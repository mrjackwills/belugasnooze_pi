use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{Error, SqlitePool};
use std::fmt;
use tracing::debug;

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
    fn unwrap_none<A>(query: Result<A, Error>) -> Result<()> {
        match query {
            Ok(_) => Ok(()),
            Err(e) => {
                debug!(%e);
                Ok(())
            }
        }
    }

    pub async fn get_all(db: &SqlitePool) -> Result<Vec<ModelAlarm>> {
        let sql = "SELECT * FROM alarm";
        let result = sqlx::query_as::<_, ModelAlarm>(sql).fetch_all(db).await?;
        Ok(result)
    }

    pub async fn add(db: &SqlitePool, data: (u8, u8, u8)) -> Result<ModelAlarm> {
        let sql = "INSERT INTO alarm(day, hour, minute) VALUES ($1, $2, $3) RETURNING alarm_id, day, hour, minute";
        let query = sqlx::query_as::<_, ModelAlarm>(sql)
            .bind(data.0)
            .bind(data.1)
            .bind(data.2)
            .fetch_one(db)
            .await?;
        Ok(query)
    }

    pub async fn delete(db: &SqlitePool, id: i64) -> Result<()> {
        let sql = "DELETE FROM alarm WHERE alarm_id = $1";
        let query = sqlx::query(sql).bind(id).fetch_all(db).await;
        Self::unwrap_none(query)
    }

    pub async fn delete_all(db: &SqlitePool) -> Result<()> {
        let sql = "DELETE FROM alarm";
        let query = sqlx::query(sql).fetch_all(db).await;
        Self::unwrap_none(query)
    }
}

// ModelAlarm tests
//
/// cargo watch -q -c -w src/ -x 'test model_alarm -- --test-threads=1 --nocapture'
#[cfg(test)]
mod tests {
    use crate::{sql::init_db, AppEnv};
    use std::{fs, sync::Arc, time::SystemTime};
    use time::UtcOffset;

    use super::*;

    async fn setup_test_db(file_name: &str) -> (Arc<SqlitePool>, AppEnv) {
        let location_sqlite = format!("/ramdrive/test_db_files/{}.db", file_name);
        let na = String::from("na");
        let env = AppEnv {
            trace: false,
            location_ip_address: na.clone(),
            location_log_combined: na.clone(),
            timezone: "America/New_York".to_owned(),
            location_log_error: na.clone(),
            location_sqlite,
            debug: true,
            start_time: SystemTime::now(),
            utc_offset: UtcOffset::from_hms(-5, 0, 0).unwrap(),
            ws_address: na.clone(),
            ws_apikey: na.clone(),
            ws_auth_address: na.clone(),
            ws_password: na,
            sql_threads: 1,
        };
        let db = Arc::new(init_db(&env).await.unwrap());
        (db, env)
    }

    fn cleanup() {
        fs::remove_dir_all("/ramdrive/test_db_files/").unwrap()
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
            "error returned from database: CHECK constraint failed: day >= 0 AND day <= 6"
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
            "error returned from database: CHECK constraint failed: hour >= 0 AND hour <= 23"
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
            "error returned from database: CHECK constraint failed: minute >= 0 AND minute <= 59"
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
        // init_db(&args).await.unwrap();
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
