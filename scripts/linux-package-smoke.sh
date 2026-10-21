#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 Racoon Typper Contributors
# Ubuntu runner/container only: install, initialize, and verify a real data row
# survives restart. GUI rendering and typed input require a separate native smoke.
set -euo pipefail
[[ $# -eq 1 ]] || { echo "usage: $0 <racoon-typper.deb>" >&2; exit 64; }
package=$(realpath "$1")
[[ -f "$package" ]] || { echo "package not found: $package" >&2; exit 66; }
script_directory=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
sudo apt-get update
sudo apt-get install -y "$package" xvfb dbus-x11 python3
python3 "$script_directory/linux-runtime-smoke.py" -- dbus-run-session -- xvfb-run -a /usr/bin/racoon-app
