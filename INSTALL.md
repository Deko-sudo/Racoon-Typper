# Installation — Racoon Typper

## Support status

The primary verified development target is Linux x86_64. Linux package smokes and Windows/macOS native candidate CI have separate recorded boundaries; they are not production support promises. Read [SUPPORT_MATRIX.md](SUPPORT_MATRIX.md) before distributing an artifact.

## From source

### Prerequisites

The Rust toolchain, Node.js/npm, and the WebKit/GTK development libraries required by Tauri must be installed.

**Arch Linux:**

```bash
sudo pacman -S rust webkit2gtk-4.1 gtk3 libayatana-appindicator librsvg npm base-devel
```

**Ubuntu 24.04:**

```bash
sudo apt update
sudo apt install -y libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev \
  librsvg2-dev build-essential curl
```

Install Rust with [rustup](https://rustup.rs/) and Node.js from the supported distribution channel if they are not already available.

### Development

```bash
git clone https://github.com/Deko-sudo/Racoon-Typper.git
cd Racoon-Typper
npm ci --prefix frontend
npm run check:version --prefix frontend
npm run tauri:dev --prefix frontend
```

The Rust toolchain is pinned by `rust-toolchain.toml`; `rustup` installs the
pinned version automatically. The frontend Node version is pinned by
`frontend/.nvmrc` (currently Node 26). CI uses Node 22; the package accepts
Node 22.12+ on the 22.x line or Node 24+. Record the actual version in candidate evidence.

### Validation and binary build

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm run check --prefix frontend
npm run build --prefix frontend
npm run tauri:build:binary --prefix frontend
```

The no-bundle command produces the Rust release binary without claiming that an installer has been validated. The configured bundle command is:

```bash
npm run tauri:build --prefix frontend
```

It may require platform-specific bundler tools. Installer support is only established by the release smoke tests described in [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md).

### Windows and macOS candidates

Build **the current reviewed source commit** on the target OS; old published
tags/assets do not acquire later fixes. Install Tauri's platform prerequisites
(Windows C++ tools and WebView2, or macOS Xcode command-line tools), then use
the same commands above. Tauri automatically merges `tauri.windows.conf.json`
or `tauri.macos.conf.json` with the common configuration.

Windows produces NSIS with the offline WebView2 installer embedded (a larger
download; installation retains the existing administrator/per-machine scope).
macOS produces `.app` and DMG; the configured minimum is 13.3. Build separately
on Intel and Apple Silicon. That minimum is a compatibility setting, not a
verified support promise. The candidate workflow produces both architectures.
No updater is included. Build/package tooling still needs network access.

The candidate workflow uploads packages and exact source/artifact hashes to
GitHub Actions, without publishing a release. Unsigned/unnotarized candidates
may be blocked by OS security checks. Do not disable OS protections; public
distribution requires the appropriate signing/notarization decision and manual
checks from [ADR 0009](docs/adr/0009-desktop-candidate-parity.md).

## Release artifacts

When a verified release exists, download artifacts from [GitHub Releases](https://github.com/Deko-sudo/Racoon-Typper/releases) and verify the published checksums. AppImage, Debian, RPM, Arch, Flatpak, and Windows installation instructions will be enabled only after the corresponding artifact passes the support matrix.

Do not use the old `v0.9.0` filenames or the obsolete `cargo tauri` commands from historical documentation.

## Data locations

The current verified Linux paths are:

| Data | Location |
|---|---|
| SQLite database | `${XDG_DATA_HOME:-$HOME/.local/share}/com.racoon.typper/data.db` |
| Settings | `${XDG_CONFIG_HOME:-$HOME/.config}/com.racoon.typper/settings.toml` |

Windows/macOS native CI resolves the platform known folders and checks a fresh disposable profile through restart. Manual upgrade, OS security prompts, Pictures-folder delivery and image paste remain platform acceptance checks; see [desktop evidence](docs/release/desktop-stability-evidence.md).

## Uninstalling source builds

Removing an executable or checkout does not remove the profile or exported PNGs.
Back up the current data/config directories above before deleting personal data.
Use Settings → Clear statistics for its documented narrower operation.

Linux also checks legacy sources whenever a current file is missing. With XDG
variables set, those sources are `$XDG_DATA_HOME/data.db` and
`$XDG_CONFIG_HOME/settings.toml`; otherwise they are under
`~/.local/share/racoon-typper` and `~/.config/racoon-typper`. Sources are copied,
not moved. Deleting only a current file can therefore import its legacy source
again on the next startup. See [privacy](docs/data/privacy.md) before removal.
Result PNGs remain in the OS Pictures folder / `Raccoon Typer` until deleted.

For candidate packaging, installation and rollback use the exact artifact and
checksum described in [Linux candidate verification](docs/release/linux-candidate.md).
