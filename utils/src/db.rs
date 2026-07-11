//! Pool and connection wiring around `sqlx::Postgres`.

use std::time::Duration;

use sqlx::postgres::{PgPoolOptions, PgRow};

pub type Connection = sqlx::Pool<sqlx::Postgres>;
pub type Row = PgRow;

pub struct DB;

impl DB {
    /// Pool sizing is deployment-specific; override via DB_MAX_CONNECTIONS. The app
    /// and dispatch services share this pool, and a bulk import can hold a
    /// connection per chunk, so the default is larger than the prior hardcoded 8.
    /// max_lifetime / idle_timeout proactively recycle connections gone stale after
    /// a Postgres or proxy restart or a network blip.
    pub async fn build_pool(url: &str) -> crate::Result<Connection> {
        let max_connections = std::env::var("DB_MAX_CONNECTIONS")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(16);
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .acquire_timeout(Duration::from_secs(10))
            .idle_timeout(Duration::from_secs(600))
            .max_lifetime(Duration::from_secs(1800))
            .connect(url)
            .await
            .map_err(|e| crate::AppError::InternalServer(format!("pool connect: {e}")))?;
        Ok(pool)
    }
}
