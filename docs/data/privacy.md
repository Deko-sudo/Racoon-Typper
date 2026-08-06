# Data, Privacy, and Retention

**Status:** Draft — retention/deletion behavior is being formalized before a
production release (Phase 4 work in `ROADMAP.md`).

## What data exists

Racoon Typper is local-first. It stores only typing practice data on the user's
own machine. There is no account, telemetry, or server component.

| Data | Location | Contents |
|---|---|---|
| SQLite database | `${XDG_DATA_HOME:-$HOME/.local/share}/racoon-typper/data.db` (Linux) | test history, replays, personal bests, daily statistics, streaks, lesson progress, custom texts, session ledger |
| Settings | `${XDG_CONFIG_HOME:-$HOME/.config}/racoon-typper/settings.toml` (Linux) | typing settings, theme, language preferences |

Platform locations other than Linux x86_64 are unverified; see
`SUPPORT_MATRIX.md`.

## Sensitive content

Typing sessions, replays, and custom texts may contain whatever text the user
types or imports. This content never leaves the machine:

- it is not transmitted over the network;
- it is not included in crash reports (crash reporting is not implemented);
- Optional local diagnostics are disabled by default. When explicitly enabled
  through `verbose_logging`, the current backup-failure event is stored as
  bounded JSONL under the data directory's `logs/` folder. Its schema contains
  only an allowlisted event name, error class, path kind, and fixed file label;
  it never records typed content, a supplied path, or a raw error payload.

## Retention

- The application keeps historical test records indefinitely until the user
  deletes them through the UI or manually removes the database file.
- Interrupted recovery records carry a recommended retention policy
  (`INTERRUPTED_SESSION_RETENTION_DAYS`, 90 days) defined in the recovery
  contracts; enforcement of that retention is Phase 4 work.

## Deletion

- Deleting a test through the UI removes the history row and its associated
  replay rows in the same transaction.
- Deleting the application data directory removes all local data. Settings and
  database live in separate directories; both must be removed for a complete
  wipe:
  - data: `${XDG_DATA_HOME:-$HOME/.local/share}/racoon-typper/`
  - settings: `${XDG_CONFIG_HOME:-$HOME/.config}/racoon-typper/`

## Backup and recovery

The data layer creates transactionally consistent, single-file SQLite backups
with SQLite's Online Backup API and retains rotating pre-migration snapshots.
Database-file restore remains a data-layer API and is not exposed through the
application UI: it must never run while a live `Database` is open on the same
path. Do not manually copy a live WAL-mode `data.db` file. See the [Profile
Transfer and Recovery Runbook](profile-transfer.md) for the current recovery
constraints.

## Export and portable profile transfer

`export_data` exports history in JSON or CSV. The current IPC also provides a
versioned portable JSON profile export/import with a no-write preview and
`merge`/`replace` policies. The portable profile includes practice data and
custom texts, but excludes settings, replays, raw SQLite backups, and operational
recovery/finalization ledgers. The detailed procedure and limits are documented
in the [Profile Transfer and Recovery Runbook](profile-transfer.md).
