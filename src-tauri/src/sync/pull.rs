//! Pull server changes to the local database.

use std::collections::HashSet;

use sqlx::SqlitePool;

use crate::caldav::{parse_vtodo, CalDavClient};
use crate::core::tasks as core_tasks;
use crate::models::SyncStatus;

use super::log::log_sync_operation;
use super::types::{PullStats, SyncError};

/// Pull changes from the server.
pub async fn pull_changes(
    pool: &SqlitePool,
    client: &CalDavClient,
    list_id: i64,
    calendar_url: &str,
    current_ctag: Option<&str>,
    account_id: Option<i64>,
) -> Result<PullStats, SyncError> {
    let mut stats = PullStats::default();

    // 1. Get current CTag from server
    let server_ctag = client.get_ctag(calendar_url).await?;

    // 2. Compare CTags - if unchanged, nothing to do
    if let (Some(current), Some(server)) = (current_ctag, server_ctag.as_deref()) {
        if current == server {
            stats.new_ctag = Some(server.to_string());
            return Ok(stats);
        }
    }

    // 3. Get list of all resources with ETags from server
    let server_resources = client.list_resources(calendar_url).await?;

    // 4. Get local tasks indexed by href
    let local_tasks = core_tasks::get_tasks_by_href(list_id, pool).await?;

    // 5. Determine what changed by comparing ETags
    let mut hrefs_to_fetch: Vec<String> = Vec::new();
    let mut server_hrefs: HashSet<String> = HashSet::new();

    for resource in &server_resources {
        server_hrefs.insert(resource.href.clone());

        match local_tasks.get(&resource.href) {
            Some(local_task) => {
                // Exists locally - check if ETag changed
                if local_task.caldav_etag.as_deref() != resource.etag.as_deref() {
                    // Changed on server - need to fetch
                    // But skip if we have local pending changes (will be handled in push)
                    if local_task.local_version <= local_task.synced_version {
                        hrefs_to_fetch.push(resource.href.clone());
                    }
                }
            }
            None => {
                // New on server
                hrefs_to_fetch.push(resource.href.clone());
            }
        }
    }

    // 6. Detect deletions (in local but not on server)
    for (href, task) in &local_tasks {
        if !server_hrefs.contains(href) {
            // Only delete if the task was synced (not pending local changes)
            if task.sync_status == SyncStatus::Synced {
                core_tasks::hard_delete_task(task.id, pool).await?;
                stats.deleted += 1;
                let _ = log_sync_operation(
                    pool,
                    account_id,
                    Some(list_id),
                    Some(task.id),
                    "pull_delete",
                    "success",
                    None,
                    None,
                )
                .await;
            }
        }
    }

    // 7. Fetch changed resources via multiget
    if !hrefs_to_fetch.is_empty() {
        let href_refs: Vec<&str> = hrefs_to_fetch.iter().map(|s| s.as_str()).collect();
        let resources = client.multiget(calendar_url, &href_refs).await?;

        for resource in resources {
            if let Some(ical) = resource.icalendar {
                let parsed = match parse_vtodo(&ical) {
                    Ok(p) => p,
                    Err(e) => {
                        let msg = format!("Failed to parse VTODO from {}: {}", resource.href, e);
                        eprintln!("{}", msg);
                        let _ = log_sync_operation(
                            pool,
                            account_id,
                            Some(list_id),
                            None,
                            "pull_parse_error",
                            "error",
                            Some(&msg),
                            None,
                        )
                        .await;
                        continue;
                    }
                };

                let due_date = parsed.due_date_rfc3339();
                let completed_at = parsed.completed_at_rfc3339();

                if local_tasks.contains_key(&resource.href) {
                    // Update existing
                    let task_id = local_tasks.get(&resource.href).map(|t| t.id);
                    core_tasks::update_task_by_href_from_server(
                        &resource.href,
                        parsed.title(),
                        parsed.description.as_deref(),
                        due_date.as_deref(),
                        parsed.priority,
                        parsed.completed,
                        completed_at.as_deref(),
                        parsed.rrule.as_deref(),
                        resource.etag.as_deref(),
                        &ical,
                        pool,
                    )
                    .await?;
                    stats.updated += 1;
                    let _ = log_sync_operation(
                        pool,
                        account_id,
                        Some(list_id),
                        task_id,
                        "pull_update",
                        "success",
                        None,
                        None,
                    )
                    .await;
                } else {
                    // Create new
                    let fallback_uid = uuid::Uuid::new_v4().to_string();
                    let uid = parsed.uid.as_deref().unwrap_or(&fallback_uid);
                    core_tasks::create_task_from_server(
                        list_id,
                        uid,
                        &resource.href,
                        parsed.title(),
                        parsed.description.as_deref(),
                        due_date.as_deref(),
                        parsed.priority,
                        parsed.completed,
                        completed_at.as_deref(),
                        parsed.rrule.as_deref(),
                        resource.etag.as_deref(),
                        &ical,
                        pool,
                    )
                    .await?;
                    stats.created += 1;
                    let _ = log_sync_operation(
                        pool,
                        account_id,
                        Some(list_id),
                        None,
                        "pull_create",
                        "success",
                        None,
                        None,
                    )
                    .await;
                }
            }
        }
    }

    stats.new_ctag = server_ctag;
    Ok(stats)
}
