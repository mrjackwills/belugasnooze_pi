use super::WsStream;
use crate::{app_env::AppEnv, app_error::AppError};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{self, connect_async, tungstenite::http::StatusCode};

#[derive(Debug, Serialize, Deserialize)]
struct PostRequest<'a> {
    key: &'a str,
    password: &'a str,
}

impl<'a> From<&'a AppEnv> for PostRequest<'a> {
    fn from(app_envs: &'a AppEnv) -> Self {
        Self {
            key: &app_envs.ws_apikey,
            password: &app_envs.ws_password,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
// This is an ULID, but probably no need to parse it as such
struct PostResponse {
    response: String,
}

/// Make a https request to get an access token
async fn get_auth_token(app_envs: &AppEnv) -> Result<String, AppError> {
    Ok(reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_millis(5000))
        .gzip(true)
        .brotli(true)
        .user_agent(format!(
            "{}/{}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        ))
        .build()?
        .post(&app_envs.ws_token_address)
        .json(&PostRequest::from(app_envs))
        .send()
        .await?
        .json::<PostResponse>()
        .await?
        .response)
}

/// Connect to wesbsocket server
pub async fn ws_upgrade(app_envs: &AppEnv) -> Result<WsStream, AppError> {
    let url = format!(
        "{}/{}",
        app_envs.ws_address,
        get_auth_token(app_envs).await?
    );
    let (socket, response) = connect_async(url).await?;
    match response.status() {
        StatusCode::SWITCHING_PROTOCOLS => Ok(socket),
        _ => Err(AppError::WsStatus),
    }
}
