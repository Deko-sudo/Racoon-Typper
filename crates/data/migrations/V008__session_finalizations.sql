-- V008__session_finalizations.sql — durable finalization claim and commit marker
--
-- This ledger records only the immutable intent association and the finalizer's
-- durable state. It intentionally does not persist any completion effects.

-- Makes the immutable V007 fingerprint addressable as part of a composite
-- foreign key without changing the accepted V007 table definition.
CREATE UNIQUE INDEX idx_session_completion_intents_session_fingerprint
    ON session_completion_intents (session_id, fingerprint);

CREATE TABLE session_finalizations (
    session_id TEXT PRIMARY KEY NOT NULL
        CHECK (typeof(session_id) = 'text' AND length(session_id) BETWEEN 1 AND 128)
        REFERENCES session_ledger(session_id) ON DELETE RESTRICT ON UPDATE RESTRICT,
    fingerprint TEXT NOT NULL
        CHECK (
            typeof(fingerprint) = 'text'
            AND length(fingerprint) = 64
            AND fingerprint NOT GLOB '*[^0-9a-f]*'
        ),
    state TEXT NOT NULL
        CHECK (state IN ('pending', 'committed', 'quarantined')),
    claimed_at TEXT NOT NULL
        CHECK (
            typeof(claimed_at) = 'text'
            AND length(claimed_at) >= 20
            AND substr(claimed_at, -1) = 'Z'
        ),
    committed_at TEXT
        CHECK (
            committed_at IS NULL OR (
                typeof(committed_at) = 'text'
                AND length(committed_at) >= 20
                AND substr(committed_at, -1) = 'Z'
            )
        ),
    quarantine_reason TEXT
        CHECK (quarantine_reason IS NULL OR quarantine_reason IN (
            'missing_completion_intent',
            'corrupt_durable_metadata',
            'fingerprint_mismatch',
            'invalid_finalization_state'
        )),
    CHECK (
        (state = 'pending' AND committed_at IS NULL AND quarantine_reason IS NULL)
        OR (state = 'committed' AND committed_at IS NOT NULL AND quarantine_reason IS NULL)
        OR (state = 'quarantined' AND committed_at IS NULL AND quarantine_reason IS NOT NULL)
    ),
    FOREIGN KEY (session_id, fingerprint)
        REFERENCES session_completion_intents(session_id, fingerprint)
        ON DELETE RESTRICT ON UPDATE RESTRICT
);

-- Diagnostic scans have a stable oldest-claim-first order without exposing
-- payload bytes.
CREATE INDEX idx_session_finalizations_diagnostic_order
    ON session_finalizations (claimed_at ASC, session_id ASC);

-- A record is initially an uncommitted claim. Quarantine and commit are
-- explicit transitions from that pending state only.
CREATE TRIGGER session_finalizations_initial_state_pending
BEFORE INSERT ON session_finalizations
WHEN NEW.state <> 'pending'
BEGIN
    SELECT RAISE(ABORT, 'session finalization must begin pending');
END;

-- Reject `OR REPLACE` before SQLite deletes the existing finalization row.
CREATE TRIGGER session_finalizations_not_replaceable
BEFORE INSERT ON session_finalizations
WHEN EXISTS (
    SELECT 1 FROM session_finalizations AS existing
    WHERE existing.session_id = NEW.session_id
)
BEGIN
    SELECT RAISE(ABORT, 'session finalizations cannot be replaced');
END;

CREATE TRIGGER session_finalizations_not_deletable
BEFORE DELETE ON session_finalizations
BEGIN
    SELECT RAISE(ABORT, 'session finalizations cannot be deleted');
END;

-- Identity and the original claim timestamp are immutable. This includes the
-- immutable V007 fingerprint association.
CREATE TRIGGER session_finalizations_identity_immutable
BEFORE UPDATE OF session_id, fingerprint, claimed_at ON session_finalizations
BEGIN
    SELECT RAISE(ABORT, 'session finalization identity is immutable');
END;

-- State transitions are intentionally small and terminal states cannot reopen.
-- A committed timestamp/reason is write-once and must match its state shape.
CREATE TRIGGER session_finalizations_transition_guard
BEFORE UPDATE ON session_finalizations
WHEN NOT (
    NEW.session_id = OLD.session_id
    AND NEW.fingerprint = OLD.fingerprint
    AND NEW.claimed_at = OLD.claimed_at
    AND (
        (OLD.state = 'pending' AND NEW.state = 'pending'
            AND NEW.committed_at IS NULL AND NEW.quarantine_reason IS NULL)
        OR (OLD.state = 'pending' AND NEW.state = 'committed'
            AND NEW.committed_at IS NOT NULL AND NEW.quarantine_reason IS NULL)
        OR (OLD.state = 'pending' AND NEW.state = 'quarantined'
            AND NEW.committed_at IS NULL AND NEW.quarantine_reason IS NOT NULL)
        OR (OLD.state = 'committed' AND NEW.state = 'committed'
            AND NEW.committed_at = OLD.committed_at AND NEW.quarantine_reason IS NULL)
        OR (OLD.state = 'quarantined' AND NEW.state = 'quarantined'
            AND NEW.committed_at IS NULL AND NEW.quarantine_reason = OLD.quarantine_reason)
    )
)
BEGIN
    SELECT RAISE(ABORT, 'invalid session finalization transition');
END;
