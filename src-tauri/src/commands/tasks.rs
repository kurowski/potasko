//! Tauri commands for task operations.
//! These are thin wrappers around core::tasks functions.

use crate::core;
use crate::models::{CreateTask, Task, UpdateTask};
use crate::DbState;
use tauri::State;

#[tauri::command]
pub async fn get_tasks(list_id: i64, db: State<'_, DbState>) -> Result<Vec<Task>, String> {
    core::tasks::get_tasks(list_id, db.0.as_ref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_task(id: i64, db: State<'_, DbState>) -> Result<Task, String> {
    core::tasks::get_task(id, db.0.as_ref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_task(data: CreateTask, db: State<'_, DbState>) -> Result<Task, String> {
    core::tasks::create_task(data, db.0.as_ref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_task(
    id: i64,
    data: UpdateTask,
    db: State<'_, DbState>,
) -> Result<Task, String> {
    core::tasks::update_task(id, data, db.0.as_ref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_task_completion(id: i64, db: State<'_, DbState>) -> Result<Task, String> {
    core::tasks::toggle_task_completion(id, db.0.as_ref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_task(id: i64, db: State<'_, DbState>) -> Result<(), String> {
    core::tasks::delete_task(id, db.0.as_ref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_tasks_today(db: State<'_, DbState>) -> Result<Vec<Task>, String> {
    core::tasks::get_tasks_today(db.0.as_ref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_tasks_overdue(db: State<'_, DbState>) -> Result<Vec<Task>, String> {
    core::tasks::get_tasks_overdue(db.0.as_ref())
        .await
        .map_err(|e| e.to_string())
}
