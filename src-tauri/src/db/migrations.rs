/// Database migration system.
/// Uses a simple version-based approach with a schema_version table.

use sqlx::SqlitePool;

use super::schema;

/// Current schema version. Increment when adding new migrations.
const CURRENT_VERSION: i32 = 1;

/// Creates the schema_version table if it doesn't exist.
const CREATE_SCHEMA_VERSION: &str = r#"
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
)
"#;

/// Gets the current schema version from the database.
async fn get_version(pool: &SqlitePool) -> Result<i32, sqlx::Error> {
    // Ensure schema_version table exists
    sqlx::query(CREATE_SCHEMA_VERSION).execute(pool).await?;

    // Get current version (0 if no rows)
    let row: Option<(i32,)> = sqlx::query_as("SELECT MAX(version) FROM schema_version")
        .fetch_optional(pool)
        .await?;

    Ok(row.and_then(|(v,)| Some(v)).unwrap_or(0))
}

/// Records that a migration version has been applied.
async fn set_version(pool: &SqlitePool, version: i32) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO schema_version (version) VALUES (?)")
        .bind(version)
        .execute(pool)
        .await?;
    Ok(())
}

/// Runs all pending migrations.
pub async fn run(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let current = get_version(pool).await?;

    if current < CURRENT_VERSION {
        // Run migrations based on current version
        if current < 1 {
            migrate_v1(pool).await?;
        }

        // Add more migration steps here as needed:
        // if current < 2 { migrate_v2(pool).await?; }
    }

    Ok(())
}

/// Migration v1: Initial schema - creates all core tables.
async fn migrate_v1(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Run all table creation statements
    for sql in schema::ALL_TABLES {
        sqlx::query(sql).execute(pool).await?;
    }

    // Create a default "Inbox" list for local tasks
    sqlx::query(
        r#"
        INSERT INTO task_lists (name, color)
        SELECT 'Inbox', '#3b82f6'
        WHERE NOT EXISTS (SELECT 1 FROM task_lists WHERE name = 'Inbox' AND account_id IS NULL)
        "#,
    )
    .execute(pool)
    .await?;

    set_version(pool, 1).await?;
    Ok(())
}
