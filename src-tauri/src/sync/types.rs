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
