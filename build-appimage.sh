#!/bin/bash
# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 Racoon Typper Contributors
# AppImage build script for Racoon Typper
# Usage: ./build-appimage.sh [version]

set -euo pipefail

if [ "$#" -gt 0 ]; then
    VERSION="$1"
else
    VERSION="$(node scripts/check-version.mjs --print)"
fi
APPDIR="AppDir"
BINARY="target/release/racoon-app"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd "$SCRIPT_DIR"

echo "=== Building release binary ==="
npm ci --prefix frontend
npm run tauri:build:binary --prefix frontend

echo "=== Preparing AppDir ==="
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/share/applications"
mkdir -p "$APPDIR/usr/share/icons/hicolor/256x256/apps"
mkdir -p "$APPDIR/usr/share/doc/racoon-typper"

cp "$BINARY" "$APPDIR/usr/bin/racoon-typper"
cp racoon-typper.desktop "$APPDIR/usr/share/applications/"
cp crates/app/icons/128x128@2x.png "$APPDIR/usr/share/icons/hicolor/256x256/apps/racoon-typper.png"
cp LICENSE "$APPDIR/usr/share/doc/racoon-typper/LICENSE"
cp THIRD_PARTY_NOTICES.md "$APPDIR/usr/share/doc/racoon-typper/THIRD_PARTY_NOTICES.md"

# AppRun
cat > "$APPDIR/AppRun" << 'APPRUN'
#!/bin/bash
set -euo pipefail
HERE="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
exec "$HERE/usr/bin/racoon-typper" "$@"
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
