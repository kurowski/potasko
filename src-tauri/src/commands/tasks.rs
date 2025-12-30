use crate::models::{CreateTask, SyncStatus, Task, UpdateTask};
use crate::DbState;
use chrono::Utc;
use tauri::State;
use uuid::Uuid;

/// Get all tasks for a list.
#[tauri::command]
pub async fn get_tasks(list_id: i64, db: State<'_, DbState>) -> Result<Vec<Task>, String> {
    let rows = sqlx::query_as!(
        TaskRow,
        r#"
        SELECT
            id as "id!",
            list_id as "list_id!",
            uid as "uid!",
            title as "title!",
            description,
            due_date,
            priority,
            completed as "completed!",
            completed_at,
            rrule,
            caldav_href,
            caldav_etag,
            raw_icalendar,
            local_version as "local_version!",
            synced_version as "synced_version!",
            sync_status as "sync_status!",
            created_at as "created_at!",
            updated_at as "updated_at!"
        FROM tasks
        WHERE list_id = ? AND sync_status != 'deleted'
        ORDER BY completed ASC, due_date ASC NULLS LAST, created_at ASC
        "#,
        list_id
    )
    .fetch_all(db.0.as_ref())
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(Task::from).collect())
}

/// Get a single task by ID.
#[tauri::command]
pub async fn get_task(id: i64, db: State<'_, DbState>) -> Result<Task, String> {
    let row = sqlx::query_as!(
        TaskRow,
        r#"
        SELECT
            id as "id!",
            list_id as "list_id!",
            uid as "uid!",
            title as "title!",
            description,
            due_date,
            priority,
            completed as "completed!",
            completed_at,
            rrule,
            caldav_href,
            caldav_etag,
            raw_icalendar,
            local_version as "local_version!",
            synced_version as "synced_version!",
            sync_status as "sync_status!",
            created_at as "created_at!",
            updated_at as "updated_at!"
        FROM tasks
        WHERE id = ?
        "#,
        id
    )
    .fetch_optional(db.0.as_ref())
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("Task with id {} not found", id))?;

    Ok(Task::from(row))
}

/// Create a new task.
#[tauri::command]
pub async fn create_task(data: CreateTask, db: State<'_, DbState>) -> Result<Task, String> {
    let now = Utc::now().to_rfc3339();
    let uid = Uuid::new_v4().to_string();
    let due_date = data.due_date.map(|d| d.to_rfc3339());

    let result = sqlx::query!(
        r#"
        INSERT INTO tasks (list_id, uid, title, description, due_date, priority, rrule,
                          completed, local_version, synced_version, sync_status,
                          created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, 0, 1, 0, 'pending', ?, ?)
        "#,
        data.list_id,
        uid,
        data.title,
        data.description,
        due_date,
        data.priority,
        data.rrule,
        now,
        now
    )
    .execute(db.0.as_ref())
    .await
    .map_err(|e| e.to_string())?;

    let id = result.last_insert_rowid();
    get_task(id, db).await
}

/// Update an existing task.
#[tauri::command]
pub async fn update_task(
    id: i64,
    data: UpdateTask,
    db: State<'_, DbState>,
) -> Result<Task, String> {
    let existing = get_task(id, db.clone()).await?;
    let now = Utc::now().to_rfc3339();

    let title = data.title.unwrap_or(existing.title);
    let description = data.description.or(existing.description);
    let due_date = data
        .due_date
        .map(|d| d.to_rfc3339())
        .or(existing.due_date.map(|d| d.to_rfc3339()));
    let priority = data.priority.or(existing.priority);
    let rrule = data.rrule.or(existing.rrule);
    let new_version = existing.local_version + 1;

    sqlx::query!(
        r#"
        UPDATE tasks
        SET title = ?, description = ?, due_date = ?, priority = ?, rrule = ?,
            local_version = ?, updated_at = ?
        WHERE id = ?
        "#,
        title,
        description,
        due_date,
        priority,
        rrule,
        new_version,
        now,
        id
    )
    .execute(db.0.as_ref())
    .await
    .map_err(|e| e.to_string())?;

    get_task(id, db).await
}

/// Toggle task completion status.
#[tauri::command]
pub async fn toggle_task_completion(id: i64, db: State<'_, DbState>) -> Result<Task, String> {
    let existing = get_task(id, db.clone()).await?;
    let now = Utc::now().to_rfc3339();

    let completed = !existing.completed;
    let completed_at: Option<String> = if completed { Some(now.clone()) } else { None };
    let new_version = existing.local_version + 1;

    sqlx::query!(
        r#"
        UPDATE tasks
        SET completed = ?, completed_at = ?, local_version = ?, updated_at = ?
        WHERE id = ?
        "#,
        completed,
        completed_at,
        new_version,
        now,
        id
    )
    .execute(db.0.as_ref())
    .await
    .map_err(|e| e.to_string())?;

    get_task(id, db).await
}

/// Delete a task (marks as deleted for sync, or hard-deletes if never synced).
#[tauri::command]
pub async fn delete_task(id: i64, db: State<'_, DbState>) -> Result<(), String> {
    let existing = get_task(id, db.clone()).await?;

    if existing.sync_status == SyncStatus::Pending && existing.caldav_href.is_none() {
        // Never synced - hard delete
        sqlx::query!("DELETE FROM tasks WHERE id = ?", id)
            .execute(db.0.as_ref())
            .await
            .map_err(|e| e.to_string())?;
    } else {
        // Mark as deleted for sync engine to handle
        let now = Utc::now().to_rfc3339();
        let new_version = existing.local_version + 1;

        sqlx::query!(
            r#"
            UPDATE tasks
            SET sync_status = 'deleted', local_version = ?, updated_at = ?
            WHERE id = ?
            "#,
            new_version,
            now,
            id
        )
        .execute(db.0.as_ref())
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Get all tasks due today (across all lists), including completed ones.
#[tauri::command]
pub async fn get_tasks_today(db: State<'_, DbState>) -> Result<Vec<Task>, String> {
    let rows = sqlx::query_as!(
        TaskRow,
        r#"
        SELECT
            id as "id!",
            list_id as "list_id!",
            uid as "uid!",
            title as "title!",
            description,
            due_date,
            priority,
            completed as "completed!",
            completed_at,
            rrule,
            caldav_href,
            caldav_etag,
            raw_icalendar,
            local_version as "local_version!",
            synced_version as "synced_version!",
            sync_status as "sync_status!",
            created_at as "created_at!",
            updated_at as "updated_at!"
        FROM tasks
        WHERE sync_status != 'deleted'
          AND due_date IS NOT NULL
          AND date(due_date) = date('now')
        ORDER BY completed ASC, due_date ASC, priority ASC NULLS LAST, created_at ASC
        "#
    )
    .fetch_all(db.0.as_ref())
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(Task::from).collect())
}

/// Get all overdue tasks (across all lists).
#[tauri::command]
pub async fn get_tasks_overdue(db: State<'_, DbState>) -> Result<Vec<Task>, String> {
    let rows = sqlx::query_as!(
        TaskRow,
        r#"
        SELECT
            id as "id!",
            list_id as "list_id!",
            uid as "uid!",
            title as "title!",
            description,
            due_date,
            priority,
            completed as "completed!",
            completed_at,
            rrule,
            caldav_href,
            caldav_etag,
            raw_icalendar,
            local_version as "local_version!",
            synced_version as "synced_version!",
            sync_status as "sync_status!",
            created_at as "created_at!",
            updated_at as "updated_at!"
        FROM tasks
        WHERE sync_status != 'deleted'
          AND completed = 0
          AND due_date IS NOT NULL
          AND date(due_date) < date('now')
        ORDER BY due_date ASC, priority ASC NULLS LAST, created_at ASC
        "#
    )
    .fetch_all(db.0.as_ref())
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(Task::from).collect())
}

// --- Row type for sqlx mapping ---

/// Intermediate type matching database columns exactly.
/// SQLite returns all integers as i64, and INTEGER columns may be nullable.
struct TaskRow {
    id: i64,
    list_id: i64,
    uid: String,
    title: String,
    description: Option<String>,
    due_date: Option<String>,
    priority: Option<i64>,        // SQLite INTEGER -> i64
    completed: i64,               // SQLite INTEGER (0/1) -> i64
    completed_at: Option<String>,
    rrule: Option<String>,
    caldav_href: Option<String>,
    caldav_etag: Option<String>,
    raw_icalendar: Option<String>,
    local_version: i64,
    synced_version: i64,
    sync_status: String,
    created_at: String,
    updated_at: String,
}

impl From<TaskRow> for Task {
    fn from(row: TaskRow) -> Self {
        Self {
            id: row.id,
            list_id: row.list_id,
            uid: row.uid,
            title: row.title,
            description: row.description,
            due_date: row.due_date.and_then(|s| parse_datetime_opt(&s)),
            priority: row.priority.map(|p| p as i32),  // i64 -> i32
            completed: row.completed != 0,              // i64 -> bool
            completed_at: row.completed_at.and_then(|s| parse_datetime_opt(&s)),
            rrule: row.rrule,
            caldav_href: row.caldav_href,
            caldav_etag: row.caldav_etag,
            raw_icalendar: row.raw_icalendar,
            local_version: row.local_version,
            synced_version: row.synced_version,
            sync_status: SyncStatus::from_str(&row.sync_status),
            created_at: parse_datetime(&row.created_at),
            updated_at: parse_datetime(&row.updated_at),
        }
    }
}

fn parse_datetime(s: &str) -> chrono::DateTime<Utc> {
    parse_datetime_opt(s).unwrap_or_else(Utc::now)
}

fn parse_datetime_opt(s: &str) -> Option<chrono::DateTime<Utc>> {
    // Try RFC3339 first (what we write), then SQLite's datetime() format
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .ok()
        .or_else(|| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                .map(|dt| dt.and_utc())
                .ok()
        })
}
