//! Utilities for easily connecting to the database

use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

/// Database connection pool type
pub type Db = Pool<Postgres>;

/// Get a database connection pool
///
/// Currently, this function returns a database pool from the
/// `DATABASE_URL` environment variable.
///
/// # Errors
///
/// - If the `DATABASE_URL` environment variable is not set
/// - If the database connection fails
pub async fn require_db() -> anyhow::Result<Db> {
    let uri = std::env::var("DATABASE_URL")
        .map_err(|_| anyhow::anyhow!("DATABASE_URL environment variable not set"))?;

    let pool: Db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&uri)
        .await?;

    Ok(pool)
}
