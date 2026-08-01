# Technical Debt — Racoon Typper

**Last updated:** 2026-07-13
**Scope:** Current debt after the Phase 2 Foundation completion. This is not a release-readiness claim.

## Active debt

| ID | Debt | Owner phase | Priority |
|---|---|---|---|
| TD1 | The private session service is intentionally not a standalone application layer with typed ports/use cases. | Phase 3 | P1 |
| TD2 | IPC uses named endpoint responses, but request/config contracts remain hand-maintained; `mode_config` and scalar setting updates are not a versioned typed algebra. | Phase 3 | P1 |
| TD3 | `App.svelte` still owns broad interaction state; feature stores and explicit cache/session ownership are not established. | Phase 3 | P1 |
| TD4 | Session handles are process-local timestamp values; durable identifiers, crash recovery, deterministic injected clocks/IDs/randomness, and restart-safe idempotency are not implemented. | Trusted Core | P0/P1 |
| TD5 | History/analytics paths still use bounded record caps and need long-history query semantics, indexes, and measured regression thresholds. | Phase 4 | P1 |
| TD6 | Backup/restore, migration preflight/recovery, privacy/retention, structured redacted logs, capability/CSP hardening, and raw-error redaction remain unfinished. | Phases 4–5 | P0/P1 |
| TD7 | Packaging signing, SBOM/reproducibility evidence, and cross-platform clean-install smoke tests remain release work. | Phase 6 | P0 |

## Dependency debt

| ID | Debt | Priority | When |
|---|---|---|---|
| DD1 | `rusqlite` 0.31 → newer releases require a compatibility review. | Low | Post-v1.0 / dependency decision |
| DD2 | `refinery` 0.8 → newer releases require a compatibility review. | Low | Post-v1.0 / dependency decision |
| DD3 | `toml` 0.8 → newer releases require a compatibility review. | Low | Post-v1.0 / dependency decision |
| DD4 | GTK3-related Tauri transitive dependencies have maintenance/advisory debt. | High | Security/dependency review |

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
