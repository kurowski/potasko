use crate::models::{CreateTaskList, TaskList, UpdateTaskList};
use chrono::Utc;
use sqlx::SqlitePool;

use super::error::CoreError;
use super::tasks::parse_datetime;

type Result<T> = std::result::Result<T, CoreError>;

/// Get all task lists.
pub async fn get_lists(pool: &SqlitePool) -> Result<Vec<TaskList>> {
    let rows = sqlx::query_as!(
        TaskListRow,
        r#"
        SELECT id, account_id, name, color, caldav_url, ctag, sync_token, created_at, updated_at
        FROM task_lists
        ORDER BY id
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(TaskList::from).collect())
}

/// Get a single task list by ID.
pub async fn get_list(id: i64, pool: &SqlitePool) -> Result<TaskList> {
    let row = sqlx::query_as!(
        TaskListRow,
        r#"
        SELECT id, account_id, name, color, caldav_url, ctag, sync_token, created_at, updated_at
        FROM task_lists
        WHERE id = ?
        "#,
        id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| CoreError::NotFound(format!("List with id {}", id)))?;

    Ok(TaskList::from(row))
}

/// Create a new task list.
pub async fn create_list(data: CreateTaskList, pool: &SqlitePool) -> Result<TaskList> {
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
    .execute(pool)
    .await?;

    let id = result.last_insert_rowid();
    get_list(id, pool).await
}

/// Update an existing task list.
pub async fn update_list(id: i64, data: UpdateTaskList, pool: &SqlitePool) -> Result<TaskList> {
    let existing = get_list(id, pool).await?;
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
    .execute(pool)
    .await?;

    get_list(id, pool).await
}

/// Delete a task list.
pub async fn delete_list(id: i64, pool: &SqlitePool) -> Result<()> {
    let result = sqlx::query!("DELETE FROM task_lists WHERE id = ?", id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(CoreError::NotFound(format!("List with id {}", id)));
    }

    Ok(())
}

// --- Row type for sqlx mapping ---

pub(crate) struct TaskListRow {
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
