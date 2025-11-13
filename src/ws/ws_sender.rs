use async_channel::Sender;
use sqlx::SqlitePool;
use std::{process, time::Instant};

use crate::C;
use crate::message_handler::Msg;
use crate::sysinfo::SysInfo;
use crate::ws_messages::{MessageValues, ParsedMessage, PiStatus, Response};
use crate::{
    app_env::AppEnv,
    db::{ModelAlarm, ModelTimezone},
    ws_messages::to_struct,
};

#[derive(Debug, Clone)]
pub struct WSSender {
    app_envs: AppEnv,
    connected_instant: Instant,
    sqlite: SqlitePool,
    tx: Sender<Msg>,
}

impl WSSender {
    pub fn new(app_envs: &AppEnv, sqlite: &SqlitePool, tx: &Sender<Msg>) -> Self {
        Self {
            app_envs: C!(app_envs),
            connected_instant: std::time::Instant::now(),
            sqlite: C!(sqlite),
            tx: C!(tx),
        }
    }

    /// Update the connected_instance time
    pub fn on_connection(&mut self) {
        self.connected_instant = std::time::Instant::now();
    }

    /// Handle text message, in this program they will all be json text
    pub async fn on_text(&self, message: String) {
        if let Some(data) = to_struct(&message) {
            match data {
                MessageValues::Invalid(error) => tracing::error!("invalid::{error:?}"),
                MessageValues::Valid(data) => match data {
                    ParsedMessage::DeleteAll => self.delete_all().await,
                    ParsedMessage::DeleteOne(id) => self.delete_one(id.alarm_id).await,
                    ParsedMessage::LedStatus => self.send_led_status().await,
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

    /// Get the current value of the light bool
    async fn get_light_value(&self) -> bool {
        let (t, r) = async_channel::bounded(1);
        self.tx.send(Msg::GetLEDStatus(t)).await.ok();
        r.recv().await.unwrap_or_default()
    }

    /// Add a new alarm to database, and update alarm_schedule alarm vector
    async fn add_alarm(&self, day: Vec<u8>, hour: u8, minute: u8) {
        for i in day {
            if let Err(e) = ModelAlarm::add(&self.sqlite, (i, hour, minute)).await {
                tracing::debug!("{e}");
            }
        }
        self.update_loop().await;
        self.send_status().await;
    }

    /// Delete all alarms in database, and update alarm_schedule alarm vector
    /// If the alarm sequence has started, and you delete all alarms, the light is still on
    /// Would need to set the light status to false, but that could also set the light off if on not during an alarm sequence
    async fn delete_all(&self) {
        ModelAlarm::delete_all(&self.sqlite).await.ok();
        self.update_loop().await;
        self.send_status().await;
    }

    /// Delete from database a given alarm, by id, and also remove from alarm_schedule alarm vector
    async fn delete_one(&self, id: i64) {
        ModelAlarm::delete(&self.sqlite, id).await.unwrap_or(());
        self.update_loop().await;
        self.send_status().await;
    }

    /// This also needs to be send from alarm sequencer
    /// return true if led light is currently turned on
    pub async fn send_led_status(&self) {
        self.send_ws_response(
            Response::LedStatus {
                status: self.get_light_value().await,
            },
            None,
        )
        .await;
    }

    /// Force quite program, assumes running in an auto-restart container, or systemd, in order to start again immediately
    async fn restart(&self) {
        self.close().await;
        process::exit(0);
    }

    /// Change the timezone in database to new given database,
    /// also update timezone in alarm scheduler
    async fn time_zone(&self, zone: String) {
        if let Ok(z) = jiff::tz::TimeZone::get(&zone) {
            match ModelTimezone::update(&self.sqlite, &z).await {
                Err(e) => {
                    tracing::error!("{e}");
                }
                _ => {
                    self.update_loop().await;
                }
            }
            self.send_status().await;
        }
    }

    /// turn light either on or off
    async fn toggle_light(&self, status: bool) {
        self.tx.send(Msg::SetLED(status)).await.ok();
    }

    /// Send a message to restar the alarm loop, used when alarms added or deleted
    async fn update_loop(&self) {
        self.tx.send(Msg::ResetAlarmLoop).await.ok();
    }

    /// Send a message to the socket
    /// cache could just be Option<()>, and if some then send true?
    async fn send_ws_response(&self, response: Response, cache: Option<bool>) {
        match self.tx.send(Msg::ToSend((response, cache))).await {
            Ok(()) => (),
            Err(e) => {
                tracing::error!("{e}");
                process::exit(1);
            }
        }
    }

    /// Generate, and send, pi information
    pub async fn send_status(&self) {
        let info = SysInfo::new(&self.sqlite, &self.app_envs).await;
        let alarms = ModelAlarm::get_all(&self.sqlite).await.unwrap_or_default();
        let info = PiStatus::new(info, alarms, self.connected_instant.elapsed().as_secs());
        self.send_ws_response(Response::Status(info), Some(true))
            .await;
    }

    /// Send a message to close the socket
    pub async fn close(&self) {
        self.tx.send(Msg::WsClose).await.ok();
    }
}
