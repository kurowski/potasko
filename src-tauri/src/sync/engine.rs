//! Sync engine: orchestrates push and pull operations.

use sqlx::SqlitePool;

use crate::caldav::CalDavClient;
use crate::core::{accounts as core_accounts, lists as core_lists};
use crate::models::TaskList;

use super::pull::pull_changes;
use super::push::push_changes;
use super::types::{SyncError, SyncResult, SyncStats};

/// Sync engine for a database connection.
pub struct SyncEngine {
    pool: SqlitePool,
}

impl SyncEngine {
    /// Create a new sync engine.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Sync a single list with its CalDAV calendar.
    pub async fn sync_list(&self, list_id: i64) -> SyncResult {
        let mut stats = SyncStats::default();

        // 1. Get list
        let list = match core_lists::get_list(list_id, &self.pool).await {
            Ok(l) => l,
            Err(e) => return SyncResult::failure(list_id, e.to_string()),
        };

        // 2. Check if list has CalDAV URL
        let caldav_url = match &list.caldav_url {
            Some(url) => url.clone(),
            None => return SyncResult::failure(list_id, "No CalDAV URL for list"),
        };

        // 3. Get account for this list
        let account = match self.get_account_for_list(&list).await {
            Ok(a) => a,
            Err(e) => return SyncResult::failure(list_id, e.to_string()),
        };

        // 4. Create CalDAV client
        let client = match CalDavClient::new(&account.server_url, &account.username, &account.password) {
            Ok(c) => c,
            Err(e) => return SyncResult::failure(list_id, e.to_string()),
        };

        // 5. Push local changes first
        match push_changes(&self.pool, &client, list_id, &caldav_url).await {
            Ok(push_stats) => {
                stats.pushed_created = push_stats.created;
                stats.pushed_updated = push_stats.updated;
                stats.pushed_deleted = push_stats.deleted;
                stats.conflicts = push_stats.conflicts;
            }
            Err(e) => {
                stats.errors.push(format!("Push failed: {}", e));
            }
        }

        // 6. Pull server changes
        match pull_changes(&self.pool, &client, list_id, &caldav_url, list.ctag.as_deref()).await {
            Ok(pull_stats) => {
                stats.pulled_created = pull_stats.created;
                stats.pulled_updated = pull_stats.updated;
                stats.pulled_deleted = pull_stats.deleted;

                // Update list ctag
                if let Some(new_ctag) = pull_stats.new_ctag {
                    if let Err(e) = self.update_list_ctag(list_id, &new_ctag).await {
                        stats.errors.push(format!("Failed to update ctag: {}", e));
                    }
                }
            }
            Err(e) => {
                stats.errors.push(format!("Pull failed: {}", e));
            }
        }

        if stats.errors.is_empty() {
            SyncResult::success(list_id, stats)
        } else {
            let error_msg = stats.errors.join("; ");
            SyncResult {
                success: false,
                list_id,
                stats,
                error: Some(error_msg),
            }
        }
    }

    /// Initial download: fetch all VTODOs from a calendar (skip push).
    pub async fn initial_download(&self, list_id: i64) -> SyncResult {
        let mut stats = SyncStats::default();

        // 1. Get list
        let list = match core_lists::get_list(list_id, &self.pool).await {
            Ok(l) => l,
            Err(e) => return SyncResult::failure(list_id, e.to_string()),
        };

        // 2. Check if list has CalDAV URL
        let caldav_url = match &list.caldav_url {
            Some(url) => url.clone(),
            None => return SyncResult::failure(list_id, "No CalDAV URL for list"),
        };

        // 3. Get account for this list
        let account = match self.get_account_for_list(&list).await {
            Ok(a) => a,
            Err(e) => return SyncResult::failure(list_id, e.to_string()),
        };

        // 4. Create CalDAV client
        let client = match CalDavClient::new(&account.server_url, &account.username, &account.password) {
            Ok(c) => c,
            Err(e) => return SyncResult::failure(list_id, e.to_string()),
        };

        // 5. Pull only (no ctag check for initial download)
        match pull_changes(&self.pool, &client, list_id, &caldav_url, None).await {
            Ok(pull_stats) => {
                stats.pulled_created = pull_stats.created;
                stats.pulled_updated = pull_stats.updated;
                stats.pulled_deleted = pull_stats.deleted;

                // Update list ctag
                if let Some(new_ctag) = pull_stats.new_ctag {
                    if let Err(e) = self.update_list_ctag(list_id, &new_ctag).await {
                        stats.errors.push(format!("Failed to update ctag: {}", e));
                    }
                }
            }
            Err(e) => {
                return SyncResult::failure(list_id, e.to_string());
            }
        }

        SyncResult::success(list_id, stats)
    }

    /// Get the account for a list.
    async fn get_account_for_list(&self, list: &TaskList) -> Result<crate::models::Account, SyncError> {
        let account_id = list.account_id.ok_or(SyncError::NoAccount(list.id))?;

        let account = core_accounts::get_account(account_id, &self.pool)
            .await
            .map_err(|e| SyncError::Database(sqlx::Error::Protocol(e.to_string())))?;

        Ok(account)
    }

    /// Update the ctag for a list.
    async fn update_list_ctag(&self, list_id: i64, ctag: &str) -> Result<(), SyncError> {
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query!(
            "UPDATE task_lists SET ctag = ?, updated_at = ? WHERE id = ?",
            ctag,
            now,
            list_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
