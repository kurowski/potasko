//! Tauri commands for preference operations.
//! These are thin wrappers around core::preferences functions.

use crate::core;
use crate::DbState;
use tauri::State;

#[tauri::command]
pub async fn get_show_local_accounts(db: State<'_, DbState>) -> Result<bool, String> {
    core::preferences::get_show_local_accounts(db.0.as_ref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_show_local_accounts(enabled: bool, db: State<'_, DbState>) -> Result<(), String> {
    core::preferences::set_show_local_accounts(enabled, db.0.as_ref())
        .await
        .map_err(|e| e.to_string())
}
