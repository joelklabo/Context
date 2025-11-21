use sqlx::{migrate::Migrator, SqlitePool};

use crate::Result;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// Run database migrations for the SQLite backend.
pub async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    MIGRATOR.run(pool).await?;
    Ok(())
}
