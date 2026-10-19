# Data, Privacy, and Retention

**Status:** Draft — retention/deletion behavior is being formalized before a
production release (Phase 4 work in `ROADMAP.md`).

## What data exists

Racoon Typper is local-first. It stores only typing practice data on the user's
own machine. There is no account, telemetry, server component, URL importer, or in-app updater. Updates are installed manually.

| Data | Location | Contents |
|---|---|---|
| SQLite database | `${XDG_DATA_HOME:-$HOME/.local/share}/com.racoon.typper/data.db` (Linux) | test history, replays, personal bests, daily statistics, streaks, lesson progress, custom texts, session ledger |
| Settings | `${XDG_CONFIG_HOME:-$HOME/.config}/com.racoon.typper/settings.toml` (Linux) | typing settings, theme, language preferences |

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
  clears statistics through Settings or removes the database while the app is
  closed. Retained backups may still contain the old records.
- Interrupted recovery records carry a recommended retention policy
  (`INTERRUPTED_SESSION_RETENTION_DAYS`, 90 days) defined in the recovery
  contracts; enforcement of that retention is Phase 4 work.

## Deletion

- Settings' clear-statistics action removes all active database statistics,
  replays and session-recovery ledgers in one transaction, while preserving
  custom texts and settings. It is rejected during an active or retryable test.
  There is no individual-test deletion command. This action does not erase
  previously retained SQLite snapshots or exported files.
- To remove the current profile, close the app and remove both its data and
  settings directories. Exported files and legacy sources outside them remain:
  - data: `${XDG_DATA_HOME:-$HOME/.local/share}/com.racoon.typper/`
  - settings: `${XDG_CONFIG_HOME:-$HOME/.config}/com.racoon.typper/`

On each Linux startup, a missing current file may be copied from a legacy
location. If `XDG_DATA_HOME` or `XDG_CONFIG_HOME` is set, the corresponding
legacy source is directly inside that directory; otherwise it is under
`~/.local/share/racoon-typper` or `~/.config/racoon-typper`. Sources are retained.
Removing a current destination can therefore import the legacy source again.
For a complete wipe, remove both current and any retained legacy files while
the app is closed. Legacy WAL/SHM companions are copied only with the main file
and only when their own destinations are missing.

## Backup and recovery

The data layer creates transactionally consistent, single-file SQLite backups
with SQLite's Online Backup API and retains rotating pre-migration snapshots.
Settings exposes whole-database restore through a file picker. The application
closes its connection, validates a private migrated copy before replacement,
retains private rollback snapshots with a local replacement journal, and performs
filesystem recovery before opening SQLite and session recovery before accepting
mutations. Cleanup keeps five completed rollback snapshots when removal succeeds;
pending, legacy/unmarked and interrupted-cleanup copies can remain.
The data-layer path API itself must never run while a live writer is open on
the same path. Do not manually copy a live WAL-mode `data.db` file. See the [Profile
Transfer and Recovery Runbook](profile-transfer.md) for the current recovery
constraints.

## Export and portable profile transfer

`export_data` exports history in JSON or CSV. The current IPC also provides a
versioned portable JSON profile export/import with a no-write preview and
`merge`/`replace` policies. The portable profile includes practice data and
custom texts, but excludes settings, replays, raw SQLite backups, and operational
recovery/finalization ledgers. The detailed procedure and limits are documented
in the [Profile Transfer and Recovery Runbook](profile-transfer.md).

### Result PNG and clipboard

Pressing **Save PNG** writes the result card to the operating system's Pictures
folder, inside `Raccoon Typer`, and also copies the image to the system clipboard.
The card contains visible result metrics and first-attempt character statistics.
It is generated locally without a network request. The UI reports the actual
saved path and clipboard outcome separately; a failed operation can be retried.
PNG files remain until you delete them and are not removed by Clear statistics.
Clipboard managers and applications where you paste can retain their own copies.
On Linux, availability after closing the app depends on the clipboard manager.
