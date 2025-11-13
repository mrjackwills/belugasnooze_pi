use async_channel::Sender;
use futures_util::{SinkExt, StreamExt, TryStreamExt};
use tokio_tungstenite::tungstenite::Message;
use tokio_util::sync::CancellationToken;

use crate::{
    alarm_schedule::ONE_SECOND_AS_MS,
    message_handler::{Msg, WSReader, WSWriter, WsStream},
    sleep,
    ws_messages::{Response, StructuredResponse},
};

#[derive(Debug)]
pub struct Socket {
    writer: WSWriter,
    incoming_msg_token: CancellationToken,
    auto_close_token: CancellationToken,
}

impl Socket {
    /// Split the stream into reader and writer, and spawn threads for incoming messages & an autocloser
    pub fn new(stream: Box<WsStream>, tx: &Sender<Msg>) -> Self {
        let (writer, reader) = stream.split();
        Self {
            incoming_msg_token: Self::start_incoming_msg_thread(reader, tx),
            auto_close_token: Self::start_auto_close(tx),
            writer,
        }
    }

    /// Send a message over the WebSocket
    pub async fn send(&mut self, response: Response, cache: Option<bool>) {
        if let Err(e) = self
            .writer
            .send(StructuredResponse::data(response, cache))
            .await
        {
            tracing::error!("{e}");
        }
    }

    /// Reset the ping handler thread
    pub fn on_ping(&mut self, tx: &Sender<Msg>) {
        self.auto_close_token.cancel();
        self.auto_close_token = Self::start_auto_close(tx);
    }

    /// Close the socket
    pub async fn close(&mut self) {
        self.auto_close_token.cancel();
        self.incoming_msg_token.cancel();
        tokio::time::timeout(std::time::Duration::from_secs(2), self.writer.close())
            .await
            .ok()
            .map(std::result::Result::ok);
    }

    /// Spawn a thread to handle incoming messages
    fn start_incoming_msg_thread(reader: WSReader, tx: &Sender<Msg>) -> CancellationToken {
        let tx = tx.clone();
        let token = CancellationToken::new();
        let t_token = token.clone();
        tokio::spawn(async move {
            t_token
                .run_until_cancelled(Self::message_recv(reader, tx))
                .await;
        });
        token
    }

    /// Actually handle incoming WS messages
    async fn message_recv(mut reader: WSReader, tx: Sender<Msg>) {
        while let Ok(Some(x)) = reader.try_next().await {
            match x {
                Message::Text(message) => {
                    tx.send(Msg::Received(message.to_string())).await.ok();
                }
                Message::Ping(_) => {
                    tx.send(Msg::Ping).await.ok();
                }
                Message::Close(_) => {
                    tx.send(Msg::WsClose).await.ok();
                    break;
                }
                _ => tracing::info!("Unexpected WS message received"),
            }
        }
    }

    /// Spawn autoclose method
    fn start_auto_close(tx: &Sender<Msg>) -> CancellationToken {
        let tx = tx.clone();
        let token = CancellationToken::new();
        let t_token = token.clone();
        tokio::spawn(async move {
            t_token.run_until_cancelled(Self::sleep_then_send(tx)).await;
        });
        token
    }

    /// Method run in the autoclose thread
    async fn sleep_then_send(tx: Sender<Msg>) {
        sleep!(40 * ONE_SECOND_AS_MS);
        tx.send(Msg::WsClose).await.ok();
    }
}
