#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 Racoon Typper Contributors
set -euo pipefail
if [[ ${GITHUB_ACTIONS:-} != true || ${RUNNER_ENVIRONMENT:-} != github-hosted || ${RUNNER_OS:-} != macOS ]]; then
  echo 'Use only on a fresh GitHub-hosted macOS runner, never a personal desktop.' >&2
  exit 1
fi
test "$#" -eq 1
test -f "$1"
workspace=$(mktemp -d)
mounted=false
cleanup() {
  if $mounted; then hdiutil detach "$workspace/volume"; fi
  rm -rf "$workspace"
}
trap cleanup EXIT
swiftc scripts/macos-window-probe.swift -o "$workspace/window-probe"
mkdir "$workspace/volume"
hdiutil attach -readonly -nobrowse -mountpoint "$workspace/volume" "$1"
mounted=true
app=$(find "$workspace/volume" -maxdepth 1 -type d -name '*.app' -print -quit)
test -n "$app"
ditto "$app" "$workspace/candidate.app"
hdiutil detach "$workspace/volume"
mounted=false
python3 scripts/desktop-runtime-smoke.py --executable "$workspace/candidate.app/Contents/MacOS/racoon-app" \
  --window-probe "$workspace/window-probe" --evidence candidate-evidence/macos-runtime
