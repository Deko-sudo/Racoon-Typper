# Racoon Typper — Agent Roadmap (GPT-5.6 Terra / Hemes Agent)

Self-contained handoff brief. This document fully replaces any earlier
conversation for the agent that receives it. Read it top to bottom and treat
the repository as the source of truth for anything not covered here.

**Repo:** https://github.com/Deko-sudo/Racoon-Typper — branch `master`
**Working copy:** `/home/yago/Desktop/racoon-typper` (mixed working tree shared with other agents)
**Product version:** 1.1.0 (source: `[workspace.package]` in root `Cargo.toml`)
**Commit author identity:** `Racoon Typper <racoon@typper.dev>`
**Updated:** 2026-08-05 — local `master` HEAD = `2007bc7`; verify with `git log --oneline -1` before starting.

---

## 1. Project state (what is already done)

Rust/Tauri/Svelte typing trainer. Backend boundary model: `racoon-domain` → `racoon-core` → `racoon-application` (infra-free) → `racoon-data` (SQLite via rusqlite bundled) → `crates/app` (Tauri wrapper) + `frontend/` (Svelte/Vite/TS).

Implemented and verified (Phase 0–3B.3, Phases 5/6/7 partial):
- Apache-2.0 licensing/provenance (Phase 1/Part1 complete): `licenses/*`, `THIRD_PARTY_NOTICES.md`, cargo-deny + npm policy in CI.
- Deterministic build topology: `scripts/tauri.mjs` wrapper, version gate `npm run check:version`.
- In-process exactly-once completion + durable crash recovery (Phases 2, 3A–3B.3): session ledger, completion intents, finalization ledger, migrations V001–V008.
- Task A: GLib/GTK3 advisory decision (ADR `docs/adr/0001-glib-gtk3-advisory.md`).
- Task B: SQLite online backup/restore data-layer `crates/data/src/backup.rs` (rusqlite Online Backup API, atomic snapshots, N=5 pre-migration rotation).
- Task C: migration matrix, `crates/data/tests/migration_matrix.rs` (16 tests): every historical schema V1..V7 → V8 through the production Refinery runner; FK/cascade/RESTRICT/NO-ACTION; backup round-trip; idempotent reopen.
- CI: green across all jobs including Windows (NSIS). Artifacts (deb, rpm, AppImage, NSIS, binary tarball) uploaded on every push.
- CSP: no `unsafe-inline` for `script-src`; data/privacy docs (`docs/data/privacy.md`), `SECURITY.md`, issue/PR templates.

**Landing commits on `master` (latest at bottom):**
```
28690e4  feat(data): SQLite online backup/restore + rotating pre-migration backups (Task B, G4)
38bbd47  docs(support): update support matrix with verified CI build evidence
de79132  feat(data): migration matrix V1..V7 -> V8 with epoch data, FK and backup round-trip tests (Task C, G4)
2007bc7  test(data): close migration-matrix FK coverage gaps (Task C)
```

### Handoff & continuity

- All earlier phases (0–3B.3) and tasks A, B, C are already merged into `master`
  (see landing commits above). The task list starts at **Task D**.
- A previous executor (GLM-5.2) is no longer available; this plan is handed to
  a fresh agent executor (GPT-5.6 Terra / Hemes Agent) with no shared memory.
  Do not assume any facts other than what you can read in the repository and
  in this file.
- Delivery cadence: one task at a time, in the order in section 3. Report back
  to the owner with files changed, verification results, and residual risks
  before starting the next task.

**Definition of done for any change (project rule):** no redundant comments; SPDX headers on new files; typed errors/retry/cancellation/shutdown considered; docs/changelog claims must match implementation; CI green; reversible.

---

## 2. Working rules (MANDATORY)

1. **Never `git push`.** Commit locally only; integration/review/push is done by the owner or reviewer agent.
   One task = one commit with a message like `feat(data): <summary> (Task X)`.
2. **Shared working copy:** other agents may stage/commit their own work around you. Before touching git, run `git status --short`
   and check the index. Never commit files you did not touch: use `git add <your-files>` explicitly, and prefer
   `git commit --only <files>` when the index is dirty. Never `git add -A`.
3. **Canonical verification (all must be green before committing):**
   - `cargo fmt --all --check`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo test --workspace`
   - `npm run check --prefix frontend`
   - `npm run build --prefix frontend`
   - `npm run check:version --prefix frontend`
   - `npm run license:check --prefix frontend`
   - `cargo deny check licenses`
   - `git diff --check`
   Never run two `cargo` commands in parallel from the same shell (target/ lock contention can hang).
4. **After each task**, update `TECH_DEBT.md` (mark the entry done / add a `TD-NEW#` resolved line) and
   `RELEASE_CHECKLIST.md` (tick checklist items that genuinely closed).
5. Do not modify migration SQL `V001..V008`; do not redesign existing APIs without calling it out in the report.
6. Code style: English doc comments, SPDX header on every new file. Follow file-local style (some legacy files have
   Russian doc comments — do not rewrite them without reason).

---

## 3. Remaining tasks, in order

### Block A — Phase 4 Data integrity (close Gate G4)

Gate G4 status: migration matrix ✓, backup/restore ✓, FK ✓. Remaining: indexes for real query patterns,
long-history ("bounded record caps"), complete aggregates, versioned export/import, migration runbook.

The "bounded record caps" live in `crates/application/src/reporting.rs:22-40`:
`MAX_REPORTING_PAGE_LIMIT=1000`, `DEFAULT_HISTORY_PAGE_LIMIT=50`, `DEFAULT_EXPORT_PAGE_LIMIT=1000`,
`DASHBOARD_ACTIVITY_HISTORY_LIMIT=1000`, `ACHIEVEMENT_HISTORY_LIMIT=500`, `ANALYTICS_HISTORY_LIMIT=100`,
`MAX_REPORTING_PAGE_OFFSET`. Concern (risk R-010): aggregates misstate progress beyond the caps.

| Task | Scope & acceptance criteria |
|---|---|
| **D.** Indexes + long-history semantics (TD5) | Measure real query patterns in repositories; add indexes proven by `EXPLAIN QUERY PLAN` (with failing tests if necessary); ensure pagination/cursor + explicit `ORDER BY` on every history surface; keep 1..10k+ fixture tests, add regression time thresholds; no behavior change to aggregate values. |
| **E.** Complete aggregates without old caps | `daily_stats` stay sparse; history/dashboard/achievement/analytics aggregate reads are correct beyond the caps above; long-history tests prove correctness at >100k records; document which surfaces paginate vs summarise. |
| **F.** Versioned export/import + restore IPC | Versioned schema, bounded input, validation-before-write, dry-run preview, conflict policy (merge/replace) and atomic replacement (temp store + swap); expose restore via Tauri IPC replacing current file-path-only `restore_from_path` (today the API deletes destination first — move recoverable behavior to temp+swap to close that risk); backup/restore runbook in `docs/`. A restore must never run while a live `Database` is open on the same path (coordinate the take-over in the IPC command). |

### Block B — Phase 5 Security (close Gate G5)

Read `ROADMAP.md §Phase 5` for full requirements.

| Task | Scope & acceptance criteria |
|---|---|
| **G.** Threat model | `docs/security/THREAT_MODEL.md` — assets, trust boundaries, abuse scenarios, mitigations, residual risk; align with the GLib/GTK3 decision (ADR 0001); record decisions. |
| **H.** Least-privilege capabilities | `crates/app/capabilities/*.json` expose only commands actually used; command allowlist; window/webview restrictions; documented exceptions; a test/audit that no command is reachable outside its intent. |
| **I.** Structured redacted logging | opt-in verbose logging with retention limits; redact typed payload content; log only path+error-class; replace `eprintln!` warn-and-continue in `crates/app/src/main.rs` (pre-migration backup warning) with a proper logger; tests that outputs never contain typed text. |
| **J.** Hostile-input/fuzz coverage | malformed IPC, oversized inputs, path traversal, malicious import/CSS, error leakage, repeated/rapid requests; whitelist-based validation; fuzz seeds or cheap fuzz tests; all run in CI. |

### Block C — Phase 6 Release engineering (Gate G6)

CI already builds deb/rpm/AppImage + Windows NSIS + binary tarball every push; actions SHA-pinned.

| Task | Scope |
|---|---|
| **K.** Release workflow | Separate PR checks / release-candidate / promotion; RC is a draft with checksum file; promote only after smoke + maintainer approval; no secrets in job logs; OIDC/least-privilege. |
| **L.** SBOM + provenance attach | CDX already produced by `license-policy`; attach to release artifacts; document reproducibility level; attach provenance. |
| **M.** Linux smoke | Clean environment (container/VM, Ubuntu 24.04): install (deb/rpm/AppImage) → first screen → short test → persist → restart → backup/export → clean exit. Automate in `scripts/` or a workflow. |
| **N.** Flatpak redesign | Source build in the manifest (not a prebuilt binary), narrow permissions, pinned runtime; install+launch smoke. |
| **O.** AppImage workflow revamp | Pin `appimagetool`, remove stale defaults, fix path/version variables, explicit failures, smoke. |
| **P.** PKGBUILD verification | Reproducible `makepkg`, `sha256sums` pinned, Apache metadata, test in a clean Arch container (may need owner). |
| **Q.** Windows clean-smoke | NSIS install → launch → first screen → save → exit on clean Windows (`windows-latest` runner or a VM the owner provisions); automate what is possible. |

### Block D — Phase 7 + Phase 8 prep

| Task | Scope |
|---|---|
| **R.** IPC contract audit/typification | Enumerate every Tauri command, DTO and error envelope; add contract tests asserting stable surfaces; document the intended client/server boundaries per Gate G3. |
| **S.** `App.svelte` decomposition (TD3) | Move interaction state into feature stores (Svelte); keep the backend-boundary model; no user-visible feature change; long-term phase 8 prep. |

---

## 4. Order of execution & dependencies

1. Strict: **D → E → F** (Gate G4) → **G → H → I → J** (Gate G5) → **K → L → M → N → O → P → Q** (G5/G6) → **R → S**.
2. F (restore IPC) may re-touch the same `Database` paths as Task B — respect the "never two connections on the same live path" constraint.
3. After every task, report to the owner: files changed, verification table results, residual risks.

## 5. Open questions for the owner

- Q: Windows clean-smoke (Task Q) — is a clean Windows VM/runner accessible, or CI-only?
- Q: Flatpak/AppImage (N/O) — container/VM available?
- Q: Strict gate order, or "practical available" reorder (e.g. F earlier than J/Q)?
- Q: Task S scope needs product sign-off (feature semantics) before delivery.

---

*This is the operating plan only; product history lives in `ROADMAP.md`.*