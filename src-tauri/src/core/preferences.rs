//! Core business logic for preferences.

use chrono::Utc;
use sqlx::SqlitePool;

use super::error::CoreError;

type Result<T> = std::result::Result<T, CoreError>;

const KEY_SHOW_LOCAL_ACCOUNTS: &str = "show_local_accounts";

/// Get the show_local_accounts preference.
/// Returns true if the preference is not set (default behavior).
pub async fn get_show_local_accounts(pool: &SqlitePool) -> Result<bool> {
    let row = sqlx::query_scalar!(
        r#"SELECT value FROM preferences WHERE key = ?"#,
        KEY_SHOW_LOCAL_ACCOUNTS
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|v| v == "true").unwrap_or(true))
}

/// Set the show_local_accounts preference.
pub async fn set_show_local_accounts(enabled: bool, pool: &SqlitePool) -> Result<()> {
    let value = if enabled { "true" } else { "false" };
    let now = Utc::now().to_rfc3339();

    sqlx::query!(
        r#"
        INSERT INTO preferences (key, value, updated_at)
        VALUES (?, ?, ?)
        ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at
        "#,
        KEY_SHOW_LOCAL_ACCOUNTS,
        value,
        now
    )
    .execute(pool)
    .await?;

    Ok(())
}
