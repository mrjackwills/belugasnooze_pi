use thiserror::Error;

use crate::blinkt;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Blinkt(#[from] blinkt::Error),
    #[error("'{0}' - sql file should end '.db'")]
    DbNameInvalid(String),
    #[error("'{0}' - file not found'")]
    FileNotFound(String),
    #[error("missing env: '{0}'")]
    MissingEnv(String),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("'{0}' - WS Connect'")]
    TungsteniteConnect(String),
    #[error("Invalid WS Status Code")]
    WsStatus,
}
