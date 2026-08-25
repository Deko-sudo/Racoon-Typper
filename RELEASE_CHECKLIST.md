# Release Checklist — Racoon Typper

This checklist describes the current baseline and the gates required for a production release. Phase 0 does not authorize a public release; licensing, foundation, security, and release-engineering gates remain mandatory.
## v1.3.0 release (2026-08-25)

- [x] Canonical version is consistently `1.3.0` (Cargo workspace, tauri.conf, frontend package/lock, PKGBUILD).
- [x] Tag `v1.3.0` recreated from the release commit `d76b1f0` after smoke fixes; PKGBUILD source checksum repinned to the final tag tarball.
- [x] Release-candidate workflow passed end-to-end: validate-tag, flatpak policy, Linux/Windows builds, AppImage launch smoke, Linux deb smoke, Windows install/first-screen/session-start/restart smoke.
- [x] `SHA256SUMS` regenerated to cover exactly the attached asset set (including updater signatures and `latest.json`) with flat basenames; all 15 entries verified against a plain release download.
- [x] Promotion published the draft with 15 assets (AppImage/NSIS + `.sig`, deb, rpm, tarball + torrent, SHA256SUMS, SBOM, provenance, release-manifest, latest.json); updater `latest.json` references signed v1.3.0 artifacts.
- [x] `release-promotion` environment now has a required-reviewer protection rule; promotion was approved through it.
- [x] **Promotion incident (found and fixed same day):** the first real run of `promote-release.yml` created a second, asset-less published release for the tag because GitHub's release-by-tag lookup excludes drafts, so the action never saw the RC draft. Recovery moved the 15 verified artifacts onto the published object byte-for-byte (checksums re-verified against public downloads; updater URLs returned 200) and removed the orphaned draft. Root cause fixed in `promote-release.yml`: promotion now PATCHes the existing draft in place and hard-fails unless the published release carries at least `min_assets` artifacts.
- [x] Known limitation recorded: typed-input persistence is not asserted on CI runners (TD-NEW23, future tauri-driver harness).

## v1.2.0 release (2026-08-21) — published

- [x] Canonical version is consistently `1.2.0` (`npm run check:version`).
- [x] Task F guarded portable profile restore UI is implemented and reviewed.
- [x] `cargo test --workspace` passes (22 suites).
- [x] Rust formatting and clippy `-D warnings` gates pass.
- [x] Frontend unit, Svelte, IPC-contract, version, license, and production-build gates pass.
- [x] Local `tauri:build:binary` produces `target/release/racoon-app`.
- [x] Changelog, release notes, transfer runbook, threat model, and release evidence boundaries are synchronized.
- [x] The stale `v1.2.0` tag (25-theme build) was recreated from the final reviewed release commit `34e2251` and force-pushed.
- [x] Release-candidate workflow passed: validate-tag, flatpak policy, Linux/Windows builds, Linux and Windows clean-install smoke.
- [x] Promotion published the draft with 15 assets (AppImage/NSIS + `.sig`, deb, rpm, tarball + torrent, SHA256SUMS, SBOM, provenance, release-manifest, latest.json).
- [x] Updater `latest.json` references the signed v1.2.0 artifacts.
- [x] Release notes match the shipped feature set (50 themes, share card, theme editor, smooth caret, pomodoro, markdown export, cheatsheet, error tails).

## v1.1.0 candidate preparation (2026-08-07) — superseded by v1.2.0

- [x] Canonical version is consistently `1.1.0`.
- [x] Task F guarded portable profile restore UI is implemented and reviewed.
- [x] `cargo test --workspace --all-targets` passes (638 tests).
- [x] Rust formatting and clippy `-D warnings` gates pass.
- [x] Frontend unit, Svelte, IPC-contract, version, license, and production-build gates pass.
- [x] Local `tauri:build:binary` produces `target/release/racoon-app`.
- [x] Changelog, release notes, transfer runbook, threat model, and release evidence boundaries are synchronized.
- [~] Recreate the stale local `v1.1.0` tag from the final reviewed release commit. The current local tag points to `89b0ed38`; no remote tag exists. — **Superseded:** v1.2.0 was released instead; the v1.1.0 tag was left as-is.
- [~] Replace the `PKGBUILD` checksum placeholder from the immutable canonical source archive. — **Superseded:** checksum is pinned for the v1.2.0 archive instead.
- [x] Push the final reviewed commit and immutable tag, then run the release-candidate workflow.
- [x] Verify the exact tagged workflow, candidate assets, checksums, evidence, and Linux/Windows clean-install smoke.
- [x] Promote only after maintainer review; this preparation does not publish a release.

## Phase 0 baseline checks

### Version and worktree

- [x] Worktree contains only reviewed changes (clean since 2026-08-01).
- [x] `Cargo.toml` `[workspace.package]` contains the intended release version.
- [x] `npm run check:version --prefix frontend` passes.
- [x] Tag is exactly `v<canonical-version>` (`v1.2.0` → `34e2251`).
- [x] [BASELINE.md](BASELINE.md) separates reviewed baseline files from work in progress.

### Code and frontend validation

- [x] `cargo fmt --all --check` passes.
- [x] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [x] `cargo test --workspace` passes.
- [x] `npm run check --prefix frontend` passes.
- [x] `npm run build --prefix frontend` passes.
- [x] `git diff --check` passes.

### Tauri topology

- [x] `npm ci --prefix frontend` succeeds from a clean checkout.
- [x] `npm run tauri:dev --prefix frontend` starts the frontend and desktop process.
- [x] `npm run tauri:build:binary --prefix frontend` locates `frontend/dist` and produces the release binary.
- [x] `npm run tauri:build --prefix frontend` completes for each claimed platform/bundle target (deb, rpm, AppImage, NSIS — built by the release-candidate workflow on Linux and Windows runners).

## Required before production release

### Licensing and provenance — Phase 1

- [x] Project-owned code/assets/content have Apache-2.0 provenance.
- [x] GPL/LGPL/AGPL and unknown project content is removed or separately cleared.
- [x] `THIRD_PARTY_NOTICES.md` and machine-readable inventory are generated.
- [x] CI license policy passes.
- [x] The release candidate attaches the checked CycloneDX SBOM, content provenance record, source-commit manifest, and checksums (`docs/release-evidence.md`; Task L).

### Foundation — Phase 2

- [x] In-process completion/retry and persistence tests pass; do not describe this as durable crash recovery.
- [x] Related persistence writes use an atomic transaction.
- [x] Monotonic timing, replay, validation, and typed endpoint responses are verified.
- [x] Startup, shutdown, error, and retry paths are tested.

### Trusted Core, architecture, and data — Phases 3–4

- [x] Stable durable session IDs, crash recovery, deterministic injected seams, and restart-safe idempotency are separately designed and verified (Phases 3A–3B.3).
- [x] Application ports/contracts and frontend state ownership have passed Phase 3 review.
- [x] Database foreign keys, migration fixtures, backup/restore, and long-history behavior are verified.
  - [x] Foreign keys enforced on every connection; cascade (replays) and RESTRICT (session ledger chain incl. V008 composite fingerprint FK) verified (`crates/data/tests/migration_matrix.rs`).
  - [x] Migration fixtures: every historical schema V1..V7 upgrades cleanly to V9 through the production Refinery runner with era-appropriate data (`crates/data/tests/migration_matrix.rs`).
  - [x] Backup/restore data-layer and rotating pre-migration backups verified (`crates/data/tests/backup_restore.rs`, Task B).
  - [x] History surfaces paginate with deterministic ordering and 10 000+ query-plan/timing evidence; dashboard and achievement summaries use complete maintained projections with >100k-history coverage. Insights/consistency intentionally summarize the documented recent window (Tasks D–E, ADR 0002).
  - [x] Portable profile transfer has strict versioned JSON validation, pre-read UI bounds, no-write preview, merge/replace policies, explicit destructive confirmation, atomic portable-table import, and backend/frontend coverage (`crates/data/tests/profile_transfer.rs`, `crates/app/src/commands/profile_transfer.rs`, `scripts/profile-transfer-ui.test.mjs`; Task F release scope).
  - [x] Whole-file restore is not exposed in v1.1.0. The data-layer API requires the live `Database` to be closed first, so lifecycle coordination and platform/manual recovery evidence remain explicitly deferred rather than being bypassed by an unsafe UI. See `docs/data/profile-transfer.md` and `TECH_DEBT.md` TD6.

### Security surface — Phase 5

- [x] Threat-model baseline inventories current assets, trust boundaries, abuse cases, implementation evidence, tests, and residual risk for IPC, local files, custom content, diagnostics, packaging, and restore (`docs/security/THREAT_MODEL.md`, Task G).
- [x] Tauri capabilities are generated from the 31 registered frontend commands and reduced to least privilege: the local `main` window receives only the corresponding application-command permissions, no `core:*` permission, and no remote-origin association. `crates/app/tests/capability_audit.rs` audits handler/frontend/manifest/capability equality (Task H).
- [x] Local backup-failure diagnostics are opt-in, bounded JSONL, and demonstrably redact typed content, supplied paths, and raw error payloads. The startup stderr fallback is a fixed generic message only (`crates/app/src/logging.rs`, `crates/app/src/main.rs`; Task I). Task J completes the separate comprehensive IPC error-redaction coverage.
- [x] Hostile-input and error-disclosure regression coverage proves malformed/oversized IPC, imported content, traversal-like values, and repeated rejected requests do not bypass validation or leak sensitive data (Task J).
- [x] GLib/GTK3 advisory-chain acceptance is documented with reachability analysis and revisit triggers; it remains monitored dependency debt, not a resolved upstream vulnerability (`docs/adr/0001-glib-gtk3-advisory.md`).

### Packaging and release engineering — Phase 6

- [x] Linux artifacts install/launch on clean supported environments (`scripts/linux-package-smoke.sh` in release-candidate).
- [x] AppImage artifact launches and persists state before a draft is created (`scripts/appimage-smoke.sh` in release-candidate; FUSE with `--appimage-extract-and-run` fallback — Task O).
- [x] Windows NSIS artifact installs/launches on clean Windows (`scripts/windows-nsis-smoke.ps1` in release-candidate).
- [x] Windows clean-smoke asserts install, first-screen rendering, live session start, and restart retention (`scripts/windows-nsis-smoke.ps1`; Task Q). Typed-input persistence is not asserted on CI runners — input delivery limitation tracked as future UI-automation work.
- [x] Checksums, source revision, version, SBOM, and provenance are attached to each release candidate (`docs/release-evidence.md`; Task L).
- [x] Signing or attestation is verified where supported (AppImage and NSIS `.sig` via Tauri signer; OIDC/SLSA remains TD7).
- [x] Release actions use least-privilege permissions and reviewed action versions (SHA-pinned).
- [x] Release-candidate creation and promotion are separate manual workflows; promotion is guarded by the repository-managed `release-promotion` environment (`docs/release-workflow.md`; Task K).
- [x] Smoke journey completes a short test, persists it, restarts, exports data, and exits cleanly (Linux smoke; Windows smoke covers install/launch/persist/restart).

### Public repository — Phase 7

- [x] README, install guide, architecture, support matrix, and roadmap are accurate (50-theme catalog, v1.2.0 status; screenshots remain pending).
- [ ] Screenshots come from a cleared release candidate (deferred by owner decision).
- [x] Security reporting and contribution workflows are published (`SECURITY.md`, issue/PR templates).
- [x] Release notes match the actual artifacts (v1.2.0 release body mirrors the changelog).

## Database and migration safety

The current schema contains migrations V001 through V009. Every future schema change must:

- include a fixture from each supported prior version;
- create and verify a backup before migration;
- validate foreign keys and indexes;
- provide a forward-fix or restore procedure;
- never rely on checking out an older binary to undo a user database migration.

## Release process

1. Complete the applicable roadmap gate and obtain maintainer approval.
2. Review the clean worktree and run the complete validation matrix.
3. Set the canonical version in the root workspace manifest and synchronize/check mirrors.
4. Update changelog and release notes with verified changes only.
5. Create an annotated tag `vX.Y.Z` from the reviewed commit.
6. Let CI build candidate artifacts and attach evidence.
7. Run clean-install and smoke checks.
8. Publish only after artifact, licensing, security, and documentation review.
9. Keep the prior known-good release available and record post-release issues.

## Rollback

- Withdraw or mark a defective release artifact rather than silently replacing it.
- Publish a forward-fixed version after the cause is understood.
- Restore user data from the pre-migration backup or follow the documented forward-fix procedure.
- Do not use `git checkout` or deleting a tag as a database rollback mechanism.
