-- V007__session_completion_intents.sql — immutable canonical completion intents
--
-- SQL stores opaque canonical bytes only. Application contracts remain the
-- sole authority for JSON schema, canonicalization, and fingerprint validity.
CREATE TABLE session_completion_intents (
    session_id TEXT PRIMARY KEY NOT NULL
        REFERENCES session_ledger(session_id) ON DELETE RESTRICT,
    canonicalization_version INTEGER NOT NULL
        CHECK (typeof(canonicalization_version) = 'integer' AND canonicalization_version >= 0),
    payload_version INTEGER NOT NULL
        CHECK (typeof(payload_version) = 'integer' AND payload_version >= 0),
    fingerprint TEXT NOT NULL
        CHECK (
            typeof(fingerprint) = 'text'
            AND length(fingerprint) = 64
            AND fingerprint NOT GLOB '*[^0-9a-f]*'
        ),
    canonical_payload BLOB NOT NULL
        CHECK (
            typeof(canonical_payload) = 'blob'
            AND length(canonical_payload) <= 8388608
        ),
    payload_byte_length INTEGER NOT NULL
        CHECK (
            typeof(payload_byte_length) = 'integer'
            AND payload_byte_length BETWEEN 0 AND 8388608
            AND payload_byte_length = length(canonical_payload)
        ),
    created_at TEXT NOT NULL
        CHECK (
            typeof(created_at) = 'text'
            AND length(created_at) >= 20
            AND substr(created_at, -1) = 'Z'
        )
);

-- A completion intent is a write-once business fact. Idempotency is handled
-- by the adapter through comparison, never replacement.
CREATE TRIGGER session_completion_intents_immutable
BEFORE UPDATE ON session_completion_intents
BEGIN
    SELECT RAISE(ABORT, 'session completion intents are immutable');
END;

-- Reject a duplicate insert before SQLite can apply `OR REPLACE` conflict
-- resolution (which otherwise deletes the old row and inserts a new one).
CREATE TRIGGER session_completion_intents_not_replaceable
BEFORE INSERT ON session_completion_intents
WHEN EXISTS (
    SELECT 1
    FROM session_completion_intents AS existing
    WHERE existing.session_id = NEW.session_id
)
BEGIN
    SELECT RAISE(ABORT, 'session completion intents cannot be replaced');
END;

-- An immutable intent is retained for corruption isolation and idempotency;
-- it must not be removed directly or through replacement semantics.
CREATE TRIGGER session_completion_intents_not_deletable
BEFORE DELETE ON session_completion_intents
BEGIN
    SELECT RAISE(ABORT, 'session completion intents cannot be deleted');
END;
