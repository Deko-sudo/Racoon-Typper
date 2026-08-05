# Release Checklist — Racoon Typper

This checklist describes the current baseline and the gates required for a production release. Phase 0 does not authorize a public release; licensing, foundation, security, and release-engineering gates remain mandatory.

## Phase 0 baseline checks

### Version and worktree

- [x] Worktree contains only reviewed changes (clean since 2026-08-01).
- [x] `Cargo.toml` `[workspace.package]` contains the intended release version.
- [x] `npm run check:version --prefix frontend` passes.
- [ ] Tag is exactly `v<canonical-version>`.
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
- [ ] `npm run tauri:build --prefix frontend` completes for each claimed platform/bundle target.

## Required before production release

### Licensing and provenance — Phase 1

- [x] Project-owned code/assets/content have Apache-2.0 provenance.
- [x] GPL/LGPL/AGPL and unknown project content is removed or separately cleared.
- [x] `THIRD_PARTY_NOTICES.md` and machine-readable inventory are generated.
- [x] CI license policy passes.
- [ ] SBOM is generated for the release.

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
  - [x] Migration fixtures: every historical schema V1..V7 upgrades cleanly to V8 through the production Refinery runner with era-appropriate data (`crates/data/tests/migration_matrix.rs`).
  - [x] Backup/restore data-layer and rotating pre-migration backups verified (`crates/data/tests/backup_restore.rs`, Task B).
  - [x] History surfaces paginate with deterministic ordering and 10 000+ query-plan/timing evidence; dashboard and achievement summaries use complete maintained projections with >100k-history coverage. Insights/consistency intentionally summarize the documented recent window (Tasks D–E, ADR 0002).

### Packaging and release engineering — Phase 6

- [ ] Linux artifacts install/launch on clean supported environments.
- [ ] Windows NSIS artifact installs/launches on clean Windows.
- [ ] Checksums, source revision, version, SBOM, and provenance are attached.
- [ ] Signing or attestation is verified where supported.
- [x] Release actions use least-privilege permissions and reviewed action versions (SHA-pinned).
- [ ] Smoke journey completes a short test, persists it, restarts, exports data, and exits cleanly.

### Public repository — Phase 7

- [ ] README, install guide, architecture, support matrix, and roadmap are accurate.
- [ ] Screenshots come from a cleared release candidate.
- [x] Security reporting and contribution workflows are published (`SECURITY.md`, issue/PR templates).
- [ ] Release notes match the actual artifacts.

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
