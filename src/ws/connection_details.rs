use std::time::{Duration, Instant};
use time::OffsetDateTime;
use tokio::time::sleep;
use tracing::{debug, info};

#[derive(Debug)]
pub struct ConnectionDetails {
    count: usize,
    wait: Wait,
    connection_instant: Option<Instant>,
    is_connected: bool,
}

#[derive(Debug)]
enum Wait {
    Short,
    Long,
}

impl Wait {
    fn as_sec(&self) -> u64 {
        match self {
            Self::Short => 5,
            Self::Long => 60,
        }
    }
}

impl ConnectionDetails {
    pub fn new() -> Self {
        Self {
            count: 0,
            wait: Wait::Short,
            is_connected: false,
            connection_instant: None,
        }
    }

    /// increase attempt count, and set delay to long if 20+ attempts
    /// Set is_connected to 0 and time to none
    pub fn fail_connect(&mut self) {
        self.count += 1;
        self.is_connected = false;
        if self.count >= 20 {
            self.wait = Wait::Long;
        }
    }

    /// delay the recconnect attempt by x seconds, depedning on ho wmany attempts already made
    pub async fn reconnect_delay(&self) {
        info!(self.count);
        if self.count > 0 {
            sleep(Duration::from_secs(self.wait.as_sec())).await
        }
    }

    /// called on each connect, to reset connection, count etc
    pub fn valid_connect(&mut self) {
        self.wait = Wait::Short;
        self.count = 1;
        self.is_connected = true;
        self.connection_instant = Some(Instant::now());
        let now = OffsetDateTime::now_utc();
        let connected_at = format!("{} {}", now.date(), now.time());
        debug!(%connected_at);
    }

    pub fn get_connect_instant(&self) -> Instant {
        self.connection_instant.unwrap_or_else(Instant::now)
    }
}
