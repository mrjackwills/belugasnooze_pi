use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;

use crate::{db::ModelAlarm, sysinfo::SysInfo};

/// Basic pi info
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PiStatus {
    pub alarms: Vec<ModelAlarm>,
    pub internal_ip: String,
    pub time_zone: String,
    pub uptime_app: u64,
    pub connected_for: u64,
    pub uptime: usize,
    pub version: String,
}
/// Combined pi into and current set alarms
impl PiStatus {
    pub fn new(sysinfo: SysInfo, alarms: Vec<ModelAlarm>, connected_for: u64) -> Self {
        Self {
            alarms,
            internal_ip: sysinfo.internal_ip,
            time_zone: sysinfo.time_zone,
            uptime_app: sysinfo.uptime_app,
            uptime: sysinfo.uptime,
            connected_for,
            version: sysinfo.version,
        }
    }
}
/// Responses, either sent as is, or nested in StructuredResponse below
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case", tag = "name", content = "data")]
pub enum Response {
    Status(PiStatus),
    LedStatus { status: bool },
}

/// These get sent to the websocket server when in structured_data mode,
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct StructuredResponse {
    data: Option<Response>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Response>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cache: Option<bool>,
}

impl StructuredResponse {
    /// Convert a ResponseMessage into a Tokio message of StructureResponse
    pub fn data(data: Response, cache: Option<bool>) -> Message {
        let x = Self {
            data: Some(data),
            error: None,
            cache,
        };
        Message::Text(serde_json::to_string(&x).unwrap_or_default().into())
    }

    /// Convert a ErrorResponse into a Tokio message of StructureResponse
    pub fn _error(data: Response) -> Message {
        let x = Self {
            error: Some(data),
            data: None,
            cache: None,
        };
        Message::Text(serde_json::to_string(&x).unwrap_or_default().into())
    }
}
