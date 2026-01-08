//! Tauri commands for task operations.
//! These are thin wrappers around core::tasks functions.

use crate::core;
use crate::models::{CreateTask, Task, UpdateTask};
use crate::sync::SyncEngine;
use crate::DbState;
use sqlx::SqlitePool;
use tauri::State;
use tokio::spawn;

/// Trigger a background push for a single task if its list is connected to CalDAV.
fn trigger_background_push(task_id: i64, list_id: i64, pool: SqlitePool) {
    spawn(async move {
        // Check if list is synced to avoid unnecessary work
        match core::lists::is_synced_list(list_id, &pool).await {
            Ok(true) => {
                let engine = SyncEngine::new(pool);
                let result = engine.push_task(task_id).await;

                if !result.success {
                    eprintln!(
                        "Background push failed for task {}: {:?}",
                        task_id, result.error
                    );
                }
            }
            Ok(false) => {
                // Local-only list, skip silently
            }
            Err(e) => {
                eprintln!("Failed to check list sync status: {:?}", e);
            }
        }
    });
}

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
    let pool = db.0.as_ref();
    let task = core::tasks::create_task(data, pool)
        .await
        .map_err(|e| e.to_string())?;

    trigger_background_push(task.id, task.list_id, pool.clone());

    Ok(task)
}

#[tauri::command]
pub async fn update_task(
    id: i64,
    data: UpdateTask,
    db: State<'_, DbState>,
) -> Result<Task, String> {
    let pool = db.0.as_ref();
    let task = core::tasks::update_task(id, data, pool)
        .await
        .map_err(|e| e.to_string())?;

    trigger_background_push(task.id, task.list_id, pool.clone());

    Ok(task)
}

#[tauri::command]
pub async fn toggle_task_completion(id: i64, db: State<'_, DbState>) -> Result<Task, String> {
    let pool = db.0.as_ref();
    let task = core::tasks::toggle_task_completion(id, pool)
        .await
        .map_err(|e| e.to_string())?;

    trigger_background_push(task.id, task.list_id, pool.clone());

    Ok(task)
}

#[tauri::command]
pub async fn delete_task(id: i64, db: State<'_, DbState>) -> Result<(), String> {
    let pool = db.0.as_ref();

    // Get task info before deleting
    let task = core::tasks::get_task(id, pool)
        .await
        .map_err(|e| e.to_string())?;
    let task_id = task.id;
    let list_id = task.list_id;
    let was_synced = task.caldav_href.is_some();

    // Delete the task (soft-delete if synced, hard-delete if not)
    core::tasks::delete_task(id, pool)
        .await
        .map_err(|e| e.to_string())?;

    // Only trigger push if the task was synced (soft-delete creates a tombstone)
    if was_synced {
        trigger_background_push(task_id, list_id, pool.clone());
    }

    Ok(())
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
