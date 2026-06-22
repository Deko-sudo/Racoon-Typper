# Racoon Typper

Local desktop touch-typing trainer for Linux. Combines Monkeytype and Stamina.

## Status

**v0.1.0 — Sprint 1 (Foundation)**

## Build (Arch Linux)

### Dependencies

```bash
sudo pacman -S rust webkit2gtk-4.1 base-devel npm
```

### Development

```bash
cd racoon-typper
npm install --prefix frontend
cargo tauri dev
```

### Production

```bash
cargo tauri build
# → target/release/racoon-typper
```

## Architecture

See `ARCHITECTURE.md`, `API_CONTRACT.md`, `IMPLEMENTATION_PLAN.md`, `REPOSITORY_BOOTSTRAP.md`.

## License

MIT