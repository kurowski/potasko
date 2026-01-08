//! Sync types: errors, stats, and results.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during sync.
#[derive(Error, Debug)]
pub enum SyncError {
    #[error("CalDAV error: {0}")]
    CalDav(#[from] crate::caldav::CalDavError),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("VTODO error: {0}")]
    VTodo(#[from] crate::caldav::VTodoError),

    #[error("Core error: {0}")]
    Core(#[from] crate::core::error::CoreError),

    #[error("No account configured for list {0}")]
    NoAccount(i64),

    #[error("List {0} has no CalDAV URL")]
    NoCalDavUrl(i64),

    #[error("Account {0} not found")]
    AccountNotFound(i64),
}

/// Statistics from a sync operation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncStats {
    pub pushed_created: u32,
    pub pushed_updated: u32,
    pub pushed_deleted: u32,
    pub pulled_created: u32,
    pub pulled_updated: u32,
    pub pulled_deleted: u32,
    pub conflicts: u32,
    pub errors: Vec<String>,
}

/// Result of syncing a single list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub success: bool,
    pub list_id: i64,
    pub stats: SyncStats,
    pub error: Option<String>,
}

impl SyncResult {
    pub fn success(list_id: i64, stats: SyncStats) -> Self {
        Self {
            success: true,
            list_id,
            stats,
            error: None,
        }
    }

    pub fn failure(list_id: i64, error: impl Into<String>) -> Self {
        Self {
            success: false,
            list_id,
            stats: SyncStats::default(),
            error: Some(error.into()),
        }
    }
}

/// Stats from push phase.
#[derive(Debug, Default)]
pub struct PushStats {
    pub created: u32,
    pub updated: u32,
    pub deleted: u32,
    pub conflicts: u32,
}

/// Stats from pull phase.
#[derive(Debug, Default)]
pub struct PullStats {
    pub created: u32,
    pub updated: u32,
    pub deleted: u32,
    pub new_ctag: Option<String>,
}

/// Result of syncing an entire account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSyncResult {
    pub success: bool,
    pub account_id: i64,
    /// Number of new calendars discovered and imported as lists.
    pub calendars_imported: u32,
    /// Results for each list synced.
    pub list_results: Vec<SyncResult>,
    pub error: Option<String>,
}

impl AccountSyncResult {
    pub fn failure(account_id: i64, error: impl Into<String>) -> Self {
        Self {
            success: false,
            account_id,
            calendars_imported: 0,
            list_results: Vec::new(),
            error: Some(error.into()),
        }
    }
}

/// Result of pushing a single task.
#[derive(Debug)]
pub struct PushTaskResult {
    pub success: bool,
    pub task_id: i64,
    /// Action taken: "created", "updated", "deleted", or "no_change"
    pub action: Option<String>,
    pub error: Option<String>,
}

impl PushTaskResult {
    pub fn success(task_id: i64, action: &str) -> Self {
        Self {
            success: true,
            task_id,
            action: Some(action.to_string()),
            error: None,
        }
    }

    pub fn failure(task_id: i64, error: impl ToString) -> Self {
        Self {
            success: false,
            task_id,
            action: None,
            error: Some(error.to_string()),
        }
    }
}
