use std::time::Instant;
use tracing::info;

use crate::sleep;

#[derive(Debug)]
pub struct ConnectionDetails {
    count: usize,
    wait: Wait,
    connection_instant: Option<Instant>,
}

#[derive(Debug)]
enum Wait {
    Short,
    Long,
}

impl Wait {
    const fn as_ms(&self) -> u64 {
        match self {
            Self::Short => 5_000,
            Self::Long => 60_000,
        }
    }
}

impl ConnectionDetails {
    pub const fn new() -> Self {
        Self {
            count: 0,
            wait: Wait::Short,
            connection_instant: None,
        }
    }

    /// increase attempt count, and set delay to long if 20+ attempts
    /// Set is_connected to 0 and time to none
    pub fn fail_connect(&mut self) {
        self.count += 1;
        if self.count >= 20 {
            self.wait = Wait::Long;
        }
    }

    /// delay the recconnect attempt by x seconds, depedning on ho wmany attempts already made
    pub async fn reconnect_delay(&self) {
        info!(self.count);
        if self.count > 0 {
            sleep!(self.wait.as_ms());
        }
    }

    /// called on each connect, to reset connection, count etc
    pub fn valid_connect(&mut self) {
        self.wait = Wait::Short;
        self.count = 0;
        self.connection_instant = Some(Instant::now());
        tracing::debug!(
            "{}",
            jiff::Zoned::now().timestamp().strftime("%Y-%m-%d %H:%M:%S")
        );
    }

    pub fn get_connect_instant(&self) -> Instant {
        self.connection_instant.unwrap_or_else(Instant::now)
    }
}
