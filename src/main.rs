#![forbid(unsafe_code)]
#![warn(
    clippy::unused_async,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::pedantic,
    clippy::nursery,
    clippy::todo
)]
#![allow(clippy::module_name_repetitions, clippy::doc_markdown)]
// Only allow when debugging
// #![allow(unused)]

mod alarm_schedule;
mod app_error;
mod db;
mod app_env;
mod light;
mod sysinfo;
mod word_art;
mod ws;
mod ws_messages;

use alarm_schedule::CronAlarm;
use app_error::AppError;
use db::init_db;
use app_env::AppEnv;
use simple_signal::{self, Signal};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::broadcast;
use word_art::Intro;
use ws::open_connection;

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

    let db = Arc::new(init_db(&app_envs).await?);
    let light_status = Arc::new(AtomicBool::new(false));

    close_signal(Arc::clone(&light_status));

    let (sx, _keep_alive) = broadcast::channel(128);

    open_connection(
        CronAlarm::init(&db, Arc::clone(&light_status), sx.clone()).await?,
        app_envs,
        db,
        light_status,
        sx,
    )
    .await
}
