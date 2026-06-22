# Contributing to Racoon Typper

## Development Setup

### Prerequisites (Arch Linux)

```bash
sudo pacman -S rust webkit2gtk-4.1 base-devel npm
```

### Build from source

```bash
git clone https://github.com/racoon-typper/racoon-typper.git
cd racoon-typper
npm install --prefix frontend
cargo tauri dev
```

## Code Style

### Rust

- Run `cargo fmt --all` before commit
- Run `cargo clippy --workspace -- -D warnings` — must pass with 0 warnings
- All public functions must have doc comments
- Tests are required for new logic

### Frontend (Svelte 5 + TypeScript)

- Components in `frontend/src/components/`
- Types in `frontend/src/lib/types/index.ts`
- IPC wrappers in `frontend/src/lib/api/ipc.ts`
- No optional chaining (`?.`) or optional params (`?`) — Svelte 5 without preprocess

## Architecture

See `ARCHITECTURE.md` — single source of truth.

Key principles:
- `domain` crate: pure types, zero logic, zero deps (except serde)
- `core` crate: engine logic, synchronous, no DB/UI deps
- `data` crate: SQLite, rusqlite, repository pattern
- `resources` crate: word packs, quotes, themes (include_str! embedded)
- `app` crate: Tauri shell, IPC commands, state management

## Commit Convention

```
feat: <description>
fix: <description>
refactor: <description>
docs: <description>
test: <description>
chore: <description>
```

## Testing

- Unit tests: in-module `#[cfg(test)] mod tests`
- Integration tests: `crates/core/tests/`
- Run all: `cargo test --workspace`
- Minimum coverage: all public functions

## Pull Requests

1. Create feature branch from `main`
2. Ensure `cargo fmt`, `cargo clippy`, `cargo test` all pass
3. Ensure `npm run build` passes
4. Squash commits before merge
5. PR description must reference the issue

## Release

1. Update version in `Cargo.toml` (workspace)
2. Update `crates/app/tauri.conf.json` version
3. Run full test suite
4. Create git tag `vX.Y.Z`
5. Create GitHub release with release notes