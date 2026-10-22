#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 Racoon Typper Contributors
# Ubuntu runner/container only. A marker row, not just file existence/checksums,
# proves restart retention; the helper terminates only its own process session.
set -euo pipefail
[[ $# -eq 1 ]] || { echo "usage: $0 <racoon-typper.AppImage>" >&2; exit 64; }
appimage=$(realpath "$1")
[[ -f "$appimage" ]] || { echo "AppImage not found: $appimage" >&2; exit 66; }
chmod +x "$appimage"
script_directory=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
sudo apt-get update -qq
sudo apt-get install -y -qq xvfb dbus-x11 python3 libwebkit2gtk-4.1-0 libgtk-3-0 libegl1 libgl1 >/dev/null
sudo apt-get install -y -qq libfuse2 >/dev/null 2>&1 || sudo apt-get install -y -qq libfuse2t64 >/dev/null 2>&1 || true
run_smoke() {
  python3 "$script_directory/linux-runtime-smoke.py" -- dbus-run-session -- xvfb-run -a "$appimage" "$@"
}
if [[ "${APPIMAGE_SMOKE_MODE:-auto}" != extract ]] && { command -v fusermount >/dev/null 2>&1 || command -v fusermount3 >/dev/null 2>&1; }; then
  if run_smoke; then exit 0; fi
  echo "FUSE smoke failed; retrying extract-and-run in a fresh disposable profile" >&2
fi
run_smoke --appimage-extract-and-run
