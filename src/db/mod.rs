mod model_alarm;
mod model_timezone;

use std::fs;

pub use model_alarm::ModelAlarm;
pub use model_timezone::ModelTimezone;

use sqlx::{sqlite::SqliteJournalMode, ConnectOptions, SqlitePool};
use tracing::error;

use crate::app_env::AppEnv;

/// If file doesn't exist on disk, create
/// Probably can be removed, as sqlx has a setting to create file if not found
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
        match fs::create_dir_all(path) {
            Ok(()) => (),
            Err(e) => {
                error!("db_create_dir::{e}");
                std::process::exit(1);
            }
        }
        match fs::File::create(filename) {
            Ok(_) => (),
            Err(e) => {
                error!("db_create::{e}");
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

    match app_envs.log_level {
        tracing::Level::TRACE | tracing::Level::DEBUG => (),
        _ => connect_options = connect_options.disable_statement_logging(),
    }

    let db = sqlx::pool::PoolOptions::<sqlx::Sqlite>::new()
        .max_connections(1)
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
            error!("create_table::{e}");
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
    use crate::{
        app_env::EnvTimeZone,
        tests::{gen_app_envs, test_cleanup},
    };

    use uuid::Uuid;

    #[tokio::test]
    async fn sql_mod_exists_created() {
        let uuid = Uuid::new_v4();
        let name = format!("{uuid}.db");

        file_exists(&name);

        let exists = fs::metadata(&name).is_ok();
        assert!(exists);

        fs::remove_file(name).unwrap();
    }

    #[tokio::test]
    async fn sql_mod_exists_invalid_name() {
        let name = "testing_file.sql";

        file_exists(name);

        let exists = fs::metadata(name).is_err();
        assert!(exists);
    }

    #[tokio::test]
    async fn sql_mod_db_created() {
        let uuid = uuid::Uuid::new_v4();
        let args = gen_app_envs(uuid);

        let db = init_db(&args).await.unwrap();

        let sql_name = format!("/dev/shm/{uuid}.db");
        let sql_sham = format!("{sql_name}-shm");
        let sql_wal = format!("{sql_name}-wal");

        assert!(fs::metadata(sql_name).is_ok());
        assert!(fs::metadata(sql_sham).is_ok());
        assert!(fs::metadata(sql_wal).is_ok());

        db.close().await;

        test_cleanup(uuid, None).await;
    }

    #[tokio::test]
    async fn sql_mod_db_created_with_timezone() {
        let uuid = uuid::Uuid::new_v4();
        let mut args = gen_app_envs(uuid);
        args.timezone = EnvTimeZone::new("America/New_York");
        init_db(&args).await.unwrap();
        let db = sqlx::pool::PoolOptions::<sqlx::Sqlite>::new()
            .max_connections(1)
            .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(&args.location_sqlite))
            .await
            .unwrap();

        let result = sqlx::query_as("SELECT * FROM timezone")
            .fetch_one(&db)
            .await;

        assert!(result.is_ok());
        let result: (i64, String) = result.unwrap();
        assert_eq!(result.0, 1);
        assert_eq!(result.1, "America/New_York");

        test_cleanup(uuid, Some(db)).await;
    }
}
