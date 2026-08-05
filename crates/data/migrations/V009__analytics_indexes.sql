-- V009__analytics_indexes.sql — ordering indexes for long-history analytics reads
--
-- Additive index-only migration (no schema, data, or trigger changes). Added to
-- keep dashboard/reporting reads fast as `tests` and `personal_bests` grow past
-- ~10 000 rows. See docs/adr/0002-long-history-metrics.md for the EXPLAIN QUERY
-- PLAN evidence and the deliberate decision NOT to index `tests.language` /
-- `lesson_progress.language` (no live caller / small cardinality).

-- Filtered test history: `WHERE mode_type = ? ORDER BY created_at DESC,
-- session_id DESC LIMIT ?`. The second order key makes offset pages stable when
-- several completions share one timestamp.
-- The pre-existing `idx_tests_mode_config(mode_type, mode_config)` selects by
-- mode but still requires a sort to deliver the history order; this
-- composite removes that sort step on long history.
CREATE INDEX IF NOT EXISTS idx_tests_mode_created_at_session_id
    ON tests (mode_type, created_at DESC, session_id DESC);

-- Unfiltered test history uses the same deterministic order. The legacy
-- created_at-only index cannot provide the session-id tie break.
CREATE INDEX IF NOT EXISTS idx_tests_created_at_session_id
    ON tests (created_at DESC, session_id DESC);

-- Personal-bests listing: `ORDER BY updated_at DESC, mode_type ASC,
-- mode_config_hash ASC`.
-- The unique `(mode_type, mode_config_hash)` index cannot provide this order;
-- this index removes the sort for an unfiltered listing.
CREATE INDEX IF NOT EXISTS idx_personal_bests_updated_at_config_hash
    ON personal_bests (updated_at DESC, mode_type ASC, mode_config_hash ASC);
