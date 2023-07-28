use serde::Deserialize;
use sqlx::SqlitePool;
use std::fmt;
use time::UtcOffset;
use time_tz::{timezones, Offset, TimeZone};

use crate::{app_env::AppEnv, app_error::AppError};

#[derive(sqlx::FromRow, Debug, Clone, Deserialize)]
pub struct ModelTimezone {
    pub timezone_id: i64,
    pub zone_name: String,
}

impl fmt::Display for ModelTimezone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "timezone_id: {}, zone_name: {}",
            self.timezone_id, self.zone_name,
        )
    }
}

impl Default for ModelTimezone {
    fn default() -> Self {
        Self {
            timezone_id: 1,
            zone_name: String::from("Etc/UTC"),
        }
    }
}

impl ModelTimezone {
    pub fn get_offset(&self) -> UtcOffset {
        timezones::get_by_name(&self.zone_name).map_or(UtcOffset::UTC, |tz| {
            tz.get_offset_utc(&time::OffsetDateTime::now_utc()).to_utc()
        })
    }

    pub async fn get(db: &SqlitePool) -> Option<Self> {
        let sql = "SELECT * FROM timezone";
        let result = sqlx::query_as::<_, Self>(sql).fetch_one(db).await;
        result.ok()
    }

    pub async fn insert(db: &SqlitePool, app_envs: &AppEnv) -> Result<Self, AppError> {
        let sql = "INSERT INTO timezone (zone_name) VALUES($1) RETURNING timezone_id, zone_name";
        let query = sqlx::query_as::<_, Self>(sql)
            .bind(&app_envs.timezone.to_string())
            .fetch_one(db)
            .await?;
        Ok(query)
    }

    pub async fn update(db: &SqlitePool, zone_name: &str) -> Result<Self, AppError> {
        let sql = "UPDATE timezone SET zone_name = $1 RETURNING timezone_id, zone_name";
        let query = sqlx::query_as::<_, Self>(sql)
            .bind(zone_name)
            .fetch_one(db)
            .await?;
        Ok(query)
    }
}

/// ModelTimezone tests
///
/// cargo watch -q -c -w src/ -x 'test model_timezone -- --test-threads=1 --nocapture'
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::{
        app_env::EnvTimeZone,
        db::{create_tables, file_exists, get_db, init_db},
    };
    use std::{fs, sync::Arc, time::SystemTime};
    use time::UtcOffset;

    use super::*;

    async fn setup_test_db(file_name: &str) -> (Arc<SqlitePool>, AppEnv) {
        let na = String::from("na");
        let location_sqlite = format!("/dev/shm/test_db_files/{file_name}.db");
        let env = AppEnv {
            location_ip_address: na.clone(),
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
    async fn model_timezone_get_empty_with_init() {
        // FIXTURES
        let na = String::from("na");
        let location_sqlite = String::from("/dev/shm/test_db_files/model_timezone_insert_ok.db");
        let app_envs = AppEnv {
            location_ip_address: na.clone(),
            location_sqlite,
            log_level: tracing::Level::INFO,
            sql_threads: 1,
            start_time: SystemTime::now(),
            timezone: EnvTimeZone::new("Europe/Berlin"),
            ws_address: na.clone(),
            ws_apikey: na.clone(),
            ws_password: na.clone(),
            ws_token_address: na,
        };

        file_exists(&app_envs.location_sqlite);
        let db = get_db(&app_envs).await.unwrap();
        create_tables(&db).await;

        // ACTIONS
        let result = ModelTimezone::get(&db).await;

        // CHECK
        assert!(result.is_none());
        cleanup();
    }

    #[tokio::test]
    async fn model_timezone_insert_ok() {
        // FIXTURES
        let na = String::from("na");
        let location_sqlite = String::from("/dev/shm/test_db_files/model_timezone_insert_ok.db");
        let app_envs = AppEnv {
            location_ip_address: na.clone(),
            location_sqlite,
            log_level: tracing::Level::INFO,
            sql_threads: 1,
            start_time: SystemTime::now(),
            timezone: EnvTimeZone::new("Europe/Berlin"),
            ws_address: na.clone(),
            ws_apikey: na.clone(),
            ws_password: na.clone(),
            ws_token_address: na,
        };

        file_exists(&app_envs.location_sqlite);
        let db = get_db(&app_envs).await.unwrap();
        create_tables(&db).await;

        // ACTIONS
        let result = ModelTimezone::insert(&db, &app_envs).await;

        // CHECK
        assert!(result.is_ok());
        let result_timezone = ModelTimezone::get(&db).await.unwrap();
        assert_eq!(result_timezone.zone_name, "Europe/Berlin");
        cleanup();
    }

    #[tokio::test]
    async fn model_timezone_get_ok_with_init() {
        // FIXTURES
        let fixtures = setup_test_db("model_timezone_get_ok_with_init").await;

        // ACTIONS
        let result = ModelTimezone::get(&fixtures.0).await;

        // CHECK
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.zone_name, "America/New_York");
        cleanup();
    }

    #[tokio::test]
    async fn model_timezone_update_ok() {
        // FIXTURES
        let fixtures = setup_test_db("model_timezone_update_ok").await;
        let data = ("Europe/Berlin", UtcOffset::from_hms(1, 0, 0).unwrap());
        let pre_update = ModelTimezone::get(&fixtures.0).await.unwrap();

        // ACTIONS
        let result = ModelTimezone::update(&fixtures.0, data.0).await;

        // CHECK
        assert_eq!(pre_update.timezone_id, 1);
        assert_eq!(pre_update.zone_name, "America/New_York");

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.timezone_id, 1);
        assert_eq!(result.zone_name, "Europe/Berlin");
        cleanup();
    }
}
