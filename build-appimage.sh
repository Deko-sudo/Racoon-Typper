#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 Racoon Typper Contributors
# Builds a pinned AppImage from the current, version-consistent source tree.
set -euo pipefail

readonly APPIMAGETOOL_VERSION="13"
readonly APPIMAGETOOL_SHA256="df3baf5ca5facbecfc2f3fa6713c29ab9cefa8fd8c1eac5d283b79cab33e4acb"
readonly APPIMAGETOOL_URL="https://github.com/AppImage/AppImageKit/releases/download/${APPIMAGETOOL_VERSION}/obsolete-appimagetool-x86_64.AppImage"

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$script_dir"

version="$(node scripts/check-version.mjs --print)"
if [[ $# -gt 1 ]]; then
  echo "usage: $0 [version]" >&2
  exit 64
fi
if [[ $# -eq 1 && "$1" != "$version" ]]; then
  echo "requested version $1 does not match project version $version" >&2
  exit 65
fi

output_dir="$script_dir/dist"
tool_dir="$script_dir/.cache/appimagetool"
tool_path="$tool_dir/appimagetool-x86_64.AppImage"
app_dir="$(mktemp -d "${TMPDIR:-/tmp}/racoon-typper-appdir.XXXXXX")"
cleanup() { rm -rf "$app_dir"; }
trap cleanup EXIT

fetch_appimagetool() {
  mkdir -p "$tool_dir"
  if [[ ! -f "$tool_path" ]]; then
    local temporary
    temporary="$(mktemp "${tool_path}.download.XXXXXX")"
    trap 'rm -f "$temporary"' RETURN
    curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 "$APPIMAGETOOL_URL" --output "$temporary"
    printf '%s  %s\n' "$APPIMAGETOOL_SHA256" "$temporary" | sha256sum --check --status
    chmod 0755 "$temporary"
    mv "$temporary" "$tool_path"
    trap - RETURN
  fi
  printf '%s  %s\n' "$APPIMAGETOOL_SHA256" "$tool_path" | sha256sum --check --status
}

printf '=== Building release binary for %s ===\n' "$version"
npm ci --prefix frontend
npm run tauri:build:binary --prefix frontend

printf '=== Preparing AppDir ===\n'
install -Dm755 target/release/racoon-app "$app_dir/usr/bin/racoon-typper"
install -Dm644 racoon-typper.desktop "$app_dir/usr/share/applications/racoon-typper.desktop"
install -Dm644 crates/app/icons/128x128@2x.png "$app_dir/usr/share/icons/hicolor/256x256/apps/racoon-typper.png"
install -Dm644 LICENSE "$app_dir/usr/share/doc/racoon-typper/LICENSE"
install -Dm644 THIRD_PARTY_NOTICES.md "$app_dir/usr/share/doc/racoon-typper/THIRD_PARTY_NOTICES.md"
install -Dm644 racoon-typper.desktop "$app_dir/racoon-typper.desktop"
cat > "$app_dir/AppRun" <<'APPRUN'
#!/usr/bin/env bash
set -euo pipefail
here="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
exec "$here/usr/bin/racoon-typper" "$@"
APPRUN
chmod 0755 "$app_dir/AppRun"

printf '=== Verifying pinned appimagetool %s ===\n' "$APPIMAGETOOL_VERSION"
fetch_appimagetool
mkdir -p "$output_dir"
output_path="$output_dir/racoon-typper-v${version}-x86_64.AppImage"
rm -f "$output_path"

printf '=== Building AppImage ===\n'
ARCH=x86_64 "$tool_path" "$app_dir" "$output_path"
test -s "$output_path"
printf '=== Done: %s ===\n' "$output_path"
sha256sum "$output_path"
