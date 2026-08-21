# Changelog — Racoon Typper v1.2.0 (DRAFT — for owner review)

> **Status:** DRAFT. Not yet merged into CHANGELOG.md. The caret-highlight
> section documents the shipped B1 behavior (neutral raised surface), matching
> the theme-pack audit and TECH_DEBT TD8.

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

## Notes for the owner

1. **Caret highlight** — the draft previously described B2 (accent color);
   the shipped implementation, theme-pack audit, and TD8 all record B1
   (neutral raised surface). This draft now documents B1.
2. **Version parity** — `CHANGELOG.md` and `RELEASE_NOTES_v1.1.md` still
   describe v1.1.0. This draft is the v1.2.0 section to prepend.
3. **Release tag** — the published `v1.2.0` release is stale (built from a tag
   with 25 themes). A re-tag/re-release from current `master` is required
   before this changelog is accurate.
