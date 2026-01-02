//! Tauri commands for task list operations.
//! These are thin wrappers around core::lists functions.

use crate::core;
use crate::models::{CreateTaskList, TaskList, UpdateTaskList};
use crate::DbState;
use tauri::State;

#[tauri::command]
pub async fn get_lists(db: State<'_, DbState>) -> Result<Vec<TaskList>, String> {
    core::lists::get_lists(db.0.as_ref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_list(id: i64, db: State<'_, DbState>) -> Result<TaskList, String> {
    core::lists::get_list(id, db.0.as_ref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_list(data: CreateTaskList, db: State<'_, DbState>) -> Result<TaskList, String> {
    core::lists::create_list(data, db.0.as_ref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_list(
    id: i64,
    data: UpdateTaskList,
    db: State<'_, DbState>,
) -> Result<TaskList, String> {
    core::lists::update_list(id, data, db.0.as_ref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_list(id: i64, db: State<'_, DbState>) -> Result<(), String> {
    core::lists::delete_list(id, db.0.as_ref())
        .await
        .map_err(|e| e.to_string())
}
