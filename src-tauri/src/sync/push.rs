//! Push local changes to the CalDAV server.

use chrono::{Duration, Utc};
use sqlx::SqlitePool;

use crate::caldav::{build_vtodo, CalDavClient, CalDavError, VTodoBuildData};
use crate::core::tasks as core_tasks;
use crate::models::{SyncStatus, Task};

use super::log::log_sync_operation;
use super::types::{PushStats, SyncError, SyncFailureType};

/// Push all pending local changes to the server.
pub async fn push_changes(
    pool: &SqlitePool,
    client: &CalDavClient,
    list_id: i64,
    calendar_url: &str,
    account_id: Option<i64>,
) -> Result<PushStats, SyncError> {
    let mut stats = PushStats::default();

    // Get tasks with pending changes
    let pending_tasks = core_tasks::get_pending_tasks(list_id, pool).await?;

    for task in pending_tasks {
        let result = match task.sync_status {
            SyncStatus::Pending if task.caldav_href.is_none() => {
                // CREATE: New task, never synced
                push_create(pool, client, &task, calendar_url).await
            }
            SyncStatus::Pending | SyncStatus::Synced => {
                // UPDATE: Existing task with local changes
                if task.local_version > task.synced_version {
                    push_update(pool, client, &task).await
                } else {
                    continue;
                }
            }
            SyncStatus::Deleted => {
                // DELETE: Soft-deleted, remove from server
                push_delete(pool, client, &task).await
            }
            SyncStatus::Conflict => {
                // Skip conflicts for now
                continue;
            }
        };

        match result {
            Ok(PushAction::Created) => {
                stats.created += 1;
                // Clear any previous error on success
                let _ = core_tasks::clear_task_sync_error(task.id, pool).await;
                let _ = log_sync_operation(
                    pool,
                    account_id,
                    Some(list_id),
                    Some(task.id),
                    "push_create",
                    "success",
                    None,
                    None,
                )
                .await;
            }
            Ok(PushAction::Updated) => {
                stats.updated += 1;
                // Clear any previous error on success
                let _ = core_tasks::clear_task_sync_error(task.id, pool).await;
                let _ = log_sync_operation(
                    pool,
                    account_id,
                    Some(list_id),
                    Some(task.id),
                    "push_update",
                    "success",
                    None,
                    None,
                )
                .await;
            }
            Ok(PushAction::Deleted) => {
                stats.deleted += 1;
                // No need to clear error - task is deleted
                let _ = log_sync_operation(
                    pool,
                    account_id,
                    Some(list_id),
                    Some(task.id),
                    "push_delete",
                    "success",
                    None,
                    None,
                )
                .await;
            }
            Err(SyncError::CalDav(CalDavError::Conflict)) => {
                // Server has newer version - fetch and overwrite local (server-wins)
                if let Err(e) = handle_conflict(pool, client, &task).await {
                    eprintln!("Failed to resolve conflict for task {}: {}", task.id, e);
                }
                stats.conflicts += 1;
                // Clear error after conflict resolution
                let _ = core_tasks::clear_task_sync_error(task.id, pool).await;
                let _ = log_sync_operation(
                    pool,
                    account_id,
                    Some(list_id),
                    Some(task.id),
                    "push_conflict",
                    "conflict",
                    Some("Server has newer version"),
                    None,
                )
                .await;
            }
            Err(ref e) => {
                let msg = e.to_string();
                eprintln!("Failed to push task {}: {}", task.id, msg);

                // Classify the error and calculate backoff
                let failure_type = classify_sync_error(e);
                let retry_after = if failure_type.should_auto_retry() {
                    // Calculate retry delay based on previous attempts
                    let attempt = count_retry_attempts(&task);
                    let delay = calculate_retry_delay(attempt);
                    Some((Utc::now() + delay).to_rfc3339())
                } else {
                    None // No auto-retry for auth/client errors
                };

                // Record the error
                let _ = core_tasks::update_task_sync_error(
                    task.id,
                    &msg,
                    retry_after.as_deref(),
                    pool,
                )
                .await;

                let _ = log_sync_operation(
                    pool,
                    account_id,
                    Some(list_id),
                    Some(task.id),
                    "push_error",
                    "error",
                    Some(&msg),
                    None,
                )
                .await;
            }
        }
    }

    Ok(stats)
}

/// Action taken when pushing a task.
pub enum PushAction {
    Created,
    Updated,
    Deleted,
}

impl PushAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            PushAction::Created => "created",
            PushAction::Updated => "updated",
            PushAction::Deleted => "deleted",
        }
    }
}

/// Push a new task to the server.
pub async fn push_create(
    pool: &SqlitePool,
    client: &CalDavClient,
    task: &Task,
    calendar_url: &str,
) -> Result<PushAction, SyncError> {
    let ical = build_vtodo(&VTodoBuildData::from(task));

    // Generate href: calendar_url/uid.ics
    let href = format!("{}/{}.ics", calendar_url.trim_end_matches('/'), task.uid);

    // PUT to server (If-None-Match: * for create)
    let response = client.put(&href, &ical, None).await?;

    // Update task with href, etag, raw_icalendar
    core_tasks::update_task_sync_metadata(task.id, &href, response.etag.as_deref(), &ical, task.local_version, pool)
        .await?;

    Ok(PushAction::Created)
}

/// Push an updated task to the server.
pub async fn push_update(
    pool: &SqlitePool,
    client: &CalDavClient,
    task: &Task,
) -> Result<PushAction, SyncError> {
    let ical = build_vtodo(&VTodoBuildData::from(task));
    let href = task.caldav_href.as_ref().ok_or(SyncError::NoCalDavUrl(task.id))?;

    // PUT with If-Match for conflict detection
    let response = client.put(href, &ical, task.caldav_etag.as_deref()).await?;

    core_tasks::update_task_sync_metadata(task.id, href, response.etag.as_deref(), &ical, task.local_version, pool)
        .await?;

    Ok(PushAction::Updated)
}

/// Delete a task from the server.
pub async fn push_delete(
    pool: &SqlitePool,
    client: &CalDavClient,
    task: &Task,
) -> Result<PushAction, SyncError> {
    if let Some(href) = &task.caldav_href {
        // DELETE with If-Match
        client.delete(href, task.caldav_etag.as_deref()).await?;
    }

    // Hard delete from database
    core_tasks::hard_delete_task(task.id, pool).await?;

    Ok(PushAction::Deleted)
}

/// Handle a conflict by fetching the server version (server-wins).
async fn handle_conflict(
    pool: &SqlitePool,
    client: &CalDavClient,
    task: &Task,
) -> Result<(), SyncError> {
    let href = match &task.caldav_href {
        Some(h) => h,
        None => return Ok(()), // Can't resolve conflict without href
    };

    // Fetch the server version
    let ical = client.get(href).await?;
    let parsed = crate::caldav::parse_vtodo(&ical)?;

    // Update local task with server data (server-wins)
    let due_date = parsed.due_date_rfc3339();
    let completed_at = parsed.completed_at_rfc3339();

    core_tasks::update_task_from_server(
        task.id,
        parsed.title(),
        parsed.description.as_deref(),
        due_date.as_deref(),
        parsed.priority,
        parsed.completed,
        completed_at.as_deref(),
        parsed.rrule.as_deref(),
        None, // We don't have the new etag here
        &ical,
        pool,
    )
    .await?;

    Ok(())
}

/// Classify a sync error to determine retry behavior.
fn classify_sync_error(error: &SyncError) -> SyncFailureType {
    match error {
        SyncError::CalDav(caldav_error) => {
            // Extract HTTP status if available
            if let Some(status) = caldav_error.http_status() {
                SyncFailureType::from_http_status(status)
            } else {
                // Network/connection errors
                SyncFailureType::Offline
            }
        }
        SyncError::Database(_) => SyncFailureType::ClientError, // Don't retry DB errors
        SyncError::VTodo(_) => SyncFailureType::ClientError,    // Don't retry parse errors
        SyncError::Core(_) => SyncFailureType::ClientError,
        SyncError::NoAccount(_) => SyncFailureType::ClientError,
        SyncError::NoCalDavUrl(_) => SyncFailureType::ClientError,
        SyncError::AccountNotFound(_) => SyncFailureType::ClientError,
    }
}

/// Count retry attempts based on current error state.
fn count_retry_attempts(task: &Task) -> u32 {
    // If there's already an error and a retry_after in the future,
    // this is a repeated attempt. Count based on the backoff duration.
    if let (Some(_error), Some(retry_after)) = (&task.last_sync_error, &task.sync_retry_after) {
        let now = Utc::now();
        if *retry_after > now {
            // Still in backoff - estimate attempt number from delay
            let remaining = (*retry_after - now).num_seconds();
            if remaining >= 3600 {
                6 // 1 hour = max backoff
            } else if remaining >= 1800 {
                5 // 30 min
            } else if remaining >= 600 {
                4 // 10 min
            } else if remaining >= 120 {
                3 // 2 min
            } else if remaining >= 30 {
                2 // 30 sec
            } else {
                1
            }
        } else {
            // Backoff expired, this is a retry
            // Estimate based on the fact we had an error before
            1
        }
    } else if task.last_sync_error.is_some() {
        // Had error but no backoff time - first retry
        1
    } else {
        // No previous error - first attempt
        0
    }
}

/// Calculate retry delay based on attempt number.
/// Uses exponential backoff with a maximum of 1 hour.
fn calculate_retry_delay(attempt: u32) -> Duration {
    match attempt {
        0 => Duration::zero(),           // Immediate
        1 => Duration::seconds(30),      // 30 seconds
        2 => Duration::seconds(120),     // 2 minutes
        3 => Duration::seconds(600),     // 10 minutes
        4 => Duration::seconds(1800),    // 30 minutes
        _ => Duration::seconds(3600),    // 1 hour (max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_retry_delay() {
        assert_eq!(calculate_retry_delay(0).num_seconds(), 0);
        assert_eq!(calculate_retry_delay(1).num_seconds(), 30);
        assert_eq!(calculate_retry_delay(2).num_seconds(), 120);
        assert_eq!(calculate_retry_delay(3).num_seconds(), 600);
        assert_eq!(calculate_retry_delay(4).num_seconds(), 1800);
        assert_eq!(calculate_retry_delay(5).num_seconds(), 3600);
        assert_eq!(calculate_retry_delay(100).num_seconds(), 3600); // Max
    }

    #[test]
    fn test_sync_failure_type_from_http() {
        assert_eq!(SyncFailureType::from_http_status(401), SyncFailureType::AuthFailure);
        assert_eq!(SyncFailureType::from_http_status(403), SyncFailureType::AuthFailure);
        assert_eq!(SyncFailureType::from_http_status(404), SyncFailureType::ClientError);
        assert_eq!(SyncFailureType::from_http_status(500), SyncFailureType::ServerError);
        assert_eq!(SyncFailureType::from_http_status(503), SyncFailureType::ServerError);
    }

    #[test]
    fn test_should_auto_retry() {
        assert!(SyncFailureType::Offline.should_auto_retry());
        assert!(SyncFailureType::ServerError.should_auto_retry());
        assert!(!SyncFailureType::AuthFailure.should_auto_retry());
        assert!(!SyncFailureType::ClientError.should_auto_retry());
    }
}
