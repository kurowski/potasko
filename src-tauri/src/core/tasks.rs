use crate::models::{CreateTask, SyncStatus, Task, UpdateTask};
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use super::error::CoreError;

type Result<T> = std::result::Result<T, CoreError>;

/// Get all tasks for a list.
pub async fn get_tasks(list_id: i64, pool: &SqlitePool) -> Result<Vec<Task>> {
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
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(Task::from).collect())
}

/// Get a single task by ID.
pub async fn get_task(id: i64, pool: &SqlitePool) -> Result<Task> {
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
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| CoreError::NotFound(format!("Task with id {}", id)))?;

    Ok(Task::from(row))
}

/// Create a new task.
pub async fn create_task(data: CreateTask, pool: &SqlitePool) -> Result<Task> {
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
    .execute(pool)
    .await?;

    let id = result.last_insert_rowid();
    get_task(id, pool).await
}

/// Update an existing task.
pub async fn update_task(id: i64, data: UpdateTask, pool: &SqlitePool) -> Result<Task> {
    let existing = get_task(id, pool).await?;
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
    .execute(pool)
    .await?;

    get_task(id, pool).await
}

/// Toggle task completion status.
/// For recurring tasks, completing creates the next occurrence.
pub async fn toggle_task_completion(id: i64, pool: &SqlitePool) -> Result<Task> {
    let existing = get_task(id, pool).await?;
    let now = Utc::now().to_rfc3339();

    let completed = !existing.completed;
    let completed_at: Option<String> = if completed { Some(now.clone()) } else { None };
    let new_version = existing.local_version + 1;

    // Update the current task
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
    .execute(pool)
    .await?;

    // If completing a recurring task with a due date, create the next occurrence
    if completed && existing.rrule.is_some() && existing.due_date.is_some() {
        if let Some(next_due) =
            calculate_next_due_date(&existing.due_date.unwrap(), existing.rrule.as_ref().unwrap())
        {
            let uid = Uuid::new_v4().to_string();
            let next_due_str = next_due.to_rfc3339();

            sqlx::query!(
                r#"
                INSERT INTO tasks (list_id, uid, title, description, due_date, priority, rrule,
                                  completed, local_version, synced_version, sync_status,
                                  created_at, updated_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, 0, 1, 0, 'pending', ?, ?)
                "#,
                existing.list_id,
                uid,
                existing.title,
                existing.description,
                next_due_str,
                existing.priority,
                existing.rrule,
                now,
                now
            )
            .execute(pool)
            .await?;
        }
    }

    get_task(id, pool).await
}

/// Delete a task (marks as deleted for sync, or hard-deletes if never synced).
pub async fn delete_task(id: i64, pool: &SqlitePool) -> Result<()> {
    let existing = get_task(id, pool).await?;

    if existing.sync_status == SyncStatus::Pending && existing.caldav_href.is_none() {
        // Never synced - hard delete
        sqlx::query!("DELETE FROM tasks WHERE id = ?", id)
            .execute(pool)
            .await?;
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
        .execute(pool)
        .await?;
    }

    Ok(())
}

/// Get all tasks due today (across all lists).
pub async fn get_tasks_today(pool: &SqlitePool) -> Result<Vec<Task>> {
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
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(Task::from).collect())
}

/// Get all overdue tasks (across all lists).
pub async fn get_tasks_overdue(pool: &SqlitePool) -> Result<Vec<Task>> {
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
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(Task::from).collect())
}

// --- Helper functions ---

/// Calculate the next due date based on RRULE frequency.
fn calculate_next_due_date(
    current_due: &chrono::DateTime<Utc>,
    rrule: &str,
) -> Option<chrono::DateTime<Utc>> {
    use chrono::{Datelike, Duration};

    if rrule.contains("FREQ=DAILY") {
        Some(*current_due + Duration::days(1))
    } else if rrule.contains("FREQ=WEEKLY") {
        Some(*current_due + Duration::weeks(1))
    } else if rrule.contains("FREQ=MONTHLY") {
        // Add one month (handle month boundaries)
        let year = current_due.year();
        let month = current_due.month();
        let day = current_due.day();

        let (new_year, new_month) = if month == 12 {
            (year + 1, 1)
        } else {
            (year, month + 1)
        };

        // Handle days that don't exist in the target month (e.g., Jan 31 -> Feb 28)
        let days_in_new_month = days_in_month(new_year, new_month);
        let new_day = day.min(days_in_new_month);

        current_due
            .with_year(new_year)
            .and_then(|d| d.with_month(new_month))
            .and_then(|d| d.with_day(new_day))
    } else if rrule.contains("FREQ=YEARLY") {
        current_due.with_year(current_due.year() + 1)
    } else {
        None
    }
}

/// Get the number of days in a month.
fn days_in_month(year: i32, month: u32) -> u32 {
    use chrono::{Datelike, NaiveDate};

    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };

    NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .and_then(|d| d.pred_opt())
        .map(|d| d.day())
        .unwrap_or(28)
}

// --- Row type for sqlx mapping ---

/// Intermediate type matching database columns exactly.
pub(crate) struct TaskRow {
    id: i64,
    list_id: i64,
    uid: String,
    title: String,
    description: Option<String>,
    due_date: Option<String>,
    priority: Option<i64>,
    completed: i64,
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
            priority: row.priority.map(|p| p as i32),
            completed: row.completed != 0,
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

// --- Datetime parsing helpers ---

pub(crate) fn parse_datetime(s: &str) -> chrono::DateTime<Utc> {
    parse_datetime_opt(s).unwrap_or_else(Utc::now)
}

pub(crate) fn parse_datetime_opt(s: &str) -> Option<chrono::DateTime<Utc>> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .ok()
        .or_else(|| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                .map(|dt| dt.and_utc())
                .ok()
        })
}

// ========== Sync helper functions ==========

/// Get tasks that need to be pushed to the server.
/// Returns tasks where local_version > synced_version OR sync_status = 'deleted'.
pub async fn get_pending_tasks(list_id: i64, pool: &SqlitePool) -> Result<Vec<Task>> {
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
        WHERE list_id = ?
          AND (local_version > synced_version OR sync_status = 'deleted')
        "#,
        list_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(Task::from).collect())
}

/// Get tasks indexed by their caldav_href (for sync comparison).
pub async fn get_tasks_by_href(list_id: i64, pool: &SqlitePool) -> Result<std::collections::HashMap<String, Task>> {
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
        WHERE list_id = ? AND caldav_href IS NOT NULL
        "#,
        list_id
    )
    .fetch_all(pool)
    .await?;

    let mut map = std::collections::HashMap::new();
    for row in rows {
        let task = Task::from(row);
        if let Some(ref href) = task.caldav_href {
            map.insert(href.clone(), task);
        }
    }

    Ok(map)
}

/// Update task sync metadata after successful push.
pub async fn update_task_sync_metadata(
    id: i64,
    href: &str,
    etag: Option<&str>,
    raw_icalendar: &str,
    synced_version: i64,
    pool: &SqlitePool,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();

    sqlx::query!(
        r#"
        UPDATE tasks
        SET caldav_href = ?, caldav_etag = ?, raw_icalendar = ?,
            synced_version = ?, sync_status = 'synced', updated_at = ?
        WHERE id = ?
        "#,
        href,
        etag,
        raw_icalendar,
        synced_version,
        now,
        id
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Update task from server data (server-wins conflict resolution).
pub async fn update_task_from_server(
    id: i64,
    title: &str,
    description: Option<&str>,
    due_date: Option<&str>,
    priority: Option<i32>,
    completed: bool,
    completed_at: Option<&str>,
    rrule: Option<&str>,
    etag: Option<&str>,
    raw_icalendar: &str,
    pool: &SqlitePool,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    let priority_i64 = priority.map(|p| p as i64);
    let completed_i64 = if completed { 1i64 } else { 0i64 };

    sqlx::query!(
        r#"
        UPDATE tasks
        SET title = ?, description = ?, due_date = ?, priority = ?,
            completed = ?, completed_at = ?, rrule = ?,
            caldav_etag = ?, raw_icalendar = ?,
            synced_version = local_version, sync_status = 'synced',
            updated_at = ?
        WHERE id = ?
        "#,
        title,
        description,
        due_date,
        priority_i64,
        completed_i64,
        completed_at,
        rrule,
        etag,
        raw_icalendar,
        now,
        id
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Create a task from server data (pulled from CalDAV).
pub async fn create_task_from_server(
    list_id: i64,
    uid: &str,
    href: &str,
    title: &str,
    description: Option<&str>,
    due_date: Option<&str>,
    priority: Option<i32>,
    completed: bool,
    completed_at: Option<&str>,
    rrule: Option<&str>,
    etag: Option<&str>,
    raw_icalendar: &str,
    pool: &SqlitePool,
) -> Result<i64> {
    let now = Utc::now().to_rfc3339();
    let priority_i64 = priority.map(|p| p as i64);
    let completed_i64 = if completed { 1i64 } else { 0i64 };

    let result = sqlx::query!(
        r#"
        INSERT INTO tasks (
            list_id, uid, title, description, due_date, priority,
            completed, completed_at, rrule,
            caldav_href, caldav_etag, raw_icalendar,
            local_version, synced_version, sync_status,
            created_at, updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 1, 1, 'synced', ?, ?)
        "#,
        list_id,
        uid,
        title,
        description,
        due_date,
        priority_i64,
        completed_i64,
        completed_at,
        rrule,
        href,
        etag,
        raw_icalendar,
        now,
        now
    )
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

/// Hard delete a task from the database.
pub async fn hard_delete_task(id: i64, pool: &SqlitePool) -> Result<()> {
    sqlx::query!("DELETE FROM tasks WHERE id = ?", id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Update task by href from server data.
pub async fn update_task_by_href_from_server(
    href: &str,
    title: &str,
    description: Option<&str>,
    due_date: Option<&str>,
    priority: Option<i32>,
    completed: bool,
    completed_at: Option<&str>,
    rrule: Option<&str>,
    etag: Option<&str>,
    raw_icalendar: &str,
    pool: &SqlitePool,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    let priority_i64 = priority.map(|p| p as i64);
    let completed_i64 = if completed { 1i64 } else { 0i64 };

    sqlx::query!(
        r#"
        UPDATE tasks
        SET title = ?, description = ?, due_date = ?, priority = ?,
            completed = ?, completed_at = ?, rrule = ?,
            caldav_etag = ?, raw_icalendar = ?,
            synced_version = local_version, sync_status = 'synced',
            updated_at = ?
        WHERE caldav_href = ?
        "#,
        title,
        description,
        due_date,
        priority_i64,
        completed_i64,
        completed_at,
        rrule,
        etag,
        raw_icalendar,
        now,
        href
    )
    .execute(pool)
    .await?;

    Ok(())
}
