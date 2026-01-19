use async_channel::Sender;
use mimalloc::MiMalloc;
use simple_signal::{self, Signal};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod alarm_schedule;
mod app_env;
mod app_error;
mod blinkt;
mod db;
mod light;
mod macros;
mod message_handler;
mod sysinfo;
mod word_art;
mod ws;
mod ws_messages;

use app_env::AppEnv;
use app_error::AppError;
use db::init_db;
use word_art::Intro;

use crate::message_handler::{MessageHandler, Msg};

fn close_signal(tx: &Sender<Msg>) {
    let tx = C!(tx);
    simple_signal::set_handler(&[Signal::Int, Signal::Term], move |_| {
        tx.send_blocking(Msg::Exit).ok();
        std::thread::sleep(std::time::Duration::from_millis(250));
        std::process::exit(1);
    });
}

fn setup_tracing(app_envs: &AppEnv) {
    tracing_subscriber::fmt()
        .with_max_level(app_envs.log_level)
        .init();
}

async fn start() -> Result<(), AppError> {
    let app_envs = AppEnv::get();
    setup_tracing(&app_envs);
    Intro::new(&app_envs).show();
    let sqlite = init_db(&app_envs).await?;
    let (tx, rx) = async_channel::bounded(2048);
    close_signal(&tx);
    MessageHandler::new(app_envs, sqlite, rx, tx).start().await
}

#[tokio::main]
async fn main() {
    tokio::spawn(async move {
        if let Err(e) = start().await {
            tracing::error!("{e:}");
        }
    })
    .await
    .ok();
}

#[cfg(test)]
#[expect(clippy::unwrap_used)]
mod tests {
    use std::{path::PathBuf, time::SystemTime};

    use sqlx::SqlitePool;
    use uuid::Uuid;

    use crate::{S, app_env::AppEnv, db::init_db};
    /// Close database connection, and delete all test files
    pub async fn test_cleanup(uuid: Uuid, db: Option<SqlitePool>) {
        if let Some(db) = db {
            db.close().await;
        }
        let sql_name = PathBuf::from(format!("/dev/shm/{uuid}.db"));
        let sql_sham = sql_name.join("-shm");
        let sql_wal = sql_name.join("-wal");
        tokio::try_join!(
            tokio::fs::remove_file(sql_wal),
            tokio::fs::remove_file(sql_sham),
            tokio::fs::remove_file(sql_name)
        )
        .ok();
    }

    pub fn gen_app_envs(uuid: Uuid) -> AppEnv {
        AppEnv {
            location_ip_address: S!("./ip.addr"),
            location_sqlite: format!("/dev/shm/{uuid}.db"),
            log_level: tracing::Level::INFO,
            start_time: SystemTime::now(),
            timezone: jiff::tz::TimeZone::get("Europe/London").unwrap(),
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
}
