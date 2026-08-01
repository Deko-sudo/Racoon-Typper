# Racoon Typper

[![CI](https://github.com/racoon-typper/racoon-typper/actions/workflows/ci.yml/badge.svg)](https://github.com/racoon-typper/racoon-typper/actions/workflows/ci.yml) [![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

Racoon Typper is a local-first desktop touch-typing trainer for focused practice, measurable progress, and offline use. It combines a Rust/Tauri desktop application with a Svelte interface and a local SQLite data store.

## Current status

The repository is undergoing a controlled modernization. The current release metadata is `1.1.0`, but the project is not claiming a production-ready artifact until the licensing, foundation, security, and release gates in [ROADMAP.md](ROADMAP.md) are complete.

The verified development baseline is Linux x86_64. See [SUPPORT_MATRIX.md](SUPPORT_MATRIX.md) for the distinction between development targets, configured packages, and supported releases.

Existing uncommitted feature and resource changes are tracked separately in [BASELINE.md](BASELINE.md). They are not automatically treated as released functionality.

## Baseline capabilities

- Time, words, quote, custom-text, and lesson-oriented typing workflows
- WPM, accuracy, heatmap, history, personal-best, and progress views
- Local SQLite persistence and replay data
- Keyboard training, weak-key analysis, and adaptive practice components
- Configurable local themes and typing settings
- Offline-first operation with no account or server requirement

Feature availability can vary between the committed baseline and the current worktree. Release notes will only claim capabilities demonstrated by a reviewed release artifact.

## Quick start from source

Install the platform prerequisites listed in [INSTALL.md](INSTALL.md), then run:

```bash
git clone https://github.com/racoon-typper/racoon-typper.git
cd racoon-typper
npm ci --prefix frontend
npm run check:version --prefix frontend
npm run tauri:dev --prefix frontend
```

The canonical commands run Tauri from `crates/app` so the Rust manifest, Tauri configuration, and frontend asset paths remain consistent.

## Build and verify

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm run check --prefix frontend
npm run build --prefix frontend

# Build the release binary without generating installers
npm run tauri:build:binary --prefix frontend

# Build configured platform bundles
npm run tauri:build --prefix frontend
```

See [ARCHITECTURE.md](ARCHITECTURE.md) for the current boundaries and [CONTRIBUTING.md](CONTRIBUTING.md) for the contributor workflow.

## Repository layout

```text
crates/domain/       Shared domain types and contracts
crates/core/         Typing engine, modes, scoring, lessons, analytics
crates/data/         SQLite connection, migrations, repositories
crates/resources/    Embedded content loading and validation
crates/app/          Tauri desktop shell and IPC adapters
frontend/            Svelte 5/Vite interface and typed IPC client
resources/           Runtime lessons, words, quotes, and themes
scripts/             Baseline verification and Tauri command wrappers
```

## Data and privacy

The application is designed to keep typing data locally. The currently verified Linux locations are:

| Data | Linux location |
|---|---|
| SQLite database | `${XDG_DATA_HOME:-$HOME/.local/share}/racoon-typper/data.db` |
| Settings | `${XDG_CONFIG_HOME:-$HOME/.config}/racoon-typper/settings.toml` |

Other platform paths remain unverified; see the support matrix. Export, retention, deletion, and migration behavior are being formalized before a production release. See [Data, Privacy, and Retention](docs/data/privacy.md) for the current policy.

## Downloads

Public release artifacts will be published on the [GitHub Releases page](https://github.com/racoon-typper/racoon-typper/releases) after the release pipeline and clean-install smoke tests pass. Configured package formats are not a support promise until they appear in [SUPPORT_MATRIX.md](SUPPORT_MATRIX.md) as verified.

## Roadmap and documentation

- [Modernization roadmap](ROADMAP.md)
- [Architecture](ARCHITECTURE.md)
- [Support matrix](SUPPORT_MATRIX.md)
- [Installation guide](INSTALL.md)
- [Contributing guide](CONTRIBUTING.md)
- [Release checklist](RELEASE_CHECKLIST.md)
- [Changelog](CHANGELOG.md)
- [Security policy](SECURITY.md)

## License

Project-owned code, resources, and metadata are released under the [Apache License 2.0](LICENSE). Third-party dependencies retain their original licenses; see [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md), the [dependency inventory](licenses/dependencies.json), the [asset provenance inventory](licenses/ASSET_PROVENANCE.md), and the [provenance record](licenses/PROVENANCE_ATTESTATION.md).
