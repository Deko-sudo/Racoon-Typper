# ADR 0004: Atomic administrative statistics reset

Date: 2026-10-17

Status: Implemented locally for review; no release validation claimed.

## Problem

The explicit Settings action to clear all statistics cannot delete finalized
history: V006–V008 intentionally prohibit deleting terminal ledger rows,
completion intents, and finalization receipts. Reordering deletes alone is
insufficient. Legacy history without ledger rows also fails when personal-best
foreign keys still reference a test deleted earlier in the transaction.

## Decision

Keep this operation an explicit administrative exception, never an ordinary
per-row ledger API. The app adapter acquires an exclusive lifecycle lease,
then the engine lock, and accepts only Idle/Persisted. It rejects running or
retry-pending completion states. Abort the completed in-memory session only
after the database transaction commits, so stale session IDs cannot be reused.

The data adapter uses one IMMEDIATE transaction with foreign keys enabled. It
reads the exact SQL of all three required delete guards before any mutation,
drops only those guards, deletes statistics in child-before-parent order,
recreates their original definitions, verifies those definitions, and commits.
Missing guards, delete failures, recreation failures, or mismatched definitions
return an error; transaction rollback restores both rows and schema changes.
Update/replace guards remain installed throughout the operation. Settings,
custom texts, migration history, and autoincrement sequences are retained.

No migration is changed. Session/recovery policy stays in its existing layers:
the app coordinates lifecycle and IPC, while SQLite operations stay in data.
Errors use the existing redacted AppError mapping at the IPC boundary.

## Limits and rollback

This is a logical transactional rollback guarantee, not a claim of durability
under physical power loss. Existing SQLite/filesystem durability limits still
apply. The data adapter requires its caller to exclude active or pending
sessions; it does not know about the in-memory engine. Ordinary code paths keep
all existing guards and terminal-state/idempotency rules.

Reverting the app/data changes together restores the earlier behavior without
schema migration. Successful administrative deletion has no undo operation;
retain a user-created backup when recovery of deleted statistics is needed.
