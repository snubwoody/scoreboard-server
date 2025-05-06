-- Add migration script here

ALTER TABLE session_tokens
ALTER COLUMN created_at
SET DEFAULT now();

ALTER TABLE refresh_tokens
ALTER COLUMN created_at
SET DEFAULT now();