use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Reqwest Error")]
    Reqwest(#[from] reqwest::Error),
    #[error("Internal Database Error")]
    Sqlx(#[from] sqlx::Error),
    #[error("WS Connect")]
    TungsteniteConnect(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("Invalid WS Status Code")]
    WsStatus
}