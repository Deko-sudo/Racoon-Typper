# Profile Transfer and Recovery Runbook

**Status:** Portable profile JSON export, preview, merge, and replace are exposed
through the Settings UI. Database-file restore remains a data-layer API only; it
is deliberately not exposed through a Tauri command or user interface while the
process owns an open database connection. This runbook does not claim a completed
cross-platform whole-file recovery smoke test.

## Choose the right operation

- Use **profile export/import** to move portable practice data between profiles or
  installations.
- Use a **SQLite backup restore** only to recover a whole database file. It is
  not a substitute for profile transfer and requires lifecycle coordination that
  is not yet available in the application UI.

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
path. It builds and integrity-checks a sibling temporary database before
atomically replacing the live main file, then removes stale WAL/SHM companions.
A failed source validation or temporary copy preserves the live database.

This API must not run while any live `Database` is open on the destination path.
The caller must first stop application activity, close the live database, perform
restore coordination, and only then reopen the application against the restored
file. The current Tauri application has no restore IPC or UI to perform that
lifecycle handoff safely. Do not attempt manual file copying of `data.db` while
WAL mode is active; use the SQLite Online Backup-based data-layer API to create
a consistent backup instead.

A whole-file restore restores the state captured in that backup, including data
outside the portable profile scope. It may therefore discard newer local writes
and restore older schema/data state. After recovery, reopen with the current
application so its normal migration and startup-recovery paths can run. Do not
use an older binary as a database downgrade mechanism.

## Current limitations

- A compromised renderer can bypass the file-size precheck and invoke IPC
  directly; the backend still enforces its 64 MiB document limit after IPC
  deserialization.
- No whole-file restore IPC/UI currently closes and recreates the live database
  around a whole-file restore.
- Platform-specific manual recovery and clean-install validation are not yet
  release evidence. See `SUPPORT_MATRIX.md` and `RELEASE_CHECKLIST.md`.
