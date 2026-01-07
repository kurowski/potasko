//! Sync engine: orchestrates push and pull operations.

use sqlx::SqlitePool;

use crate::caldav::CalDavClient;
use crate::core::{accounts as core_accounts, lists as core_lists};
use crate::models::TaskList;

use super::pull::pull_changes;
use super::push::push_changes;
use super::types::{AccountSyncResult, SyncError, SyncResult, SyncStats};

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

    /// Sync an entire account: discover calendars, import new ones, sync all lists.
    pub async fn sync_account(&self, account_id: i64) -> AccountSyncResult {
        // 1. Get account
        let account = match core_accounts::get_account(account_id, &self.pool).await {
            Ok(a) => a,
            Err(e) => return AccountSyncResult::failure(account_id, e.to_string()),
        };

        // 2. Create CalDAV client
        let client = match CalDavClient::new(&account.server_url, &account.username, &account.password) {
            Ok(c) => c,
            Err(e) => return AccountSyncResult::failure(account_id, e.to_string()),
        };

        // 3. Discover calendars
        let calendars = match client.discover().await {
            Ok((_, _, cals)) => cals,
            Err(e) => return AccountSyncResult::failure(account_id, format!("Discovery failed: {}", e)),
        };

        // 4. Import new calendars as lists
        let mut calendars_imported = 0u32;
        for cal in calendars.iter().filter(|c| c.supports_vtodo) {
            // Resolve calendar href to absolute URL
            let caldav_url = self.resolve_calendar_url(&client.base_url, &cal.href);

            // Check if list already exists for this calendar
            let exists = self.list_exists_for_calendar(account_id, &caldav_url).await;
            if !exists {
                let name = cal.display_name.as_deref().unwrap_or("Untitled");
                let color = cal.color.as_deref();

                match core_lists::create_list_from_calendar(account_id, &caldav_url, name, color, &self.pool).await {
                    Ok(_) => {
                        calendars_imported += 1;
                        println!("Imported calendar: {}", name);
                    }
                    Err(e) => {
                        eprintln!("Failed to import calendar '{}': {}", name, e);
                    }
                }
            }
        }

        // 5. Get all lists for this account and sync them
        let lists = match self.get_lists_for_account(account_id).await {
            Ok(l) => l,
            Err(e) => return AccountSyncResult::failure(account_id, format!("Failed to get lists: {}", e)),
        };

        let mut list_results = Vec::new();
        let mut all_success = true;

        for list in lists {
            let result = self.sync_list(list.id).await;
            if !result.success {
                all_success = false;
            }
            list_results.push(result);
        }

        AccountSyncResult {
            success: all_success,
            account_id,
            calendars_imported,
            list_results,
            error: None,
        }
    }

    /// Check if a list already exists for a given CalDAV URL.
    async fn list_exists_for_calendar(&self, account_id: i64, caldav_url: &str) -> bool {
        let result = sqlx::query!(
            "SELECT id FROM task_lists WHERE account_id = ? AND caldav_url = ?",
            account_id,
            caldav_url
        )
        .fetch_optional(&self.pool)
        .await;

        matches!(result, Ok(Some(_)))
    }

    /// Get all lists for an account.
    async fn get_lists_for_account(&self, account_id: i64) -> Result<Vec<TaskList>, SyncError> {
        // Use core function but filter by account_id
        let all_lists = core_lists::get_lists(&self.pool).await?;
        Ok(all_lists.into_iter().filter(|l| l.account_id == Some(account_id)).collect())
    }

    /// Resolve a calendar href to an absolute URL.
    fn resolve_calendar_url(&self, base_url: &str, href: &str) -> String {
        if href.starts_with("http://") || href.starts_with("https://") {
            href.to_string()
        } else if href.starts_with('/') {
            // Extract scheme + host from base URL
            if let Ok(url) = reqwest::Url::parse(base_url) {
                let host = url.host_str().unwrap_or("");
                let port_part = url.port().map(|p| format!(":{}", p)).unwrap_or_default();
                format!("{}://{}{}{}", url.scheme(), host, port_part, href)
            } else {
                format!("{}{}", base_url, href)
            }
        } else {
            format!("{}/{}", base_url, href)
        }
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
