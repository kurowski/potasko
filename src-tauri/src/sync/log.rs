//! Sync logging utilities.

use chrono::Utc;
use sqlx::SqlitePool;

use super::types::SyncLogEntry;

/// Log a sync operation to the sync_log table.
pub async fn log_sync_operation(
    pool: &SqlitePool,
    account_id: Option<i64>,
    list_id: Option<i64>,
    task_id: Option<i64>,
    operation: &str,
    status: &str,
    message: Option<&str>,
    http_status: Option<u16>,
) -> Result<(), sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    let http_status_i64 = http_status.map(|s| s as i64);

    sqlx::query!(
        r#"
        INSERT INTO sync_log (account_id, list_id, task_id, operation, status, message, http_status, timestamp)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        account_id,
        list_id,
        task_id,
        operation,
        status,
        message,
        http_status_i64,
        now
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Get sync log entries with pagination.
pub async fn get_sync_log(
    pool: &SqlitePool,
    limit: u32,
    offset: u32,
) -> Result<Vec<SyncLogEntry>, sqlx::Error> {
    let limit_i64 = limit as i64;
    let offset_i64 = offset as i64;

    let rows = sqlx::query_as!(
        SyncLogEntry,
        r#"
        SELECT
            id as "id!",
            account_id,
            list_id,
            task_id,
            operation as "operation!",
            status as "status!",
            message,
            http_status,
            timestamp as "timestamp!"
        FROM sync_log
        ORDER BY timestamp DESC
        LIMIT ? OFFSET ?
        "#,
        limit_i64,
        offset_i64
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}
