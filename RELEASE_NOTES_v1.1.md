# Release Notes — Racoon Typper v1.1.0

**Status:** Release candidate preparation — not published
**Planned tag:** `v1.1.0`

Racoon Typper 1.1 focuses on durable local data, safe profile transfer,
long-history performance, security hardening, and desktop release verification.

## Highlights

### Safe profile backup and restore

Settings now includes a complete portable profile workflow:

- export a versioned JSON profile;
- reject empty, non-JSON, and over-64-MiB files before reading them;
- preview all collection counts without writing data;
- merge with the current profile or replace portable profile data atomically;
- require an explicit post-preview acknowledgement before replace;
- invalidate the preview whenever the selected file or policy changes.

Portable profiles include tests, personal bests, daily statistics, streaks,
custom texts, and lesson progress. They intentionally exclude settings, replays,
raw SQLite files, and operational recovery ledgers.

### Durable session recovery

- Backend-issued UUIDv7 session identities.
- Durable session, completion-intent, and finalization ledgers.
- Deterministic startup recovery with retry-safe, exactly-once finalization.
- Pre-migration SQLite backups and validated restore primitives.
- Forward-only migrations through schema V009 with historical upgrade fixtures.

### Accurate long-history reporting

- Stable pagination and explicit tie-break ordering.
- Maintained global projections for dashboard and achievement summaries.
- Query-plan and timing coverage for 10,000+ records.
- Correct global best and streak results beyond 100,000 historical records.

### Typing and interface improvements

- Refined physical keyboard layout with aligned navigation and special keys.
- Natural left/right hand silhouettes that highlight the required finger.
- Theme-aware SVG success and error icons instead of emoji status stickers.
- Existing sound, Zen mode, achievements, replay, lessons, weak-key training,
  themes, and analytics remain available.

### Security and privacy

- Least-privilege Tauri command capability generated from the registered IPC set.
- Stable redacted public error envelopes.
- Opt-in bounded diagnostics that exclude typed content and supplied paths.
- Hostile-input coverage for malformed profile data and repeated rejected requests.
- Apache-2.0 project licensing, content provenance records, dependency policy,
  and CycloneDX SBOM evidence.

### Release engineering

- Read-only CI checks are separated from draft candidate creation and protected
  release promotion.
- Linux and Windows artifacts are rebuilt from an immutable version-matching tag.
- Candidate artifacts include SHA-256 checksums, SBOM, provenance, and a source
  commit manifest.
- Automated Linux package and Windows NSIS install/restart smoke workflows.

## Upgrade behavior

The current application runs forward-only database migrations and creates a
pre-migration backup before changing an older schema. Existing local profile data
is preserved by the migration matrix verified from every supported historical
schema fixture.

## Known limitations

- Whole-file SQLite restore is not exposed in the running application because it
  requires closing and recreating the process-owned database connection. Use the
  guarded portable profile workflow for user-facing backup and restore.
- A portable profile is not a complete filesystem/database snapshot.
- Native artifacts are source-to-artifact traceable but are not claimed to be
  byte-for-byte reproducible or cryptographically signed.
- The Arch `PKGBUILD` checksum remains blocked until the final immutable source
  tag/archive exists at the canonical upstream.
- Public release remains blocked until final tagged CI, clean-install artifact
  smoke, release evidence review, and maintainer promotion approval complete.