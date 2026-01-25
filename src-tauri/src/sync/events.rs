//! Tauri event emission for sync operations.

use super::SyncResult;
use tauri::{AppHandle, Emitter};

/// Emit an event when a list sync completes.
/// Frontend can listen to "sync:list-completed" to refresh task views.
pub fn emit_sync_completed(app: &AppHandle, result: &SyncResult) {
    let _ = app.emit("sync:list-completed", result);
}
