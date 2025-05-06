-- Add migration script here
ALTER TABLE session_tokens
ALTER COLUMN created_at
TYPE TIMESTAMPTZ;