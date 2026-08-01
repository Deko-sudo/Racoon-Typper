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
- logs, when structured logging is introduced, must redact typed content by
  policy (Phase 5 work).

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

## Backup

Before any migration, users should back up the database file (`data.db` plus
`data.db-wal` and `data.db-shm` while the app is closed). A structured backup
and restore workflow is planned in Phase 4.

## Export

`export_data` currently exports history in JSON or CSV. Full profile export and
import (settings, custom texts, lessons, preferences) is Phase 4 work and is not
yet available.
