mod commands;
mod db;
mod models;

pub use models::{
    Task, CreateTask, UpdateTask, SyncStatus,
    TaskList, CreateTaskList, UpdateTaskList,
};

use commands::{
    // List commands
    get_lists, get_list, create_list, update_list, delete_list,
    // Task commands
    get_tasks, get_task, create_task, update_task, delete_task, toggle_task_completion,
    get_tasks_today, get_tasks_overdue,
};

use sqlx::SqlitePool;
use std::sync::Arc;
use tauri::Manager;

/// Database pool wrapper for Tauri state.
pub struct DbState(pub Arc<SqlitePool>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Get app data directory for database file
            let app_data_dir = app.path().app_data_dir()?;
            let db_path = app_data_dir.join("potasko.db");

            println!("Database path: {}", db_path.display());

            // Initialize database (runs migrations)
            let pool = tauri::async_runtime::block_on(async {
                db::init(&db_path).await
            })?;

            // Store pool in app state
            app.manage(DbState(Arc::new(pool)));

            println!("Database initialized successfully");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // List commands
            get_lists,
            get_list,
            create_list,
            update_list,
            delete_list,
            // Task commands
            get_tasks,
            get_task,
            create_task,
            update_task,
            delete_task,
            toggle_task_completion,
            get_tasks_today,
            get_tasks_overdue,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
