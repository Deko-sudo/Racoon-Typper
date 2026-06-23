# Changelog — Racoon Typper

## v1.1.0 (2026-06-23) — Community & Polish Release

### New Features

- **Sound Engine**: key press, error, lesson complete, achievement sounds with volume control and cooldown
- **Zen Mode**: distraction-free typing — hides navbar, stats, panels during test
- **3 New Themes**: Dracula, Catppuccin Mocha, Nord (total: 6 themes)
- **Achievement Notifications**: toast notifications when achievements unlock
- **Session Recovery**: restore previous session after app crash/close
- **Extended Statistics**: best day, most active hour, avg session duration, total chars/words
- **Profile Export**: full profile export (settings, tests, lessons, PBs, custom texts) as JSON
- **Sound Settings**: sound_enabled, sound_volume in settings.toml
- **Zen Mode Setting**: zen_mode_enabled in settings.toml

### Improvements

- Dashboard extended with 5 new stat cards
- Settings page: 3 new controls (sound toggle, volume slider, zen mode toggle)
- 6 themes total (was 3)

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
- 3 themes: Serika Dark, Serika Light, Racoon Dark
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