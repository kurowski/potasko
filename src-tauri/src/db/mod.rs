pub mod migrations;
pub mod schema;

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;

/// Creates a connection pool to the SQLite database.
/// The database file is created if it doesn't exist.
pub async fn create_pool(db_path: &Path) -> Result<SqlitePool, sqlx::Error> {
    // Ensure parent directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

    SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
}

/// Initializes the database: creates connection and runs migrations.
pub async fn init(db_path: &Path) -> Result<SqlitePool, sqlx::Error> {
    let pool = create_pool(db_path).await?;
    migrations::run(&pool).await?;
    Ok(pool)
}
