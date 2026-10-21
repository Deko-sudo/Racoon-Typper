# Linux stability candidate verification

This is an unreleased candidate procedure. Identify artifacts by source commit
and SHA-256, even while metadata is still 1.3.1. Do not replace an existing tag
or label a new binary as the previously published v1.3.1 artifact.

## Build and isolate

Run CONTRIBUTING checks plus the opt-in crash target, optimized GLib regression,
macro/provenance/reference gates and `python3 -m unittest discover -s scripts -p
linux_smoke_test.py -v`. Build the production binary/package in the oldest stated
supported userspace (currently Ubuntu 24.04 for the Debian candidate). Host Arch
build success does not prove Ubuntu ABI compatibility.

Create a manifest containing commit/tree, toolchains, build/runtime OS, package
hashes, commands/results and limitations. Verify the package hash before testing.
Use a new container/VM and a disposable profile; do not install an audit package
or point test scripts at the user's real profile. A container is not a clean
hardware desktop or an all-GPU compatibility test.

## Runtime and upgrade

1. Install the exact `.deb` into the isolated Ubuntu runtime. Launch as a normal
   user with private XDG data/config/cache directories and no external network.
2. Complete onboarding and short RU/EN sessions; inspect results/history, repeat,
   abort, PNG save and image paste. Restart and verify retained records.
3. Keep a copy of a synthetic older-version profile. Open its copy with the
   candidate and compare records/settings, integrity and foreign keys. Never
   downgrade a migrated live profile with the old binary.
4. Test valid restore and foreign/changed/corrupt/empty refusal against a
   synthetic nonempty profile. Check snapshots and retained records after errors.
5. Record actual IME preedit/commit/cancel and assistive-technology checks where
   available. Event-model/DOM tests do not substitute for native IME or speech.

The shell smokes are Ubuntu CI/container scripts and call apt; do not run them
on a personal Arch installation. `linux-runtime-smoke.py` controls only its own
new process session and verifies a persisted custom-text row across two launches.
It does not assert typed-input completion or visual rendering. AppImage fallback
starts a separate disposable profile after a failed FUSE attempt, while the two
launches within a successful smoke must reuse one profile.

## Retention, signatures and publication

Control exported PNGs and snapshots separately from statistics deletion. Review
profile-transfer/privacy runbooks before deleting a current or legacy profile.
SHA-256 binds bytes but does not authenticate a publisher: installer signatures
require an independently trusted maintainer public key. Never label an unsigned
local candidate as signed. Existing signing/promotion credentials are not read
or replaced by this procedure.

Keep the previous known-good package and a pre-upgrade profile copy. On failure,
stop all instances and restore the copy according to the runbook; uninstalling a
package alone does not reverse migrations or delete user data.

Public promotion is separate from candidate validation. Windows/macOS are
explicitly deferred for this candidate; physical power-loss and desktop/GPU
coverage must remain accurately bounded in release notes.
