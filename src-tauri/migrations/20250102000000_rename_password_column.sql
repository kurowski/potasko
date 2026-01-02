-- Rename password_encrypted to password for development simplicity.
-- Note: Password stored in plaintext for development.
-- TODO: Implement proper credential storage before release.

-- Requires SQLite 3.25.0+ (2018) for RENAME COLUMN support.
ALTER TABLE accounts RENAME COLUMN password_encrypted TO password;

-- Make password NOT NULL (it was optional before, now required)
-- SQLite doesn't support ALTER COLUMN, so we recreate if needed.
-- For now, the column remains nullable since existing rows may have NULL.
