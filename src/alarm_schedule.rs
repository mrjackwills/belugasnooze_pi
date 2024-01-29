use sqlx::SqlitePool;
use std::sync::{atomic::AtomicBool, Arc};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};

use crate::{
    app_error::AppError,
    db::{ModelAlarm, ModelTimezone},
    light::LightControl,
    ws::InternalTx,
};

const ONE_SECOND: u64 = 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CronMessage {
    ResetLoop,
    Light,
}

pub type CronTx = Sender<CronMessage>;
pub type CronRx = Receiver<CronMessage>;

#[derive(Debug)]
pub struct AlarmSchedule {
    c_rx: CronRx,
    c_tx: CronTx,
    i_tx: InternalTx,
    light_status: Arc<AtomicBool>,
    looper: Option<JoinHandle<()>>,
    sqlite: SqlitePool,
}

impl AlarmSchedule {
    pub async fn init(
        i_tx: InternalTx,
        light_status: Arc<AtomicBool>,
        sqlite: SqlitePool,
    ) -> Result<CronTx, AppError> {
        let (c_tx, c_rx) = tokio::sync::mpsc::channel(128);

        let mut alarm_schedule = Self {
            c_rx,
            c_tx: c_tx.clone(),
            i_tx,
            light_status,
            looper: None,
            sqlite,
        };
        alarm_schedule.generate_alarm_loop().await?;
        tokio::spawn(async move {
            alarm_schedule.message_looper().await;
        });

        Ok(c_tx)
    }

    async fn message_looper(&mut self) {
        while let Some(msg) = self.c_rx.recv().await {
            match msg {
                CronMessage::ResetLoop => {
                    if let Some(looper) = self.looper.as_ref() {
                        looper.abort();
                    }
                    if let Err(e) = self.generate_alarm_loop().await {
                        println!("Can't generate new alarm loop");
                        println!("{e}");
                    }
                }
                CronMessage::Light => {
                    LightControl::alarm_illuminate(
                        Arc::clone(&self.light_status),
                        self.i_tx.clone(),
                    );
                }
            }
        }
    }

    async fn generate_alarm_loop(&mut self) -> Result<(), AppError> {
        let alarms = ModelAlarm::get_all(&self.sqlite).await?;
        let tz = ModelTimezone::get(&self.sqlite).await.unwrap_or_default();
        let sx = self.c_tx.clone();
        self.looper = Some(tokio::spawn(async move {
            Self::init_alarm_loop(alarms, sx, tz).await;
        }));
        Ok(())
    }

    // loop every 1 second,check if current time & day matches alarm, and if so execute alarm illuminate
    // is private, so that it can only be executed during the self.init() method, so that it is correctly spawned onto it's own tokio thread
    async fn init_alarm_loop(alarms: Vec<ModelAlarm>, c_tx: CronTx, time_zone: ModelTimezone) {
        loop {
            let start = std::time::Instant::now();
            if let Some(current_time) = time_zone.to_time() {
                let week_day = time_zone
                    .now_with_offset()
                    .weekday()
                    .number_days_from_monday();

                if alarms.iter().filter(|i| i.day == week_day).any(|i| {
                    i.hour == current_time.hour()
                        && i.minute == current_time.minute()
                        && current_time.second() == 0
                }) {
                    c_tx.send(CronMessage::Light).await.ok();
                }
            }
            let to_sleep = ONE_SECOND
                .saturating_sub(u64::try_from(start.elapsed().as_millis()).unwrap_or(ONE_SECOND));
            tokio::time::sleep(std::time::Duration::from_millis(to_sleep)).await;
        }
    }
}
