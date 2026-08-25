# Release workflow

Releases are deliberately split into verification, a draft release candidate, and an approved promotion. A pushed tag does not publish an artifact.

## 1. Verify the candidate

The normal `CI` workflow runs on pull requests, protected branches, and tags with read-only repository permissions. It verifies Rust, frontend, licensing, and build artifacts but cannot create a release.

## 2. Build a draft release candidate

A maintainer runs **Release candidate** manually and supplies an existing immutable tag such as `v1.2.3`.

The workflow checks that the tag exists and matches the project version, rebuilds Linux and Windows artifacts from that tag, generates `SHA256SUMS`, and creates a draft prerelease. Verify the checksums before installation. A draft release is not public and is not a promotion decision.

The Linux job builds all Linux bundle targets through the Tauri bundler (`deb`, `rpm`, `appimage`). Draft creation is gated on three isolated runtime smokes: the Debian package, the AppImage (`scripts/appimage-smoke.sh` — launch, persisted SQLite state across a restart, clean termination; falls back to `--appimage-extract-and-run` when FUSE is unavailable), and the Windows NSIS installer.

## 3. Promote after approval

A maintainer runs **Promote release** with the same tag only after release evidence is reviewed. Its `release-promotion` environment must be protected in GitHub repository settings with the required maintainer reviewers. This is the approval boundary: the workflow makes the existing draft non-draft and non-prerelease.

## Security properties

- Workflow defaults are `contents: read`; only the draft publisher and protected promotion job receive `contents: write`.
- Workflows do not print tokens or repository secrets.
- Candidate and promotion both validate the exact tag name before mutation.
- GitHub environment protection is repository configuration, not a source-controlled guarantee; configure `release-promotion` before using promotion.

## Scope limits

This workflow produces release artifacts and checksums, attaches the checked
CycloneDX SBOM and repository provenance record, and blocks draft creation on the
isolated Linux, AppImage, and Windows package smokes. These records are source-to-artifact
evidence, not cryptographic signing or a byte-for-byte reproducibility claim. The
runtime smoke intentionally does not claim browser-download export coverage;
profile transfer remains covered by backend integration tests and the guarded UI
flow until a stable desktop automation contract exists.

### AppImage supply-chain boundary

The `.AppImage` is produced by the Tauri bundler from the tag checkout. The
bundler downloads its own `linuxdeploy` tool image at build time; `tauri-cli`
currently exposes no configuration to pin that download by checksum. This is a
recorded acceptance (same class as the dependency-debt entries in
`TECH_DEBT.md`): the artifact is built inside a pinned-actions, tag-rebuilt
workflow, launch-tested before draft creation, and covered by `SHA256SUMS`.
Revisit if upstream adds a pinning knob or when signing moves to OIDC/SLSA
attestation (TD7).
