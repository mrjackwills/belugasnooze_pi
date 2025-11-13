mod connect;
mod connection_details;
mod socket;
mod ws_sender;

use async_channel::Sender;
use connect::ws_upgrade;
use tracing::{error, info};

use crate::{app_env::AppEnv, message_handler::Msg};

pub use connection_details::ConnectionDetails;
pub use socket::Socket;
pub use ws_sender::WSSender;

/// try to open WS connection, and spawn a ThreadChannel message handler
pub async fn open_connection(
    app_envs: &AppEnv,
    tx: &Sender<Msg>,
    connection_details: &mut ConnectionDetails,
) {
    info!("in connection loop, awaiting delay then try to connect");
    connection_details.reconnect_delay().await;

    match ws_upgrade(app_envs).await {
        Ok(socket) => {
            info!("connected in ws_upgrade match");
            connection_details.valid_connect();
            tx.send(Msg::WsConnected(Box::new(socket))).await.ok();
        }
        Err(e) => {
            error!("connection::{e}");
            connection_details.fail_connect();
            tx.send(Msg::WsClose).await.ok();
        }
    }
    // });
}
