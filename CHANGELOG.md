# Changelog — Racoon Typper

## Modernization baseline (2026-08-01) — unreleased development state

This section records foundation and release-engineering work merged into
`master` after the v1.1.0 release notes. It is development work and is not a
production release claim (see `ROADMAP.md`).

### Foundation — Phases 0–3B.3

- Apache-2.0 licensing migration: workspace license, `PKGBUILD` metadata,
  removed imported GPL theme catalog, provenance inventories under `licenses/`,
  `THIRD_PARTY_NOTICES.md`, CI license policy (`cargo-deny` + npm policy).
- Deterministic build topology: canonical Tauri wrapper (`scripts/tauri.mjs`),
  version source of truth in `[workspace.package]`, `check:version` gate.
- In-process exactly-once completion: explicit engine lifecycle, retry-safe
  persistence, backend-authoritative scoring, single `IMMEDIATE` transaction.
- Durable session identity (Phase 3A): backend-issued UUIDv7 `SessionId`,
  migration `V005` with deterministic legacy backfill and immutability guards.
- Application layer (Phase 3B.1–3B.2): infrastructure-free `racoon-application`
  crate, `SessionKernel`, provider ports (identity, monotonic time, wall clock,
  randomness).
- Crash recovery (Phase 3B.3): durable recovery vocabulary, canonical
  SHA-256 completion intents, session ledger (`V006`), completion intents
  (`V007`), finalization ledger (`V008`), startup recovery coordinator,
  test-only process-crash campaign (`crash-test-support` feature).

### Release engineering — Phase 6 partial

- GitHub Actions pinned to full commit SHAs; dependabot group for action
  updates; checkout upgraded to v5 (Node 24 runtime).
- Toolchain pinning: `rust-toolchain.toml` (Rust 1.96.0), `.nvmrc` + `engines`
  (Node >=22).
- CRLF-safe version check and `.gitattributes` (normalizes line endings so the
  version gate works on Windows checkouts).

### Security — Phase 5 partial

- CSP: removed `unsafe-inline` from `script-src`; inline style exception
  retained and documented.

### Repository and documentation — Phase 7 partial

- Issue templates (bug, feature), pull request template, `SECURITY.md`.
- Data, privacy, and retention documentation in `docs/data/privacy.md`.

### Tests

- 587 workspace tests passing; crash-recovery campaign (16 default + 115
  extended child-process crashes) passing.

## v1.1.0 (2026-06-23) — Community & Polish Release

### New Features

- **Sound Engine**: key press, error, lesson complete, achievement sounds with volume control and cooldown
- **Zen Mode**: distraction-free typing — hides navbar, stats, panels during test
- **Original Racoon Themes**: dark, light, and high-contrast practice palettes
- **Achievement Notifications**: toast notifications when achievements unlock
- **Session Recovery**: restore previous session after app crash/close
- **Extended Statistics**: best day, most active hour, avg session duration, total chars/words
- **Profile Export**: full profile export (settings, tests, lessons, PBs, custom texts) as JSON
- **Sound Settings**: sound_enabled, sound_volume in settings.toml
- **Zen Mode Setting**: zen_mode_enabled in settings.toml

### Improvements

- Dashboard extended with 5 new stat cards
- Settings page: 3 new controls (sound toggle, volume slider, zen mode toggle)
- 3 original themes with documented provenance

### Testing

- 418 tests (was 405)
- 0 clippy warnings
- 13 new sound engine tests

## v1.0.0 (2026-06-23) — Initial Release

### Features

- 4 typing modes: Time, Words, Quote, Custom
- 2 languages: English, Russian
- Lesson system: 8 modules per language (80 lessons total)
- Adaptive learning: FrequencyAdaptiveGenerator
- Weak Keys engine: analysis, training generation
- Dashboard: streak, avg WPM/accuracy, tests today/week/total
- Analytics: consistency, burst detection, achievements (15), insights, export
- Replay system: play/pause/seek/speed (0.5x/1x/2x/4x)
- Heatmap with finger hints
- KeyboardTrainer: next-key highlight, finger mapping
- HandPositionGuide: 8-finger visual guide
- TypingWarnings: layout detection, Caps Lock detection
- NotificationStack: smart side notifications (max 3, 5s auto-remove)
- Progress charts: SVG WPM + accuracy (7d/30d/90d)
- 3 original Racoon themes
- 9 settings: font size, caret style, live WPM, accuracy, keyboard trainer, hand guide, layout warnings, CapsLock warnings
- SQLite persistence (7 tables, 3 migrations)
- TOML settings (~/.config/racoon-typper/settings.toml)
- Fully offline, no network requests

### Packaging

- PKGBUILD (Arch Linux)
- AppImage (build-appimage.sh)
- Flatpak manifest (com.racoon.typper.json)
- NSIS installer (Windows, EN+RU)
- GitHub Actions CI/CD pipeline

### Testing

- 405 tests (0 failed)
- 0 clippy warnings
- cargo audit: 0 vulnerabilities
