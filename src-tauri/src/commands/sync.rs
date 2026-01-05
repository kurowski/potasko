//! Tauri commands for sync operations.

use crate::sync::{SyncEngine, SyncResult};
use crate::DbState;
use sqlx::SqlitePool;
use tauri::State;

/// Sync a single list with its CalDAV calendar.
#[tauri::command]
pub async fn sync_list(list_id: i64, db: State<'_, DbState>) -> Result<SyncResult, String> {
    let engine = SyncEngine::new(pool_from_state(&db));
    Ok(engine.sync_list(list_id).await)
}

/// Initial download: fetch all tasks from a CalDAV calendar.
#[tauri::command]
pub async fn initial_download(list_id: i64, db: State<'_, DbState>) -> Result<SyncResult, String> {
    let engine = SyncEngine::new(pool_from_state(&db));
    Ok(engine.initial_download(list_id).await)
}

/// Sync all lists for an account.
#[tauri::command]
pub async fn sync_account(account_id: i64, db: State<'_, DbState>) -> Result<Vec<SyncResult>, String> {
    let pool = pool_from_state(&db);
    let engine = SyncEngine::new(pool.clone());

    // Get all lists for this account
    let rows = sqlx::query!(
        r#"SELECT id as "id!" FROM task_lists WHERE account_id = ?"#,
        account_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for row in rows {
        results.push(engine.sync_list(row.id).await);
    }

    Ok(results)
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
        last_sync,
    })
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
    pub last_sync: Option<String>,
}
