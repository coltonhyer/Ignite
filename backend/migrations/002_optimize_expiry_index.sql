DROP INDEX IF EXISTS idx_secrets_expires_at;

CREATE INDEX IF NOT EXISTS idx_secrets_expires_at_datetime
ON secrets(datetime(expires_at));
