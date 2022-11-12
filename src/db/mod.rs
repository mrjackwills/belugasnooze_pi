mod model_alarm;
mod model_timezone;

use std::fs;

pub use model_alarm::ModelAlarm;
pub use model_timezone::ModelTimezone;

use sqlx::{sqlite::SqliteJournalMode, ConnectOptions, SqlitePool};
use tracing::error;

use crate::env::AppEnv;

/// If file doesn't exist on disk, create
/// Probably can be removes, as sqlx has a setting to create file if not found
fn file_exists(filename: &str) {
    if !std::path::Path::new(filename)
        .extension()
        .map_or(false, |ext| ext.eq_ignore_ascii_case("db"))
    {
        return;
    }
    if fs::metadata(filename).is_err() {
        let path = filename
            .split_inclusive('/')
            .filter(|f| {
                !std::path::Path::new(f)
                    .extension()
                    .map_or(false, |ext| ext.eq_ignore_ascii_case("db"))
            })
            .collect::<String>();
        match fs::create_dir_all(&path) {
            Ok(_) => (),
            Err(e) => {
                error!(%e);
                std::process::exit(1);
            }
        }
        match fs::File::create(filename) {
            Ok(_) => (),
            Err(e) => {
                error!(%e);
                std::process::exit(1);
            }
        }
    };
}

/// Open Sqlite pool connection, and return
/// `max_connections` need to be 1, [see issue](https://github.com/launchbadge/sqlx/issues/816)
async fn get_db(app_envs: &AppEnv) -> Result<SqlitePool, sqlx::Error> {
    let mut connect_options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(&app_envs.location_sqlite)
        .journal_mode(SqliteJournalMode::Wal);
    if !app_envs.trace {
        connect_options.disable_statement_logging();
    }
    let db = sqlx::pool::PoolOptions::<sqlx::Sqlite>::new()
        .max_connections(app_envs.sql_threads)
        .connect_with(connect_options)
        .await?;
    Ok(db)
}

/// Check if timezone in db, if not then insert
async fn insert_env_timezone(db: &SqlitePool, app_envs: &AppEnv) {
    if ModelTimezone::get(db).await.is_none() {
        ModelTimezone::insert(db, app_envs)
            .await
            .unwrap_or_default();
    }
}

async fn create_tables(db: &SqlitePool) {
    let init_db = include_str!("init_db.sql");
    match sqlx::query(init_db).execute(db).await {
        Ok(_) => (),
        Err(e) => {
            error!(%e);
            std::process::exit(1);
        }
    }
}

/// Init db connection, works if folder/files exists or not
pub async fn init_db(app_envs: &AppEnv) -> Result<SqlitePool, sqlx::Error> {
    file_exists(&app_envs.location_sqlite);
    let db = get_db(app_envs).await?;
    create_tables(&db).await;
    insert_env_timezone(&db, app_envs).await;
    Ok(db)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
/// Sql Test
///
/// cargo watch -q -c -w src/ -x 'test sql_mod -- --test-threads=1 --nocapture'
mod tests {
    use super::*;
    use std::{fs, time::SystemTime};
    use time::UtcOffset;

    fn cleanup() {
        fs::remove_dir_all("/dev/shm/test_db_files/").unwrap();
    }

    fn gen_args(timezone: String, hour_offset: i8, location_sqlite: String) -> AppEnv {
        let na = String::from("na");
        AppEnv {
            debug: true,
            location_ip_address: na.clone(),
            location_sqlite,
            sql_threads: 1,
            start_time: SystemTime::now(),
            timezone,
            trace: false,
            // utc_offset: UtcOffset::from_hms(hour_offset, 0, 0).unwrap(),
            ws_address: na.clone(),
            ws_apikey: na.clone(),
            ws_password: na.clone(),
            ws_token_address: na,
        }
    }

    #[tokio::test]
    async fn sql_mod_exists_created() {
        // FIXTURES
        let name = "testing_file.db";

        // ACTION
        file_exists(name);

        // CHECK
        let exists = fs::metadata(name).is_ok();
        assert!(exists);

        // CLEANUP
        fs::remove_file(name).unwrap();
    }

    #[tokio::test]
    async fn sql_mod_exists_nested_created() {
        // FIXTURES
        let path = "/dev/shm/test_db_files/";
        let name = format!("{path}/testing_file.db");

        // ACTION
        file_exists(&name);

        // CHECK
        let dir_exists = fs::metadata(path).unwrap().is_dir();
        let exists = fs::metadata(&name).is_ok();
        assert!(exists);
        assert!(dir_exists);

        // CLEANUP
        cleanup();
    }

    #[tokio::test]
    async fn sql_mod_exists_invalid_name() {
        // FIXTURES
        let name = "testing_file.sql";

        // ACTION
        file_exists(name);

        // CHECK
        let exists = fs::metadata(name).is_err();
        assert!(exists);
    }

    #[tokio::test]
    async fn sql_mod_db_created() {
        // FIXTURES
        let sql_name = String::from("/dev/shm/test_db_files/sql_file_db_created.db");
        let sql_sham = format!("{sql_name}-shm");
        let sql_wal = format!("{sql_name}-wal");

        let args = gen_args("America/New_York".into(), -5, sql_name.clone());

        // ACTION
        init_db(&args).await.unwrap();
        // CHECK
        assert!(fs::metadata(&sql_name).is_ok());
        assert!(fs::metadata(&sql_sham).is_ok());
        assert!(fs::metadata(&sql_wal).is_ok());

        // CLEANUP
        cleanup();
    }

    #[tokio::test]
    async fn sql_mod_db_created_with_timezone() {
        // FIXTURES
        let sql_name = String::from("/dev/shm/test_db_files/sql_file_db_created_with_timezone.db");
        let timezone = "America/New_York";
        let args = gen_args(timezone.into(), -5, sql_name.clone());
        init_db(&args).await.unwrap();
        let db = sqlx::pool::PoolOptions::<sqlx::Sqlite>::new()
            .max_connections(1)
            .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(&args.location_sqlite))
            .await
            .unwrap();

        // ACTION
        let result = sqlx::query_as("SELECT * FROM timezone")
            .fetch_one(&db)
            .await;

        // CHECK
        assert!(result.is_ok());
        let result: (i64, String, i64, i64, i64) = result.unwrap();
        assert_eq!(result.0, 1);
        assert_eq!(result.1, "America/New_York");
        assert_eq!(result.2, -5);
        assert_eq!(result.3, 0);
        assert_eq!(result.4, 0);

        // CLEANUP
        cleanup();
    }
}
