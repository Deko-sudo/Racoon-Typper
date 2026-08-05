# Support Matrix

**Status:** Foundation (Phases 0–3B) + Phase 6 partial  
**Last reviewed:** 2026-08-01

This matrix distinguishes the environments used for development from platforms that are actually supported for public releases. A configured bundle target is not evidence of support.

| Platform / artifact | Development | CI/build target | Release support | Baseline evidence |
|---|---:|---:|---:|---|
| Linux x86_64 — Arch Linux | **Primary** | Not the hosted CI runner | **Primary development target** | Repository reviewed and local Rust/frontend checks run on Arch Linux x86_64 |
| Linux x86_64 — Ubuntu 24.04 | Supported build environment candidate | **Verified — every CI run** | Pending Phase 6 smoke validation | CI runner is `ubuntu-latest` (24.04); full workspace checks and Linux bundles build and upload green on every push |
| Linux x86_64 — AppImage | Build target configured | **Verified build on every CI run** | **Experimental / not supported yet** | `tauri:build:ci` produces the AppImage as a CI artifact; clean-install launch evidence still required (Phase 6 smoke) |
| Linux x86_64 — Debian package | Build target configured | **Verified build on every CI run** | **Experimental / not supported yet** | `tauri:build:ci` produces the `.deb` as a CI artifact; package install/launch smoke test still required |
| Linux x86_64 — RPM package | Build target configured | **Verified build on every CI run** | **Experimental / not supported yet** | `tauri:build:ci` produces the `.rpm` as a CI artifact; RPM toolchain and install/launch test still required |
| Arch package (`PKGBUILD`) | Packaging recipe present | Not currently validated in CI | **Community recipe / not supported yet** | Reproducible `makepkg` verification belongs to Phase 6 |
| Windows x86_64 — NSIS | Not locally verified | **Verified build on every CI run** | **Not supported yet** | NSIS installer is produced and uploaded by `build-windows`; local launch and clean-Windows smoke testing belong to Phase 6 (Task Q) |
| macOS x86_64/Apple Silicon | Not configured | Not configured | **Unsupported** | No build workflow or artifact is claimed |
| Linux ARM/aarch64 | Not configured | Not configured | **Unsupported** | No target, performance baseline, or package is claimed |
| Flatpak | Manifest present | Not a verified source build | **Unsupported / pending redesign** | Current manifest installs a prebuilt binary and needs Phase 6 work |

## Support policy

- “Primary” means maintainers actively develop and validate the platform.
- “Configured” means files exist, not that the artifact is safe to publish.
- “Experimental” means a maintainer may test it, but users should not rely on release support.
- “Unsupported” means no compatibility promise is made.
- A platform moves to release support only after clean-install, launch, short typing session, persistence, upgrade, and uninstall smoke evidence is attached to a release.

## Current baseline environment

- OS: Arch Linux, x86_64
- Kernel: Linux 7.0.14-zen1-1-zen
- Rust: cargo 1.96.0 (2026-05-25)
- Node.js: v26.4.0
- npm: 11.16.0
- Tauri CLI package: 2.11.3 in the checked-out frontend installation

These versions describe the review environment, not a long-term support promise. Toolchain pinning and a release support policy belong to Phase 6.
