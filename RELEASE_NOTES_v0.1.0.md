# Racoon Typper v0.1.0 — Release Notes

**Date**: 2026-06-22
**Status**: First public release

## Overview

Racoon Typper is a local desktop touch-typing trainer for Linux, combining the best of Monkeytype (statistics, modes) and Stamina (structured learning — planned for v0.2+).

## Features

### Test Modes
- **Time**: 15s, 30s, 60s, 120s with auto-generated text
- **Words**: 10, 25, 50, 100 words from dictionary
- **Quote**: Random quotes from built-in database
- **Custom Text**: User-created texts (full CRUD)

### Languages
- English (500+ words, 20 quotes)
- Russian (500+ words, 20 proverbs)

### Statistics
- WPM (Net)
- Raw WPM
- Accuracy (Net + Raw)
- Heatmap (keyboard visualization with error highlighting)
- Personal Bests (per mode configuration)
- Test History (SQLite persistence)

### Customization
- 3 built-in themes (Serika Dark, Serika Light, Racoon Dark)
- Settings: font size, caret style, live WPM/accuracy display
- Settings stored in `~/.config/racoon-typper/settings.toml`

### Data Storage
- SQLite database: `~/.local/share/racoon-typper/data.db`
- Settings: `~/.config/racoon-typper/settings.toml`
- XDG-compliant paths
- Fully offline — no network requests

## Architecture

- **Backend**: Rust + Tauri 2.x (synchronous architecture)
- **Frontend**: Svelte 5 + Vite (SPA, no SSR)
- **Database**: SQLite (rusqlite, WAL mode)
- **5 crates**: domain, core, data, resources, app

## Test Results

- 155 tests passing (146 unit/integration + 9 e2e)
- 0 clippy warnings
- cargo fmt clean

## Known Limitations

- No Lessons/Stamina module (planned v0.2+)
- No Consistency metric (planned v0.2)
- No Graph visualization (planned v0.2)
- No Daily Stats / Streaks (planned v0.2)
- No Weak Keys Engine (planned v0.2+)
- No Plugin System (planned post-v1.0)
- Linux only (Arch Linux primary target)

## Installation (Arch Linux)

### From source

```bash
sudo pacman -S rust webkit2gtk-4.1 base-devel npm
git clone https://github.com/racoon-typper/racoon-typper.git
cd racoon-typper
makepkg -si
```

### From AUR (planned)

```bash
yay -S racoon-typper
```

## Roadmap

### v0.2.0
- Consistency metric
- Graph (WPM/accuracy over time)
- Daily stats aggregation
- Streaks
- Lessons (Stamina-style course)

### Post v1.0
- Weak Keys Engine
- Adaptive training
- Plugin System (WASM)
- More languages (DE, FR, ES)

## License

Apache-2.0