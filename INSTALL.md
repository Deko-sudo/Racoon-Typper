# Installation — Racoon Typper

## Arch Linux (AUR / PKGBUILD)

### From PKGBUILD

```bash
git clone https://github.com/racoon-typper/racoon-typper.git
cd racoon-typper
makepkg -si
```

### From AUR (planned)

```bash
yay -S racoon-typper
```

## AppImage

```bash
chmod +x racoon-typper-v0.9.0.AppImage
./racoon-typper-v0.9.0.AppImage
```

## Flatpak

```bash
flatpak-builder build-dir com.racoon.typper.json
flatpak install --user build-dir
flatpak run com.racoon.typper
```

## Windows (NSIS)

1. Download `racoon-typper-v0.9.0-setup.exe`
2. Run installer
3. Launch from Start Menu

## From Source (any Linux)

### Dependencies

**Arch Linux:**
```bash
sudo pacman -S rust webkit2gtk-4.1 base-devel npm
```

**Ubuntu 24.04:**
```bash
sudo apt install -y libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev build-essential curl
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash -
sudo apt install -y nodejs
```

**Fedora:**
```bash
sudo dnf install -y webkit2gtk4.1-devel gtk3-devel librsvg2-devel openssl-devel
sudo dnf group install "Development Tools"
```

### Build

```bash
git clone https://github.com/racoon-typper/racoon-typper.git
cd racoon-typper
npm install --prefix frontend
cargo tauri dev    # development
cargo tauri build  # production
```

Binary: `target/release/racoon-app`

## Data Locations

| Platform | Database | Settings |
|----------|----------|----------|
| Linux | `~/.local/share/racoon-typper/data.db` | `~/.config/racoon-typper/settings.toml` |
| Windows | `%LOCALAPPDATA%\racoon-typper\data.db` | `%APPDATA%\racoon-typper\settings.toml` |

## Uninstall

### Arch Linux
```bash
sudo pacman -R racoon-typper
```

### AppImage
```bash
rm racoon-typper-v*.AppImage
```

### From source
```bash
rm -rf racoon-typper/
rm ~/.local/share/racoon-typper/data.db
rm ~/.config/racoon-typper/settings.toml
```