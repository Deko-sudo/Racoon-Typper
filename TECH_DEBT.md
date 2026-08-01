# Technical Debt — Racoon Typper

**Last updated:** 2026-08-01
**Scope:** Current debt after the Phase 0–3B.3 baseline commit, Phase 6 CI hardening, the GLib/GTK3 dependency-decision (ADR 0001), and the SQLite backup/restore data-layer (Task B).

## Active debt

| ID | Debt | Owner phase | Priority |
|---|---|---|---|
| TD1 | The private session service is intentionally not a standalone application layer with typed ports/use cases. | Phase 3 | P1 |
| TD2 | IPC uses named endpoint responses, but request/config contracts remain hand-maintained; `mode_config` and scalar setting updates are not a versioned typed algebra. | Phase 3 | P1 |
| TD3 | `App.svelte` still owns broad interaction state; feature stores and explicit cache/session ownership are not established. | Phase 3 | P1 |
| TD4 | Durable crash recovery is implemented for the recovery/finalization ledger; backup/restore, restart-safe live completion rewiring, and deterministic injected runtime providers in production are not fully shipped. | Trusted Core | P0/P1 |
| TD5 | History/analytics paths still use bounded record caps and need long-history query semantics, indexes, and measured regression thresholds. | Phase 4 | P1 |
| TD6 | Backup/restore data-layer API and rotating pre-migration backups are implemented (Task B); restore IPC wiring/runbook, migration preflight/recovery, privacy/retention enforcement, structured redacted logs, capability/CSP hardening, and raw-error redaction remain unfinished. | Phases 4–5 | P0/P1 |
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
| TD-NEW9 | SQLite online backup/restore data-layer API (`crates/data/src/backup.rs`) using rusqlite's Online Backup API for transactionally consistent snapshots; rotating N=5 pre-migration backups with warn-and-continue; restore removes destination + WAL/SHM before writing. Tauri restore IPC remains Task F. | Phase 4 G4 |
