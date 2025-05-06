-- Add migration script here
ALTER TABLE users
ADD COLUMN encrypted_password TEXT NULL,
ADD COLUMN created_at TIMESTAMPTZ NOT NULL DEFAULT now();