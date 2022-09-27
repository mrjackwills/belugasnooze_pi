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
    response: String,
}

/// Make a http request to get an access token
async fn get_auth_token(app_envs: &AppEnv) -> Result<String, AppError> {
    let new_post = PostRequest {
        key: &app_envs.ws_apikey,
        password: &app_envs.ws_password,
    };
    let request: PostResponse = reqwest::Client::new()
        .post(&app_envs.ws_token_address)
        .json(&new_post)
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

// ws connect tests
//
// cargo watch -q -c -w src/ -x 'test ws_connect -- --test-threads=1 --nocapture'
// #[cfg(test)]
// #[allow(clippy::unwrap_used)]
// mod tests {
//     use super::*;

//     const VALID_ADDRESS: &str = "wss://some.domain.com";
//     const INVALID_ADDRESS: &str = "ws://some.domain.com";

//     // fn get_header_key<'a>(request: &'a Request, key: &str) -> &'a str {
//     //     request.headers().get(key).unwrap().to_str().unwrap()
//     // }

//     // #[test]
//     // fn ws_connect_extract_host_fail() {
//     //     let result = extract_host("invalid_address");
//     //     assert!(result.is_err());
//     //     assert_eq!(
//     //         result.unwrap_err().to_string(),
//     //         "unable to extract host from address: invalid_address"
//     //     );

//     //     let result = extract_host(INVALID_ADDRESS);
//     //     assert!(result.is_err());
//     //     assert_eq!(
//     //         result.unwrap_err().to_string(),
//     //         format!("unable to extract host from address: {}", INVALID_ADDRESS)
//     //     );
//     // }

//     // #[test]
//     // fn ws_connect_extract_host_ok() {
//     //     let result = extract_host(VALID_ADDRESS);
//     //     assert!(result.is_ok());
//     //     assert_eq!(result.unwrap().to_string(), "some.domain.com");
//     // }

//     // #[test]
//     // fn ws_connect_build_request_ok() {
//     //     let result = build_request(VALID_ADDRESS, "some_api_key", "auth_token");
//     //     assert!(result.is_ok());

//     //     let result = result.unwrap();
//     //     assert!(result.headers().contains_key("Connection"));
//     //     assert!(result.headers().contains_key("host"));
//     //     assert!(result.headers().contains_key("sec-websocket-key"));
//     //     assert!(result.headers().contains_key("sec-websocket-protocol"));
//     //     assert!(result.headers().contains_key("sec-websocket-version"));
//     //     assert!(result.headers().contains_key("upgrade"));
//     //     assert_eq!(get_header_key(&result, "Connection"), "Upgrade");
//     //     assert_eq!(get_header_key(&result, "host"), "some.domain.com");
//     //     assert_eq!(
//     //         get_header_key(&result, "sec-websocket-protocol"),
//     //         "some_api_key"
//     //     );
//     //     assert_eq!(get_header_key(&result, "sec-websocket-version"), "13");
//     //     assert_eq!(get_header_key(&result, "upgrade"), "websocket");
//     // }

//     // #[test]
//     // fn ws_connect_build_request_fail() {
//     //     let result = build_request(INVALID_ADDRESS, "some_api_key", "auth_token");
//     //     assert!(result.is_err());
//     //     assert_eq!(
//     //         result.unwrap_err().to_string(),
//     //         format!("unable to extract host from address: {}", INVALID_ADDRESS)
//     //     );
//     // }
// }
