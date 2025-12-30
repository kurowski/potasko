/// SQL statements for creating database tables.

/// Creates the `accounts` table for CalDAV server credentials.
pub const CREATE_ACCOUNTS: &str = r#"
CREATE TABLE IF NOT EXISTS accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    server_url TEXT NOT NULL,
    username TEXT NOT NULL,
    -- Password stored encrypted (handled at application level)
    password_encrypted TEXT,
    -- CalDAV discovery URLs (populated after successful discovery)
    principal_url TEXT,
    calendar_home_url TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
)
"#;

/// Creates the `task_lists` table for local lists mapped to CalDAV calendars.
pub const CREATE_TASK_LISTS: &str = r#"
CREATE TABLE IF NOT EXISTS task_lists (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER REFERENCES accounts(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    color TEXT,
    -- CalDAV sync metadata
    caldav_url TEXT,
    ctag TEXT,
    sync_token TEXT,
    -- Timestamps
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
)
"#;

/// Creates the `tasks` table for VTODO data with sync metadata.
pub const CREATE_TASKS: &str = r#"
CREATE TABLE IF NOT EXISTS tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    list_id INTEGER NOT NULL REFERENCES task_lists(id) ON DELETE CASCADE,
    -- iCalendar UID (unique identifier for sync)
    uid TEXT NOT NULL,
    -- Core task fields
    title TEXT NOT NULL,
    description TEXT,
    due_date TEXT,
    priority INTEGER CHECK (priority IS NULL OR (priority >= 1 AND priority <= 9)),
    completed INTEGER NOT NULL DEFAULT 0,
    completed_at TEXT,
    -- Recurrence rule (RFC 5545 RRULE string)
    rrule TEXT,
    -- CalDAV sync metadata
    caldav_href TEXT,
    caldav_etag TEXT,
    -- Preserve unknown iCalendar properties for round-trip
    raw_icalendar TEXT,
    -- Change tracking for sync
    local_version INTEGER NOT NULL DEFAULT 1,
    synced_version INTEGER NOT NULL DEFAULT 0,
    -- pending | synced | conflict | deleted
    sync_status TEXT NOT NULL DEFAULT 'pending',
    -- Timestamps
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    -- Ensure UID is unique within a list
    UNIQUE(list_id, uid)
)
"#;

/// Creates the `sync_log` table for audit trail and debugging.
pub const CREATE_SYNC_LOG: &str = r#"
CREATE TABLE IF NOT EXISTS sync_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER REFERENCES accounts(id) ON DELETE CASCADE,
    list_id INTEGER REFERENCES task_lists(id) ON DELETE CASCADE,
    task_id INTEGER REFERENCES tasks(id) ON DELETE SET NULL,
    -- Operation type: push_create, push_update, push_delete, pull_create, pull_update, pull_delete, conflict
    operation TEXT NOT NULL,
    -- success | error
    status TEXT NOT NULL,
    -- Human-readable message or error details
    message TEXT,
    -- HTTP status code if applicable
    http_status INTEGER,
    timestamp TEXT NOT NULL DEFAULT (datetime('now'))
)
"#;

/// Creates indexes for common query patterns.
pub const CREATE_INDEXES: &str = r#"
CREATE INDEX IF NOT EXISTS idx_tasks_list_id ON tasks(list_id);
CREATE INDEX IF NOT EXISTS idx_tasks_sync_status ON tasks(sync_status);
CREATE INDEX IF NOT EXISTS idx_tasks_due_date ON tasks(due_date);
CREATE INDEX IF NOT EXISTS idx_task_lists_account_id ON task_lists(account_id);
CREATE INDEX IF NOT EXISTS idx_sync_log_timestamp ON sync_log(timestamp);
"#;

/// All table creation statements in order (respecting foreign keys).
pub const ALL_TABLES: &[&str] = &[
    CREATE_ACCOUNTS,
    CREATE_TASK_LISTS,
    CREATE_TASKS,
    CREATE_SYNC_LOG,
    CREATE_INDEXES,
];
