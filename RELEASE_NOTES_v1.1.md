# Release Notes — Racoon Typper v1.1.0

**Release Date**: 2026-06-23
**Tag**: v1.1.0
**Tests**: 418 (0 failed)
**Clippy**: 0 warnings

---

## What's New

### Sound Engine
Typing feedback sounds for key presses, errors, lesson completions, and achievement unlocks. Configurable volume and enable/disable toggle in Settings.

### Zen Mode
Distraction-free typing mode. When enabled, the navbar, statistics, and panels hide during a test — leaving only the text, caret, and progress. Toggle in Settings → Zen Mode.

### New Themes
Three community-requested themes added:
- **Dracula** — dark purple
- **Catppuccin Mocha** — warm dark
- **Nord** — cold blue dark

### Achievement Notifications
Toast notifications now appear when achievements unlock (First Test, 50 WPM, 100 WPM, 7 Day Streak, etc.).

### Session Recovery
If the app closes during an active test, it offers to restore the previous session on next launch — including text, typed characters, mode, and elapsed time.

### Extended Statistics
Dashboard now shows:
- Best day (WPM + date)
- Most active hour
- Average session duration
- Total typed characters
- Total typed words

### Profile Export
Full profile export (settings, tests, lessons, personal bests, custom texts) as JSON file.

---

## Settings Changes

New settings in settings.toml:
- `sound_enabled` (default: false)
- `sound_volume` (default: 0.5)
- `zen_mode_enabled` (default: false)

---

## Upgrading from v1.0.0

No database migration needed. New settings default to safe values. Existing data preserved.

---

## Known Limitations

- Sound is tone-based (no external audio files)
- No import profile yet (export only)
- Linux primary platform; Windows via NSIS but untested on CI
- 17 unmaintained dependency warnings (all Tauri transitive)