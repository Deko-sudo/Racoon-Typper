#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 Racoon Typper Contributors
# All package installation happens in a new disposable Fedora container.
set -euo pipefail
[[ $# -eq 1 ]] || { echo "usage: $0 <candidate.rpm>" >&2; exit 64; }
package=$(realpath "$1")
[[ -f "$package" ]] || { echo 'RPM package not found' >&2; exit 66; }
script_directory=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
evidence=$(realpath -m candidate-evidence/rpm-runtime)
mkdir -p "$evidence"
docker run --rm -i \
  --mount "type=bind,source=$package,target=/input/candidate.rpm,readonly" \
  --mount "type=bind,source=$script_directory,target=/scripts,readonly" \
  --mount "type=bind,source=$evidence,target=/evidence" \
  --env "SMOKE_UID=$(id -u)" --env "SMOKE_GID=$(id -g)" \
  fedora:43@sha256:a651ddf48ea28a06ed4e1e6519f51c9f47e7a5a138722ade87369b8fbb7e5b42 bash -se -o pipefail <<'CONTAINER'
dnf install -y --setopt=install_weak_deps=False /input/candidate.rpm \
  xorg-x11-server-Xvfb xorg-x11-xauth dbus-x11 python3 mesa-libEGL mesa-libGL
# Avoid a privileged WebView; the host UID only owns this evidence directory.
uid=$SMOKE_UID
[[ $uid -ne 0 ]] || uid=1000
useradd -m -u "$uid" audit
chown "$uid" /evidence
cat /etc/os-release > /evidence/os-release.txt
rpm -q racoon-typper webkit2gtk4.1 > /evidence/packages.txt
runuser -u audit -- python3 /scripts/linux-runtime-smoke.py --workspace /evidence/profile \
  -- dbus-run-session -- xvfb-run -a /usr/bin/racoon-app | tee /evidence/runtime-smoke.json
CONTAINER
