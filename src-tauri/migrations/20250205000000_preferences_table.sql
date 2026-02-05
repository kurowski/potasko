-- Preferences table for storing user settings
CREATE TABLE IF NOT EXISTS preferences (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Insert default value for show_local_accounts
INSERT INTO preferences (key, value, updated_at)
VALUES ('show_local_accounts', 'true', datetime('now'));
