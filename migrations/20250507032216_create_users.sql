-- Add migration script here

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE users(
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    email TEXT NULL UNIQUE,
    created_at TIMESTAMPTZ DEFAULT now() NOT NULL,
    user_name TEXT NULL,
    phone_number TEXT NULL,
    encrypted_password TEXT NULL,
    is_anonymous BOOLEAN NOT NULL DEFAULT false
);


ALTER TABLE users
ADD CONSTRAINT non_anon_users_have_passwords
CHECK(
    is_anonymous OR encrypted_password IS NOT NULL
);


COMMENT ON COLUMN users.encrypted_password IS 'Hashed password, this field is required if the user is not null';

