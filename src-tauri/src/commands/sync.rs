//! Tauri commands for sync operations.

use crate::core::tasks as core_tasks;
use crate::sync::{emit_sync_completed, get_sync_log, AccountSyncResult, SyncEngine, SyncLogEntry, SyncResult};
use crate::DbState;
use sqlx::SqlitePool;
use tauri::{AppHandle, State};

/// Sync a single list with its CalDAV calendar.
#[tauri::command]
pub async fn sync_list(list_id: i64, db: State<'_, DbState>, app: AppHandle) -> Result<SyncResult, String> {
    let engine = SyncEngine::new(pool_from_state(&db));
    let result = engine.sync_list(list_id).await;
    emit_sync_completed(&app, &result);
    Ok(result)
}

/// Initial download: fetch all tasks from a CalDAV calendar.
#[tauri::command]
pub async fn initial_download(list_id: i64, db: State<'_, DbState>, app: AppHandle) -> Result<SyncResult, String> {
    let engine = SyncEngine::new(pool_from_state(&db));
    let result = engine.initial_download(list_id).await;
    emit_sync_completed(&app, &result);
    Ok(result)
}

/// Sync all lists for an account: discovers calendars, imports new ones, syncs all lists.
#[tauri::command]
pub async fn sync_account(account_id: i64, db: State<'_, DbState>, app: AppHandle) -> Result<AccountSyncResult, String> {
    let engine = SyncEngine::new(pool_from_state(&db));
    let result = engine.sync_account(account_id).await;
    // Emit event for each synced list
    for list_result in &result.list_results {
        emit_sync_completed(&app, list_result);
    }
    Ok(result)
}

/// Get sync status for a list.
#[tauri::command]
pub async fn get_sync_status(list_id: i64, db: State<'_, DbState>) -> Result<ListSyncStatus, String> {
    let pool = pool_from_state(&db);

    // Count pending tasks (local_version > synced_version or sync_status != 'synced')
    let row = sqlx::query!(
        r#"
        SELECT COUNT(*) as count FROM tasks
        WHERE list_id = ? AND (local_version > synced_version OR sync_status != 'synced')
        "#,
        list_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?;
    let pending_count = row.count as u32;

    // Count failed tasks and get last error
    let failed_count = core_tasks::count_failed_tasks(list_id, &pool)
        .await
        .map_err(|e| e.to_string())?;
    let last_error = core_tasks::get_list_last_error(list_id, &pool)
        .await
        .map_err(|e| e.to_string())?;

    // Get list info (for caldav_url and last sync)
    let list = sqlx::query!(
        "SELECT caldav_url, ctag, updated_at FROM task_lists WHERE id = ?",
        list_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let (has_caldav, last_sync) = match list {
        Some(l) => (l.caldav_url.is_some(), l.ctag.map(|_| l.updated_at)),
        None => (false, None),
    };

    Ok(ListSyncStatus {
        list_id,
        has_caldav,
        pending_changes: pending_count,
        failed_changes: failed_count,
        last_error,
        last_sync,
    })
}

/// Clear backoff for failed tasks and trigger an immediate sync retry.
#[tauri::command]
pub async fn retry_failed_sync(list_id: i64, db: State<'_, DbState>, app: AppHandle) -> Result<SyncResult, String> {
    let pool = pool_from_state(&db);

    // Clear the retry backoff for all failed tasks in this list
    core_tasks::clear_list_sync_retry(list_id, &pool)
        .await
        .map_err(|e| e.to_string())?;

    // Trigger an immediate sync
    let engine = SyncEngine::new(pool);
    let result = engine.sync_list(list_id).await;
    emit_sync_completed(&app, &result);
    Ok(result)
}

/// Helper to get SqlitePool from state.
fn pool_from_state(db: &State<'_, DbState>) -> SqlitePool {
    SqlitePool::clone(db.0.as_ref())
}

/// Sync status for a list.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListSyncStatus {
    pub list_id: i64,
    pub has_caldav: bool,
    pub pending_changes: u32,
    pub failed_changes: u32,
    pub last_error: Option<String>,
    pub last_sync: Option<String>,
}

/// Get sync log entries with pagination.
#[tauri::command]
pub async fn get_sync_log_entries(
    limit: Option<u32>,
    offset: Option<u32>,
    db: State<'_, DbState>,
) -> Result<Vec<SyncLogEntry>, String> {
    let pool = pool_from_state(&db);
    get_sync_log(&pool, limit.unwrap_or(50), offset.unwrap_or(0))
        .await
        .map_err(|e| e.to_string())
}
