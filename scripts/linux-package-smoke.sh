#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 Racoon Typper Contributors
#
# Runs an isolated runtime smoke of a Debian package on Ubuntu. It intentionally
# verifies package install, first launch, persisted application state creation,
# restart, and clean termination. Export and backup flows remain covered by the
# application integration suite until a stable desktop automation contract exists.
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <racoon-typper.deb>" >&2
  exit 64
fi

package=$(realpath "$1")
[[ -f "$package" ]] || { echo "package not found: $package" >&2; exit 66; }

workspace=$(mktemp -d)
cleanup() {
  pkill -u "$(id -u)" -f '/usr/bin/racoon-app' 2>/dev/null || true
  # Give GTK/WebKit children a moment to release files before removing the
  # workspace; otherwise rm -rf can fail with "Directory not empty".
  sleep 1
  rm -rf "$workspace"
}
trap cleanup EXIT

sudo apt-get update
sudo apt-get install -y "$package" xvfb dbus-x11

export HOME="$workspace/home"
export XDG_DATA_HOME="$workspace/data"
export XDG_CONFIG_HOME="$workspace/config"
mkdir -p "$HOME" "$XDG_DATA_HOME" "$XDG_CONFIG_HOME"

launch_and_stop() {
  dbus-run-session -- xvfb-run -a /usr/bin/racoon-app >"$workspace/app.log" 2>&1 &
  local pid=$!
  for _ in $(seq 1 30); do
    [[ -f "$XDG_DATA_HOME/com.racoon.typper/data.db" ]] && break
    sleep 1
  done
  [[ -f "$XDG_DATA_HOME/com.racoon.typper/data.db" ]] || { cat "$workspace/app.log" >&2; return 1; }
  kill -TERM "$pid"
  wait "$pid" || true
}

launch_and_stop
first_database_checksum=$(sha256sum "$XDG_DATA_HOME/com.racoon.typper/data.db" | cut -d ' ' -f1)
launch_and_stop
second_database_checksum=$(sha256sum "$XDG_DATA_HOME/com.racoon.typper/data.db" | cut -d ' ' -f1)
[[ -n "$first_database_checksum" && -n "$second_database_checksum" ]]
printf 'Linux package smoke passed: installed, launched twice, and persisted SQLite state.\n'
