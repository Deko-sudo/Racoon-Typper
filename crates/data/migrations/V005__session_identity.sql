-- V005__session_identity.sql — durable backend-owned session identities

-- Keep the existing integer row id as the relational key used by replays and
-- personal-best references. The durable identity is additive so old data and
-- replay foreign keys remain valid.
ALTER TABLE tests ADD COLUMN session_id TEXT;

-- Existing rows predate durable identities. Their deterministic compatibility
-- values are stable across restarts and never collide with generated UUIDv7s.
UPDATE tests
SET session_id = printf('legacy-test-%016x', id)
WHERE session_id IS NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_tests_session_id ON tests(session_id);

-- Session identity is immutable once a completed test is recorded. SQLite
-- cannot add NOT NULL/immutable constraints with ALTER TABLE, so equivalent
-- guards keep the additive migration safe without rebuilding the table.
CREATE TRIGGER IF NOT EXISTS tests_session_id_required
BEFORE INSERT ON tests
WHEN NEW.session_id IS NULL OR length(NEW.session_id) = 0
BEGIN
    SELECT RAISE(ABORT, 'tests.session_id is required');
END;

CREATE TRIGGER IF NOT EXISTS tests_session_id_immutable
BEFORE UPDATE OF session_id ON tests
WHEN OLD.session_id IS NOT NEW.session_id
BEGIN
    SELECT RAISE(ABORT, 'tests.session_id is immutable');
END;
