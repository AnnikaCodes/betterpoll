-- PostgreSQL schema for bettervote

BEGIN TRANSACTION;

CREATE TABLE polls (
    -- Doubles as the URL
    id TEXT NOT NULL PRIMARY KEY,
    title TEXT NOT NULL,
    candidates TEXT[] NOT NULL,
    prohibit_double_vote_by_ip BOOLEAN NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    num_winners INTEGER NOT NULL,
    -- 0 for Schulze
    method INTEGER NOT NULL
);

CREATE TABLE votes (
    poll_id TEXT NOT NULL REFERENCES polls(id),
    voter_ip INET NOT NULL,
    -- Reserved for future use
    voter_fingerprint TEXT,
    -- 1st choice is in preferences[0], etc.
    preferences TEXT[] NOT NULL
);

CREATE TABLE db_info (
    version INTEGER NOT NULL
);

INSERT INTO db_info (version) VALUES (1);

COMMIT;
