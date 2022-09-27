#![forbid(unsafe_code)]
#![warn(clippy::unused_async, clippy::unwrap_used, clippy::expect_used)]
// Wanring - These are indeed pedantic
// #![warn(clippy::pedantic)]
// #![warn(clippy::nursery)]
// #![allow(clippy::module_name_repetitions, clippy::doc_markdown)]
// Only allow when debugging
// #![allow(unused)]

mod alarm_schedule;
mod app_error;
mod env;
mod light;
mod db;
mod sysinfo;
mod word_art;
mod ws;
mod ws_messages;

use alarm_schedule::CronAlarm;
use app_error::AppError;
use env::AppEnv;
use simple_signal::{self, Signal};
use db::init_db;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::broadcast;
use tracing::Level;
use word_art::Intro;
use ws::open_connection;

fn close_signal(light_status: Arc<AtomicBool>) {
    simple_signal::set_handler(&[Signal::Int, Signal::Term], move |_| {
        light_status.store(false, Ordering::SeqCst);
        std::thread::sleep(std::time::Duration::from_millis(250));
        std::process::exit(1);
    });
}

fn setup_tracing(app_envs: &AppEnv) {
    let level = if app_envs.trace {
        Level::TRACE
    } else if app_envs.debug {
        Level::DEBUG
    } else {
        Level::INFO
    };
    tracing_subscriber::fmt().with_max_level(level).init();
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let app_envs = AppEnv::get().await;
    setup_tracing(&app_envs);
    Intro::new(&app_envs).show();

    // Should probably handle this better
    let db = Arc::new(init_db(&app_envs).await?);
    let light_status = Arc::new(AtomicBool::new(false));

    close_signal(Arc::clone(&light_status));

    let (sx, _keep_alive) = broadcast::channel(128);

    let cron_alarm = CronAlarm::init(&db, Arc::clone(&light_status), sx.clone()).await?;

    open_connection(
        Arc::clone(&cron_alarm),
        app_envs,
        Arc::clone(&db),
        Arc::clone(&light_status),
        sx,
    )
    .await?;

    Ok(())
}
