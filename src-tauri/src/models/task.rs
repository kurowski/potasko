use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Sync status for a task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SyncStatus {
    #[default]
    Pending,
    Synced,
    Conflict,
    Deleted,
}

impl SyncStatus {
    /// Convert from database string value.
    pub fn from_str(s: &str) -> Self {
        match s {
            "synced" => Self::Synced,
            "conflict" => Self::Conflict,
            "deleted" => Self::Deleted,
            _ => Self::Pending,
        }
    }

    /// Convert to database string value.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Synced => "synced",
            Self::Conflict => "conflict",
            Self::Deleted => "deleted",
        }
    }
}

/// A task (maps to VTODO in CalDAV).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Database row ID.
    pub id: i64,
    /// Parent list ID.
    pub list_id: i64,
    /// iCalendar UID (unique identifier for sync).
    pub uid: String,

    // Core task fields
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    /// Priority 1-9 (1 = highest, 9 = lowest). None means no priority.
    pub priority: Option<i32>,
    pub completed: bool,
    pub completed_at: Option<DateTime<Utc>>,
    /// RFC 5545 RRULE string for recurrence.
    pub rrule: Option<String>,

    // CalDAV sync metadata
    /// Resource URL on the CalDAV server.
    pub caldav_href: Option<String>,
    /// ETag for conflict detection.
    pub caldav_etag: Option<String>,
    /// Raw iCalendar data to preserve unknown properties.
    pub raw_icalendar: Option<String>,

    // Change tracking
    /// Incremented on each local change.
    pub local_version: i64,
    /// Set to local_version after successful sync.
    pub synced_version: i64,
    pub sync_status: SyncStatus,

    // Sync error tracking
    /// Error message from last failed sync attempt.
    pub last_sync_error: Option<String>,
    /// Timestamp when next retry is allowed (for backoff).
    pub sync_retry_after: Option<DateTime<Utc>>,

    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Data for creating a new task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTask {
    pub list_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub priority: Option<i32>,
    pub rrule: Option<String>,
}

/// Data for updating an existing task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTask {
    pub title: Option<String>,
    pub description: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub priority: Option<i32>,
    pub rrule: Option<String>,
}
