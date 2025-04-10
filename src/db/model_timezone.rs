use jiff::{Zoned, civil::Time};
use serde::Deserialize;
use sqlx::SqlitePool;
use std::fmt;
// use time::{OffsetDateTime, Time, UtcOffset};
// use time_tz::{timezones, Offset, TimeZone};

use crate::{S, app_env::AppEnv, app_error::AppError};

#[derive(sqlx::FromRow, Debug, Clone, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
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
            zone_name: S!("Etc/UTC"),
        }
    }
}

impl ModelTimezone {
    // Get the current time as OffsetDateTime with the ModelTimezone zone accounted for
    pub fn now_with_offset(&self) -> jiff::Zoned {
        jiff::Timestamp::now()
            .in_tz(&self.zone_name)
            .unwrap_or_else(|_| Zoned::now())
    }

    /// Get the current timezone in HMS
    pub fn to_time(&self) -> Time {
        self.now_with_offset().time()
    }

    pub async fn get(db: &SqlitePool) -> Option<Self> {
        let sql = "SELECT * FROM timezone";
        let result = sqlx::query_as::<_, Self>(sql).fetch_one(db).await;
        result.ok()
    }

    pub async fn insert(db: &SqlitePool, app_envs: &AppEnv) -> Result<Self, AppError> {
        let sql = "INSERT INTO timezone (zone_name) VALUES($1) RETURNING timezone_id, zone_name";
        let query = sqlx::query_as::<_, Self>(sql)
            .bind(app_envs.timezone.iana_name())
            .fetch_one(db)
            .await?;
        Ok(query)
    }

    pub async fn update(db: &SqlitePool, zone_name: &jiff::tz::TimeZone) -> Result<Self, AppError> {
        let sql = "UPDATE timezone SET zone_name = $1 RETURNING timezone_id, zone_name";
        let query = sqlx::query_as::<_, Self>(sql)
            .bind(zone_name.iana_name())
            .fetch_one(db)
            .await?;
        Ok(query)
    }
}

/// ModelTimezone tests
///
/// cargo watch -q -c -w src/ -x 'test model_timezone -- --test-threads=1 --nocapture'
#[cfg(test)]
#[expect(clippy::unwrap_used)]
mod tests {
    use crate::{
        db::{create_tables, file_exists, get_db},
        tests::{gen_app_envs, test_cleanup, test_setup},
    };
    use uuid::Uuid;

    use super::*;

    #[tokio::test]
    async fn model_timezone_get_empty_with_init() {
        let uuid = Uuid::new_v4();

        let app_envs = gen_app_envs(uuid);

        file_exists(&app_envs.location_sqlite);
        let db = get_db(&app_envs).await.unwrap();
        create_tables(&db).await;

        let result = ModelTimezone::get(&db).await;

        assert!(result.is_none());
        test_cleanup(uuid, Some(db)).await;
    }

    #[tokio::test]
    async fn model_timezone_insert_ok() {
        let uuid = Uuid::new_v4();

        let mut app_envs = gen_app_envs(uuid);
        app_envs.timezone = jiff::tz::TimeZone::get("America/New_York").unwrap();

        file_exists(&app_envs.location_sqlite);
        let db = get_db(&app_envs).await.unwrap();
        create_tables(&db).await;

        let result: Result<ModelTimezone, AppError> = ModelTimezone::insert(&db, &app_envs).await;

        assert!(result.is_ok());
        let result_timezone = ModelTimezone::get(&db).await.unwrap();
        assert_eq!(result_timezone.zone_name, "America/New_York");
        test_cleanup(uuid, Some(db)).await;
    }

    #[tokio::test]
    async fn model_timezone_get_ok_with_init() {
        let (_, db, uuid) = test_setup().await;
        let result = ModelTimezone::get(&db).await;

        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.zone_name, "Europe/London");
        test_cleanup(uuid, Some(db)).await;
    }

    #[tokio::test]
    async fn model_timezone_update_ok() {
        let (_, db, uuid) = test_setup().await;
        let result = ModelTimezone::get(&db).await.unwrap();
        assert_eq!(result.zone_name, "Europe/London");

        let result =
            ModelTimezone::update(&db, &jiff::tz::TimeZone::get("America/New_York").unwrap()).await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.timezone_id, 1);
        assert_eq!(result.zone_name, "America/New_York");
        test_cleanup(uuid, Some(db)).await;
    }
}
