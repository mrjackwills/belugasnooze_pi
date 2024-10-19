use futures_util::lock::Mutex;
use futures_util::SinkExt;
use sqlx::SqlitePool;
use std::process;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Instant;
use time_tz::timezones;
use tracing::{debug, error, trace};

use crate::alarm_schedule::{CronMessage, CronTx};
use crate::sysinfo::SysInfo;
use crate::ws_messages::{MessageValues, ParsedMessage, PiStatus, Response, StructuredResponse};
use crate::C;
use crate::{
    app_env::AppEnv,
    db::{ModelAlarm, ModelTimezone},
    light::LightControl,
    ws_messages::to_struct,
};

use super::{InternalTx, WSWriter};

#[derive(Debug, Clone)]
pub struct WSSender {
    app_envs: AppEnv,
    c_tx: CronTx,
    connected_instant: Instant,
    db: SqlitePool,
    i_tx: InternalTx,
    light_status: Arc<AtomicBool>,
    writer: Arc<Mutex<WSWriter>>,
}

impl WSSender {
    pub fn new(
        app_envs: &AppEnv,
        c_tx: CronTx,
        connected_instant: Instant,
        db: &SqlitePool,
        i_tx: InternalTx,
        light_status: &Arc<AtomicBool>,
        writer: Arc<Mutex<WSWriter>>,
    ) -> Self {
        Self {
            app_envs: C!(app_envs),
            connected_instant,
            db: C!(db),
            light_status: Arc::clone(light_status),
            c_tx,
            i_tx,
            writer,
        }
    }

    /// Handle text message, in this program they will all be json text
    pub async fn on_text(&self, message: String) {
        if let Some(data) = to_struct(&message) {
            match data {
                MessageValues::Invalid(error) => error!("invalid::{error:?}"),
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
    async fn add_alarm(&self, day: Vec<u8>, hour: u8, minute: u8) {
        let handles = day
            .into_iter()
            .map(|i| ModelAlarm::add(&self.db, (i, hour, minute)))
            .collect::<Vec<_>>();
        for handle in handles {
            if let Err(e) = handle.await {
                debug!("{e}");
            }
        }
        tracing::info!("update alarm scheduler");
        self.c_tx.send(CronMessage::ResetLoop).await.ok();
        tracing::info!("send status");
        self.send_status().await;
    }

    /// Delete all alarms in database, and update alarm_schedule alarm vector
    /// If the alarm sequence has started, and you delete all alarms, the light is still on
    /// Would need to set the light status to false, but that could also set the light off if on not during an alarm sequence
    async fn delete_all(&self) {
        ModelAlarm::delete_all(&self.db).await.ok();
        self.c_tx.send(CronMessage::ResetLoop).await.ok();
        self.send_status().await;
    }

    /// Delete from database a given alarm, by id, and also remove from alarm_schedule alarm vector
    async fn delete_one(&self, id: i64) {
        ModelAlarm::delete(&self.db, id).await.unwrap_or(());
        self.c_tx.send(CronMessage::ResetLoop).await.ok();
        self.send_status().await;
    }

    /// This also needs to be send from alarm sequencer
    /// return true if led light is currently turned on
    pub async fn led_status(&self) {
        let status = self.light_status.load(Ordering::Relaxed);
        let response = Response::LedStatus { status };
        self.send_ws_response(response, None).await;
    }

    /// Force quite program, assumes running in an auto-restart container, or systemd, in order to start again immediately
    async fn restart(&self) {
        self.close().await;
        process::exit(0);
    }

    /// Change the timezone in database to new given database,
    /// also update timezone in alarm scheduler
    async fn time_zone(&self, zone: String) {
        if timezones::get_by_name(&zone).is_some() {
            if let Err(e) = ModelTimezone::update(&self.db, &zone).await {
                tracing::error!("{e}");
            } else {
                self.c_tx.send(CronMessage::ResetLoop).await.ok();
            }
            self.send_status().await;
        }
    }

    /// turn light either on or off
    async fn toggle_light(&self, new_status: bool) {
        if new_status && !self.light_status.load(Ordering::Relaxed) {
            self.light_status.store(true, Ordering::Relaxed);
            let response = Response::LedStatus { status: new_status };
            self.send_ws_response(response, None).await;
            LightControl::turn_on(Arc::clone(&self.light_status), &self.i_tx).await;
        } else if !new_status {
            self.light_status.store(false, Ordering::Relaxed);
            self.led_status().await;
        }
    }

    /// Send a message to the socket
    /// cache could just be Option<()>, and if some then send true?
    async fn send_ws_response(&self, response: Response, cache: Option<bool>) {
        match self
            .writer
            .lock()
            .await
            .send(StructuredResponse::data(response, cache))
            .await
        {
            Ok(()) => trace!("Message sent"),
            Err(e) => {
                error!("send_ws_response::SEND-ERROR::{e:?}");
                process::exit(1);
            }
        }
    }

    /// Generate, and send, pi information
    pub async fn send_status(&self) {
        let info = SysInfo::new(&self.db, &self.app_envs).await;
        let alarms = ModelAlarm::get_all(&self.db).await.unwrap_or_default();
        let info = PiStatus::new(info, alarms, self.connected_instant.elapsed().as_secs());
        self.send_ws_response(Response::Status(info), Some(true))
            .await;
    }

    /// close connection, uses a 2 second timeout
    pub async fn close(&self) {
        tokio::time::timeout(
            std::time::Duration::from_secs(2),
            self.writer.lock().await.close(),
        )
        .await
        .ok()
        .map(std::result::Result::ok);
    }
}
