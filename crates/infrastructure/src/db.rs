use std::time::Duration;

use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbErr, Schema};

use crate::persistence::seaorm::post;

pub async fn connect(db_url: &str) -> Result<DatabaseConnection, DbErr> {
    let mut options = ConnectOptions::new(db_url.to_owned());
    options
        .max_connections(env_u32("DB_POOL_MAX_CONNECTIONS", 10))
        .min_connections(env_u32("DB_POOL_MIN_CONNECTIONS", 1))
        .connect_timeout(Duration::from_secs(env_u64(
            "DB_POOL_CONNECT_TIMEOUT_SECS",
            10,
        )))
        .acquire_timeout(Duration::from_secs(env_u64(
            "DB_POOL_ACQUIRE_TIMEOUT_SECS",
            10,
        )))
        .idle_timeout(Duration::from_secs(env_u64(
            "DB_POOL_IDLE_TIMEOUT_SECS",
            600,
        )))
        .max_lifetime(Duration::from_secs(env_u64(
            "DB_POOL_MAX_LIFETIME_SECS",
            1800,
        )))
        .map_sqlx_postgres_opts(|pg| pg.options(pg_session_options()));

    let db = Database::connect(options).await?;
    setup_schema(&db).await?;
    Ok(db)
}

/// PostgreSQL session options applied on each pooled connection.
fn pg_session_options() -> [(&'static str, String); 4] {
    [
        (
            "lock_timeout",
            timeout_from_secs_env("DB_LOCK_TIMEOUT_SECS", 5),
        ),
        (
            "statement_timeout",
            timeout_from_secs_env("DB_STATEMENT_TIMEOUT_SECS", 30),
        ),
        (
            "idle_in_transaction_session_timeout",
            timeout_from_secs_env("DB_IDLE_IN_TRANSACTION_TIMEOUT_SECS", 60),
        ),
        (
            "deadlock_timeout",
            timeout_from_ms_env("DB_DEADLOCK_TIMEOUT_MS", 1000),
        ),
    ]
}

fn timeout_from_secs_env(key: &str, default_secs: u64) -> String {
    format!("{}s", env_u64(key, default_secs))
}

fn timeout_from_ms_env(key: &str, default_ms: u64) -> String {
    format!("{}ms", env_u64(key, default_ms))
}

fn env_u32(key: &str, default: u32) -> u32 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

async fn setup_schema(db: &DatabaseConnection) -> Result<(), DbErr> {
    let builder = db.get_database_backend();
    let schema = Schema::new(builder);

    let mut stmt = schema.create_table_from_entity(post::Entity);
    stmt.if_not_exists();

    db.execute(builder.build(&stmt)).await?;
    migrate_posts_status(db).await?;

    Ok(())
}

async fn migrate_posts_status(db: &DatabaseConnection) -> Result<(), DbErr> {
    use sea_orm::Statement;

    let backend = db.get_database_backend();
    let sql = match backend {
        sea_orm::DatabaseBackend::Postgres => {
            "ALTER TABLE posts ADD COLUMN IF NOT EXISTS status VARCHAR(20) NOT NULL DEFAULT 'draft'"
        }
        sea_orm::DatabaseBackend::Sqlite => {
            "ALTER TABLE posts ADD COLUMN status TEXT NOT NULL DEFAULT 'draft'"
        }
        sea_orm::DatabaseBackend::MySql => {
            "ALTER TABLE posts ADD COLUMN status VARCHAR(20) NOT NULL DEFAULT 'draft'"
        }
    };

    db.execute(Statement::from_string(backend, sql.to_string()))
        .await?;

    Ok(())
}
