# Technical Debt — Racoon Typper

**Last updated:** 2026-08-06
**Scope:** Current debt after the Phase 0–3B.3 baseline commit, Phase 6 CI hardening, the GLib/GTK3 dependency decision (ADR 0001), the SQLite backup/restore data layer (Task B), migration-matrix upgrade evidence (Task C), long-history reporting (Tasks D–E), committed Task F portable profile transfer, the Task G threat-model baseline, the Task H–I security hardening, the Task J hostile-input/error-redaction regression, and the Task K release-workflow split.

## Active debt

| ID | Debt | Owner phase | Priority |
|---|---|---|---|
| TD1 | The private session service is intentionally not a standalone application layer with typed ports/use cases. | Phase 3 | P1 |
| TD2 | IPC uses named endpoint responses, but request/config contracts remain hand-maintained; `mode_config` and scalar setting updates are not a versioned typed algebra. | Phase 3 | P1 |
| TD3 | `App.svelte` still owns broad interaction state; feature stores and explicit cache/session ownership are not established. | Phase 3 | P1 |
| TD4 | Durable crash recovery is implemented for the recovery/finalization ledger; backup/restore, restart-safe live completion rewiring, and deterministic injected runtime providers in production are not fully shipped. | Trusted Core | P0/P1 |
| TD6 | SQLite backup/restore data-layer API, rotating pre-migration backups (Task B), migration-matrix evidence (Task C), and the Task F portable JSON transfer API/IPC/runbook are implemented. Whole-file restore IPC/UI still needs lifecycle coordination that closes the live `Database`; no restore handoff or platform/manual recovery smoke is complete. Privacy/retention enforcement remains unfinished. Task I adds bounded opt-in redacted backup diagnostics, and Task J makes the `AppError` IPC, display, and debug boundaries use stable public codes/messages rather than dynamic payloads. The Task H application-command ACL limits the local main window to audited frontend commands, but typed/versioned IPC contracts remain Task R work. The implementation evidence and residual risks are inventoried in `docs/security/THREAT_MODEL.md`; that document does not close these items. | Phases 4–5 | P0/P1 |
| TD7 | Packaging signing, SBOM/reproducibility evidence, and cross-platform clean-install smoke tests remain release work. | Phase 6 | P0 |

## Dependency debt

| ID | Debt | Priority | When |
|---|---|---|---|
| DD1 | `rusqlite` 0.31 → newer releases require a compatibility review. | Low | Post-v1.0 / dependency decision |
| DD2 | `refinery` 0.8 → newer releases require a compatibility review. | Low | Post-v1.0 / dependency decision |
| DD3 | `toml` 0.8 → newer releases require a compatibility review. | Low | Post-v1.0 / dependency decision |
| DD4 | GTK3-related Tauri transitive dependencies have maintenance/advisory debt (RUSTSEC-2024-0429 `glib` unsound + gtk3-rs unmaintained cluster). **Decision recorded:** documented acceptance scoped to the Linux-only Tauri webview backend; the unsound `VariantStrIter` path is transitive and not reached by our code. Revisit on Tauri gtk4-rs/glib≥0.20 migration or any reachable soundness advisory. See `docs/adr/0001-glib-gtk3-advisory.md`. | High → monitored | Phase 5 G5 / upstream Tauri gtk4 migration |

## Resolved foundation debt

| ID | Debt | Resolved in |
|---|---|---|
| TD-OLD1 | `type_text()` unused test helper | Sprint 8.5 |
| TD-OLD2 | `make_text()` unused test helper | Sprint 8.5 |
| TD-OLD3 | Stringly command errors | Sprint 8 |
| TD-OLD4 | Hardcoded mode type | Sprint 6 |
| TD-OLD5 | Nested engine/database completion locking and repeated completion persistence | Phase 2 |
| TD-OLD6 | Stub lesson repository | Phase 2 |
| TD-OLD7 | Missing release profile stripping | Phase 0 |
| TD-OLD8 | Missing daily statistics/streak persistence | Phase 2 |
| TD-OLD9 | Missing consistency/graph output and persisted text length | Phase 2 |
| TD-OLD10 | Command-module raw SQL and monolithic `commands.rs` | Phase 2 |

## Resolved baseline work

| ID | Work | Resolved in |
|---|---|---|
| TD-NEW1 | Durable session identity (UUIDv7), session ledger, completion intents, finalization ledger, startup recovery coordinator, process-crash campaign | Phase 3B.3 |
| TD-NEW2 | GitHub Actions pinned to full commit SHAs; dependabot update group added | Phase 6 partial |
| TD-NEW3 | Toolchain pinning: `rust-toolchain.toml` (Rust 1.96.0), `.nvmrc` + `engines` (Node >=22) | Phase 6 partial |
| TD-NEW4 | CSP removed `unsafe-inline` from `script-src`; inline style exception retained and documented | Phase 5 partial |
| TD-NEW5 | Bug/feature issue templates, PR template, SECURITY.md | Phase 7 partial |
| TD-NEW6 | Data privacy/retention documentation in `docs/data/privacy.md` | Phase 4 partial |
| TD-NEW7 | Unused `@tauri-apps/plugin-shell` dependency removed | Phase 5 |
| TD-NEW8 | GLib/GTK3 advisory chain (DD4, R-009) decided: documented acceptance scoped to the Linux-only Tauri webview backend, with bounded revisit trigger. ADR `docs/adr/0001-glib-gtk3-advisory.md`. | Phase 5 G5 |
| TD-NEW9 | SQLite online backup/restore data-layer API (`crates/data/src/backup.rs`) using rusqlite's Online Backup API for transactionally consistent snapshots; rotating N=5 pre-migration backups with warn-and-continue; restore validates a sibling temporary replacement before atomically replacing the live main file and removing stale WAL/SHM companions. Tauri restore IPC remains unfinished. | Phase 4 G4 |
| TD-NEW10 | Migration matrix evidence (`crates/data/tests/migration_matrix.rs`, 16 tests): every historical schema V1..V7 upgrades cleanly to V8 through the production Refinery runner with era-appropriate seed data; asserts refinery history, full schema (10 tables + 15 indexes across V1..V8), `foreign_keys=ON`, `journal_mode=wal`, epoch data survival incl. the V005 `session_id` backfill, idempotent reopen, and a V1→V8 pre-migration backup round-trip via the Task B seam. FK coverage: `test_replays` `ON DELETE CASCADE`; `session_completion_intents` orphan-insert rejected and `session_ledger` delete blocked when an intent exists (V007 `ON DELETE RESTRICT`); `session_finalizations` orphan-session and mismatched-fingerprint rejected (V008 `ON DELETE RESTRICT` + composite fingerprint FK); `personal_bests.best_wpm_test_id` `NO ACTION` blocks deleting a referenced `tests` row. Observation recorded: `session_ledger` (V006) has no foreign keys by design; FK enforcement begins at V007 (child → ledger) and V008 (composite fingerprint → intents); `personal_bests` references use implicit NO ACTION. | Phase 4 G4 |
| TD-NEW11 | Long-history reporting semantics (Tasks D–E): V009 deterministic ordering indexes; explicit tie-break ordering for history and personal-best lists; 10k planner/timing and >100k complete-aggregate evidence; dashboard/achievement global best and streak reads use maintained projections; intentionally recent consistency window documented in ADR 0002. | Phase 4 G4 |
| TD-NEW12 | Task F portable profile transfer: strict versioned JSON (`racoon-typper-profile` schema v1), 64 MiB/100k collection bounds, complete validation before writes, no-write import preview, merge/replace policies, transactional portable-table import, Tauri IPC, and recovery/profile-transfer runbook (`docs/data/profile-transfer.md`). It deliberately excludes settings, replays, raw SQLite backups, and recovery/finalization ledgers; whole-file restore remains data-layer only. | Phase 4 G4 (partial) |
| TD-NEW13 | Task G threat model: evidence-based inventory of IPC, local files, custom content, diagnostics, packaging, and restore boundaries; records controls, tests, and open residual risks. It cross-references ADR 0001 and does not assert completion of Task H–P controls. See `docs/security/THREAT_MODEL.md`. | Phase 5 G5 (documentation baseline) |
| TD-NEW14 | Task H least-privilege Tauri capability: a build-time `AppManifest` generates per-command permissions for the exact 31 frontend IPC wrappers; the local `main` window receives only those permissions, with no Tauri core window/webview/event/app or remote-origin permission. `crates/app/tests/capability_audit.rs` fails on unexpected or missing handler/frontend/manifest/capability coverage. | Phase 5 G5 |
| TD-NEW15 | Task I redacted local diagnostics: `verbose_logging` is false by default and persists through `SettingsStore`; the backup-failure event uses an allowlisted JSONL schema, bounded rotating files, and no fallback diagnostic write on filesystem errors. Unit tests prove disabled mode, retention bounds, and absence of typed text/profile content/supplied paths. | Phase 5 G5 |
| TD-NEW16 | Task K release-workflow split: `.github/workflows/ci.yml` is PR-checks only (rust/frontend/license, `contents: read`); `release-candidate.yml` rebuilds Linux+Windows artifacts from an immutable tag, validates the tag against the project version, generates `SHA256SUMS`, and creates a draft prerelease; `promote-release.yml` is a `workflow_dispatch` gated by the protected `release-promotion` environment that flips the draft to a published release with generated notes (no rebuild, no secrets). Model and scope limits documented in `docs/release-workflow.md`. SBOM/provenance attach (Task L), signing (TD7), and clean-install smoke (Tasks M/Q) remain separate release tasks. | Phase 6 G6 |
