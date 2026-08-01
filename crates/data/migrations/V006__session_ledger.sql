-- V006__session_ledger.sql — durable recovery lifecycle records
--
-- This table is intentionally separate from completed historical tests. It
-- records only newly accepted recoverable sessions; no historical backfill is
-- performed by this additive migration.
CREATE TABLE session_ledger (
    session_id TEXT PRIMARY KEY NOT NULL
        CHECK (typeof(session_id) = 'text' AND length(session_id) BETWEEN 1 AND 128),
    state TEXT NOT NULL
        CHECK (state IN (
            'running',
            'awaiting_persistence',
            'finalization_pending',
            'finalized',
            'aborted',
            'interrupted',
            'quarantined'
        )),
    mode_type TEXT NOT NULL
        CHECK (typeof(mode_type) = 'text' AND length(mode_type) BETWEEN 1 AND 64),
    -- StartedSession requires this to be sanitized metadata, never typed text.
    mode_descriptor TEXT NOT NULL
        CHECK (
            typeof(mode_descriptor) = 'text'
            AND length(mode_descriptor) <= 16384
            AND json_valid(mode_descriptor)
        ),
    language TEXT NOT NULL
        CHECK (typeof(language) = 'text' AND length(language) BETWEEN 1 AND 64),
    created_at TEXT NOT NULL
        CHECK (
            typeof(created_at) = 'text'
            AND length(created_at) >= 20
            AND substr(created_at, -1) = 'Z'
        ),
    updated_at TEXT NOT NULL
        CHECK (
            typeof(updated_at) = 'text'
            AND length(updated_at) >= 20
            AND substr(updated_at, -1) = 'Z'
        ),
    interruption_reason TEXT
        CHECK (interruption_reason IS NULL OR interruption_reason = 'process_restart'),
    abort_reason TEXT
        CHECK (abort_reason IS NULL OR abort_reason = 'explicit_abort'),
    quarantine_reason TEXT
        CHECK (quarantine_reason IS NULL OR quarantine_reason IN (
            'unsupported_canonicalization_version',
            'unsupported_intent_version',
            'corrupt_completion_payload',
            'missing_completion_intent',
            'conflicting_completion_intent',
            'invalid_state_record',
            'fingerprint_mismatch',
            'inconsistent_durable_metadata'
        )),
    CHECK (interruption_reason IS NULL OR length(interruption_reason) <= 128),
    CHECK (abort_reason IS NULL OR length(abort_reason) <= 128),
    CHECK (quarantine_reason IS NULL OR length(quarantine_reason) <= 128)
);

-- Candidate ordering is explicit: oldest accepted session first, with the
-- stable session identity as the tie-breaker.
CREATE INDEX idx_session_ledger_recovery_order
    ON session_ledger (created_at ASC, session_id ASC);

CREATE INDEX idx_session_ledger_state_recovery_order
    ON session_ledger (state ASC, created_at ASC, session_id ASC);

-- SQLite `INSERT OR REPLACE` resolves a primary-key conflict by deleting the
-- old row and inserting a new one. Reject the replacement before conflict
-- resolution when the existing record is terminal, so it cannot bypass the
-- update/delete guards below.
CREATE TRIGGER session_ledger_terminal_row_not_replaceable
BEFORE INSERT ON session_ledger
WHEN EXISTS (
    SELECT 1
    FROM session_ledger AS existing
    WHERE existing.session_id = NEW.session_id
      AND existing.state IN ('finalized', 'aborted', 'interrupted', 'quarantined')
)
BEGIN
    SELECT RAISE(ABORT, 'terminal session ledger row cannot be replaced');
END;

-- Terminal records are retained as historical recovery decisions and must
-- never be reopened by a later adapter write.
CREATE TRIGGER session_ledger_terminal_state_immutable
BEFORE UPDATE OF state ON session_ledger
WHEN OLD.state IN ('finalized', 'aborted', 'interrupted', 'quarantined')
     AND NEW.state <> OLD.state
BEGIN
    SELECT RAISE(ABORT, 'terminal session ledger state cannot be reopened');
END;

-- Preserve terminal recovery decisions even when a caller attempts a direct
-- delete or an SQLite replacement path.
CREATE TRIGGER session_ledger_terminal_row_not_deletable
BEFORE DELETE ON session_ledger
WHEN OLD.state IN ('finalized', 'aborted', 'interrupted', 'quarantined')
BEGIN
    SELECT RAISE(ABORT, 'terminal session ledger row cannot be deleted');
END;
