use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::fmt;

use crate::app_error::AppError;

#[derive(sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct ModelAlarm {
    pub alarm_id: i64,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
}

impl fmt::Display for ModelAlarm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "alarm_id: {}, day:{}, hour:{}, minute:{}",
            self.alarm_id, self.day, self.hour, self.minute
        )
    }
}

impl ModelAlarm {
    pub async fn get_all(db: &SqlitePool) -> Result<Vec<Self>, AppError> {
        let sql = "SELECT * FROM alarm";
        let result = sqlx::query_as::<_, Self>(sql).fetch_all(db).await?;
        Ok(result)
    }

    pub async fn add(db: &SqlitePool, data: (u8, u8, u8)) -> Result<Self, AppError> {
        let sql = "INSERT INTO alarm(day, hour, minute) VALUES ($1, $2, $3) RETURNING alarm_id, day, hour, minute";
        let query = sqlx::query_as::<_, Self>(sql)
            .bind(data.0)
            .bind(data.1)
            .bind(data.2)
            .fetch_one(db)
            .await?;
        Ok(query)
    }

    pub async fn delete(db: &SqlitePool, id: i64) -> Result<(), AppError> {
        let sql = "DELETE FROM alarm WHERE alarm_id = $1";
        sqlx::query(sql).bind(id).execute(db).await?;
        Ok(())
    }

    pub async fn delete_all(db: &SqlitePool) -> Result<(), AppError> {
        let sql = "DELETE FROM alarm";
        sqlx::query(sql).execute(db).await?;
        Ok(())
    }
}

// ModelAlarm tests
//
/// cargo watch -q -c -w src/ -x 'test model_alarm -- --test-threads=1 --nocapture'
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::{app_env::EnvTimeZone, db::init_db, AppEnv};
    use std::{fs, sync::Arc, time::SystemTime};

    use super::*;

    async fn setup_test_db(file_name: &str) -> (Arc<SqlitePool>, AppEnv) {
        let location_sqlite = format!("/dev/shm/test_db_files/{file_name}.db");
        let na = String::from("na");
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
    async fn model_alarm_add_ok() {
        // FIXTURES
        let fixtures = setup_test_db("model_alarm_add_ok").await;
        let data = (1, 10, 10);

        // ACTIONS
        let result = ModelAlarm::add(&fixtures.0, data).await;

        // CHECK
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.alarm_id, 1);
        assert_eq!(result.day, 1);
        assert_eq!(result.hour, 10);
        assert_eq!(result.minute, 10);
        cleanup();
    }

    #[tokio::test]
    async fn model_alarm_add_err_invalid_day() {
        // FIXTURES
        let fixtures = setup_test_db("model_alarm_add_err_invalid_day").await;
        let data = (10, 10, 10);

        // ACTIONS
        let result = ModelAlarm::add(&fixtures.0, data).await;

        // CHECK
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Internal Database Error: error returned from database: (code: 275) CHECK constraint failed: day >= 0 AND day <= 6"
        );
        cleanup();
    }

    #[tokio::test]
    async fn model_alarm_add_err_invalid_hour() {
        // FIXTURES
        let fixtures = setup_test_db("model_alarm_add_err_invalid_hour").await;
        let data = (1, 25, 10);

        // ACTIONS
        let result = ModelAlarm::add(&fixtures.0, data).await;

        // CHECK
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Internal Database Error: error returned from database: (code: 275) CHECK constraint failed: hour >= 0 AND hour <= 23"
        );
        cleanup();
    }

    #[tokio::test]
    async fn model_alarm_add_err_invalid_minute() {
        // FIXTURES
        let fixtures = setup_test_db("model_alarm_add_err_invalid_minute").await;
        let data = (1, 10, 60);

        // ACTIONS
        let result = ModelAlarm::add(&fixtures.0, data).await;

        // CHECK
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Internal Database Error: error returned from database: (code: 275) CHECK constraint failed: minute >= 0 AND minute <= 59"
        );
        cleanup();
    }

    #[tokio::test]
    async fn model_alarm_get_all_ok() {
        // FIXTURES
        let fixtures = setup_test_db("model_alarm_get_all_ok").await;
        for i in 0..6 {
            let data = (i, i, i);
            ModelAlarm::add(&fixtures.0, data).await.unwrap();
        }

        // ACTIONS
        let result = ModelAlarm::get_all(&fixtures.0).await;

        // CHECK
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.len(), 6);
        assert_eq!(result[0].day, 0);
        assert_eq!(result[1].hour, 1);
        assert_eq!(result[2].minute, 2);
        cleanup();
    }

    #[tokio::test]
    async fn model_alarm_delete_one_ok() {
        // FIXTURES
        let fixtures = setup_test_db("model_alarm_delete_one_ok").await;
        let data = (1, 10, 10);
        let alarm = ModelAlarm::add(&fixtures.0, data).await.unwrap();

        // ACTIONS
        let result = ModelAlarm::delete(&fixtures.0, alarm.alarm_id).await;
        let alarm = ModelAlarm::get_all(&fixtures.0).await.unwrap();

        // CHECK
        assert!(result.is_ok());
        assert!(alarm.is_empty());
        cleanup();
    }

    #[tokio::test]
    async fn model_alarm_multiple_delete_one_ok() {
        // FIXTURES
        let fixtures = setup_test_db("model_alarm_multiple_delete_one_ok").await;
        for i in 0..6 {
            let data = (i, i, i);
            ModelAlarm::add(&fixtures.0, data).await.unwrap();
        }

        // ACTIONS
        let result = ModelAlarm::delete(&fixtures.0, 1).await;
        let alarm = ModelAlarm::get_all(&fixtures.0).await.unwrap();

        // CHECK
        assert!(result.is_ok());
        assert_eq!(alarm.len(), 5);
        cleanup();
    }

    #[tokio::test]
    async fn model_alarm_delete_all_ok() {
        // FIXTURES
        let fixtures = setup_test_db("model_alarm_delete_all_ok").await;
        for i in 0..6 {
            let data = (i, i, i);
            ModelAlarm::add(&fixtures.0, data).await.unwrap();
        }

        // ACTIONS
        let result = ModelAlarm::delete_all(&fixtures.0).await;
        let alarm = ModelAlarm::get_all(&fixtures.0).await.unwrap();

        // CHECK
        assert!(result.is_ok());
        assert!(alarm.is_empty());
        cleanup();
    }

    #[tokio::test]
    async fn model_alarm_delete_err() {
        // FIXTURES
        let fixtures = setup_test_db("model_alarm_delete_err").await;
        let data = (1, 10, 10);
        ModelAlarm::add(&fixtures.0, data).await.unwrap();

        // ACTIONS
        let result = ModelAlarm::delete(&fixtures.0, 2).await;
        let alarm = ModelAlarm::get_all(&fixtures.0).await.unwrap();

        // CHECK
        assert!(result.is_ok());
        assert_eq!(alarm.len(), 1);
        cleanup();
    }
}
