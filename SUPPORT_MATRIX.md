# Support Matrix

**Status:** Release Baseline (Phase 0)  
**Last reviewed:** 2026-07-12

This matrix distinguishes the environments used for development from platforms that are actually supported for public releases. A configured bundle target is not evidence of support.

| Platform / artifact | Development | CI/build target | Release support | Baseline evidence |
|---|---:|---:|---:|---|
| Linux x86_64 — Arch Linux | **Primary** | Not currently the hosted runner | **Primary development target** | Repository reviewed and local Rust/frontend checks run on Arch Linux x86_64 |
| Linux x86_64 — Ubuntu 24.04 | Supported build environment candidate | **Configured** | Pending Phase 6 smoke validation | Dependencies are documented; clean artifact launch is not yet evidenced |
| Linux x86_64 — AppImage | Build target configured | Configured but not release-verified | **Experimental / not supported yet** | Clean AppImage build and launch evidence is still required |
| Linux x86_64 — Debian package | Build target configured | Configured but not release-verified | **Experimental / not supported yet** | Package install/launch smoke test is still required |
| Linux x86_64 — RPM package | Build target configured | Configured but not release-verified | **Experimental / not supported yet** | RPM toolchain and install/launch test are still required |
| Arch package (`PKGBUILD`) | Packaging recipe present | Not currently validated in CI | **Community recipe / not supported yet** | Reproducible `makepkg` verification belongs to Phase 6 |
| Windows x86_64 — NSIS | Not locally verified | **Configured target** | **Not supported yet** | Workflow invocation was corrected in Phase 0; real Windows smoke testing belongs to Phase 6 |
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
