# Changelog — Racoon Typper

## Unreleased

### Fixed

- **Hand guide highlight geometry** — the active finger highlight no longer
  spills onto the palm: band bottoms stop at the finger/palm merge lines
  measured from the rendered hand path, with edges tightened to each finger.
- **Hand guide finger mapping** — the hand visualization now resolves the
  next character against the active layout's physical key positions instead
  of per-character guesses: A/Ф light up the left pinky, S/Ы the left ring,
  K/Л the right middle, and so on for every supported layout (QWERTY,
  ЙЦУКЕН, Dvorak). Number-row and punctuation keys received their standard
  touch-typing fingers, space uses the right thumb, and unknown keys no
  longer highlight anything. The keyboard key coloring shares the same
  position-derived tables.

### Dependencies

- **DD1–DD3 dependency window closed** — `rusqlite` 0.31→0.39,
  `refinery` 0.8→0.9.2 (resolver-coupled pair; refinery-core caps rusqlite
  ≤0.39), and `toml` 0.8→1.1. Zero source changes required. Regression
  evidence: migration matrix V1..V7→V9 through the production Refinery
  runner (16/16), Online Backup API round-trips (9/9), existing-data and WAL
  preservation, settings TOML round-trips (26/26), full workspace suite green.

## v1.3.1

### Release engineering

- **Actions refreshed to Node 24** — `actions/setup-node` v6.5.0,
  `actions/download-artifact` v8.0.1, and `softprops/action-gh-release` v2.6.2
  replace their Node 20-targeting pins.
- **WebDriver harness groundwork** — pinned `tauri-driver` provisioning,
  a selenium-webdriver client, and exact-version msedgedriver matching were
  built and exercised on CI. Session creation currently fails inside the
  runner's desktop session (EdgeDriver↔WebView2 attach limitation), so the
  Windows smoke retains its proven install / first-screen / live-session /
  restart assertions; findings recorded in TECH_DEBT (TD-NEW23) for the
  WebdriverIO-service route.

## v1.3.0

### Highlights

- **Weekly summaries** — the dashboard gained a "Last 8 Weeks" strip: per-week
  activity bars, goal-day counters, and weighted WPM/accuracy aggregates built
  from persisted daily statistics.
- **First-run onboarding** — a one-time three-step setup (interface language,
  practice language, daily goal) with an explicit skip; the practice language
  is now a persisted setting that drives tests and lessons instead of a
  hard-coded default.
- **Keyboard layouts** — new `keyboard_layout` setting (QWERTY / ЙЦУКЕН /
  Dvorak) driving finger hints, the hand guide, result heatmap, and weak-key
  finger tags; Cyrillic always resolves to JCUKEN, matching the core maps.
- **Text packs** — versioned `racoon-typper-text-pack` interchange for custom
  texts: JSON export/preview/import plus Anki-style TSV, RFC4180 CSV, and
  plain-text block sources with explicit mapping rules; merge or
  language-scoped replace policies, all inside one sandboxed transaction.
- **Language resources audit** — every bundled language now has a pinned
  completeness bar (courses ≥3 modules / ≥12 lessons, ≥10 attributed quotes,
  ≥300 unique words); fixed Czech word-list shortfall and deduplicated the
  generated CJK lists.
- **Release evidence hardening** — AppImage artifacts are launch-tested before
  a draft is created (FUSE with extract-and-run fallback), and the Windows
  clean-smoke now proves silent install, first-screen rendering, a live
  practice session start, and restart data retention on every candidate.
  Typed-input persistence on headless CI runners remains a documented
  limitation pending a desktop UI automation harness.

### Fixes

- Whole-file restore lifecycle coordination (TD6), durable live-completion
  routing (TD4), typed ModeConfig contract pinning (Task R), and reporting
  adapters behind application use cases (TD1) landed during the cycle.
- Performance baselines for every reporting adapter at 10k records guard the
  dashboard query paths against accidental full scans.

## v1.2.0

### Highlights

- **50 built-in themes** — the theme catalog grew from 25 to 50 original
  Apache-2.0 themes spanning graphite, light, warm, nature, terminal, pastel,
  retro, and high-contrast variants.
- **52 achievements** — the achievement catalog expanded from 15 to 52.
- **PNG result share card** — a 900×470 share card (WPM/raw/accuracy/time,
  mode, date, mini keyboard heatmap) rendered in the active theme colors;
  embedded DejaVu fonts, one-click Blob download from the result screen.
- **Custom theme editor** — 48 theme variables grouped by surface/text/typing/
  keyboard/charts/misc, live swatches, Save/Reset/Randomize; the custom theme
  is stored as a validated JSON setting and applied without a bundled CSS file.
- **Smooth caret** — a single absolutely-positioned caret element glides
  between characters (80 ms ease-out, monkeytype-style) with thin/thick/
  bubble/off styles, before/after positions, and `prefers-reduced-motion`
  support.
- **Caret animation** — new `blink` (default) / `pulse` setting plus a live
  "Hello world" caret preview next to the caret settings.
- **Pomodoro timer** — dedicated view with work/break phases (configurable
  minutes), 4-cycle long break (15 min), start/pause/reset, and a phase-change
  sound; drift-free timing across background tabs.
- **Markdown export** — analytics export gains a Markdown format (summary +
  history table) alongside JSON and CSV.
- **Hotkey cheatsheet** — press `?` for an overlay of navigation, Vim, test,
  and global shortcuts (Esc/click/`?` closes).
- **Error tails** — positions where a mistake was made keep a thin red
  underline that survives backspacing and retyping.
- **Auto-update** — `tauri-plugin-updater` wired to the GitHub Releases
  endpoint, with signed AppImage/NSIS artifacts and a `latest.json` manifest.
- **Contribution calendar** — a GitHub-style activity calendar with a metric
  switcher (tests / time / lessons).
- **Vim mode** — extended with `gg`/`G`/`r` and a mode indicator, extracted into
  a testable module.
- **In-test restart** — restart the current test without leaving the view.
- **Clear-all-statistics** — a confirmed action to reset typing statistics.
- **Settings polish** — toggle switches replace native checkboxes; keyboard
  heatmap alignment fixes.

### Session recovery (Phase 3B)

- Durable session identity (UUIDv7), session ledger, completion intents,
  finalization ledger, and startup recovery coordinator (accepted in the
  Phase 3B.3 series).
- Exactly-once completion persistence in a single `IMMEDIATE` transaction
  (test + replay + personal bests + daily stats + streaks + goals + lesson
  completion).
- Pre-migration rotating SQLite backups and validated restore primitives.

### Caret highlight (B1 — shipped)

- The current-character highlight uses a **neutral raised surface**
  (`--color-surface-active`), per the B1 decision recorded in TECH_DEBT TD8.
  Contrast is provided by `--color-typing-current`, set per theme to meet
  WCAG AA (≥4.5:1) against that surface.
- All 50 themes now pass the `theme-pack` contrast suite; the suite measures
  the current character against `--color-surface-active`, the real render
  surface in TestView.

### Fixes

- Honest accuracy reporting and strict lesson gating.
- Keyboard input filters (no phantom characters from Arrow/Delete/Home/F keys).
- Session leaks, dead settings, and pagination fixes (medium-severity audit).
- Unicode-safe HTML stripping for URL text import: ASCII case-insensitive
  search in the original string (lowercased copies can change UTF-8 length
  and shifted byte offsets panicked or dropped text around `İ`, `ẞ`, Cyrillic).
- Daily stats and dashboard day boundaries use local calendar days
  consistently with persistence.
- Windows: hide console window in release builds.
- AUR PKGBUILD source URL and LTO linking corrected.

### Release engineering

- Updater artifacts signed and referenced from subdirectories; `latest.json`
  published with correct platform URLs and signatures.
- Linux package smoke and Windows NSIS install/restart smoke in the
  release-candidate workflow.
- SBOM, provenance, and release manifest attached to each candidate.
- License policy covers the embedded DejaVu fonts (third-party-font
  provenance) and the `ab_glyph` dependency family.

---

## v1.1.0 — release candidate (not published)

This section records the audited release-candidate baseline on `master`. Public
release still requires tagged CI, artifact smoke, evidence review, and maintainer
promotion approval.

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

### Product and data transfer

- Sound controls, Zen mode, achievement notifications, extended statistics,
  original Racoon themes, and durable session recovery.
- Guarded portable profile export, no-write import preview, merge, and
  destructive replace with pre-read bounds and explicit acknowledgement.
- Stable long-history reporting beyond 100,000 records.
- Refined physical keyboard geometry, theme-aware status icons, and hand
  silhouettes that highlight the required finger.

### Tests

- 638 workspace/all-target tests passing; crash-recovery campaign (16 default + 115
  extended child-process crashes) passing.

Product-facing highlights, upgrade behavior, and current limitations are in
[RELEASE_NOTES_v1.1.md](RELEASE_NOTES_v1.1.md).

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
