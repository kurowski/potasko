use crate::caldav::CalDavClient;
use crate::models::{CreateTaskList, TaskList, UpdateTaskList};
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use super::accounts::get_account;
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
///
/// If `account_id` is provided, the list will be created on the CalDAV server
/// using MKCALENDAR, and then stored locally with the `caldav_url` set.
pub async fn create_list(data: CreateTaskList, pool: &SqlitePool) -> Result<TaskList> {
    let now = Utc::now().to_rfc3339();

    // If account_id is provided, create on CalDAV server first
    if let Some(account_id) = data.account_id {
        let account = get_account(account_id, pool).await?;

        // Need calendar_home_url to create a new calendar
        let calendar_home_url = account.calendar_home_url.ok_or_else(|| {
            CoreError::CalDav("Account does not have a calendar home URL".to_string())
        })?;

        // Generate a unique slug for the calendar URL
        let slug = generate_calendar_slug(&data.name);
        let calendar_url = format!(
            "{}/{}/",
            calendar_home_url.trim_end_matches('/'),
            slug
        );

        // Create the calendar on the server
        let client = CalDavClient::new(&account.server_url, &account.username, &account.password)
            .map_err(|e| CoreError::CalDav(e.to_string()))?;

        client
            .mkcalendar(&calendar_url, &data.name, data.color.as_deref())
            .await
            .map_err(|e| CoreError::CalDav(e.to_string()))?;

        // Store locally with the caldav_url
        let result = sqlx::query!(
            r#"
            INSERT INTO task_lists (account_id, caldav_url, name, color, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            account_id,
            calendar_url,
            data.name,
            data.color,
            now,
            now
        )
        .execute(pool)
        .await?;

        let id = result.last_insert_rowid();
        return get_list(id, pool).await;
    }

    // Local-only list
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

/// Generate a URL-safe slug for a calendar name.
fn generate_calendar_slug(name: &str) -> String {
    // Create a base slug from the name
    let base: String = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    // Add a short UUID suffix to ensure uniqueness
    let suffix = &Uuid::new_v4().to_string()[..8];
    if base.is_empty() {
        format!("list-{}", suffix)
    } else {
        format!("{}-{}", base, suffix)
    }
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
///
/// If the list is synced to a CalDAV server (has caldav_url), it will be
/// deleted from the server first, then from the local database.
pub async fn delete_list(id: i64, pool: &SqlitePool) -> Result<()> {
    // Get the list to check if it's synced
    let list = get_list(id, pool).await?;

    // If synced to CalDAV, delete from server first
    if let (Some(account_id), Some(caldav_url)) = (list.account_id, list.caldav_url) {
        let account = get_account(account_id, pool).await?;

        let client = CalDavClient::new(&account.server_url, &account.username, &account.password)
            .map_err(|e| CoreError::CalDav(e.to_string()))?;

        client
            .delete_calendar(&caldav_url)
            .await
            .map_err(|e| CoreError::CalDav(e.to_string()))?;
    }

    // Delete from local database
    let result = sqlx::query!("DELETE FROM task_lists WHERE id = ?", id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(CoreError::NotFound(format!("List with id {}", id)));
    }

    Ok(())
}

/// Check if a list has a CalDAV URL (i.e., is synced to a server).
pub async fn is_synced_list(list_id: i64, pool: &SqlitePool) -> Result<bool> {
    let result = sqlx::query_scalar!(
        "SELECT caldav_url FROM task_lists WHERE id = ?",
        list_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.flatten().is_some())
}

/// Create a task list from a discovered CalDAV calendar.
/// This creates a list already linked to an account with a caldav_url set.
pub async fn create_list_from_calendar(
    account_id: i64,
    caldav_url: &str,
    name: &str,
    color: Option<&str>,
    pool: &SqlitePool,
) -> Result<TaskList> {
    let now = Utc::now().to_rfc3339();

    let result = sqlx::query!(
        r#"
        INSERT INTO task_lists (account_id, caldav_url, name, color, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
        account_id,
        caldav_url,
        name,
        color,
        now,
        now
    )
    .execute(pool)
    .await?;

    let id = result.last_insert_rowid();
    get_list(id, pool).await
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
