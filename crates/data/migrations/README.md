# Session ledger migrations

## Migration level summary

Each migration is additive and forward-only. This table records only facts
expressed by the SQL files; it is not a behavioral spec.

| Level | Migration file | What it changes | Data it touches |
|---|---|---|---|
| V001 | `V001__initial.sql` | Creates the six base tables (`tests`, `personal_bests`, `lesson_progress`, `daily_stats`, `streaks`, `custom_texts`) and their indexes | None (fresh schema) |
| V002 | `V002__lesson_language.sql` | `ALTER TABLE lesson_progress ADD COLUMN language TEXT NOT NULL DEFAULT 'en'` | Existing `lesson_progress` rows receive the `'en'` default |
| V003 | `V003__replays.sql` | Creates `test_replays` with `test_id … REFERENCES tests(id) ON DELETE CASCADE` and indexes | None (new table) |
| V004 | `V004__custom_text_language.sql` | `ALTER TABLE custom_texts ADD COLUMN language TEXT NOT NULL DEFAULT 'en'` | Existing `custom_texts` rows receive the `'en'` default |
| V005 | `V005__session_identity.sql` | `ALTER TABLE tests ADD COLUMN session_id TEXT`; deterministic backfill; unique index; `required`/`immutable` triggers | Pre-V005 `tests` rows are backfilled as `legacy-test-%016x` of `id` |
| V006 | `V006__session_ledger.sql` | Creates `session_ledger` (no foreign keys; CHECK + terminal-state triggers) and recovery-order indexes | None — historical `tests` rows are intentionally not backfilled |
| V007 | `V007__session_completion_intents.sql` | Creates `session_completion_intents` with `session_id … REFERENCES session_ledger(session_id) ON DELETE RESTRICT`; immutability/delete/replace triggers | None (new table) |
| V008 | `V008__session_finalizations.sql` | Unique index on `(session_id, fingerprint)`; creates `session_finalizations` with `session_id … REFERENCES session_ledger … ON DELETE RESTRICT` and a composite `FOREIGN KEY (session_id, fingerprint) … ON DELETE RESTRICT ON UPDATE RESTRICT`; state-transition triggers | None (new table) |

The matrix upgrade tests in `crates/data/tests/migration_matrix.rs` exercise
every path V1..V7 → V8 through the production Refinery runner, seeding
era-appropriate data before the upgrade and asserting schema, foreign keys,
journal mode, and data survival (including the V005 backfill) afterwards.

## Session ledger detail

`V006__session_ledger.sql` adds durable lifecycle records for newly accepted
sessions. It does not backfill historical `tests` rows, because completed test
history is not a recoverable active session. The table stores only bounded,
application-validated sanitized session descriptor metadata and redacted reason
codes; typed text, replay frames, and completion payloads are excluded.
Terminal rows reject direct update, delete, and replacement attempts.

`V007__session_completion_intents.sql` adds one immutable opaque canonical
payload per ledger session. SQLite enforces the foreign key, lowercase digest
shape, exact recorded byte length, the 8 MiB limit, and update/delete/replacement
resistance, while the application layer remains responsible for canonical JSON,
schema, and fingerprint validation.

`V008__session_finalizations.sql` adds one effect-free finalization record per
session with an immutable identity/fingerprint association. Its fingerprint has
a composite foreign key to the V007 intent fingerprint, so SQL itself rejects
mismatched associations. Records
begin `pending` and may become `committed` or terminal `quarantined`; triggers
protect identity, fingerprint, claim timestamp, deletion, replacement, and
terminal reopening. The V008 adapter uses `IMMEDIATE` transactions to claim or
mark this ledger only—it does not insert any completion effect or mark V006
`finalized`. Quarantine revalidates the expected, V008, and current V007
fingerprints; durable mismatch, missing-intent, or corrupt-metadata reasons take
precedence over caller-supplied reasons, while terminal records remain unchanged.

Phase 3B.3.4 adds no migration. Its unwired `SqliteSessionFinalizer` uses the
same `IMMEDIATE` transaction boundary to strictly validate the immutable V007
payload and V006/V008 consistency, apply the existing completion effects from
that payload, then commit V008 and finalize V006. A retry is identified by the
session and immutable fingerprint and rechecks the complete immutable test and
replay evidence before reporting success; terminal corruption is not repaired
or rewritten. Time-goal evaluation preserves zero and fractional-minute
behavior. Its rollback failpoints are available only through racoon-data's
non-default `test-support` feature. Phase 3B.3.5 adds no migration: its
application-owned startup coordinator composes the V006/V007/V008 adapters by
first listing header-only candidates, then strictly loading V007 only for
eligible finalization. It claims V008 through the accepted port and invokes
the finalizer without adding completion effects of its own. Its process-local
readiness gate blocks recovery-relevant commands until the scan reaches
`Ready`; global failures remain `Blocked`, while safely quarantined row-local
records are reported and do not stop unrelated recovery. Normal live
completion remains unchanged.

Phase 3B.3.6 adds no migration. Its non-default `crash-test-support` feature
extends `test-support` only for a file-backed integration-test campaign. An
independent child process synchronizes a bounded checkpoint marker and aborts
without unwinding; its parent reopens the same SQLite/WAL file, snapshots the
durable state, and runs the accepted startup coordinator. The matrix covers
V006/V007/V008 crash boundaries, all meaningful pre-commit finalizer effects,
one lesson boundary, and post-commit ambiguous success. A dedicated
1,000-ms `time(0.01)` fixture performs the real daily-goal false-to-true write
before its pre-commit checkpoint; abort rolls it back, recovery applies it
once, and reopen does not duplicate it. It proves process-crash recovery only,
not physical power-loss durability. No production build or migration exposes
the crash seam. Phase 3B.3 recovery architecture is accepted and complete.
Phase 3B.3.7 final acceptance is complete. Later Phase 3B milestones remain
not started and require separate approval.

These migrations are additive and forward-only. Downgrading to an older binary
after V006/V007/V008 is unsupported. No destructive down migration is supplied;
backup restore or a forward fix is the operational recovery path. The data
adapter uses header-only scans for recovery candidates and separately validates
payload bytes only when loading a specific intent. Historical upgrade fixtures
are created with Refinery history and upgraded through the production runner;
structured SQLite busy/locked codes remain available to the adapter for
retryable-failure classification.
