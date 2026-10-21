# ADR 0003: Offline operation and profile lifecycle

Date: 2026-10-16

Status: Implemented locally for review; no release validation claimed.

## Decision

The owner selected strictly offline operation. Remove URL text import and the
in-app updater, including their IPC/capabilities, direct dependencies, and updater
release manifest. Keep local file import/export, manual installers and detached
signatures. Verifying a signature requires a separately obtained trusted public
key; this change does not provide a complete manual verification procedure.
The build toolchain can still download dependencies; this decision concerns the
application's runtime features, not an air-gapped build environment.

Acquire a standard-library OS lock on `.profile.lock` before legacy copying,
SQLite migrations, and startup recovery. Keep the file handle in Tauri-managed
state until shutdown. Do not delete the lock file. A second cooperating instance
must not interrupt a live session during its own startup recovery.

Profile initialization runs after `Builder::build` and before `App::run`, outside
the setup hook. In Tauri 2.11.3 an error returned by that hook panics from the
Ready event rather than reaching `Builder::run`'s error result. Expected lock/IO
failures therefore use an explicit startup boundary: exit 1, a fixed actionable
message for an occupied profile, and a generic redacted message for other local
initialization errors. The lock, migrations and recovery order is unchanged.

During whole-file restore, acquire an exclusive application lifecycle lease
before the engine and database locks. Mutators retain shared leases for the
entire command and fail admission during restore, rather than queue old IDs for
a different database. Reopen and recover the restored file with a fresh gate;
failed recovery leaves mutations blocked. Keep recovery policy in `application`,
SQL and backups in `data`, and lifetime/IPC coordination in `app`.

A completion retry reuses an existing immutable intent's policy only after its
version, session facts, replay, timestamp, language and canonical payload match.
Changing daily goals must not rewrite a previously recorded completion intent.

## Limits

The profile lock requires cooperating versions and filesystem locking support.
It does not exclude older binaries or arbitrary SQLite tools. Close older
instances before running this build. Windows/macOS and native UI recovery are
not validated by Linux unit tests. Restore retains its existing rollback
snapshots, schema compatibility rules, and lack of a physical power-loss
protocol. No migrations or dependencies are added.

## Rollback

Revert runtime/IPC and frontend removals together if the owner changes the
offline policy; do not publish an updater manifest for an offline build. Revert
the profile/lifecycle changes only while the application is closed. These changes
do not alter the schema or file format; preserve backups before choosing any
older application version, whose existing migration compatibility still applies.
