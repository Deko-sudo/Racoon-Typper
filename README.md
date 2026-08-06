<p align="center">
  <img src="crates/app/icons/icon.png" width="140" height="140" alt="Racoon Typper logo" />
</p>

<h1 align="center">Racoon Typper</h1>

<p align="center">
  A local-first desktop touch-typing trainer — focused practice, measurable progress, fully offline.
  <br />
  Rust · Tauri 2 · Svelte 5 · SQLite
</p>

<p align="center">
  <a href="https://github.com/Deko-sudo/Racoon-Typper/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/Deko-sudo/Racoon-Typper/actions/workflows/ci.yml/badge.svg" /></a>
  <a href="LICENSE"><img alt="License: Apache-2.0" src="https://img.shields.io/badge/license-Apache--2.0-blue.svg" /></a>
  <img alt="Rust" src="https://img.shields.io/badge/Rust-1.96.0-orange?logo=rust" />
  <img alt="Tauri" src="https://img.shields.io/badge/Tauri-2-black?logo=tauri" />
  <img alt="Svelte" src="https://img.shields.io/badge/Svelte-5-ff3e00?logo=svelte" />
  <img alt="Made for Linux" src="https://img.shields.io/badge/platform-Linux%20x86__64-success?logo=linux" />
</p>

---

## ⬇️ Download

<p align="center">
  <a href="https://github.com/Deko-sudo/Racoon-Typper/releases/latest">
    <img alt="Download the latest release" src="https://img.shields.io/badge/%E2%AC%87%EF%B8%8F%20Download-Latest%20Release-blue?style=for-the-badge&logo=github" />
  </a>
  &nbsp;&nbsp;
  <a href="https://github.com/Deko-sudo/Racoon-Typper/releases/latest" title="Each release also ships a .torrent for the Linux tarball">
    <img alt="Download via torrent" src="https://img.shields.io/badge/%E2%9A%A1%20via%20Torrent-P2P-orange?style=for-the-badge&logo=github" />
  </a>
</p>

> Pick your artifact (`.deb` / `.rpm` / `.AppImage` / Windows `.exe`) from the
> [Releases page](https://github.com/Deko-sudo/Racoon-Typper/releases). Each
> release also ships a `.torrent` for the Linux tarball — open it in any
> BitTorrent client; it carries a GitHub webseed, so it works even with zero
> peers. Verify with the release's `SHA256SUMS` before installing.

<!-- 📸 Showcase scaffold — captures pending. See docs/SCREENSHOTS.md.
     To enable: drop PNGs into docs/screenshots/, then uncomment the image
     lines below and remove the italic placeholders. -->

## 📸 Showcase

<!-- Hero: the running typing test — caret, colored chars, live stats, and the
     next-key glow on the virtual keyboard. The single image that sells the app. -->
> _Screenshot pending — `hero-test.png` (Test view, mid-test, `racoon_dark`)_
<!-- <p align="center"><img src="docs/screenshots/hero-test.png" alt="Racoon Typper typing test in progress" width="720" /></p> -->

### 🎨 Three built-in themes

<!-- Side-by-side theme comparison. Capture the same frame in each theme. -->
| Dark | Light | High Contrast |
| :---: | :---: | :---: |
| _pending_ | _pending_ | _pending_ |
<!-- | ![](docs/screenshots/theme-dark.png) | ![](docs/screenshots/theme-light.png) | ![](docs/screenshots/theme-hc.png) | -->

### ✨ Key screens

<!-- Feature highlight grid — one captioned screenshot per row. -->
| | |
| :--- | :--- |
| _pending_ — **Results & heatmap** · 4-stat summary plus per-key accuracy heatmap | _pending_ — **Dashboard** · streak, stat cards, 30-day progress chart |
| _pending_ — **Weak-key trainer** · accuracy-tinted keyboard + adaptive practice | _pending_ — **Replay** · scrub through a past test frame by frame |

<!-- | ![](docs/screenshots/results-heatmap.png) <br/>**Results & heatmap** — 4-stat summary plus per-key accuracy heatmap | ![](docs/screenshots/dashboard.png) <br/>**Dashboard** — streak, stat cards, 30-day progress chart |
| ![](docs/screenshots/weakkeys.png) <br/>**Weak-key trainer** — accuracy-tinted keyboard + adaptive practice | ![](docs/screenshots/replay.png) <br/>**Replay** — scrub through a past test frame by frame | -->

_Capture instructions and the full shot list live in
[docs/SCREENSHOTS.md](docs/SCREENSHOTS.md)._

---

Racoon Typper is a local-first desktop touch-typing trainer for focused
practice, measurable progress, and offline use. It combines a Rust/Tauri
desktop process with a Svelte interface, embedded learning resources, and a
local SQLite data store. **No accounts, no servers, no telemetry** — your
typing data never leaves your machine.

> **Status:** The repository is undergoing a controlled modernization.
> Release metadata is `1.1.0`, but this is **not** a production-ready release
> claim until the [roadmap gates](ROADMAP.md) are complete. The verified
> development target is **Linux x86_64**. See
> [SUPPORT_MATRIX.md](SUPPORT_MATRIX.md) for the distinction between
> development targets, configured packages, and supported releases.

## ✨ Features

- **Typing modes** — time, words, quotes, custom text, and guided lessons
- **Rich analytics** — live WPM & accuracy, per-key heatmap, history,
  personal bests, progress charts, daily goals, and streaks
- **Multi-language courses** — original Apache-2.0 lesson content for
  **15 languages** (en, ru, de, es, fr, it, pt, pl, cs, ro, uk, ja, ko,
  zh-hk, zh-tw)
- **Keyboard training** — weak-key analysis and adaptive practice
- **Themes** — original Racoon themes (dark, light, high-contrast),
  configurable locally
- **Replay & review** — replay past tests, frame by frame
- **Crash-safe** — durable session identity, crash recovery, and rotating
  pre-migration database backups
- **Offline-first** — runs entirely on-device; no network required

## 🧱 Tech stack

| Layer | Technology |
| --- | --- |
| Backend | Rust (edition 2021), modular workspace |
| Desktop shell | Tauri 2 |
| Frontend | Svelte 5, Vite 5, TypeScript 5 |
| Storage | SQLite (WAL mode) via rusqlite, settings in TOML |
| Resources | Embedded lessons, word lists, quotes, themes |
| Toolchain | Rust 1.96.0, Node 22+ (CI uses 22) |

### Architecture

A modular monolith with a strict backend boundary — dependencies point inward,
toward pure domain logic:

```
racoon-application   (session contracts & ports — no Tauri/SQLite/FS)
   ├── racoon-core       (typing engine, modes, scoring, analytics)
   └── racoon-domain     (shared types & contracts)
racoon-resources     (embedded content adapter)  ──► racoon-application
racoon-app           (Tauri + SQLite adapters)   ──► racoon-application
```

See [ARCHITECTURE.md](ARCHITECTURE.md) for the full boundary model and runtime
flow.

## 🚀 Quick start (from source)

**1. Install platform prerequisites** — see [INSTALL.md](INSTALL.md) for
details. On Ubuntu 24.04:

```bash
sudo apt install -y libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev \
  librsvg2-dev build-essential curl
```

**2. Clone, install, and run:**

```bash
git clone https://github.com/Deko-sudo/Racoon-Typper.git
cd Racoon-Typper
npm ci --prefix frontend
npm run check:version --prefix frontend   # verify version parity
npm run tauri:dev --prefix frontend       # launch the dev window
```

## 🔨 Build & verify

```bash
# Canonical checks (must all pass before commit)
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm run check --prefix frontend
npm run build --prefix frontend

# Release binary only (no installers)
npm run tauri:build:binary --prefix frontend

# Configured platform bundles (.deb / .rpm / .AppImage / NSIS)
npm run tauri:build --prefix frontend
```

CI runs these on every push and pull request — see
[CONTRIBUTING.md](CONTRIBUTING.md) for the contributor workflow.

## 📦 Platforms & packages

| Target | Status |
| --- | --- |
| **Linux x86_64** (Arch, Ubuntu 24.04) | ✅ Verified development target |
| `.deb`, `.rpm`, `.AppImage` | ⚙️ CI build targets — experimental, not yet supported releases |
| Windows x86_64 (NSIS) | ⚙️ CI build target — not yet supported |
| Arch `PKGBUILD` | 📄 Community recipe — not a supported release |
| Flatpak | 🚧 Manifest present, pending redesign |
| macOS · Linux ARM | ❌ Unsupported (no config) |

Public release artifacts will appear on the
[Releases page](https://github.com/Deko-sudo/Racoon-Typper/releases) once the
release pipeline and clean-install smoke tests pass.

## 📁 Repository layout

```
crates/domain/       Shared domain types and contracts
crates/core/         Typing engine, modes, scoring, lessons, analytics
crates/data/         SQLite connection, migrations, repositories
crates/resources/    Embedded content loading and validation
crates/application/  Session contracts and ports (infra-free)
crates/app/          Tauri desktop shell and IPC adapters
frontend/            Svelte 5 / Vite interface and typed IPC client
resources/           Runtime lessons, words, quotes, and themes
scripts/             Baseline verification and Tauri command wrappers
licenses/            Provenance inventories, SBOM, asset records
```

## 🔒 Data & privacy

The application keeps all typing data locally. Verified Linux locations:

| Data | Linux location |
| --- | --- |
| SQLite database | `${XDG_DATA_HOME:-$HOME/.local/share}/racoon-typper/data.db` |
| Settings | `${XDG_CONFIG_HOME:-$HOME/.config}/racoon-typper/settings.toml` |

Export, retention, deletion, and migration behavior are being formalized before
a production release. See the [Data, Privacy & Retention
policy](docs/data/privacy.md) and the [Profile Transfer & Recovery
Runbook](docs/data/profile-transfer.md) for the current state and safe-transfer
constraints.

## 📚 Documentation

- [Roadmap](ROADMAP.md) — modernization gates and milestones
- [Architecture](ARCHITECTURE.md) — boundaries and runtime flow
- [Support matrix](SUPPORT_MATRIX.md) — verified vs. configured targets
- [Installation guide](INSTALL.md) — platform prerequisites
- [Contributing guide](CONTRIBUTING.md) — contributor workflow
- [Security policy](SECURITY.md)
- [Changelog](CHANGELOG.md)
- [Release checklist](RELEASE_CHECKLIST.md)

## 🧪 Quality

- **~640 workspace tests** including crash-recovery campaigns
- Forward-only Refinery migrations (V001–V009) with a full historical
  upgrade-matrix (V1..V7 → V9) verified through the production runner
- Least-privilege Tauri capabilities audited against the registered command
  surface; redacted IPC error boundary (no dynamic payload leakage)
- All GitHub Actions pinned to full commit SHAs; Apache-2.0-only license
  policy enforced in CI

## 📄 License

Project-owned code, resources, and metadata are released under the
[Apache License 2.0](LICENSE). Third-party dependencies retain their original
licenses — see [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md), the
[dependency inventory](licenses/dependencies.json), the
[asset provenance inventory](licenses/ASSET_PROVENANCE.md), and the
[provenance record](licenses/PROVENANCE_ATTESTATION.md).
