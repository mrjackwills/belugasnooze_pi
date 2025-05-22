use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("'{0}' - sql file should end '.db'")]
    DbNameInvalid(String),
    #[error("'{0}' - file not found'")]
    FileNotFound(String),
    #[error("missing env: '{0}'")]
    MissingEnv(String),
    #[error("Reqwest Error")]
    Reqwest(#[from] reqwest::Error),
    #[error("Internal Database Error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("'{0}' - WS Connect'")]
    TungsteniteConnect(String),
    #[error("Invalid WS Status Code")]
    WsStatus,
}
