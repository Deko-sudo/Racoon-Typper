# Profile Transfer and Recovery Runbook

**Status:** Portable profile JSON export, preview, merge, and replace, as well as
database-file restore, are exposed through Settings. Whole-file restore closes
the application connection and coordinates replacement and recovery. This runbook
does not claim a completed cross-platform whole-file recovery smoke test.

## Choose the right operation

- Use **profile export/import** to move portable practice data between profiles or
  installations.
- Use a **SQLite backup restore** only to recover a whole database file. It is
  not a substitute for portable profile transfer; finish the active test first
  and keep other database writers closed.

A portable profile is JSON, not a SQLite backup. Treat both files as sensitive:
test history, custom texts, and typed-data-derived statistics can contain private
content. Store them only where the recipient is authorized to read them.

## Portable profile transfer

The IPC contract provides `export_profile`, `preview_profile_import`, and
`import_profile`. The current document is strict and versioned:

- `format` is `racoon-typper-profile` and `schema_version` is `1`.
- Unknown JSON fields, unsupported versions, invalid values, documents larger
  than 64 MiB, and collections larger than 100,000 rows are rejected.
- Export reads one consistent database connection snapshot.
- Import validates the complete document before starting its write transaction.

The portable payload contains test records, personal bests, daily statistics,
streaks, custom texts, and lesson progress. It does **not** contain settings,
replays, raw SQLite files, or operational session-recovery/finalization ledgers.
Consequently, importing a profile does not restore an interrupted active session
or reproduce every locally stored record.

### Recommended transfer procedure

1. Finish or cancel any active typing test before beginning. Import is rejected
   while the engine is running or finalizing; application startup recovery must
   also be ready.
2. In Settings, export the source profile and save the downloaded JSON outside
   the application data directory.
3. On the target profile, select the JSON and choose merge or replace. The UI
   rejects empty, non-JSON, and over-64-MiB files before reading them, then calls
   `preview_profile_import` with the exact document and policy. This dry run
   returns incoming, existing, and `to_insert` counts without changing the database.
4. Review the policy and preview. Keep the exported JSON until the target data
   has been inspected after import.
5. For replace, acknowledge the destructive portable-data replacement after the
   preview. Any file or policy change invalidates both preview and acknowledgement.
6. Apply the restore. The UI sends the exact previewed document and policy to
   `import_profile`; any import error rolls back all portable-table writes. After
   success, the application reloads so every view reads the restored profile.

### Import policies

- `merge` preserves existing test and custom-text identities and skips matching
  incoming entries. Personal bests, daily statistics, streaks, and lesson
  progress use their documented identity keys and are updated from the import
  when they conflict. It is not a field-by-field reconciliation policy.
- `replace` deletes and repopulates only the portable profile tables in one
  SQLite transaction. It does not swap the database file and does not delete or
  restore recovery/finalization ledgers. Treat it as destructive for the target
  profile data; export the target profile first and require a reviewed dry run.

The preview is advisory: another write between preview and import can change the
observed existing counts. Re-run the preview immediately before a destructive
replace if the target may have changed.

## Database-file recovery constraints

`restore_from_path` accepts a separate regular-file backup and live database
path. It copies the backup into a private sibling directory, applies migrations,
checks integrity, foreign keys, the reference application schema, and the
terminal-state trigger with recursive triggers off and on. Source validation
does not replace the live database. Historical V1–V8 backups are migrated on the
private copy; additional schema objects and differently quoted definitions may
be rejected even when semantically equivalent.

This API must not run while any live `Database` is open on the destination path.
The Settings restore command takes an exclusive lifecycle lease and the engine
lock, rejects an active session, closes the database, performs replacement, then
reopens and runs recovery with a fresh gate before reporting success. New
mutations are rejected during this operation; failed recovery blocks them until
the problem is resolved and the application restarts. Read-only reporting may
temporarily fail while the connection is closed. External writers must also be
closed. Do not manually copy a live WAL-mode database. Whole-file backups are
available as retained pre-migration snapshots or through the SQLite Online
Backup-based data-layer API; Settings JSON export is a portable profile, not a
SQLite backup.

Before replacement, the old main/WAL/SHM/journal files are copied to a private
`data.db.restore-rollback-*` sibling directory. Each file is synced, and a
versioned `pending` manifest records the expected file sizes. The manifest and
parent directory are synced before detaching old companions and renaming the
replacement. A `committed` marker is published only after syncing the installed
file's directory entries. An error before commit attempts rollback; failure or
an uncertain commit keeps the connection closed.

On startup and reopen, filesystem recovery runs **before SQLite opens and before
the pre-migration backup hook**. One complete pending snapshot restores the old
main and companions, removes companions that were originally absent, syncs the
result and records `aborted`. Recovery can be repeated after another process
interruption. A committed or aborted snapshot never overwrites later user data.
Malformed/ambiguous journals, symlinks and missing/truncated recovery files block
opening; preserve the directory for manual recovery rather than deleting it.

Cleanup retains the five most recently completed (committed or aborted)
snapshots, ordered by terminal-marker modification time. It never prunes pending
or unmarked legacy directories. Before deletion, a completed directory is
renamed out of the recovery namespace and that rename is synced, so interrupted
cleanup cannot create a false pending restore. Cleanup is best effort: IO errors,
unmarked preparation directories and interrupted `.restore-trash-*` deletion can
leave extra copies. These may contain personal text; remove only after a verified
backup and after confirming no recovery is pending. There is no settings-file
rollback: whole-database restore replaces SQLite data only.

All restore/open/reopen operations must be serialized by the caller and use a
closed profile. The application lifecycle lease and OS profile lock cover
cooperating application instances; unrelated SQLite tools must remain closed.
The protocol assumes a local filesystem honoring file/directory sync and
same-filesystem atomic rename. Process-interruption tests do not prove physical
power-loss durability; directory sync is currently implemented on Unix only.
The manifest checks file presence/type/size, not cryptographic content checksums.

On Linux, find them with:

```bash
find "${XDG_DATA_HOME:-$HOME/.local/share}/com.racoon.typper" \
  -maxdepth 1 -type d -name 'data.db.restore-rollback-*' -print
```

A whole-file restore restores the state captured in that backup, including data
outside the portable profile scope. It may therefore discard newer local writes
and restore older schema/data state. After recovery, reopen with the current
application so its normal migration and startup-recovery paths can run. Do not
use an older binary as a database downgrade mechanism.

## Current limitations

- A compromised renderer can bypass the file-size precheck and invoke IPC
  directly; the backend still enforces its 64 MiB document limit after IPC
  deserialization.
- Whole-file restore does not guarantee physical power-loss recovery or
  coordinate with external SQLite writers. The profile lock excludes only
  application versions that implement it.
- Platform-specific manual recovery and clean-install validation are not yet
  release evidence. See `SUPPORT_MATRIX.md` and `RELEASE_CHECKLIST.md`.
