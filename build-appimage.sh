#!/bin/bash
# AppImage build script for Racoon Typper
# Usage: ./build-appimage.sh [version]

set -e

VERSION=${1:-0.9.0}
APPDIR="AppDir"
BINARY="target/release/racoon-app"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd "$SCRIPT_DIR"

echo "=== Building release binary ==="
cargo build --release -p racoon-app

echo "=== Preparing AppDir ==="
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/share/applications"
mkdir -p "$APP_DIR/usr/share/icons/hicolor/256x256/apps"

cp "$BINARY" "$APPDIR/usr/bin/racoon-typper"
cp racoon-typper.desktop "$APPDIR/usr/share/applications/"
cp crates/app/icons/128x128@2x.png "$APPDIR/usr/share/icons/hicolor/256x256/apps/racoon-typper.png"

# AppRun
cat > "$APPDIR/AppRun" << 'APPRUN'
#!/bin/bash
exec "${APPDIR}/usr/bin/racoon-typper" "$@"
APPRUN
chmod +x "$APPDIR/AppRun"

# .desktop for AppImage
cp racoon-typper.desktop "$APPDIR/racoon-typper.desktop"

echo "=== Downloading appimagetool ==="
if [ ! -f "appimagetool" ]; then
    wget -q "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage" -O appimagetool
    chmod +x appimagetool
fi

echo "=== Building AppImage ==="
ARCH=x86_64 ./appimagetool "$APPDIR" "racoon-typper-v${VERSION}.AppImage"

echo "=== Done: racoon-typper-v${VERSION}.AppImage ==="
ls -lh "racoon-typper-v${VERSION}.AppImage"