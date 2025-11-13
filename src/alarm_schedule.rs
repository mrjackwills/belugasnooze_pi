use async_channel::Sender;
use sqlx::SqlitePool;
use tokio_util::sync::CancellationToken;

use crate::{
    C,
    app_error::AppError,
    db::{ModelAlarm, ModelTimezone},
    message_handler::Msg,
    sleep,
};

pub const ONE_SECOND_AS_MS: u64 = 1000;

#[derive(Debug)]
pub struct AlarmSchedule {
    tx: Sender<Msg>,
    token: Option<CancellationToken>,
}

impl AlarmSchedule {
    pub fn new(tx: &Sender<Msg>) -> Self {
        Self {
            tx: C!(tx),
            token: None,
        }
    }

    /// Cancel the current token, set a new one, and return it
    fn get_set_cancel_token(&mut self) -> CancellationToken {
        if let Some(token) = &self.token {
            token.cancel();
        }
        let token = CancellationToken::new();
        self.token = Some(C!(token));
        token
    }
    /// Start the alarm looper thread
    pub async fn start_alarm_thread(&mut self, sqlite: &SqlitePool) -> Result<(), AppError> {
        let alarms = ModelAlarm::get_all(sqlite).await?;
        let tz = ModelTimezone::get(sqlite).await.unwrap_or_default();
        let tx = self.tx.clone();
        let token = self.get_set_cancel_token();
        tokio::spawn(async move {
            token
                .run_until_cancelled(Self::init_alarm_loop(alarms, tz, tx))
                .await
        });
        Ok(())
    }

    /// loop every 1 second,check if current time & day matches alarm, and if so execute alarm illuminate
    async fn init_alarm_loop(alarms: Vec<ModelAlarm>, time_zone: ModelTimezone, tx: Sender<Msg>) {
        loop {
            let start = std::time::Instant::now();
            let current_time = time_zone.to_time();
            let week_day = time_zone
                .now_with_offset()
                .weekday()
                .to_monday_zero_offset();

            if alarms.iter().filter(|i| i.day == week_day).any(|i| {
                i.hour == current_time.hour()
                    && i.minute == current_time.minute()
                    && current_time.second() == 0
            }) {
                tx.send(Msg::StartAlarm).await.ok();
            }
            sleep!(ONE_SECOND_AS_MS.saturating_sub(
                u64::try_from(start.elapsed().as_millis()).unwrap_or(ONE_SECOND_AS_MS)
            ));
        }
    }
}
