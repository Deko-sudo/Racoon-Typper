# Racoon Typper

Local desktop touch-typing trainer for Linux. Combines Monkeytype and Stamina.

## Features

- **4 Test Modes**: Time (15/30/60/120s), Words (10/25/50/100), Quote, Custom Text
- **2 Languages**: English, Russian (500+ words each, 20 quotes each)
- **Statistics**: WPM, Raw WPM, Accuracy, Raw Accuracy, Heatmap
- **Personal Bests**: Automatic tracking per mode configuration
- **Test History**: Full history with SQLite persistence
- **Custom Texts**: Create, edit, delete, search, and test with your own texts
- **Themes**: 3 built-in themes (Serika Dark, Serika Light, Racoon Dark)
- **Settings**: Font size, caret style, live WPM/accuracy display
- **Fully Offline**: No network requests, all data stored locally

## Installation

See [INSTALL.md](INSTALL.md) for all installation methods.

### Quick Start (Arch Linux)

```bash
sudo pacman -S rust webkit2gtk-4.1 base-devel npm
git clone https://github.com/racoon-typper/racoon-typper.git
cd racoon-typper
makepkg -si
```

## Repository Structure

```
racoon-typper/
├── crates/
│   ├── domain/      # Pure types (TestRecord, FinalStats, Settings, etc.)
│   ├── core/        # Engine logic (Input, Typing, Stats, TestMode trait)
│   ├── data/        # SQLite layer (rusqlite, migrations, repositories)
│   ├── resources/   # Word packs, quote loaders (include_str! embedded)
│   └── app/         # Tauri shell (IPC commands, state, event bridge)
├── frontend/        # Svelte 5 + Vite SPA
│   ├── src/
│   │   ├── components/    # TestView, HistoryView, BestsView, etc.
│   │   ├── lib/api/       # Typed IPC wrappers
│   │   └── lib/types/     # TypeScript type definitions
├── resources/       # Word packs, quotes, themes
│   ├── words/       # en.txt, ru.txt
│   ├── quotes/      # en.toml, ru.toml
│   └── themes/      # serika_dark, serika_light, racoon_dark
└── Cargo.toml       # Workspace root
```

## Architecture

- **Backend**: Rust (Tauri 2.x), synchronous architecture
- **Frontend**: Svelte 5 + Vite (SPA, no SSR)
- **Database**: SQLite (rusqlite, WAL mode, no connection pool)
- **Settings**: TOML file (~/.config/racoon-typper/settings.toml)
- **Data**: SQLite (~/.local/share/racoon-typper/data.db)

See `ARCHITECTURE.md` for full specification.

## Screenshots

*(Screenshots will be added before public release)*

## Roadmap

### v0.1.0 (Current)
- [x] Time/Words/Quote/Custom modes
- [x] WPM, Raw WPM, Accuracy
- [x] Heatmap data
- [x] SQLite persistence
- [x] Personal Bests
- [x] Custom Texts CRUD
- [x] 3 themes, 5 settings
- [x] EN/RU dictionaries

### v0.2.0
- [ ] Consistency metric
- [ ] Graph (WPM/accuracy over time)
- [ ] Daily stats aggregation
- [ ] Streaks

### Post v1.0
- [ ] Lessons (Stamina-style course)
- [ ] Weak Keys Engine
- [ ] Adaptive training
- [ ] Plugin System (WASM)
- [ ] More languages

## License

Apache-2.0