-- Add migration script here

CREATE TABLE leaderboards(
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE leaderboard_members(
    id SERIAL PRIMARY KEY,
    leaderboard INTEGER NOT NULL REFERENCES leaderboards(id),
    player_alias TEXT NULL,
    player UUID NOT NULL REFERENCES users(id)
);

CREATE TABLE points(
    id SERIAL PRIMARY KEY,
    leaderboard INTEGER NOT NULL REFERENCES leaderboards(id),
    value NUMERIC(10,4) NOT NULL,
    player UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);