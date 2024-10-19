mod alarm_schedule;
mod app_env;
mod app_error;
mod db;
mod light;
mod sysinfo;
mod word_art;
mod ws;
mod ws_messages;

use alarm_schedule::AlarmSchedule;
use app_env::AppEnv;
use app_error::AppError;
use db::init_db;
use simple_signal::{self, Signal};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use word_art::Intro;
use ws::open_connection;

/// Simple macro to create a new String, or convert from a &str to  a String - basically just gets rid of String::from() / .to_owned() etc
#[macro_export]
macro_rules! S {
    () => {
        String::new()
    };
    ($s:expr) => {
        String::from($s)
    };
}

/// Simple macro to call `.clone()` on whatever is passed in
#[macro_export]
macro_rules! C {
    ($i:expr) => {
        $i.clone()
    };
}

fn close_signal(light_status: Arc<AtomicBool>) {
    simple_signal::set_handler(&[Signal::Int, Signal::Term], move |_| {
        light_status.store(false, Ordering::Relaxed);
        std::thread::sleep(std::time::Duration::from_millis(250));
        std::process::exit(1);
    });
}

fn setup_tracing(app_envs: &AppEnv) {
    tracing_subscriber::fmt()
        .with_max_level(app_envs.log_level)
        .init();
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let app_envs = AppEnv::get();
    setup_tracing(&app_envs);
    Intro::new(&app_envs).show();

    let db = init_db(&app_envs).await?;
    let light_status = Arc::new(AtomicBool::new(false));

    close_signal(Arc::clone(&light_status));

    let (i_tx, _keep_alive) = tokio::sync::broadcast::channel(128);

    let cron_sx = AlarmSchedule::init(C!(i_tx), Arc::clone(&light_status), C!(db)).await?;

    open_connection(app_envs, cron_sx, db, i_tx, light_status).await
}

#[cfg(test)]
#[expect(clippy::unwrap_used)]
mod tests {
    use std::{path::PathBuf, time::SystemTime};

    use sqlx::SqlitePool;
    use uuid::Uuid;

    use crate::{
        app_env::{AppEnv, EnvTimeZone},
        db::init_db,
    };
    /// CLose database connection, and delete all test files
    pub async fn test_cleanup(uuid: Uuid, db: Option<SqlitePool>) {
        if let Some(db) = db {
            db.close().await;
        }
        let sql_name = PathBuf::from(format!("/dev/shm/{uuid}.db"));
        let sql_sham = sql_name.join("-shm");
        let sql_wal = sql_name.join("-wal");
        tokio::fs::remove_file(sql_wal).await.ok();
        tokio::fs::remove_file(sql_sham).await.ok();
        tokio::fs::remove_file(sql_name).await.ok();
    }

    pub fn gen_app_envs(uuid: Uuid) -> AppEnv {
        AppEnv {
            location_ip_address: S!("./ip.addr"),
            location_sqlite: format!("/dev/shm/{uuid}.db"),
            log_level: tracing::Level::INFO,
            start_time: SystemTime::now(),
            rainbow: None,
            timezone: EnvTimeZone::new("Europe/London"),
            ws_address: S!("ws_address"),
            ws_apikey: S!("ws_apikey"),
            ws_password: S!("ws_password"),
            ws_token_address: S!("ws_token_address"),
        }
    }

    pub async fn test_setup() -> (AppEnv, SqlitePool, Uuid) {
        let uuid = Uuid::new_v4();
        let app_envs = gen_app_envs(uuid);
        let db = init_db(&app_envs).await.unwrap();
        (app_envs, db, uuid)
    }

    #[macro_export]
    /// Sleep for a given number of milliseconds, is an async fn.
    /// If no parameter supplied, defaults to 1000ms
    macro_rules! sleep {
        () => {
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        };
        ($ms:expr) => {
            tokio::time::sleep(std::time::Duration::from_millis($ms)).await;
        };
    }
}
