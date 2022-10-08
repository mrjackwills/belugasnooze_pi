use super::WsStream;
use crate::{app_error::AppError, env::AppEnv};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{self, connect_async, tungstenite::http::StatusCode};

#[derive(Debug, Serialize, Deserialize)]
struct PostRequest<'a> {
    key: &'a str,
    password: &'a str,
}

#[derive(Debug, Serialize, Deserialize)]
struct PostResponse {
    // This is an UUID, but probably no need to parse it as such
    response: String,
}

/// Make a https request to get an access token
async fn get_auth_token(app_envs: &AppEnv) -> Result<String, AppError> {
    let request_body = PostRequest {
        key: &app_envs.ws_apikey,
        password: &app_envs.ws_password,
    };
    let request: PostResponse = reqwest::Client::new()
        .post(&app_envs.ws_token_address)
        .json(&request_body)
        .send()
        .await?
        .json()
        .await?;
    Ok(request.response)
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
