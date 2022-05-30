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
use std::sync::{atomic::AtomicBool, Arc};
use time::{OffsetDateTime, UtcOffset};
use tokio::{
    net::TcpStream,
    sync::{
        broadcast::{Receiver, Sender},
        Mutex as TokioMutex,
    },
};
use tokio_tungstenite::{self, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{error, info, trace};

use crate::{
    alarm_schedule::AlarmSchedule, env::AppEnv, light::LightControl, sql::ModelTimezone,
    ws::ws_sender::WSSender,
};

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
type WSReader = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
type WSWriter = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

mod ws_sender;

#[derive(Debug, Clone, Copy)]
pub enum InternalMessage {
    Light,
}

/// handle each incoming ws message
async fn incoming_ws_message(mut reader: WSReader, ws_sender: WSSender) {
    loop {
        let mut ws = ws_sender.clone();

        // server sends a ping every 30 seconds, so just wait 45 seconds for any message, if not received then break
        let message_timeout =
            tokio::time::timeout(std::time::Duration::from_secs(45), reader.try_next()).await;

        match message_timeout {
            Ok(some_message) => match some_message {
                Ok(Some(m)) => {
                    tokio::spawn(async move {
                        match m {
                            m if m.is_close() => ws.close().await,
                            m if m.is_text() => ws.on_text(m.to_string().as_str()).await,
                            m if m.is_ping() => ws.ping().await,
                            _ => (),
                        };
                    });
                }
                Ok(None) => {
                    error!("None in incoming_ws_message");
                    ws.close().await;
                    break;
                }
                Err(e) => {
                    error!(%e);
                    error!("Error in incoming_ws_message");
                    ws.close().await;
                    break;
                }
            },
            Err(_) => {
                trace!("timeout error");
                ws.close().await;
                break;
            }
        }
    }
}

async fn incoming_internal_message(mut rx: Receiver<InternalMessage>, mut ws_sender: WSSender) {
    ws_sender.send_status().await;
    ws_sender.led_status().await;
    while let Ok(_message) = rx.recv().await {
        ws_sender.led_status().await;
    }
}

// need to spawn a new receiver on each connect
/// try to open WS connection, and spawn a ThreadChannel message handler
pub async fn open_connection(
    cron_alarm: Arc<TokioMutex<AlarmSchedule>>,
    app_envs: AppEnv,
    db: Arc<SqlitePool>,
    light_status: Arc<AtomicBool>,
    sx: Sender<InternalMessage>,
) {
    let mut connection_details = ConnectionDetails::new();
    loop {
        info!("in connection loop, awaiting delay then try to connect");
        connection_details.reconnect_delay().await;

        // something here with is_alive

        match ws_upgrade(&app_envs).await {
            Ok(socket) => {
                info!("connected in ws_upgrade match");
                connection_details.valid_connect();

                let (writer, reader) = socket.split();
                let writer = Arc::new(Mutex::new(writer));

                let db_timezone = ModelTimezone::get(&db).await.unwrap_or_default();

                let allowable = 7u8..=22;
                if allowable.contains(
                    &OffsetDateTime::now_utc()
                        .to_offset(
                            UtcOffset::from_hms(
                                db_timezone.offset_hour,
                                db_timezone.offset_minute,
                                db_timezone.offset_second,
                            )
                            .unwrap(),
                        )
                        .hour(),
                ) {
                    LightControl::rainbow(Arc::clone(&light_status)).await;
                }

                let cron_alarm = Arc::clone(&cron_alarm);
                let light_status = Arc::clone(&light_status);
                let db = Arc::clone(&db);

                let ws_sender = WSSender::new(
                    cron_alarm,
                    app_envs.clone(),
                    connection_details.get_connect_instant(),
                    db,
                    light_status,
                    sx.clone(),
                    writer,
                );

                let in_ws_sender = ws_sender.clone();
                let rx = sx.subscribe();
                let internal_message_thread = tokio::spawn(async move {
                    incoming_internal_message(rx, in_ws_sender).await;
                });

                incoming_ws_message(reader, ws_sender).await;

                internal_message_thread.abort();
                info!("aborted spawns, incoming_ws_message done, reconnect next");
            }
            Err(e) => {
                let connect_error = format!("{}", e);
                error!(%connect_error);
                connection_details.fail_connect();
            }
        }
    }
}
