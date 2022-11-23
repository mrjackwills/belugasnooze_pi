use sqlx::SqlitePool;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use time::{OffsetDateTime, Time};
use tokio::sync::{broadcast::Sender, Mutex};
use tracing::trace;

use crate::{
    app_error::AppError,
    db::{ModelAlarm, ModelTimezone},
    light::LightControl,
    ws::InternalMessage,
};

const ONE_SECOND: u64 = 1000;
#[derive(Debug)]
pub struct AlarmSchedule {
    alarms: Vec<ModelAlarm>,
    time_zone: ModelTimezone,
}

impl AlarmSchedule {
    async fn new(db: &SqlitePool) -> Result<Self, AppError> {
        let alarms = ModelAlarm::get_all(db).await?;
        let time_zone = ModelTimezone::get(db).await.unwrap_or_default();

        Ok(Self {
            alarms,
            time_zone,
        })
    }

    /// Remove all alarms from vector
    pub fn clear_all(&mut self) {
        self.alarms.clear();
    }

    /// Clear current alarms, get alarms from db and set as self.alarms
    /// Also update + replace timezone + offset
    pub async fn refresh_alarms(&mut self, db: &SqlitePool) {
        Self::clear_all(self);
        self.alarms = ModelAlarm::get_all(db).await.unwrap_or_default();
        Self::refresh_timezone(self, db).await;
    }

    /// Get timezone from db and store into self, also update offset
    pub async fn refresh_timezone(&mut self, db: &SqlitePool) {
        if let Some(time_zone) = ModelTimezone::get(db).await {
            self.time_zone = time_zone;
        }
    }

    /// Remove alarm from vector by id
    pub fn remove_alarm(&mut self, id: i64) {
        let alarm_item = self.alarms.iter().enumerate().find(|i| i.1.alarm_id == id);
        if let Some((index, _)) = alarm_item {
            self.alarms.remove(index);
        }
    }
}

pub struct CronAlarm {
    alarm_schedule: Arc<Mutex<AlarmSchedule>>,
    light_status: Arc<AtomicBool>,
}

impl CronAlarm {
    /// create a looper and spawn into it's own async thread
    pub async fn init(
        db: &SqlitePool,
        light_status: Arc<AtomicBool>,
        sx: Sender<InternalMessage>,
    ) -> Result<Arc<Mutex<AlarmSchedule>>, AppError> {
        let alarm_schedule = Arc::new(Mutex::new(AlarmSchedule::new(db).await?));
        let mut looper = Self {
            alarm_schedule: Arc::clone(&alarm_schedule),
            light_status: Arc::clone(&light_status),
        };
        tokio::spawn(async move {
            looper.init_loop(sx).await;
        });
        Ok(alarm_schedule)
    }

    /// loop every 1 second,check if current time & day matches alarm, and if so execute alarm illuminate
    /// is private, so that it can only be executed during the self.init() method, so that it is correctly spawned onto it's own tokio thread
    async fn init_loop(&mut self, sx: Sender<InternalMessage>) {
        trace!("alarm looper started");
        loop {
			let start = std::time::Instant::now();
            if !self.alarm_schedule.lock().await.alarms.is_empty() {
				let offset = self.alarm_schedule.lock().await.time_zone.get_offset();
                let now_as_utc_offset = OffsetDateTime::now_utc().to_offset(offset);
                if let Ok(current_time) = Time::from_hms(
                    now_as_utc_offset.hour(),
                    now_as_utc_offset.minute(),
                    now_as_utc_offset.second(),
                ) {
                    let current_weekday = now_as_utc_offset
                        .to_offset(offset)
                        .weekday()
                        .number_days_from_monday();
                    for i in &self.alarm_schedule.lock().await.alarms {
                        if i.day == current_weekday
                            && i.hour == current_time.hour()
                            && i.minute == current_time.minute()
                            && !self.light_status.load(Ordering::SeqCst)
                        {
                            trace!("sending lighton message to via internal channels");
                            LightControl::alarm_illuminate(
                                Arc::clone(&self.light_status),
                                sx.clone(),
                            )
                            .await;
                        }
                    }
                }
            }
            let sleep_for =
                ONE_SECOND - u64::try_from(start.elapsed().as_millis()).unwrap_or(ONE_SECOND);
            tokio::time::sleep(std::time::Duration::from_millis(sleep_for)).await;
        }
    }
}
