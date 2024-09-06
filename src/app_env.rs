use std::{collections::HashMap, env, fmt, time::SystemTime};
use time_tz::timezones;

use crate::app_error::AppError;

type EnvHashMap = HashMap<String, String>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EnvTimeZone(String);

impl EnvTimeZone {
    pub fn new(x: impl Into<String>) -> Self {
        let x = x.into();
        if timezones::get_by_name(&x).is_some() {
            Self(x)
        } else {
            Self("Etc/UTC".into())
        }
    }
}

impl fmt::Display for EnvTimeZone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct AppEnv {
    pub location_ip_address: String,
    pub location_sqlite: String,
    pub log_level: tracing::Level,
    pub rainbow: Option<()>,
    pub start_time: SystemTime,
    pub timezone: EnvTimeZone,
    pub ws_address: String,
    pub ws_apikey: String,
    pub ws_password: String,
    pub ws_token_address: String,
}

impl AppEnv {
    /// Check a given file actually exists on the file system
    fn check_file_exists(filename: String) -> Result<String, AppError> {
        match std::fs::metadata(&filename) {
            Ok(_) => Ok(filename),
            Err(_) => Err(AppError::FileNotFound(filename)),
        }
    }

    /// Parse "true" or "false" to bool, else false
    fn parse_boolean(key: &str, map: &EnvHashMap) -> bool {
        map.get(key).map_or(false, |value| value == "true")
    }

    /// Make sure database file ends .db
    fn parse_db_name(key: &str, map: &EnvHashMap) -> Result<String, AppError> {
        match map.get(key) {
            None => Err(AppError::MissingEnv(key.into())),
            Some(value) => {
                if std::path::Path::new(value)
                    .extension()
                    .map_or(false, |ext| ext.eq_ignore_ascii_case("db"))
                {
                    return Ok(value.into());
                }
                Err(AppError::DbNameInvalid(key.into()))
            }
        }
    }

    fn parse_string(key: &str, map: &EnvHashMap) -> Result<String, AppError> {
        map.get(key)
            .map_or(Err(AppError::MissingEnv(key.into())), |value| {
                Ok(value.into())
            })
    }

    /// Check that a given timezone is valid, else return UTC
    fn parse_timezone(map: &EnvHashMap) -> EnvTimeZone {
        EnvTimeZone::new(
            map.get("TZ")
                .map_or_else(String::new, std::borrow::ToOwned::to_owned),
        )
    }

    /// Parse debug and/or trace into tracing level
    fn parse_rainbow(map: &EnvHashMap) -> Option<()> {
        if Self::parse_boolean("RAINBOW", map) {
            Some(())
        } else {
            None
        }
    }

    /// Parse debug and/or trace into tracing level
    fn parse_log(map: &EnvHashMap) -> tracing::Level {
        if Self::parse_boolean("LOG_TRACE", map) {
            tracing::Level::TRACE
        } else if Self::parse_boolean("LOG_DEBUG", map) {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        }
    }

    /// Load, and parse .env file, return `AppEnv`
    fn generate() -> Result<Self, AppError> {
        let env_map = env::vars()
            .map(|i| (i.0, i.1))
            .collect::<HashMap<String, String>>();

        Ok(Self {
            location_ip_address: Self::check_file_exists(Self::parse_string(
                "LOCATION_IP_ADDRESS",
                &env_map,
            )?)?,
            location_sqlite: Self::parse_db_name("LOCATION_SQLITE", &env_map)?,
            log_level: Self::parse_log(&env_map),
            rainbow: Self::parse_rainbow(&env_map),
            start_time: SystemTime::now(),
            timezone: Self::parse_timezone(&env_map),
            ws_address: Self::parse_string("WS_ADDRESS", &env_map)?,
            ws_apikey: Self::parse_string("WS_APIKEY", &env_map)?,
            ws_password: Self::parse_string("WS_PASSWORD", &env_map)?,
            ws_token_address: Self::parse_string("WS_TOKEN_ADDRESS", &env_map)?,
        })
    }

    pub fn get() -> Self {
        let local_env = ".env";
        let app_env = "/app_env/.env";

        let env_path = if std::fs::metadata(app_env).is_ok() {
            app_env
        } else if std::fs::metadata(local_env).is_ok() {
            local_env
        } else {
            println!("\n\x1b[31munable to load env file\x1b[0m\n");
            std::process::exit(1);
        };

        dotenvy::from_path(env_path).ok();
        match Self::generate() {
            Ok(s) => s,
            Err(e) => {
                println!("\n\x1b[31m{e}\x1b[0m\n");
                std::process::exit(1);
            }
        }
    }
}

/// Run tests with
///
/// cargo watch -q -c -w src/ -x 'test env_ -- --nocapture'
#[cfg(test)]
#[expect(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn env_missing_env() {
        let mut map = HashMap::new();
        map.insert("not_fish".to_owned(), "not_fish".to_owned());

        let result = AppEnv::parse_string("fish", &map);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "missing env: 'fish'");
    }

    #[test]
    fn env_parse_string_valid() {
        let mut map = HashMap::new();
        map.insert("LOCATION_SQLITE".to_owned(), "/alarms.db".to_owned());

        let result = AppEnv::parse_string("LOCATION_SQLITE", &map).unwrap();

        assert_eq!(result, "/alarms.db");
    }

    #[test]
    fn env_parse_boolean_ok() {
        let mut map = HashMap::new();
        map.insert("valid_true".to_owned(), "true".to_owned());
        map.insert("valid_false".to_owned(), "false".to_owned());
        map.insert("invalid_but_false".to_owned(), "as".to_owned());

        let result01 = AppEnv::parse_boolean("valid_true", &map);
        let result02 = AppEnv::parse_boolean("valid_false", &map);
        let result03 = AppEnv::parse_boolean("invalid_but_false", &map);
        let result04 = AppEnv::parse_boolean("missing", &map);

        assert!(result01);
        assert!(!result02);
        assert!(!result03);
        assert!(!result04);
    }

    #[test]
    fn env_parse_rainbow() {
        let mut map = HashMap::new();
        map.insert("RAINBOW".to_owned(), "true".to_owned());

        let result = AppEnv::parse_rainbow(&map);

        assert!(result.is_some());

        let mut map = HashMap::new();
        map.insert("RAINBOW".to_owned(), "FALSE".to_owned());

        let result = AppEnv::parse_rainbow(&map);

        assert!(result.is_none());
    }

    #[test]
    fn env_parse_db_location_ok() {
        let mut map = HashMap::new();
        map.insert("LOCATION_SQLITE".to_owned(), "file.db".to_owned());

        let result = AppEnv::parse_db_name("LOCATION_SQLITE", &map);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "file.db");

        let mut map = HashMap::new();
        map.insert(
            "LOCATION_SQLITE".to_owned(),
            "some/nested/location/file.db".to_owned(),
        );

        let result = AppEnv::parse_db_name("LOCATION_SQLITE", &map);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "some/nested/location/file.db");
    }

    #[test]
    fn env_parse_db_location_format_err() {
        let mut map = HashMap::new();
        map.insert("LOCATION_SQLITE".to_owned(), "file.sql".to_owned());

        let result = AppEnv::parse_db_name("LOCATION_SQLITE", &map);

        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::DbNameInvalid(value) => assert_eq!(value, "LOCATION_SQLITE"),
            _ => unreachable!(),
        }
    }

    #[test]
    fn env_parse_db_location_missing_err() {
        let map = HashMap::new();

        let result = AppEnv::parse_db_name("LOCATION_SQLITE", &map);

        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::MissingEnv(value) => assert_eq!(value, "LOCATION_SQLITE"),
            _ => unreachable!(),
        }
    }

    #[test]
    fn env_parse_log_valid() {
        let map = HashMap::from([("RANDOM_STRING".to_owned(), "123".to_owned())]);

        let result = AppEnv::parse_log(&map);

        assert_eq!(result, tracing::Level::INFO);

        let map = HashMap::from([("LOG_DEBUG".to_owned(), "false".to_owned())]);

        let result = AppEnv::parse_log(&map);

        assert_eq!(result, tracing::Level::INFO);

        let map = HashMap::from([("LOG_TRACE".to_owned(), "false".to_owned())]);

        let result = AppEnv::parse_log(&map);

        assert_eq!(result, tracing::Level::INFO);

        let map = HashMap::from([
            ("LOG_DEBUG".to_owned(), "false".to_owned()),
            ("LOG_TRACE".to_owned(), "false".to_owned()),
        ]);

        let result = AppEnv::parse_log(&map);

        assert_eq!(result, tracing::Level::INFO);

        let map = HashMap::from([
            ("LOG_DEBUG".to_owned(), "true".to_owned()),
            ("LOG_TRACE".to_owned(), "false".to_owned()),
        ]);

        let result = AppEnv::parse_log(&map);

        assert_eq!(result, tracing::Level::DEBUG);

        let map = HashMap::from([
            ("LOG_DEBUG".to_owned(), "true".to_owned()),
            ("LOG_TRACE".to_owned(), "true".to_owned()),
        ]);

        let result = AppEnv::parse_log(&map);

        assert_eq!(result, tracing::Level::TRACE);

        let map = HashMap::from([
            ("LOG_DEBUG".to_owned(), "false".to_owned()),
            ("LOG_TRACE".to_owned(), "true".to_owned()),
        ]);

        let result = AppEnv::parse_log(&map);

        assert_eq!(result, tracing::Level::TRACE);
    }

    #[test]
    fn env_parse_timezone_ok() {
        let mut map = HashMap::new();
        map.insert("TZ".to_owned(), "America/New_York".to_owned());

        let result = AppEnv::parse_timezone(&map);

        assert_eq!(result.0, "America/New_York");

        let mut map = HashMap::new();
        map.insert("TZ".to_owned(), "Europe/Berlin".to_owned());

        let result = AppEnv::parse_timezone(&map);

        assert_eq!(result.0, "Europe/Berlin");

        let map = HashMap::new();

        let result = AppEnv::parse_timezone(&map);

        assert_eq!(result.0, "Etc/UTC");
    }

    #[test]
    fn env_parse_timezone_err() {
        let mut map = HashMap::new();
        map.insert("TIMEZONE".to_owned(), "america/New_York".to_owned());

        let result = AppEnv::parse_timezone(&map);

        assert_eq!(result.0, "Etc/UTC");

        // No timezone present

        let map = HashMap::new();
        let result = AppEnv::parse_timezone(&map);

        assert_eq!(result.0, "Etc/UTC");
    }

    #[test]
    fn env_panic_appenv() {
        let result = AppEnv::generate();

        assert!(result.is_err());
    }

    #[test]
    fn env_return_appenv() {
        dotenvy::dotenv().ok();

        let result = AppEnv::generate();

        assert!(result.is_ok());
    }

    #[test]
    fn env_check_file_exists_ok() {
        let result = AppEnv::check_file_exists("Cargo.lock".into());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Cargo.lock");
    }

    #[test]
    fn env_check_file_exists_er() {
        let result = AppEnv::check_file_exists("file.sql".into());

        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::FileNotFound(value) => assert_eq!(value, "file.sql"),
            _ => unreachable!(),
        }
    }
}
