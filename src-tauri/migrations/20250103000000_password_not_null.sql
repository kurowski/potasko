-- Make password NOT NULL since CalDAV connections require it.
-- SQLite doesn't support ALTER COLUMN, so we recreate the table.

-- Step 1: Create new table with NOT NULL constraint
CREATE TABLE accounts_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    server_url TEXT NOT NULL,
    username TEXT NOT NULL,
    password TEXT NOT NULL DEFAULT '',
    principal_url TEXT,
    calendar_home_url TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Step 2: Copy data (use empty string for any NULL passwords)
INSERT INTO accounts_new (id, name, server_url, username, password, principal_url, calendar_home_url, created_at, updated_at)
SELECT id, name, server_url, username, COALESCE(password, ''), principal_url, calendar_home_url, created_at, updated_at
FROM accounts;

-- Step 3: Drop old table
DROP TABLE accounts;

-- Step 4: Rename new table
ALTER TABLE accounts_new RENAME TO accounts;
