CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE users(
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY NOT NULL,
    email TEXT UNIQUE NULL,
    user_name TEXT NULL
);

CREATE TABLE session_tokens(
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    token TEXT UNIQUE NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMP NOT NULL,
    user_id UUID NOT NULL  REFERENCES users(id)
);

CREATE TABLE refresh_tokens(
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    token TEXT UNIQUE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    session_id UUID NOT NULL REFERENCES session_tokens(id),
    active BOOLEAN NOT NULL
);