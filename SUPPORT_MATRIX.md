# Support Matrix

**Last reviewed:** 2026-10-21. Scope: unreleased Linux stability candidate.
Version metadata `1.3.1` is not a claim that historical release assets contain
these changes. Every candidate must record its exact source commit, artifact
hash, build OS and runtime OS in its evidence manifest.

| Platform / artifact | Current candidate evidence | Support boundary |
|---|---|---|
| Arch Linux x86_64 / Omarchy Wayland | Local Rust/frontend gates; native RU/EN, repeat/abort, history, PNG/clipboard smoke | Primary development target; final artifact checks remain separate |
| Linux X11 / WebKitGTK | Native synthetic-profile typing, restore refusal/success, PNG/image-paste and restart checks | Software-rendered test environment; does not prove all GPUs/desktops |
| Ubuntu 24.04 x86_64 / `.deb` | Production build, fresh-container apt install, two launches retaining data, synthetic v1.3.1-profile upgrade and native CJK/PNG smoke | Verified Ubuntu userspace/container path; not a full VM or hardware-GPU claim |
| AppImage | Configured bundler and repaired restart-retention smoke | Experimental until this candidate's exact artifact passes the smoke |
| RPM | Configured bundler | Build configuration is not an RPM install test |
| Arch `PKGBUILD` | Recipe exists | Community recipe; source checksum/version must match a published immutable tag |
| Windows x86_64 / NSIS | Current-source candidate workflow and offline WebView2 installer configured; results must be recorded per commit | Experimental until native CI and manual DPI/input/PNG/clipboard gates pass |
| macOS Intel / Apple Silicon, `.app` / DMG | Platform overrides and native candidate workflow configured; minimum bundle OS 13.3 | Experimental; no hardware, Developer ID or notarization claim |
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
[Linux stability evidence](docs/release/linux-stability-evidence.md).

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
