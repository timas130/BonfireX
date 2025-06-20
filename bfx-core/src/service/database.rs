//! Utilities for easily connecting to the database

use crate::service::environment::require_env;
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
/// - If the `DATABASE_URL` environment variable is not set (see [`require_env`])
/// - If the database connection fails
pub async fn require_db() -> anyhow::Result<Db> {
    let uri = require_env("DATABASE_URL")?;

    let pool: Db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&uri)
        .await?;

    Ok(pool)
}

pub trait DbResultExt {
    fn is_unique_violation(&self) -> bool;
}

impl DbResultExt for sqlx::Error {
    fn is_unique_violation(&self) -> bool {
        self.as_database_error()
            .is_some_and(|err| err.kind() == sqlx::error::ErrorKind::UniqueViolation)
    }
}

impl<T> DbResultExt for Result<T, sqlx::Error> {
    fn is_unique_violation(&self) -> bool {
        if let Err(err) = self {
            err.is_unique_violation()
        } else {
            false
        }
    }
}
