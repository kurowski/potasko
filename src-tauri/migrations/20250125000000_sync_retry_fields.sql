-- Add fields for sync error tracking and retry backoff
ALTER TABLE tasks ADD COLUMN last_sync_error TEXT;
ALTER TABLE tasks ADD COLUMN sync_retry_after TEXT;
