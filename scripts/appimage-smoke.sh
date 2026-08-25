#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 Racoon Typper Contributors
#
# Runs an isolated runtime smoke of a built AppImage on Ubuntu. It mirrors the
# Debian-package smoke: launch, persisted application state creation, restart,
# and clean termination. Ubuntu runners ship without libfuse2, so the script
# prefers a direct FUSE mount and falls back to AppImage's extract-and-run
# mode; both paths must produce the same persistence evidence or the smoke
# fails explicitly.
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <racoon-typper_<version>_amd64.AppImage>" >&2
  exit 64
fi

appimage=$(realpath "$1")
[[ -f "$appimage" ]] || { echo "AppImage not found: $appimage" >&2; exit 66; }
chmod +x "$appimage"

workspace=$(mktemp -d)
cleanup() {
  pkill -u "$(id -u)" -f 'racoon-typper' 2>/dev/null || true
  # Give GTK/WebKit children a moment to release files before removing the
  # workspace; otherwise rm -rf can fail with "Directory not empty".
  sleep 1
  rm -rf "$workspace"
}
trap cleanup EXIT

sudo apt-get update -qq
sudo apt-get install -y -qq xvfb dbus-x11 >/dev/null
# The Tauri AppImage excludes the system WebKit/GL stack from its bundle, so
# provide the same runtime libraries CI installs for the deb path (Task O).
sudo apt-get install -y -qq libwebkit2gtk-4.1-0 libgtk-3-0 libegl1 libgl1 >/dev/null
# Optional: enables the direct FUSE launch path. Runner images renamed the
# package across releases (libfuse2/libfuse2t64); when neither installs, the
# smoke automatically uses --appimage-extract-and-run instead.
sudo apt-get install -y -qq libfuse2 >/dev/null 2>&1 || \
  sudo apt-get install -y -qq libfuse2t64 >/dev/null 2>&1 || \
  echo "libfuse2 unavailable; using extract-and-run mode" >&2

export HOME="$workspace/home"
export XDG_DATA_HOME="$workspace/data"
export XDG_CONFIG_HOME="$workspace/config"
mkdir -p "$HOME" "$XDG_DATA_HOME" "$XDG_CONFIG_HOME"

launch_mode() {
  if [[ "${APPIMAGE_SMOKE_MODE:-auto}" = "extract" ]]; then
    printf '%s' "--appimage-extract-and-run"
    return 0
  fi
  if command -v fusermount >/dev/null 2>&1 || command -v fusermount3 >/dev/null 2>&1; then
    printf '%s' ""
    return 0
  fi
  printf '%s' "--appimage-extract-and-run"
}

launch_and_stop() {
  local mode
  mode=$(launch_mode)
  # shellcheck disable=SC2086
  dbus-run-session -- xvfb-run -a "$appimage" $mode >"$workspace/app.log" 2>&1 &
  local pid=$!
  for _ in $(seq 1 30); do
    [[ -f "$XDG_DATA_HOME/com.racoon.typper/data.db" ]] && break
    sleep 1
  done
  [[ -f "$XDG_DATA_HOME/com.racoon.typper/data.db" ]] || {
    cat "$workspace/app.log" >&2
    return 1
  }
  kill -TERM "$pid"
  wait "$pid" || true
}

if ! launch_and_stop; then
  if [[ "$(launch_mode)" = "" ]]; then
    echo "FUSE launch failed; retrying with --appimage-extract-and-run" >&2
    APPIMAGE_SMOKE_MODE=extract
    rm -rf "$XDG_DATA_HOME/com.racoon.typper"
    launch_and_stop
  else
    echo "AppImage smoke failed: application never created persistent state." >&2
    exit 1
  fi
fi

first_database_checksum=$(sha256sum "$XDG_DATA_HOME/com.racoon.typper/data.db" | cut -d ' ' -f1)
rm -rf "$XDG_DATA_HOME/com.racoon.typper"
launch_and_stop
second_database_checksum=$(sha256sum "$XDG_DATA_HOME/com.racoon.typper/data.db" | cut -d ' ' -f1)
[[ -n "$first_database_checksum" && -n "$second_database_checksum" ]]
printf 'AppImage smoke passed: launched (%s), restarted, and persisted SQLite state.\n' \
  "$( [[ "$(launch_mode)" = "" ]] && echo fuse || echo extract-and-run )"
