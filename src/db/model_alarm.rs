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
    use crate::tests::{test_cleanup, test_setup};

    use super::*;

    #[tokio::test]
    async fn model_alarm_add_ok() {
        let (_app_env, db, uuid) = test_setup().await;
        let data = (1, 10, 10);

        let result = ModelAlarm::add(&db, data).await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.alarm_id, 1);
        assert_eq!(result.day, 1);
        assert_eq!(result.hour, 10);
        assert_eq!(result.minute, 10);

        test_cleanup(uuid, Some(db)).await;
    }

    #[tokio::test]
    async fn model_alarm_add_err_invalid_day() {
        let (_app_env, db, uuid) = test_setup().await;
        let data = (10, 10, 10);

        let result = ModelAlarm::add(&db, data).await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Internal Database Error: error returned from database: (code: 275) CHECK constraint failed: day >= 0\n\t\tAND day <= 6"
        );
        test_cleanup(uuid, Some(db)).await;
    }

    #[tokio::test]
    async fn model_alarm_add_err_invalid_hour() {
        let (_app_env, db, uuid) = test_setup().await;
        let data = (1, 25, 10);

        let result = ModelAlarm::add(&db, data).await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Internal Database Error: error returned from database: (code: 275) CHECK constraint failed: hour >= 0\n\t\tAND hour <= 23"
        );
        test_cleanup(uuid, Some(db)).await;
    }

    #[tokio::test]
    async fn model_alarm_add_err_invalid_minute() {
        let (_app_env, db, uuid) = test_setup().await;
        let data = (1, 10, 60);

        let result = ModelAlarm::add(&db, data).await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Internal Database Error: error returned from database: (code: 275) CHECK constraint failed: minute >= 0\n\t\tAND minute <= 59"
        );
        test_cleanup(uuid, Some(db)).await;
    }

    #[tokio::test]
    async fn model_alarm_get_all_ok() {
        let (_app_env, db, uuid) = test_setup().await;
        for i in 0..6 {
            let data = (i, i, i);
            ModelAlarm::add(&db, data).await.unwrap();
        }

        let result = ModelAlarm::get_all(&db).await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.len(), 6);
        assert_eq!(result[0].day, 0);
        assert_eq!(result[1].hour, 1);
        assert_eq!(result[2].minute, 2);
        test_cleanup(uuid, Some(db)).await;
    }

    #[tokio::test]
    async fn model_alarm_delete_one_ok() {
        let (_app_env, db, uuid) = test_setup().await;
        let data = (1, 10, 10);
        let alarm = ModelAlarm::add(&db, data).await.unwrap();

        let result = ModelAlarm::delete(&db, alarm.alarm_id).await;
        let alarm = ModelAlarm::get_all(&db).await.unwrap();

        assert!(result.is_ok());
        assert!(alarm.is_empty());
        test_cleanup(uuid, Some(db)).await;
    }

    #[tokio::test]
    async fn model_alarm_multiple_delete_one_ok() {
        let (_app_env, db, uuid) = test_setup().await;
        for i in 0..6 {
            let data = (i, i, i);
            ModelAlarm::add(&db, data).await.unwrap();
        }

        let result = ModelAlarm::delete(&db, 1).await;
        let alarm = ModelAlarm::get_all(&db).await.unwrap();

        assert!(result.is_ok());
        assert_eq!(alarm.len(), 5);
        test_cleanup(uuid, Some(db)).await;
    }

    #[tokio::test]
    async fn model_alarm_delete_all_ok() {
        let (_app_env, db, uuid) = test_setup().await;
        for i in 0..6 {
            let data = (i, i, i);
            ModelAlarm::add(&db, data).await.unwrap();
        }

        let result = ModelAlarm::delete_all(&db).await;
        let alarm = ModelAlarm::get_all(&db).await.unwrap();

        assert!(result.is_ok());
        assert!(alarm.is_empty());
        test_cleanup(uuid, Some(db)).await;
    }

    #[tokio::test]
    async fn model_alarm_delete_err() {
        let (_app_env, db, uuid) = test_setup().await;
        let data = (1, 10, 10);
        ModelAlarm::add(&db, data).await.unwrap();

        let result = ModelAlarm::delete(&db, 2).await;
        let alarm = ModelAlarm::get_all(&db).await.unwrap();

        assert!(result.is_ok());
        assert_eq!(alarm.len(), 1);
        test_cleanup(uuid, Some(db)).await;
    }
}
