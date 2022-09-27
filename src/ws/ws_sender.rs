use futures_util::lock::Mutex;
use futures_util::SinkExt;
use sqlx::SqlitePool;
use std::process;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Instant;
use time_tz::{timezones, Offset, TimeZone};
use tokio::sync::{broadcast::Sender, Mutex as TokioMutex};
// use tokio_tungstenite;
use tracing::{debug, error, trace};

use crate::alarm_schedule::AlarmSchedule;
use crate::sysinfo::SysInfo;
use crate::ws_messages::{MessageValues, ParsedMessage, PiStatus, Response, StructuredResponse};
use crate::{
    env::AppEnv,
    light::LightControl,
    sql::{ModelAlarm, ModelTimezone},
    ws_messages::to_struct,
};

use super::{InternalMessage, WSWriter};

#[derive(Debug, Clone)]
pub struct WSSender {
    alarm_scheduler: Arc<TokioMutex<AlarmSchedule>>,
    app_envs: AppEnv,
    connected_instant: Instant,
    db: Arc<SqlitePool>,
    light_status: Arc<AtomicBool>,
    sx: Sender<InternalMessage>,
    writer: Arc<Mutex<WSWriter>>,
}

impl WSSender {
    pub fn new(
        alarm_scheduler: Arc<TokioMutex<AlarmSchedule>>,
        app_envs: AppEnv,
        connected_instant: Instant,
        db: Arc<SqlitePool>,
        light_status: Arc<AtomicBool>,
        sx: Sender<InternalMessage>,
        writer: Arc<Mutex<WSWriter>>,
    ) -> Self {
        Self {
            alarm_scheduler,
            app_envs,
            connected_instant,
            db,
            light_status,
            sx,
            writer,
        }
    }

    /// Handle text message, in this program they will all be json text
    pub async fn on_text(&mut self, message: &str) {
        if let Some(data) = to_struct(message) {
            match data {
                MessageValues::Invalid(error) => error!("{:?}", error),
                MessageValues::Valid(data) => match data {
                    ParsedMessage::DeleteAll => self.delete_all().await,
                    ParsedMessage::DeleteOne(id) => self.delete_one(id.alarm_id).await,
                    ParsedMessage::LedStatus => self.led_status().await,
                    ParsedMessage::Restart => self.restart().await,
                    ParsedMessage::TimeZone(timezone) => self.time_zone(timezone.zone).await,
                    ParsedMessage::AddAlarm(data) => {
                        self.add_alarm(data.days, data.hour, data.minute).await;
                    }
                    ParsedMessage::Light { status } => self.toggle_light(status).await,
                    ParsedMessage::Status => self.send_status().await,
                },
            }
        }
    }

    /// Add a new alarm to database, and update alarm_schedule alarm vector
    async fn add_alarm(&mut self, day: Vec<u8>, hour: u8, minute: u8) {
        let mut handles = vec![];
        for i in day {
            handles.push(ModelAlarm::add(&self.db, (i, hour, minute)));
        }
        for handle in handles {
            match handle.await {
                Ok(_) => (),
                Err(e) => debug!(%e),
            }
        }
        self.alarm_scheduler
            .lock()
            .await
            .refresh_alarms(&self.db)
            .await;
        self.send_status().await;
    }

    // /// Handle websocket close event
    // pub async fn ping(self) {
    //     self.writer
    //         .lock()
    //         .await
    //         .send(Message::Pong(vec![]))
    //         .await
    //         .unwrap_or(());
    // }

    /// Delete all alarms in database, and update alarm_schedule alarm vector
    /// If the alarm sequence has started, and you delete all alarms, the light is still on
    /// Would need to set the light status to false, but that could also set the light off if on not during an alarm sequence
    async fn delete_all(&mut self) {
        ModelAlarm::delete_all(&self.db).await.unwrap_or(());
        self.alarm_scheduler
            .lock()
            .await
            .refresh_alarms(&self.db)
            .await;
        self.send_status().await;
    }

    /// Delete from database a given alarm, by id, and also remove from alarm_schedule alarm vector
    async fn delete_one(&mut self, id: i64) {
        ModelAlarm::delete(&self.db, id).await.unwrap_or(());
        self.alarm_scheduler.lock().await.remove_alarm(id);
        self.send_status().await;
    }

    /// This also needs to be send from alarm sequencer
    /// return true if led light is currently turned on
    pub async fn led_status(&mut self) {
        let status = self.light_status.load(Ordering::SeqCst);
        let response = Response::LedStatus { status };
        self.send_ws_response(response, None).await;
    }

    /// Force quite program, assumes running in an auto-restart container, or systemd, in order to start again immediately
    async fn restart(&mut self) {
        self.close().await;
        process::exit(0);
    }

    /// Change the timezone in database to new given database,
    /// also update timezone in alarm scheduler
    async fn time_zone(&mut self, zone: String) {
        if let Some(tz) = timezones::get_by_name(&zone) {
            let offset = tz.get_offset_utc(&time::OffsetDateTime::now_utc()).to_utc();
            ModelTimezone::update(&self.db, &zone, offset)
                .await
                .unwrap_or_default();
            self.alarm_scheduler
                .lock()
                .await
                .refresh_timezone(&self.db)
                .await;
        }
        self.send_status().await;
    }

    /// turn light either on or off
    async fn toggle_light(&mut self, new_status: bool) {
        if new_status && !self.light_status.load(Ordering::SeqCst) {
            self.light_status.store(true, Ordering::SeqCst);
            let response = Response::LedStatus { status: new_status };
            self.send_ws_response(response, None).await;
            LightControl::turn_on(Arc::clone(&self.light_status), &self.sx).await;
        } else if !new_status {
            self.light_status.store(false, Ordering::SeqCst);
            self.led_status().await;
        }
    }

    /// Send a message to the socket
    async fn send_ws_response(&mut self, response: Response, cache: Option<bool>) {
        match self
            .writer
            .lock()
            .await
            .send(StructuredResponse::data(response, cache))
            .await
        {
            Ok(_) => trace!("Message sent"),
            Err(e) => {
                error!("send_ws_response::SEND-ERROR::{:?}", e);
                process::exit(1);
            }
        }
    }

    /// Generate, and send, pi information
    pub async fn send_status(&mut self) {
        let info = SysInfo::new(&self.db, &self.app_envs).await;
        let alarms = ModelAlarm::get_all(&self.db).await.unwrap_or_default();
        let info = PiStatus::new(info, alarms, self.connected_instant.elapsed().as_secs());
        let response = Response::Status(info);
        self.send_ws_response(response, Some(true)).await;
    }

    /// close connection
    pub async fn close(&mut self) {
        self.writer.lock().await.close().await.unwrap_or_default();
    }
}
