use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A task list (maps to a CalDAV calendar with VTODO support).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskList {
    /// Database row ID.
    pub id: i64,
    /// Associated CalDAV account (None for local-only lists).
    pub account_id: Option<i64>,
    pub name: String,
    /// Hex color code (e.g., "#ff0000").
    pub color: Option<String>,

    // CalDAV sync metadata
    /// Calendar URL on the CalDAV server.
    pub caldav_url: Option<String>,
    /// Collection tag for change detection.
    pub ctag: Option<String>,
    /// Sync token for incremental sync.
    pub sync_token: Option<String>,

    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Data for creating a new task list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskList {
    pub name: String,
    pub color: Option<String>,
    /// If provided, the list will be created on the CalDAV server for this account.
    pub account_id: Option<i64>,
}

/// Data for updating an existing task list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskList {
    pub name: Option<String>,
    pub color: Option<String>,
}
