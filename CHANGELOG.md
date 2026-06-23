# Changelog — Racoon Typper

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
- Migration tests: V001→V002→V003
- Stress tests: 50k tests, 10k replays, 90-day aggregation
- Performance benchmarks included

### Architecture

- 5 Rust crates: domain, core, data, resources, app
- Tauri 2.x + Svelte 5 + Vite
- SQLite (rusqlite 0.31, refinery 0.8)
- Synchronous architecture (no Tokio, no async)

### Known Limitations

- Linux only (Arch, Ubuntu, Fedora). Windows via NSIS but untested on CI.
- No plugin system (planned post-v1.0)
- No Zen mode (planned post-v1.0)
- No sound effects (planned post-v1.0)
- No multiplayer (planned post-v1.0)
- 17 unmaintained dependency warnings (all Tauri transitive)
- No auto-update mechanism