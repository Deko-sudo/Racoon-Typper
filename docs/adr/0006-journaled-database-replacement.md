# ADR 0006: Journal whole-database replacement before opening SQLite

- Status: implemented in the local stability branch; upstream review pending
- Scope: data-layer filesystem replacement, not session-ledger migrations

## Context

Private-copy validation prevents incompatible backups from replacing live data.
It does not make replacement of the main database and its companions one atomic
operation. An interrupted detach/rename can leave a mixed filesystem state.
The user approved an independent durable-protocol follow-up to the compact fix.

## Decision

Retain the validated replacement and original main/WAL/SHM/journal files in an
exclusively created 0700 sibling directory (files 0600 on Unix). Sync all copied
files, then atomically publish and sync a versioned pending manifest containing
presence and expected lengths. Sync the parent before detaching any companion.
After installing the replacement, sync both affected directories before marking
committed. Ordinary pre-commit errors attempt rollback; failed rollback or an
uncertain commit keeps the application connection closed.

Database open/reopen runs filesystem recovery before opening SQLite, applying
pragmas or invoking the pre-migration backup hook. A unique pending operation
restores all original files and original absences from the retained copy, syncs
the result and marks aborted. Repeating recovery is safe after interruption.
Conflicting journals, symlinks and incomplete snapshots fail closed. Terminal
snapshots never overwrite later user data.

Keep five completed snapshots, ordered by terminal-marker modification time.
Pruning refuses unresolved pending operations, renames a completed directory
out of the recovery namespace, syncs that rename and only then deletes it.
Failure is best effort: pending/legacy/preparation directories and retired trash
may remain. No migration, direct dependency, IPC command or network service is
added; filesystem and SQL remain in data and the existing lifecycle/OS profile
locks remain in app.

## Assumptions and limits

The caller serializes restore/open/reopen and closes all writers. The OS profile
lock covers cooperating app versions, not arbitrary SQLite tools or a malicious
process running as the same user. The local filesystem must honor atomic rename
and file/directory sync ordering. Directory sync is implemented on Unix; Windows
and macOS have no runtime evidence here. The manifest detects missing/truncated
files, not equal-length content corruption. Settings TOML is outside this DB
transaction. Old unmarked snapshots cannot be safely classified and are retained.

Tests use a real committed, uncheckpointed WAL and abrupt child-process exit at
11 replacement and 5 rollback boundaries, plus typed injected errors and a Linux
LD_PRELOAD rename/unlink refusal fixture. They do not exhaust all possible IO
errors or prove physical power-loss durability, hot rollback-journal recovery,
network-filesystem behavior or clean packaged installation.
