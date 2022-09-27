use dotenvy::dotenv;
use std::{collections::HashMap, env, time::SystemTime};
use thiserror::Error;
use time::UtcOffset;
use time_tz::{timezones, Offset, TimeZone};
use tokio::fs;
use tracing::error;

type EnvHashMap = HashMap<String, String>;

#[derive(Debug, Error)]
enum EnvError {
    #[error("missing env: '{0}'")]
    NotFound(String),
    #[error("'{0}' - sql file should end '.db'")]
    DbNameInvalid(String),
    #[error("'{0}' - file not found'")]
    FileNotFound(String),
}

#[derive(Debug, Clone)]
pub struct AppEnv {
    pub sql_threads: u32,
    pub trace: bool,
    pub location_ip_address: String,
    pub location_log_combined: String,
    pub location_log_error: String,
    pub location_sqlite: String,
    pub debug: bool,
    pub start_time: SystemTime,
    pub timezone: String,
    pub utc_offset: UtcOffset,
    pub ws_address: String,
    pub ws_apikey: String,
    pub ws_token_address: String,
    pub ws_password: String,
}

impl AppEnv {
    /// Check a given file actually exists on the file system
    async fn check_file_exists(filename: String) -> Result<String, EnvError> {
        match fs::metadata(&filename).await {
            Ok(_) => Ok(filename),
            Err(_) => Err(EnvError::FileNotFound(filename)),
        }
    }

    /// Parse "true" or "false" to bool, else false
    fn parse_boolean(key: &str, map: &EnvHashMap) -> bool {
        match map.get(key) {
            Some(value) => value == "true",
            None => false,
        }
    }

    /// Make sure database file ends .db
    fn parse_db_name(key: &str, map: &EnvHashMap) -> Result<String, EnvError> {
        match map.get(key) {
            None => Err(EnvError::NotFound(key.into())),
            Some(value) => {
                if value.ends_with(".db") {
                    return Ok(value.into());
                }
                Err(EnvError::DbNameInvalid(key.into()))
            }
        }
    }

    /// Return offset for given timezone, else utc
    fn parse_offset(map: &EnvHashMap) -> UtcOffset {
        if let Some(data) = map.get("TIMEZONE") {
            if let Some(value) = timezones::get_by_name(data) {
                return value
                    .get_offset_utc(&time::OffsetDateTime::now_utc())
                    .to_utc();
            }
        }
        UtcOffset::from_hms(0, 0, 0).unwrap()
    }

    fn parse_string(key: &str, map: &EnvHashMap) -> Result<String, EnvError> {
        match map.get(key) {
            Some(value) => Ok(value.into()),
            None => Err(EnvError::NotFound(key.into())),
        }
    }
    /// Check that a given timezone is valid, else return UTC
    fn parse_timezone(map: &EnvHashMap) -> String {
        if let Some(data) = map.get("TIMEZONE") {
            if timezones::get_by_name(data).is_some() {
                return data.clone();
            }
        }
        "Etc/UTC".to_owned()
    }

    /// Parse string to u32, else return 1
    fn parse_u32(key: &str, map: &EnvHashMap) -> u32 {
        let default = 1u32;
        if let Some(data) = map.get(key) {
            return match data.parse::<u32>() {
                Ok(d) => d,
                Err(_) => default,
            };
        }
        default
    }

    /// Load, and parse .env file, return `AppEnv`
    async fn generate() -> Result<Self, EnvError> {
        let env_map = env::vars()
            .into_iter()
            .map(|i| (i.0, i.1))
            .collect::<HashMap<String, String>>();

        Ok(Self {
            trace: Self::parse_boolean("TRACE", &env_map),
            location_ip_address: Self::check_file_exists(Self::parse_string(
                "LOCATION_IP_ADDRESS",
                &env_map,
            )?)
            .await?,
            location_log_combined: Self::parse_string("LOCATION_LOG_COMBINED", &env_map)?,
            location_log_error: Self::parse_string("LOCATION_LOG_ERROR", &env_map)?,
            location_sqlite: Self::parse_db_name("LOCATION_SQLITE", &env_map)?,
            debug: Self::parse_boolean("DEBUG", &env_map),
            start_time: SystemTime::now(),
            timezone: Self::parse_timezone(&env_map),
            utc_offset: Self::parse_offset(&env_map),
            ws_address: Self::parse_string("WS_ADDRESS", &env_map)?,
            ws_apikey: Self::parse_string("WS_APIKEY", &env_map)?,
            ws_token_address: Self::parse_string("WS_TOKEN_ADDRESS", &env_map)?,
            ws_password: Self::parse_string("WS_PASSWORD", &env_map)?,
            sql_threads: Self::parse_u32("SQL_THREADS", &env_map),
        })
    }

    pub async fn get() -> Self {
        dotenv().ok();
        match Self::generate().await {
            Ok(s) => s,
            Err(e) => {
                println!("\n\x1b[31m{}\x1b[0m\n", e);
                std::process::exit(1);
            }
        }
    }
}

/// Run tests with
///
/// cargo watch -q -c -w src/ -x 'test env_ -- --nocapture'
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn env_missing_env() {
        let mut map = HashMap::new();
        map.insert("not_fish".to_owned(), "not_fish".to_owned());
        // ACTION
        let result = AppEnv::parse_string("fish", &map);

        // CHECK
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "missing env: 'fish'");
    }

    #[tokio::test]
    async fn env_parse_string_valid() {
        // FIXTURES
        let mut map = HashMap::new();
        map.insert("LOCATION_SQLITE".to_owned(), "/alarms.db".to_owned());

        // ACTION
        let result = AppEnv::parse_string("LOCATION_SQLITE", &map).unwrap();

        // CHECK
        assert_eq!(result, "/alarms.db");
    }

    #[tokio::test]
    async fn env_parse_boolean_ok() {
        // FIXTURES
        let mut map = HashMap::new();
        map.insert("valid_true".to_owned(), "true".to_owned());
        map.insert("valid_false".to_owned(), "false".to_owned());
        map.insert("invalid_but_false".to_owned(), "as".to_owned());

        // ACTION
        let result01 = AppEnv::parse_boolean("valid_true", &map);
        let result02 = AppEnv::parse_boolean("valid_false", &map);
        let result03 = AppEnv::parse_boolean("invalid_but_false", &map);
        let result04 = AppEnv::parse_boolean("missing", &map);

        // CHECK
        assert!(result01);
        assert!(!result02);
        assert!(!result03);
        assert!(!result04);
    }

    #[tokio::test]
    async fn env_parse_db_location_ok() {
        // FIXTURES
        let mut map = HashMap::new();
        map.insert("LOCATION_SQLITE".to_owned(), "file.db".to_owned());

        // ACTION
        let result = AppEnv::parse_db_name("LOCATION_SQLITE", &map);

        // CHECK
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "file.db");

        // FIXTURES
        let mut map = HashMap::new();
        map.insert(
            "LOCATION_SQLITE".to_owned(),
            "some/nested/location/file.db".to_owned(),
        );

        // ACTION
        let result = AppEnv::parse_db_name("LOCATION_SQLITE", &map);

        // CHECK
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "some/nested/location/file.db");
    }

    #[tokio::test]
    async fn env_parse_db_location_format_err() {
        // FIXTURES
        let mut map = HashMap::new();
        map.insert("LOCATION_SQLITE".to_owned(), "file.sql".to_owned());

        // ACTION
        let result = AppEnv::parse_db_name("LOCATION_SQLITE", &map);

        // CHECK
        assert!(result.is_err());
        match result.unwrap_err() {
            EnvError::DbNameInvalid(value) => assert_eq!(value, "LOCATION_SQLITE"),
            _ => unreachable!(),
        }
    }

    #[tokio::test]
    async fn env_parse_db_location_missing_err() {
        // FIXTURES
        let map = HashMap::new();

        // ACTION
        let result = AppEnv::parse_db_name("LOCATION_SQLITE", &map);

        // CHECK
        assert!(result.is_err());
        match result.unwrap_err() {
            EnvError::NotFound(value) => assert_eq!(value, "LOCATION_SQLITE"),
            _ => unreachable!(),
        }
    }

    // Need to work on this test, can fail during the odd period of the year when NY and Berling is only 5 hours seperated, rather than the usualy 6
    #[tokio::test]
    async fn env_parse_offset_ok() {
        // FIXTURES
        let mut map = HashMap::new();
        map.insert("TIMEZONE".to_owned(), "America/New_York".to_owned());

        // ACTION
        let result = AppEnv::parse_offset(&map);

        // CHECK
        assert_eq!(result, UtcOffset::from_hms(-4, 0, 0).unwrap());

        // FIXTURES
        let mut map = HashMap::new();
        map.insert("TIMEZONE".to_owned(), "Europe/Berlin".to_owned());

        // ACTION
        let result = AppEnv::parse_offset(&map);

        // CHECK
        assert_eq!(result, UtcOffset::from_hms(2, 0, 0).unwrap());

        // FIXTURES
        let map = HashMap::new();

        // ACTION
        let result = AppEnv::parse_offset(&map);

        // CHECK
        assert_eq!(result, UtcOffset::from_hms(0, 0, 0).unwrap());
    }

    #[tokio::test]
    async fn env_parse_offset_err() {
        // typo time zone
        // FIXTURES
        let mut map = HashMap::new();
        map.insert("TIMEZONE".to_owned(), "america/New_York".to_owned());

        // ACTION
        let result = AppEnv::parse_offset(&map);
        // CHECK
        assert_eq!(result, UtcOffset::from_hms(0, 0, 0).unwrap());

        // No timezone present
        // FIXTURES
        let map = HashMap::new();
        let result = AppEnv::parse_offset(&map);

        // CHECK
        assert_eq!(result, UtcOffset::from_hms(0, 0, 0).unwrap());
    }

    #[tokio::test]
    async fn env_parse_timezone_ok() {
        // FIXTURES
        let mut map = HashMap::new();
        map.insert("TIMEZONE".to_owned(), "America/New_York".to_owned());

        // ACTION
        let result = AppEnv::parse_timezone(&map);

        // CHECK
        assert_eq!(result, "America/New_York");

        let mut map = HashMap::new();
        map.insert("TIMEZONE".to_owned(), "Europe/Berlin".to_owned());

        // ACTION
        let result = AppEnv::parse_timezone(&map);

        // CHECK
        assert_eq!(result, "Europe/Berlin");

        // FIXTURES
        let map = HashMap::new();

        // ACTION
        let result = AppEnv::parse_timezone(&map);

        // CHECK
        assert_eq!(result, "Etc/UTC");
    }

    #[tokio::test]
    async fn env_parse_timezone_err() {
        // FIXTURES
        let mut map = HashMap::new();
        map.insert("TIMEZONE".to_owned(), "america/New_York".to_owned());

        // ACTION
        let result = AppEnv::parse_timezone(&map);
        // CHECK
        assert_eq!(result, "Etc/UTC");

        // No timezone present
        // FIXTURES
        let map = HashMap::new();
        let result = AppEnv::parse_timezone(&map);

        // CHECK
        assert_eq!(result, "Etc/UTC");
    }
    #[tokio::test]
    async fn env_panic_appenv() {
        // ACTION
        let result = AppEnv::generate().await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn env_return_appenv() {
        // FIXTURES
        dotenv().ok();

        // ACTION
        let result = AppEnv::generate().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn env_check_file_exists_ok() {
        // ACTION
        let result = AppEnv::check_file_exists("Cargo.lock".into()).await;

        // CHECK
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Cargo.lock");
    }

    #[tokio::test]
    async fn env_check_file_exists_er() {
        // ACTION
        let result = AppEnv::check_file_exists("file.sql".into()).await;

        // CHECK
        assert!(result.is_err());
        match result.unwrap_err() {
            EnvError::FileNotFound(value) => assert_eq!(value, "file.sql"),
            _ => unreachable!(),
        }
    }

    #[tokio::test]
    async fn env_parse_u32_ok() {
        // FIXTURES
        let mut map = HashMap::new();
        map.insert("U32_TEST".to_owned(), "88".to_owned());

        // ACTION
        let result = AppEnv::parse_u32("U32_TEST", &map);

        // CHECK
        assert_eq!(result, 88);
    }

    #[tokio::test]
    async fn env_parse_u32_default_ok() {
        // FIXTURES
        let map = HashMap::new();
        //   map.insert("U32_TEST".to_owned(), "88".to_owned());

        // ACTION
        let result = AppEnv::parse_u32("U32_TEST", &map);

        // CHECK
        assert_eq!(result, 1);

        // FIXTURES
        let mut map = HashMap::new();
        map.insert("U32_TEST".to_owned(), "U32_TEST".to_owned());

        // ACTION
        let result = AppEnv::parse_u32("U32_TEST", &map);

        // CHECK
        assert_eq!(result, 1);
    }
}
