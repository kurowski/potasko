//! Push local changes to the CalDAV server.

use sqlx::SqlitePool;

use crate::caldav::{build_vtodo, CalDavClient, CalDavError, VTodoBuildData};
use crate::core::tasks as core_tasks;
use crate::models::{SyncStatus, Task};

use super::types::{PushStats, SyncError};

/// Push all pending local changes to the server.
pub async fn push_changes(
    pool: &SqlitePool,
    client: &CalDavClient,
    list_id: i64,
    calendar_url: &str,
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
            Ok(PushAction::Created) => stats.created += 1,
            Ok(PushAction::Updated) => stats.updated += 1,
            Ok(PushAction::Deleted) => stats.deleted += 1,
            Err(SyncError::CalDav(CalDavError::Conflict)) => {
                // Server has newer version - fetch and overwrite local (server-wins)
                if let Err(e) = handle_conflict(pool, client, &task).await {
                    eprintln!("Failed to resolve conflict for task {}: {}", task.id, e);
                }
                stats.conflicts += 1;
            }
            Err(e) => {
                eprintln!("Failed to push task {}: {}", task.id, e);
            }
        }
    }

    Ok(stats)
}

enum PushAction {
    Created,
    Updated,
    Deleted,
}

/// Push a new task to the server.
async fn push_create(
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
async fn push_update(
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
async fn push_delete(
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
