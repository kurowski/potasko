use crate::models::{CreateTaskList, TaskList, UpdateTaskList};
use crate::DbState;
use chrono::Utc;
use tauri::State;

/// Get all task lists.
#[tauri::command]
pub async fn get_lists(db: State<'_, DbState>) -> Result<Vec<TaskList>, String> {
    let rows = sqlx::query_as!(
        TaskListRow,
        r#"
        SELECT id, account_id, name, color, caldav_url, ctag, sync_token, created_at, updated_at
        FROM task_lists
        ORDER BY id
        "#
    )
    .fetch_all(db.0.as_ref())
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(TaskList::from).collect())
}

/// Get a single task list by ID.
#[tauri::command]
pub async fn get_list(id: i64, db: State<'_, DbState>) -> Result<TaskList, String> {
    let row = sqlx::query_as!(
        TaskListRow,
        r#"
        SELECT id, account_id, name, color, caldav_url, ctag, sync_token, created_at, updated_at
        FROM task_lists
        WHERE id = ?
        "#,
        id
    )
    .fetch_optional(db.0.as_ref())
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("List with id {} not found", id))?;

    Ok(TaskList::from(row))
}

/// Create a new task list.
#[tauri::command]
pub async fn create_list(data: CreateTaskList, db: State<'_, DbState>) -> Result<TaskList, String> {
    let now = Utc::now().to_rfc3339();

    let result = sqlx::query!(
        r#"
        INSERT INTO task_lists (name, color, created_at, updated_at)
        VALUES (?, ?, ?, ?)
        "#,
        data.name,
        data.color,
        now,
        now
    )
    .execute(db.0.as_ref())
    .await
    .map_err(|e| e.to_string())?;

    let id = result.last_insert_rowid();
    get_list(id, db).await
}

/// Update an existing task list.
#[tauri::command]
pub async fn update_list(
    id: i64,
    data: UpdateTaskList,
    db: State<'_, DbState>,
) -> Result<TaskList, String> {
    let existing = get_list(id, db.clone()).await?;
    let now = Utc::now().to_rfc3339();

    let name = data.name.unwrap_or(existing.name);
    let color = data.color.or(existing.color);

    sqlx::query!(
        r#"
        UPDATE task_lists
        SET name = ?, color = ?, updated_at = ?
        WHERE id = ?
        "#,
        name,
        color,
        now,
        id
    )
    .execute(db.0.as_ref())
    .await
    .map_err(|e| e.to_string())?;

    get_list(id, db).await
}

/// Delete a task list.
#[tauri::command]
pub async fn delete_list(id: i64, db: State<'_, DbState>) -> Result<(), String> {
    let result = sqlx::query!("DELETE FROM task_lists WHERE id = ?", id)
        .execute(db.0.as_ref())
        .await
        .map_err(|e| e.to_string())?;

    if result.rows_affected() == 0 {
        return Err(format!("List with id {} not found", id));
    }

    Ok(())
}

// --- Row type for sqlx mapping ---

/// Intermediate type matching database columns exactly.
/// SQLite stores dates as TEXT, so we parse them when converting to TaskList.
struct TaskListRow {
    id: i64,
    account_id: Option<i64>,
    name: String,
    color: Option<String>,
    caldav_url: Option<String>,
    ctag: Option<String>,
    sync_token: Option<String>,
    created_at: String,
    updated_at: String,
}

impl From<TaskListRow> for TaskList {
    fn from(row: TaskListRow) -> Self {
        Self {
            id: row.id,
            account_id: row.account_id,
            name: row.name,
            color: row.color,
            caldav_url: row.caldav_url,
            ctag: row.ctag,
            sync_token: row.sync_token,
            created_at: parse_datetime(&row.created_at),
            updated_at: parse_datetime(&row.updated_at),
        }
    }
}

fn parse_datetime(s: &str) -> chrono::DateTime<Utc> {
    // Try RFC3339 first (what we write), then SQLite's datetime() format
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                .map(|dt| dt.and_utc())
        })
        .unwrap_or_else(|_| Utc::now())
}
