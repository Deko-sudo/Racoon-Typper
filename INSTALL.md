# Installation — Racoon Typper

## Support status

The current verified development target is Linux x86_64. Package formats and Windows are currently build targets or experiments, not production support promises. Read [SUPPORT_MATRIX.md](SUPPORT_MATRIX.md) before distributing an artifact.

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
cd racoon-typper
npm ci --prefix frontend
npm run check:version --prefix frontend
npm run tauri:dev --prefix frontend
```

The Rust toolchain is pinned by `rust-toolchain.toml`; `rustup` installs the
pinned version automatically. The frontend Node version is pinned by
`frontend/.nvmrc` (use `nvm use`/`fnm use` or install Node >= 22 manually).

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

## Release artifacts

When a verified release exists, download artifacts from [GitHub Releases](https://github.com/Deko-sudo/Racoon-Typper/releases) and verify the published checksums. AppImage, Debian, RPM, Arch, Flatpak, and Windows installation instructions will be enabled only after the corresponding artifact passes the support matrix.

Do not use the old `v0.9.0` filenames or the obsolete `cargo tauri` commands from historical documentation.

## Data locations

The current verified Linux paths are:

| Data | Location |
|---|---|
| SQLite database | `${XDG_DATA_HOME:-$HOME/.local/share}/racoon-typper/data.db` |
| Settings | `${XDG_CONFIG_HOME:-$HOME/.config}/racoon-typper/settings.toml` |

Windows and other platform-specific paths remain unverified until the lifecycle/platform phase completes.

## Uninstalling source builds

Removing the checkout does not remove user data. Delete it only after exporting or backing up anything you want to keep:

```bash
rm -rf racoon-typper/
rm -rf "${XDG_DATA_HOME:-$HOME/.local/share}/racoon-typper"
rm -rf "${XDG_CONFIG_HOME:-$HOME/.config}/racoon-typper"
```

Package-manager uninstall instructions will be documented with each verified package format.
