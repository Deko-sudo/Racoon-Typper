#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 Racoon Typper Contributors
set -euo pipefail
if [[ ${GITHUB_ACTIONS:-} != true || ${RUNNER_ENVIRONMENT:-} != github-hosted || ${RUNNER_OS:-} != macOS ]]; then
  echo 'Use only on a fresh GitHub-hosted macOS runner, never a personal desktop.' >&2
  exit 1
fi
[[ $# -eq 1 || ( $# -eq 2 && $2 == --ui-probe ) ]]
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
probe_arguments=()
evidence_directory=candidate-evidence/macos-runtime
if [[ ${2:-} == --ui-probe ]]; then
  swiftc scripts/macos-ui-probe.swift -o "$workspace/ui-probe"
  probe_arguments=(--ui-probe "$workspace/ui-probe")
  evidence_directory=candidate-evidence/macos-ui
fi
python3 scripts/desktop-runtime-smoke.py --executable "$workspace/candidate.app/Contents/MacOS/racoon-app" \
  --window-probe "$workspace/window-probe" --evidence "$evidence_directory" "${probe_arguments[@]}"
