use std::path::{Path, PathBuf};

use async_channel::{Receiver, Sender};
use sqlx::SqlitePool;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::{
    alarm_schedule::AlarmSchedule,
    app_env::AppEnv,
    app_error::AppError,
    light::{LightControl, LightMsg},
    ws::{self, ConnectionDetails, Socket, WSSender, open_connection},
    ws_messages::Response,
};

pub type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
pub type WSReader =
    futures_util::stream::SplitStream<Box<WebSocketStream<MaybeTlsStream<TcpStream>>>>;
pub type WSWriter = futures_util::stream::SplitSink<
    Box<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    tokio_tungstenite::tungstenite::Message,
>;

#[derive(Debug)]
struct StatusFile(PathBuf);

impl StatusFile {
    fn new(app_env: &AppEnv) -> Self {
        Self(Path::new(&app_env.location_status_dir).join(&app_env.status_file_name))
    }

    /// Check if status fie exists
    async fn exists(&self) -> bool {
        match tokio::fs::try_exists(&self.0).await {
            Ok(exists) => exists,
            Err(e) => {
                tracing::error!("{e}");
                false
            }
        }
    }

    /// Try to create a status file
    async fn create(&self) {
        if !self.exists().await
            && let Err(e) = tokio::fs::File::create_new(&self.0).await
        {
            tracing::error!("{e}")
        }
    }

    /// Remove the status file
    async fn remove(&self) {
        if self.exists().await
            && let Err(e) = tokio::fs::remove_file(&self.0).await
        {
            tracing::error!("{e}");
        }
    }

    async fn toggle(&self, create: Option<()>) {
        if create.is_some() {
            self.create().await
        } else {
            self.remove().await
        }
    }
}
#[derive(Debug)]
pub enum Msg {
    Exit,
    GetLEDStatus(Sender<bool>),
    Ping,
    Received(String),
    ResetAlarmLoop,
    SendLEDStatus,
    SetLED(bool),
    StartAlarm,
    StatusFile(Option<()>),
    ToSend((Response, Option<bool>)),
    WsClose,
    WsConnected(Box<WsStream>),
}

#[derive(Debug)]
pub struct MessageHandler {
    alarm_schedule: AlarmSchedule,
    app_env: AppEnv,
    connection_details: ConnectionDetails,
    light_tx: Sender<LightMsg>,
    rx: Receiver<Msg>,
    socket: Option<Socket>,
    status_file: StatusFile,
    sqlite: SqlitePool,
    tx: Sender<Msg>,
    ws_sender: WSSender,
}

impl MessageHandler {
    /// Send a status update, will be spawned in own thread before sending back to message handler here
    fn send_status(&self) {
        let ws_sender = self.ws_sender.clone();
        tokio::spawn(async move {
            ws_sender.send_status().await;
        });
    }

    /// Send a LED update, will be spawned in own thread before sending back to message handler here
    fn send_led_status(&self) {
        let ws_sender = self.ws_sender.clone();
        tokio::spawn(async move {
            ws_sender.send_led_status().await;
        });
    }

    /// Start the message handler
    pub async fn start(&mut self) -> Result<(), AppError> {
        tokio::join!(
            self.alarm_schedule.start_alarm_thread(&self.sqlite),
            open_connection(&self.app_env, &self.tx, &mut self.connection_details)
        )
        .0?;

        // Turn the light off at start
        self.light_tx.send(LightMsg::Off).await.ok();

        while let Ok(msg) = self.rx.recv().await {
            match msg {
                Msg::Exit => {
                    self.light_tx.send(LightMsg::Exit).await.ok();
                    if let Some(socket) = &mut self.socket {
                        socket.close().await;
                    }
                }
                Msg::GetLEDStatus(sender) => {
                    self.light_tx.send(LightMsg::Get(sender)).await.ok();
                }
                Msg::StatusFile(create) => self.status_file.toggle(create).await,
                Msg::Ping => {
                    if let Some(socket) = &mut self.socket {
                        socket.on_ping(&self.tx);
                    }
                }
                Msg::Received(msg) => {
                    let ws_sender = self.ws_sender.clone();
                    tokio::spawn(async move {
                        ws_sender.on_text(msg).await;
                    });
                }
                Msg::ResetAlarmLoop => {
                    self.alarm_schedule.start_alarm_thread(&self.sqlite).await?;
                    self.send_status();
                }

                Msg::SendLEDStatus => self.send_led_status(),
                Msg::SetLED(status) => {
                    self.light_tx.send(LightMsg::Toggle(status)).await.ok();
                }
                Msg::StartAlarm => {
                    self.light_tx.send(LightMsg::Alarm).await.ok();
                }
                Msg::ToSend((response, cache)) => {
                    if let Some(socket) = &mut self.socket {
                        socket.send(response, cache).await;
                    }
                }
                Msg::WsClose => {
                    if let Some(socket) = &mut self.socket {
                        socket.close().await;
                    }
                    open_connection(&self.app_env, &self.tx, &mut self.connection_details).await;
                    self.ws_sender.on_connection();
                }
                Msg::WsConnected(stream) => {
                    self.socket = Some(Socket::new(stream, &self.tx));
                    self.send_status();
                    self.send_led_status();
                }
            }
        }
        Ok(())
    }

    pub fn new(app_env: AppEnv, sqlite: SqlitePool, rx: Receiver<Msg>, tx: Sender<Msg>) -> Self {
        let ws_sender = ws::WSSender::new(&app_env, &sqlite, &tx);
        let alarm_schedule = AlarmSchedule::new(&tx);
        let status_file = StatusFile::new(&app_env);

        Self {
            alarm_schedule,
            app_env,
            connection_details: ConnectionDetails::new(),
            light_tx: LightControl::init(&tx),
            rx,
            socket: None,
            status_file,
            sqlite,
            tx,
            ws_sender,
        }
    }
}
