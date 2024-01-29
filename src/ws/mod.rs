mod connect;
mod connection_details;

use connect::ws_upgrade;
use connection_details::ConnectionDetails;
use futures_util::{
    lock::Mutex,
    stream::{SplitSink, SplitStream},
    StreamExt, TryStreamExt,
};
use sqlx::SqlitePool;
use std::{
    ops::RangeInclusive,
    sync::{atomic::AtomicBool, Arc},
};
use time::OffsetDateTime;
use tokio::{net::TcpStream, task::JoinHandle};
use tokio_tungstenite::{self, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{error, info};

use crate::{
    alarm_schedule::CronTx, app_env::AppEnv, app_error::AppError, db::ModelTimezone,
    light::LightControl, ws::ws_sender::WSSender,
};

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
type WSReader = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
type WSWriter = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

mod ws_sender;

pub type InternalTx = tokio::sync::broadcast::Sender<InternalMessage>;

#[derive(Debug, Clone, Copy)]
pub enum InternalMessage {
    Light,
}

const ALLOWABLE_HOURS: RangeInclusive<u8> = 7u8..=22;

#[derive(Debug, Default)]
struct AutoClose(Option<JoinHandle<()>>);

/// Will close the connection after 40 seconds unless a ping message is received
impl AutoClose {
    fn init(&mut self, ws_sender: &WSSender) {
        if let Some(handle) = self.0.as_ref() {
            handle.abort();
        };
        let mut ws_sender = ws_sender.clone();
        self.0 = Some(tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(40)).await;
            ws_sender.close().await;
        }));
    }
}

/// Handle each incoming ws message
async fn incoming_ws_message(mut reader: WSReader, mut ws_sender: WSSender) {
    let mut auto_close = AutoClose::default();
    auto_close.init(&ws_sender);
    while let Ok(Some(message)) = reader.try_next().await {
        match message {
            Message::Text(message) => {
                let mut ws_sender = ws_sender.clone();
                tokio::spawn(async move {
                    ws_sender.on_text(message).await;
                });
            }
            Message::Ping(_) => auto_close.init(&ws_sender),
            Message::Close(_) => {
                ws_sender.close().await;
                break;
            }
            _ => (),
        };
    }
    info!("incoming_ws_message done");
}

/// Send pi status message  and light status message to connect client, for when light turns off
fn incoming_internal_message(tx: &InternalTx, ws_sender: &WSSender) -> JoinHandle<()> {
    let mut ws_sender = ws_sender.clone();

    let mut rx = tx.subscribe();
    tokio::spawn(async move {
        ws_sender.send_status().await;
        ws_sender.led_status().await;
        while let Ok(message) = rx.recv().await {
            match message {
                InternalMessage::Light => {
                    info!("sending led status");
                    ws_sender.led_status().await;
                }
            }
        }
    })
}

/// If the current time is in the alllowable range, then illuminate the lights in a rainbow sequence
async fn rainbow(db: &SqlitePool, light_status: &Arc<AtomicBool>, app_envs: &AppEnv) {
    let db_timezone = ModelTimezone::get(db).await.unwrap_or_default();
    if ALLOWABLE_HOURS.contains(
        &OffsetDateTime::now_utc()
            .to_offset(db_timezone.get_offset())
            .hour(),
    ) {
        LightControl::rainbow(Arc::clone(light_status), app_envs).await;
    }
}

/// need to spawn a new receiver on each connect
/// try to open WS connection, and spawn a ThreadChannel message handler
pub async fn open_connection(
    app_envs: AppEnv,
    c_tx: CronTx,
    db: SqlitePool,
    i_tx: InternalTx,
    light_status: Arc<AtomicBool>,
) -> Result<(), AppError> {
    let mut connection_details = ConnectionDetails::new();
    loop {
        info!("in connection loop, awaiting delay then try to connect");
        connection_details.reconnect_delay().await;

        match ws_upgrade(&app_envs).await {
            Ok(socket) => {
                info!("connected in ws_upgrade match");
                connection_details.valid_connect();

                let (writer, reader) = socket.split();

                rainbow(&db, &light_status, &app_envs).await;

                let ws_sender = WSSender::new(
                    &app_envs,
                    c_tx.clone(),
                    connection_details.get_connect_instant(),
                    &db,
                    i_tx.clone(),
                    &light_status,
                    Arc::new(Mutex::new(writer)),
                );

                let internal_message_thread = incoming_internal_message(&i_tx, &ws_sender);

                incoming_ws_message(reader, ws_sender).await;
                internal_message_thread.abort();

                info!("aborted spawns, incoming_ws_message done, reconnect next");
            }
            Err(e) => {
                error!("connection::{e}");
                connection_details.fail_connect();
            }
        }
    }
}
