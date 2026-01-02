//! Tauri commands for account operations.
//! These are thin wrappers around core::accounts functions.

use crate::caldav::AccountTestResult;
use crate::core;
use crate::models::{Account, CreateAccount, UpdateAccount};
use crate::DbState;
use tauri::State;

#[tauri::command]
pub async fn get_accounts(db: State<'_, DbState>) -> Result<Vec<Account>, String> {
    core::accounts::get_accounts(db.0.as_ref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_account(id: i64, db: State<'_, DbState>) -> Result<Account, String> {
    core::accounts::get_account(id, db.0.as_ref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_account(data: CreateAccount, db: State<'_, DbState>) -> Result<Account, String> {
    core::accounts::create_account(data, db.0.as_ref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_account(
    id: i64,
    data: UpdateAccount,
    db: State<'_, DbState>,
) -> Result<Account, String> {
    core::accounts::update_account(id, data, db.0.as_ref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_account(id: i64, db: State<'_, DbState>) -> Result<(), String> {
    core::accounts::delete_account(id, db.0.as_ref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn test_account_connection(
    server_url: String,
    username: String,
    password: String,
) -> Result<AccountTestResult, String> {
    core::accounts::test_account_connection(&server_url, &username, &password)
        .await
        .map_err(|e| e.to_string())
}
