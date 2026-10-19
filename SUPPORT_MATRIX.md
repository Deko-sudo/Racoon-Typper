# Support Matrix

**Last reviewed:** 2026-10-20. Scope: unreleased Linux stability candidate.
Version metadata `1.3.1` is not a claim that historical release assets contain
these changes. Every candidate must record its exact source commit, artifact
hash, build OS and runtime OS in its evidence manifest.

| Platform / artifact | Current candidate evidence | Support boundary |
|---|---|---|
| Arch Linux x86_64 / Omarchy Wayland | Local Rust/frontend gates; native RU/EN, repeat/abort, history, PNG/clipboard smoke | Primary development target; final artifact checks remain separate |
| Linux X11 / WebKitGTK | Native synthetic-profile typing, restore refusal/success, PNG/image-paste and restart checks | Software-rendered test environment; does not prove all GPUs/desktops |
| Ubuntu 24.04 x86_64 / `.deb` | Candidate source-build/install/runtime verification required; see candidate manifest | Linux candidate target; do not reuse an Arch-built binary as Ubuntu evidence |
| AppImage | Configured bundler and repaired restart-retention smoke | Experimental until this candidate's exact artifact passes the smoke |
| RPM | Configured bundler | Build configuration is not an RPM install test |
| Arch `PKGBUILD` | Recipe exists | Community recipe; source checksum/version must match a published immutable tag |
| Windows x86_64 / NSIS | Historical CI records exist; current candidate not manually checked | Deferred by owner; no new support claim |
| macOS | No current candidate validation | Deferred; unsupported by this Linux candidate |
| Linux ARM, Flatpak | No verified candidate install/runtime | Unsupported / pending separate work |

A container install validates its userspace and package path, not a full desktop
VM, hardware GPU, portal integration or all distributions. Screen-reader and
IME event-model tests are separate from real assistive technology/input methods.
Windows/macOS do not gate a Linux-only candidate and must not be advertised as
verified because Linux tests pass.

## Toolchains and evidence

Rust is pinned by `rust-toolchain.toml` (1.96.0). `frontend/.nvmrc` selects Node
26; the package permits Node 22.12+ on the 22.x line or Node 24+, and CI uses
22. Tauri, Svelte and Vite versions are determined by the checked-in lockfiles.
Record actual versions used for each build; this document does not infer them
from the host or an old report.

Before promoting an artifact: clean install, launch, short session, persistence
through restart/upgrade, applicable restore tests and an uninstall/rollback plan.
See [candidate procedure](docs/release/linux-candidate.md) and
[release checklist](RELEASE_CHECKLIST.md). Automatic CI currently targets only
`main` and `master` PRs/pushes; branch existence is not proof of CI success.
