//! Persistent work graph primitives for goals, tasks, approvals, and runs.

pub mod error;
pub mod portability;
pub mod store;
pub mod types;

pub use {
    error::{Error, Result},
    store::SqliteWorkStore,
};

/// Run database migrations for the work crate.
pub async fn run_migrations(pool: &sqlx::SqlitePool) -> Result<()> {
    sqlx::migrate!("./migrations")
        .set_ignore_missing(true)
        .run(pool)
        .await
        .map_err(Error::from)?;
    Ok(())
}
