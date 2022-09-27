use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::fmt;
use time::UtcOffset;

use crate::{app_error::AppError, env::AppEnv};

#[derive(sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct ModelTimezone {
    pub timezone_id: i64,
    pub zone_name: String,
    pub offset_hour: i8,
    pub offset_minute: i8,
    pub offset_second: i8,
}

impl fmt::Display for ModelTimezone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "timezone_id: {}, zone_name: {}, offset_hour: {}, offset_minute: {}, offset_second: {}",
            self.timezone_id,
            self.zone_name,
            self.offset_hour,
            self.offset_minute,
            self.offset_second
        )
    }
}

impl Default for ModelTimezone {
    fn default() -> Self {
        Self {
            timezone_id: 1,
            zone_name: String::from("Etc/UTC"),
            offset_hour: 0,
            offset_minute: 0,
            offset_second: 0,
        }
    }
}

impl ModelTimezone {
    pub async fn get(db: &SqlitePool) -> Option<Self> {
        let sql = "SELECT * FROM timezone";
        let result = sqlx::query_as::<_, Self>(sql).fetch_one(db).await;
        match result {
            Ok(data) => Some(data),
            Err(_) => None,
        }
    }

    pub async fn insert(db: &SqlitePool, app_envs: &AppEnv) -> Result<Self, AppError> {
        let sql = "INSERT INTO timezone (zone_name, offset_hour, offset_minute, offset_second) VALUES($1, $2, $3, $4) RETURNING timezone_id, zone_name, offset_hour, offset_minute, offset_second";
        let query = sqlx::query_as::<_, Self>(sql)
            .bind(&app_envs.timezone)
            .bind(app_envs.utc_offset.whole_hours())
            .bind(app_envs.utc_offset.minutes_past_hour())
            .bind(app_envs.utc_offset.seconds_past_minute())
            .fetch_one(db)
            .await?;
        Ok(query)
    }

    pub async fn update(
        db: &SqlitePool,
        zone_name: &str,
        offset: UtcOffset,
    ) -> Result<Self, AppError> {
        let sql = "UPDATE timezone SET zone_name = $1, offset_hour = $2, offset_minute = $3, offset_second = $4 RETURNING timezone_id, zone_name, offset_hour, offset_minute, offset_second";
        let query = sqlx::query_as::<_, Self>(sql)
            .bind(zone_name)
            .bind(offset.whole_hours())
            .bind(offset.minutes_past_hour())
            .bind(offset.seconds_past_minute())
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
    use crate::db::{create_tables, file_exists, get_db, init_db};
    use std::{fs, sync::Arc, time::SystemTime};
    use time::UtcOffset;

    use super::*;

    async fn setup_test_db(file_name: &str) -> (Arc<SqlitePool>, AppEnv) {
        let na = String::from("na");
        let location_sqlite = format!("/dev/shm/test_db_files/{}.db", file_name);
        let env = AppEnv {
            debug: true,
            location_ip_address: na.clone(),
            location_sqlite,
            sql_threads: 1,
            start_time: SystemTime::now(),
            timezone: "America/New_York".to_owned(),
            trace: false,
            utc_offset: UtcOffset::from_hms(-5, 0, 0).unwrap(),
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
            debug: true,
            location_ip_address: na.clone(),
            location_sqlite,
            sql_threads: 1,
            start_time: SystemTime::now(),
            timezone: "Europe/Berlin".to_owned(),
            trace: false,
            utc_offset: UtcOffset::from_hms(1, 0, 0).unwrap(),
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
            debug: true,
            location_ip_address: na.clone(),
            location_sqlite,
            sql_threads: 1,
            start_time: SystemTime::now(),
            timezone: "Europe/Berlin".to_owned(),
            trace: false,
            utc_offset: UtcOffset::from_hms(1, 0, 0).unwrap(),
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
        assert_eq!(result_timezone.offset_hour, 1);
        assert_eq!(result_timezone.offset_minute, 0);
        assert_eq!(result_timezone.offset_second, 0);
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
        assert_eq!(result.offset_hour, -5);
        assert_eq!(result.offset_minute, 0);
        assert_eq!(result.offset_second, 0);
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
        let result = ModelTimezone::update(&fixtures.0, data.0, data.1).await;

        // CHECK
        assert_eq!(pre_update.timezone_id, 1);
        assert_eq!(pre_update.offset_hour, -5);
        assert_eq!(pre_update.offset_minute, 0);
        assert_eq!(pre_update.offset_second, 0);
        assert_eq!(pre_update.zone_name, "America/New_York");

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.offset_hour, 1);
        assert_eq!(result.offset_minute, 0);
        assert_eq!(result.offset_second, 0);
        assert_eq!(result.timezone_id, 1);
        assert_eq!(result.zone_name, "Europe/Berlin");
        cleanup();
    }
}