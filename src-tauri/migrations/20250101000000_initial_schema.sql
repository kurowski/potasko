-- Initial schema for Potasko

-- CalDAV server credentials
CREATE TABLE IF NOT EXISTS accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    server_url TEXT NOT NULL,
    username TEXT NOT NULL,
    password_encrypted TEXT,
    principal_url TEXT,
    calendar_home_url TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Task lists (maps to CalDAV calendars)
CREATE TABLE IF NOT EXISTS task_lists (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER REFERENCES accounts(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    color TEXT,
    caldav_url TEXT,
    ctag TEXT,
    sync_token TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Tasks (maps to VTODO)
CREATE TABLE IF NOT EXISTS tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    list_id INTEGER NOT NULL REFERENCES task_lists(id) ON DELETE CASCADE,
    uid TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    due_date TEXT,
    priority INTEGER CHECK (priority IS NULL OR (priority >= 1 AND priority <= 9)),
    completed INTEGER NOT NULL DEFAULT 0,
    completed_at TEXT,
    rrule TEXT,
    caldav_href TEXT,
    caldav_etag TEXT,
    raw_icalendar TEXT,
    local_version INTEGER NOT NULL DEFAULT 1,
    synced_version INTEGER NOT NULL DEFAULT 0,
    sync_status TEXT NOT NULL DEFAULT 'pending',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(list_id, uid)
);

-- Sync audit log
CREATE TABLE IF NOT EXISTS sync_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER REFERENCES accounts(id) ON DELETE CASCADE,
    list_id INTEGER REFERENCES task_lists(id) ON DELETE CASCADE,
    task_id INTEGER REFERENCES tasks(id) ON DELETE SET NULL,
    operation TEXT NOT NULL,
    status TEXT NOT NULL,
    message TEXT,
    http_status INTEGER,
    timestamp TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_tasks_list_id ON tasks(list_id);
CREATE INDEX IF NOT EXISTS idx_tasks_sync_status ON tasks(sync_status);
CREATE INDEX IF NOT EXISTS idx_tasks_due_date ON tasks(due_date);
CREATE INDEX IF NOT EXISTS idx_task_lists_account_id ON task_lists(account_id);
CREATE INDEX IF NOT EXISTS idx_sync_log_timestamp ON sync_log(timestamp);

-- Default inbox list
INSERT INTO task_lists (name, created_at, updated_at)
VALUES ('Inbox', datetime('now'), datetime('now'));
