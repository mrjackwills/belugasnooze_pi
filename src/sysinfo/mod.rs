use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tokio::fs::read_to_string;

use crate::{db::ModelTimezone, env::AppEnv};

#[derive(Debug, Serialize, Deserialize)]
pub struct SysInfo {
    pub uptime: usize,
    pub version: String,
    pub internal_ip: String,
    pub uptime_app: u64,
    pub time_zone: String,
}

impl SysInfo {
    async fn get_ip(app_envs: &AppEnv) -> String {
        let na = || String::from("N/A");
        let ip = read_to_string(&app_envs.location_ip_address)
            .await
            .unwrap_or_else(|_| na());
        let output = if ip.len() > 1 {
            ip.trim().to_owned()
        } else {
            na()
        };
        output
    }

    async fn get_uptime() -> usize {
        let uptime = read_to_string("/proc/uptime").await.unwrap_or_default();
        let (uptime, _) = uptime.split_once('.').unwrap_or_default();
        uptime.parse::<usize>().unwrap_or(0)
    }

    pub async fn new(db: &SqlitePool, app_envs: &AppEnv) -> Self {
        let model_timezone = ModelTimezone::get(db).await.unwrap_or_default();
        Self {
            internal_ip: Self::get_ip(app_envs).await,
            uptime: Self::get_uptime().await,
            uptime_app: std::time::SystemTime::now()
                .duration_since(app_envs.start_time)
                .map_or(0, |value| value.as_secs()),
            time_zone: model_timezone.zone_name,
            version: env!("CARGO_PKG_VERSION").into(),
        }
    }
}

// SysInfo tests
//
/// cargo watch -q -c -w src/ -x 'test sysinfo -- --test-threads=1 --nocapture'
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::{db::init_db, env::EnvTimeZone};
    use std::{fs, sync::Arc, time::SystemTime};

    use super::*;

    async fn setup_test_db(
        file_name: &str,
        location_ip_address: String,
    ) -> (Arc<SqlitePool>, AppEnv) {
        let location_sqlite = format!("/dev/shm/test_db_files/{file_name}.db");
        let na = String::from("na");
        let env = AppEnv {
            location_ip_address,
            location_sqlite,
			log_level: tracing::Level::INFO,
            sql_threads: 1,
            start_time: SystemTime::now(),
            timezone: EnvTimeZone::new("America/New_York"),
            ws_address: na.clone(),
            ws_apikey: na.clone(),
            ws_password: na.clone(),
            ws_token_address: na,
        };
        let db = Arc::new(init_db(&env).await.unwrap());
        (db, env)
    }

    fn cleanup() {
        fs::remove_dir_all("/dev/shm/test_db_files/").unwrap();
    }

    #[tokio::test]
    async fn sysinfo_getuptime_ok() {
        // FIXTURES
        setup_test_db("sysinfo_getuptime_ok", String::new()).await;

        // ACTIONS
        let result = SysInfo::get_uptime().await;

        // CHECK
        // Assumes ones computer has been turned on for one minute
        assert!(result > 60);
        cleanup();
    }

    #[tokio::test]
    async fn sysinfo_get_ip_na() {
        // FIXTURES
        let app_envs = setup_test_db("sysinfo_get_ip_na", String::new()).await;

        // ACTIONS
        let result = SysInfo::get_ip(&app_envs.1).await;

        // CHECK
        assert_eq!(result, "N/A");
        cleanup();
    }

    #[tokio::test]
    async fn sysinfo_get_ip_ok() {
        // FIXTURES
        let app_envs = setup_test_db("sysinfo_get_ip_ok", "./ip.addr".to_owned()).await;

        // ACTIONS
        let result = SysInfo::get_ip(&app_envs.1).await;

        // CHECK
        assert_eq!(result, "127.0.0.1");
        cleanup();
    }

    #[tokio::test]
    async fn sysinfo_get_sysinfo_ok() {
        // FIXTURES
        let app_envs = setup_test_db("sysinfo_get_sysinfo_ok", "./ip.addr".to_owned()).await;
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        // ACTIONS
        let result = SysInfo::new(&app_envs.0, &app_envs.1).await;

        // CHECK
        assert_eq!(result.internal_ip, "127.0.0.1");
        assert_eq!(result.time_zone, "America/New_York");
        assert_eq!(result.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(result.uptime_app, 1);
        // TODO need to check pi_time with regex?
        // assert!(result.pi_time.len() == 8);
        // Again assume ones computer has been turned on for one minute
        assert!(result.uptime > 60);
        cleanup();
    }
}
