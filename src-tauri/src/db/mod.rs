use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::Path;
use std::str::FromStr;

/// Creates a connection pool to the SQLite database.
/// The database file is created if it doesn't exist.
pub async fn create_pool(db_path: &Path) -> Result<SqlitePool, sqlx::Error> {
    // Ensure parent directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

    // Enable foreign keys for CASCADE deletes to work
    let options = SqliteConnectOptions::from_str(&db_url)?
        .foreign_keys(true);

    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
}

/// Initializes the database: creates connection and runs migrations.
pub async fn init(db_path: &Path) -> Result<SqlitePool, sqlx::Error> {
    let pool = create_pool(db_path).await?;

    // Run migrations embedded at compile time from migrations/ folder
    sqlx::migrate!().run(&pool).await?;

    Ok(pool)
}
