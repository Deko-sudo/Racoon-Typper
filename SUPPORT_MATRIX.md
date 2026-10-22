# Support Matrix

**Last reviewed:** 2026-10-22. Scope: unreleased desktop stability candidate.
Version metadata `1.3.1` is not a claim that historical release assets contain
these changes. Every candidate must record its exact source commit, artifact
hash, build OS and runtime OS in its evidence manifest.

| Platform / artifact | Current candidate evidence | Support boundary |
|---|---|---|
| Arch Linux x86_64 / Omarchy Wayland | Local Rust/frontend gates; native RU/EN, repeat/abort, history, PNG/clipboard smoke | Primary development target; final artifact checks remain separate |
| Linux X11 / WebKitGTK | Native synthetic-profile typing, restore refusal/success, PNG/image-paste and restart checks | Software-rendered test environment; does not prove all GPUs/desktops |
| Ubuntu 24.04 x86_64 / `.deb` | Production build, fresh-container apt install, two launches retaining data, synthetic v1.3.1-profile upgrade and native CJK/PNG smoke | Verified Ubuntu userspace/container path; not a full VM or hardware-GPU claim |
| AppImage x86_64 | Ubuntu 24.04 build and two extract-and-run launches retaining data | Container extraction verified; kernel FUSE, hardware GPU and desktop integration remain separate |
| RPM x86_64 | Ubuntu-built RPM installed in a fresh Fedora 43 container; two launches, retained marker, integrity/FK | Verified Fedora userspace path; not every RPM distribution or hardware desktop |
| Arch `PKGBUILD` | Recipe exists | Community recipe; source checksum/version must match a published immutable tag |
| Windows x86_64 / NSIS | Native Windows Server 2022 CI: strict Clippy, workspace/frontend tests, install, visible native window and SQLite restart retention | Experimental; physical Windows 10/11, DPI, RU/EN and clipboard acceptance remain manual |
| macOS Intel / Apple Silicon, `.app` / DMG | Native macOS 15.7.9 CI on both architectures: tests, DMG/app and window/SQLite restart; configured minimum OS 13.3 | Experimental; no macOS 13.3, hardware, Developer ID or notarization claim |
| Linux ARM, Flatpak | No verified candidate install/runtime | Unsupported / pending separate work |

A container install validates its userspace and package path, not a full desktop
VM, hardware GPU, portal integration or all distributions. Screen-reader and
IME event-model tests are separate from real assistive technology/input methods.
One real Fcitx Pinyin/WebKitGTK scenario is recorded separately from the
15-language synthetic-input matrix. Orca speech-generation evidence is a
bounded accessibility smoke, not a complete screen-reader usability review.
Windows/macOS do not gate a Linux-only candidate and must not be advertised as
verified because Linux tests pass.

The separate `Windows and macOS candidates` workflow builds the current PR head,
not an older release tag. It uploads unsigned/unpromoted NSIS/DMG candidates,
source hashes and startup/restart evidence. Native window existence is not a
rendering, typing, screenshot or clipboard test. Follow
[desktop candidate parity](docs/adr/0009-desktop-candidate-parity.md) for limits.

Current source/CI, artifact hashes and exact test boundaries are recorded in
[desktop stability evidence](docs/release/desktop-stability-evidence.md).

## Toolchains and evidence

Rust is pinned by `rust-toolchain.toml` (1.96.0). `frontend/.nvmrc` selects Node
26; the package permits Node 22.12+ on the 22.x line or Node 24+, and CI uses
22. Tauri, Svelte and Vite versions are determined by the checked-in lockfiles.
Record actual versions used for each build; this document does not infer them
from the host or an old report.

Before promoting an artifact: clean install, launch, short session, persistence
through restart/upgrade, applicable restore tests and an uninstall/rollback plan.
See [candidate procedure](docs/release/linux-candidate.md) and
[release checklist](RELEASE_CHECKLIST.md). Canonical CI targets `main`/`master`
PRs/pushes; the separate desktop candidate workflow accepts PRs to any base.
Branch existence is not proof of CI success. The optional native Windows/macOS UI
experiment records its own outcome and cannot replace the manual acceptance gate.
