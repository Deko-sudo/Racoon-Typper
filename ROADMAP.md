# Racoon Typper — Modernization and Production Roadmap

**Status:** Approved implementation specification — Phases 0, 1, and 2 are implemented; Phase 3A durable session identity, Phase 3B.1–3B.2 application/runtime-provider work, Phase 3B.3.1 recovery contracts, Phase 3B.3.1a, Phase 3B.3.2, Phase 3B.3.3, and Phase 3B.3.4 are accepted; Phase 3B.3.5 and Phase 3B.3.6 are accepted after independent re-review; Phase 3B.3 recovery architecture accepted and complete; Phase 3B.3.7 final acceptance complete; later milestones remain not started and require explicit approval  
**Review date:** 2026-07-17  
**Product:** Racoon Typper, a Rust/Tauri/Svelte desktop typing trainer  
**License objective:** Apache-2.0-compatible distribution, with third-party components retaining their original licenses and required notices  
**Document role:** Single source of truth for modernization, foundation hardening, release readiness, and subsequent product delivery

> This document is a plan, not authorization to perform the work. Each phase requires explicit approval, must be completed and validated, and must be committed before the next phase begins.

## Implementation status

- **Phase 0 — Release Baseline:** approved and implemented in the current working tree; commit remains pending.
- **Phase 1 — Apache-2.0 Migration:** approved and implemented in the current working tree; commit remains pending.
- **Phase 2 — Runtime foundation:** complete in the current working tree; its final validation evidence is recorded with the phase handoff.
- **Phase 3A — Durable session identity:** complete in the current working tree; UUIDv7 identity, additive persistence migration, immutable lifecycle correlation, and regression coverage are implemented.
- **Phase 3B.1:** application kernel and session ports complete in the current working tree.
- **Phase 3B.2:** deterministic runtime provider seams complete in the current working tree; recovery and persistence policy remain deferred.
- **Phase 3B.3.1:** recovery contracts accepted after independent re-review; no persistence or runtime recovery was introduced.
- **Phase 3B.3.2:** accepted after independent re-review.
- **Phase 3B.3.3:** accepted after independent re-review.
- **Phase 3B.3.4:** accepted after independent re-review.
- **Phase 3B.3.5:** accepted after independent re-review.
- **Phase 3B.3.6:** accepted after independent re-review.
- **Phase 3B.3.7:** final acceptance complete; the durable recovery subsystem is accepted. Normal live-completion wiring through that protocol remains separately deferred. Later Phase 3B milestones remain not started and require separate approval.

Historical **Phase 0 — Release Baseline** remains limited to release topology,
baseline/version tooling, support documentation, and outdated release-command
corrections. Existing feature/resource work in the dirty tree remains separately
identified in `BASELINE.md`.

### Current execution stages — 2026-08-07

This file remains the repository's canonical execution roadmap. The historical
modernization phases above are preserved as accepted records. The following
uniquely named execution stages reconcile independently accepted local work with
the published repository; they do not change the status or meaning of any
historical phase:

1. **Stage S0 — repository reconciliation and accepted-work publication.**
   Publish the separately independently reviewed and accepted 25-theme pack with
   its resources, registry, settings compatibility, selector integration,
   documentation, tests, and provenance records. Preserve unrelated local work.
   The accepted durable recovery subsystem is not reopened; its normal
   live-completion wiring remains deferred.

   **Stage S0 publication gate:** Only independently reviewed accepted work may
   enter the publication branch. Unrelated dirty-worktree changes remain outside
   the commit and outside the published baseline. Publication uses a clean,
   isolated commit/branch containing only accepted Stage S0 work; the original
   dirty worktree is preserved rather than cleaned destructively.
2. **Stage S1 — small correctness fixes.** Do not begin before Stage S0 is
   published and reviewed. Start with replay-pagination metadata consistency
   (`offset`, `total`, and `has_more`) and its negative regression coverage.
3. **Stage S2 — reporting architecture migration.** Migrate one read-only
   reporting vertical slice at a time from Tauri command adapters to the
   existing application reporting use cases and ports. Do not create a second
   reporting architecture or change reporting semantics during Stage S0.

No Stage S1 or Stage S2 implementation is authorized by this Stage S0
publication work.

## 1. Executive decision

The project should remain a desktop-first modular monolith built with Rust, Tauri, Svelte, and SQLite. A technology rewrite, microservice split, ORM adoption, or asynchronous back-end rewrite is not justified by the current product or evidence. The largest risks are boundaries and operational discipline, not the choice of core technologies.

The two existing roadmap documents are not safe release plans:

| Roadmap | What it does well | Why it cannot remain authoritative |
|---|---|---|
| `ROADMAP_v1.1.md` | Smaller feature scope; lower-risk product direction; reasonable near-term learning features | Older; omits the newer worktree direction; lacks dependencies, acceptance criteria, release gates, licensing, security, migration, and rollback planning |
| `ROADMAP_v1.2.md` | More complete feature inventory; captures export, onboarding, themes, languages, and goals | Claims completion that is not supported by the current code; treats native plugins as a normal feature without a threat model; omits foundation and release blockers; mixes product ideas with implementation claims |
| **This document** | Evidence-based baseline; foundation-first sequencing; legal and operational gates; explicit rollback and acceptance criteria | Proposed and not effective until approved |

The implementation is closer to a partially completed v1.1 release plus work-in-progress v1.2 changes than to either roadmap's “completed” sections. The current committed baseline has a working domain/core/data/app/frontend stack and passing workspace tests, while the working tree contains additional uncommitted changes around themes, languages, settings, replay, and CI. Those changes must be reviewed independently and are not treated as released functionality by this roadmap.

### Initial quality assessment

**Current roadmap quality: 4/10.** The existing documents communicate product ambition, but they are not sufficiently accurate or operationally useful for a commercial release. They do not define the system's actual boundaries, legal provenance, migration safety, package verification, security posture, or measurable release gates. The replacement plan below raises the bar by making evidence, risk, and acceptance criteria first-class.

### Highest-priority findings

1. The configured Tauri asset path is wrong for the current invocation, so a production build reaches the frontend build and then fails to find its web assets.
2. CI and packaging scripts disagree about Tauri commands, versions, targets, and output artifacts; Windows uses `--target nsis`, which is a bundle-type value rather than a Rust target triple.
3. The pre-existing imported GPL theme catalog was a release blocker; Phase 1 removes it and replaces the shipped theme set with original Racoon assets.
4. Project-owned theme, icon, course, word, and quote provenance is not documented well enough to establish clean distribution rights.
5. **Resolved in Phase 2:** command adapters are grouped, completion orchestration is private, and production app-layer SQL is repository-owned; the broader Phase 3B use-case/port split remains deferred.
6. **Resolved in Phase 2/3A for the running process:** completion is an explicit retry-safe lifecycle transition and each persisted test has a backend-issued durable UUIDv7 identity; durable restart recovery remains deferred Trusted Core work.
7. **Resolved in Phase 2/3A for the running process:** completion writes related records and the immutable session identity in one transaction; durable idempotency and backup/restore remain deferred.
8. **Resolved in Phase 2/3A at the current IPC boundary:** the backend derives scores/completion and identity, dedicated endpoint responses are typed, and stale identity tokens are rejected; versioned/generated universal contracts remain Phase 3B work.
9. Database lifecycle, migration rollback, foreign-key enforcement, cross-platform paths, panic handling, and startup failure behavior are under-specified.
10. Current test success is useful but insufficient: packaging, cross-platform smoke tests, license scans, security gates, and release artifact verification are missing or unreliable.

## 2. Scope, principles, and definitions

### 2.1 Scope

This roadmap covers:

- the Rust workspace in `crates/`;
- the Svelte/Tauri frontend in `frontend/`;
- SQLite schema, migrations, repositories, and persistence workflows;
- resources, themes, icons, scripts, and generated assets;
- CI, packaging, installers, release automation, and support documentation;
- public repository documentation and contribution workflows;
- product features listed in the legacy roadmaps, but only after foundation gates pass.

It does not authorize deleting legacy roadmaps, rewriting the application, changing the product name, or selecting a new license without explicit approval.

### 2.2 Planning principles

- **Evidence before claims.** A feature is “released” only when code, tests, documentation, and a release artifact demonstrate it.
- **One boundary, one owner.** Tauri adapts transport; application services coordinate use cases; domain owns business invariants; data owns persistence; resources own content loading.
- **Authoritative state lives on the back end.** The frontend may request actions and render results, but it must not be the authority for scores, completion, entitlements, or persistence outcomes.
- **Atomic user-visible outcomes.** Related records are committed together or not at all; retries are safe.
- **Explicit failure.** Unsupported input, missing content, migration failure, and persistence failure must return typed, actionable errors rather than silently falling back.
- **Small reversible steps.** Each phase has a narrow goal, a validation contract, and a rollback strategy.
- **Security is a release property.** Capabilities, input limits, file access, export/import, custom content, and future plugins are threat-modeled before implementation.
- **Compatibility is intentional.** Third-party dependencies retain their licenses; project-owned work is Apache-2.0; all redistributed notices are tracked.
- **Measure before optimizing.** Performance work requires representative benchmarks and regression thresholds.

### 2.3 Priority and complexity

- **P0 — release blocker:** must be resolved before a supported production release.
- **P1 — foundation:** required before adding material product scope.
- **P2 — product:** valuable after P0/P1 gates pass.
- **P3 — exploratory:** deferred until user value and operational cost are demonstrated.

Complexity estimates are relative implementation size, not calendar promises:

- **S:** up to several focused files and low migration risk;
- **M:** one bounded subsystem, tests and documentation included;
- **L:** cross-layer change, migration or packaging matrix, multiple validation environments;
- **XL:** substantial new capability or external security/operations surface.

## 3. Current-state inventory and evidence baseline

### 3.1 Technology baseline

- Rust workspace with `domain`, `core`, `data`, `resources`, and `app` crates.
- Tauri desktop shell and Svelte frontend.
- SQLite through `rusqlite`, with `refinery` migrations.
- Local resources for lessons, words, quotes, and themes.
- Workspace/package version currently represented as 1.1.0 in the Rust/Tauri side, while `frontend/package.json` remains 0.1.0.
- Release profile already enables stripped, thin-LTO builds; older performance/debt documents contain stale claims that should not drive work.
- No fonts were found in the repository during this review.

### 3.2 Baseline validation already observed

The following checks pass on the current working tree, subject to the existing uncommitted changes:

- `cargo fmt --all --check`;
- `cargo clippy --workspace --all-targets -- -D warnings`;
- `cargo test --workspace` — 435 tests observed across the workspace;
- frontend type checking via `npm run check`;
- frontend production build via `npm run build`;
- `git diff --check`.

These are not release approval because they do not prove package creation, cross-platform behavior, licensing, migration recovery, security boundaries, or artifact integrity.

### 3.3 Baseline failures and uncertainty

- `npx --prefix frontend tauri build --ci --no-bundle` builds the frontend and then fails because `frontendDist` resolves relative to `crates/app/tauri.conf.json` as `crates/frontend/dist`, not the repository's `frontend/dist`.
- The documented `cargo tauri dev` command is unavailable in the current environment unless the cargo subcommand is installed; the repository lacks one canonical, verified development command.
- Linux packaging claims are not demonstrated by the current CI artifact paths.
- The Flatpak manifest installs a prebuilt binary, uses broad permissions, and is not a reproducible source build.
- `PKGBUILD` still declares `MIT`, uses a skipped source checksum, and depends on a command that is not available in the baseline environment.
- `build-appimage.sh` has a stale default version, a path-variable typo, and an AppRun variable expansion problem; it downloads an unpinned continuous tool without verification.
- Cargo audit's local advisory scan found no vulnerability advisory in the available database but reported 17 unmaintained/unsound dependency warnings, including an affected GLib line. A complete transitive license and dependency decision requires a network-enabled, reproducible CI scan.
- The frontend lockfile's locally parsed production dependency licenses are permissive (MIT, Apache-2.0, BSD-3-Clause, ISC, 0BSD, and dual permissive expressions); this is not a substitute for a CI-generated transitive SBOM and license policy.

## 4. Target architecture

### 4.1 Recommended shape

Retain a modular monolith and make the boundaries explicit:

```text
Svelte UI
  │ typed IPC DTOs; loading/error states; presentation-only state
  ▼
Tauri command adapter
  │ authorization, input limits, serialization, tracing context
  ▼
Application/use-case layer
  │ start test, process key, finish test, save settings, export profile
  │ owns orchestration and transaction requests
  ├──────────────► Domain
  │                invariants, value objects, state machines, policies
  ├──────────────► Resource ports
  │                validated content/theme/catalog loading
  └──────────────► Data ports
                   repositories, transaction unit of work, migrations
                              ▼
                         SQLite/filesystem
```

### 4.2 Boundary rules

- `app` command handlers must be thin adapters. They should not contain SQL, analytics algorithms, content fallback policy, or multi-record persistence orchestration.
- The domain must not depend on Tauri, SQLite, filesystem paths, environment variables, or frontend DTOs. It may depend on serialization only where that is a deliberate boundary decision; otherwise move wire/persistence representations outward.
- The application layer should expose use cases with typed request/response/error models and explicit ports for clock, ID generation, randomness, content, and persistence.
- `data` owns SQL, schema mapping, transactions, connection configuration, backup hooks, and repository implementations.
- `resources` validates all loaded data and reports malformed content with source context. It must not silently substitute English or discard parse failures.
- Frontend types should be generated or checked against a single contract source. Avoid hand-maintained nested `any` payloads.
- Keep a single SQLite writer model unless measured requirements justify a more complex pool. Remove the redundant nested mutex design while preserving safe serialization of access.
- Use immutable value objects or validated constructors for IDs, durations, language codes, mode configuration, score inputs, and custom text limits.

### 4.3 Technology recommendations and trade-offs

| Decision | Recommendation | Benefit | Trade-off |
|---|---|---|---|
| Desktop architecture | Keep Tauri + Rust + Svelte | Existing code and product fit; small local footprint; native packaging | Requires careful IPC and platform testing |
| Backend structure | Add a small `application` crate or clearly isolated application module | Makes use cases and transaction boundaries testable without Tauri | Adds a workspace boundary and migration effort |
| IDs | **Phase 3A selected UUIDv7; Phase 3B.2 injects the source:** backend-generated immutable `SessionId` values with a canonical text form and an application-owned identity port | Standard UUID/cloud interoperability, time ordering, collision resistance, replay/debug/synchronization keys | Adds a dependency and additive migration/serialization work |
| Stable config identity | Canonical typed representation plus a stable cryptographic digest | Prevents PB identity changing with map/order/hasher implementation | Requires a versioned canonicalization rule |
| IPC contracts | Prefer generated TypeScript bindings from Rust or a checked JSON schema; choose one in an ADR | Removes drift and unsafe `any` payloads | Generation step and CI maintenance |
| Local logging | Structured `tracing` with redaction and bounded local files | Debuggability without leaking user text | Log rotation and privacy policy are required |
| Property testing | `proptest` for text-buffer, mode, score, and state-machine invariants | Finds edge cases beyond examples | Test generation can complicate debugging |
| License/SBOM gate | `cargo-deny`/`cargo-about`, npm license tooling, and CycloneDX or equivalent SBOM | Repeatable legal and supply-chain evidence | Tool configuration and exception review cost |
| UI tests | Component tests plus a small set of Tauri/E2E smoke journeys | Faster diagnosis than only full-app tests | Browser/runtime test setup cost |
| Plugins | Defer native plugins; if demand is validated, start with a narrow capability-based WASM design | Smaller blast radius and clearer sandbox boundary | WASM limits APIs and may be slower or more complex |
| Database | Keep SQLite; improve schema/transaction discipline before considering another store | Correct fit for local single-user desktop data | Requires careful migrations and backup/restore |

Do not adopt an ORM, server database, microservices, or a general plugin ABI in the foundation phases. Each would enlarge the change surface without addressing the observed risks.

## 5. Roadmap comparison and corrected product scope

### 5.1 What is actually supported

The current implementation visibly contains typing modes, lessons, adaptive/weak-key analysis, statistics, replay, settings, themes, and local persistence. The workspace test suite provides meaningful coverage. It does not establish that every feature claimed by the legacy release notes is complete or production-ready.

The working tree contains partial or unverified work around:

- additional theme data and an import script;
- more language resources and frontend language selection;
- settings and replay persistence changes;
- i18n and UI changes;
- CI changes.

These changes should be reviewed as normal change sets before being declared part of a release.

### 5.2 Features explicitly moved behind foundation gates

The following are not deleted, but are not allowed to drive the foundation schedule:

- daily goals and weekly summaries;
- onboarding and achievement gallery;
- import/export and full profile restore;
- custom theme editor or user CSS;
- additional languages and layout support;
- Dvorak/Colemak support;
- course editor and Anki/CSV interchange;
- plugin system;
- multiplayer/LAN, cloud sync, mobile companion, ML adaptation, voice narration, and eye tracking.

Each must have a separate product brief, telemetry/feedback hypothesis, security review where applicable, and acceptance criteria before implementation.

## 6. Apache-2.0 compatibility and provenance plan

### 6.1 Licensing policy

“Apache-2.0 compatible” does not mean that every third-party package can be relabeled Apache-2.0. Project-owned source, original assets, and project-authored content should be Apache-2.0 where the contributors can grant those rights. Third-party dependencies and resources retain their original licenses, and the distribution must preserve their notices and comply with their terms.

Permissive MIT, BSD, ISC, 0BSD, and Apache-2.0 dependencies are generally compatible with an Apache-2.0 application when their notices and conditions are preserved. GPL-family material is a separate copyleft obligation and cannot be treated as Apache-2.0 project material. Legal review is required for any ambiguous or mixed-origin resource.

### 6.2 Current inventory findings

| Material | Current finding | Classification | Required action |
|---|---|---|---|
| Project Rust manifests and `LICENSE` | Workspace declares Apache-2.0 and the root license is Apache-2.0 | Likely clear for project-owned code | Preserve copyright notices; add automated policy checks |
| `PKGBUILD` | Still declares `MIT` | Inconsistent | Change to Apache-2.0 in the licensing phase and validate package metadata |
| npm lockfile | Locally observed licenses are MIT, Apache-2.0, BSD-3-Clause, ISC, 0BSD, and permissive dual licenses; no GPL/LGPL/AGPL detected locally | Permissive but incomplete evidence | Generate a reproducible inventory in CI; preserve notices; review every exception |
| Rust transitive dependencies | Full license metadata was not available offline; advisory scan reports unmaintained/unsound lines | Unverified | Add `cargo-deny`/`cargo-about` policy; review GTK/GLib and other warnings; document accepted platform constraints |
| Pre-existing imported theme catalog | Copied catalog and license material had a GPL-family license and uncleared source provenance | Removed in Phase 1 | Keep only the original Racoon theme set and enforce asset inventory checks |
| Existing theme JSON/CSS | Several themes had no source, author, or license manifest and some were named after external palettes | Provenance unclear | Replace with original Racoon palettes, add SPDX/provenance metadata, and remove the old names |
| Application icons | No visible source/provenance metadata; duplicate hashes exist among size variants | Provenance unclear, not proven incompatible | Obtain contributor attestation/source license or replace with original source SVG/PNG; generate sizes from one source and record it |
| Fonts | No font files found | No current font licensing issue found | Keep the inventory check in CI so future additions require provenance |
| Courses, quotes, and word lists | Content source and license metadata are not documented comprehensively | Unclear; quotes have copyright risk | Add a content manifest and contributor attestations; remove or replace content that cannot be cleared; prefer original, public-domain, or clearly licensed material |
| Pre-existing theme import script | The importer extracted external source and executed an object with `Function(...)` | Legal and supply-chain risk; unsafe tooling design | Removed in Phase 1; no external theme importer is permitted |
| Tauri schema URL | Configuration references a remote schema URL | Reproducibility concern, not a runtime asset | Pin to a project-approved version or use a local/generated schema where practical |

### 6.3 Required legal deliverables

Create during Phase 1, not before approval:

- `THIRD_PARTY_NOTICES.md` with package/resource name, version or commit, origin, license, copyright, and distribution obligations;
- a machine-readable asset/content manifest under `licenses/` or `docs/legal/`;
- SPDX identifiers for every distributed third-party component;
- contributor provenance attestations for project-authored themes, icons, words, quotes, and courses;
- a CI license policy that fails on GPL/LGPL/AGPL or unknown licenses unless an explicitly reviewed exception exists;
- a release SBOM attached to every production release.

### 6.4 Preferred migration for incompatible or unclear assets

1. Freeze distribution of the questionable asset.
2. Record its exact path, source, commit, license, and dependency in the inventory.
3. Remove it from the build/resource loader or place it outside the distributable tree while the decision is pending.
4. Replace it with an original asset or an asset whose license explicitly permits the intended distribution.
5. Add attribution and regression checks so the replacement cannot be silently reintroduced.
6. Have a maintainer review the resulting manifest before the licensing gate is marked complete.

## 7. Foundation audit and required corrections

| Area | Evidence/risk | Required correction | Priority |
|---|---|---|---|
| Test state machine | **Phase 2 complete:** `CoreEngine` and command adapters reject invalid replacement/reset transitions and retain a retryable completion | Preserve `Idle → Running → AwaitingPersistence → Persisting → Persisted`; keep durable crash recovery out of this in-memory boundary | P0 |
| Time mode semantics | Current policy completes on duration expiry or supplied-text exhaustion; generated time-mode content is sized conservatively | Text looping/backfill or a changed source-exhaustion policy is product semantics work, not this Foundation refinement | P2 |
| Persistence atomicity | **Phase 2/3A complete in-process:** completion-related records and the immutable session identity commit in one transaction and a failed transaction retries the same in-memory result | Restart-safe idempotency and backup/restore work remain Trusted Core/Phase 4 | P0 |
| Frontend authority | **Phase 2/3A complete:** frontend sends input plus the backend-issued session token; backend validates identity and derives elapsed time, score, completion, and persistence payloads | Preserve backend authority as later contracts/features evolve | P0 |
| Raw SQL in commands | **Phase 2 complete:** no production SQL remains in the app command/service layer; streak SQL is owned by a repository | Phase 3B.1 establishes the application boundary; later Phase 3B owns broader analytics/use-case migration | P1 |
| Nested locking | **Phase 2 complete:** `Database` owns the connection mutex and production callers use `with_connection`/`with_transaction` | Keep lock ordering/documentation under review if concurrency changes | P1 |
| ID generation | **Phase 3A/3B.2 complete:** backend-generated UUIDv7 values are immutable, persisted with completed tests, ordered, correlated at IPC boundaries, and supplied through an application identity port; migrated rows receive deterministic `legacy-test-<id>` values | Crash recovery and restart-safe identity handling remain Phase 3B.3 Trusted Core work | P1 |
| Clock handling | **Phase 2/3B.2 complete:** replay/scoring use the process-monotonic elapsed clock; completion metadata snapshots one UTC wall-clock instant supplied through a wall-clock port | Durable recovery and restart-safe completion remain Trusted Core work | P1 |
| Config identity | `DefaultHasher` over serialized data is not a stable persisted identity | Canonicalize a versioned config and hash it with a stable digest; add migration/version semantics | P1 |
| Validation | **Phase 2 complete:** command/repository validation bounds input and rejects unsupported content rather than substituting fallback text | Typed request/config algebra and generated contracts remain Phase 3B work | P0 |
| Error handling | **Partially complete:** startup/settings paths use typed errors and atomic replacement; raw internal-error redaction and structured logging remain unfinished | Complete redaction/logging in the security/observability phases | P1 |
| Database pragmas | **Phase 2 complete:** every database connection enables and tests foreign keys, WAL, synchronous mode, and a busy timeout | Revisit only with a data-layer redesign | P0 |
| Migrations | **Phase 3A V005 complete:** additive `session_id` column, deterministic legacy backfill, unique index, and required/immutable guards preserve integer/replay keys | Back up before migration; future schema work still needs restore/forward-fix procedures and upgrade fixtures | P0 |
| Analytics | Several queries cap history at 100/500/1000 records and use hard-coded or fallback values | Define metric semantics, query complete aggregates, add indexes, and test long-history correctness | P1 |
| Lesson policy | `LessonMode` reports `Custom`; lesson completion can be marked without server-side threshold enforcement | Make lesson configuration and eligibility domain policies authoritative and transactional | P1 |
| Resource loading | Invalid resource data is silently ignored or replaced with English/default text | Fail startup only for required catalogs; report optional catalog errors with source context; never hide data corruption | P1 |
| Filesystem paths | **Phase 2 complete:** Tauri-managed paths and non-destructive Linux migration replace direct home/XDG assembly | Cross-platform smoke evidence remains Phase 6 work | P0 |
| State ownership | `App.svelte` is a large component with duplicated/manual types and broad mutable state | Split feature stores and view components; establish server/cache/session ownership; keep UI state local where possible | P1 |
| IPC contracts | **Phase 2/3A complete:** endpoint responses use named serializable types, frontend wrappers no longer expose opaque analytics/weak-key results, and session identity is backend-issued and mismatch-checked | Versioned/generated universal contracts and typed polymorphic mode configuration remain Phase 3B work | P1 |
| Custom CSS/themes | Dynamic style injection would make future user CSS an injection and persistence risk | Store structured theme tokens by default; if CSS is supported, sanitize/limit it, scope it, and threat-model import/export | P0 |
| Shell/plugin surface | `@tauri-apps/plugin-shell` is present but appears unused and is not needed for current product behavior | Remove unused dependency and permission; add capabilities only for a demonstrated use case | P1 |
| Tauri capabilities | The capability file describes broad IPC access and does not demonstrate least-privilege command exposure | Define a minimal command allowlist, restrict windows/webviews, and review permissions as code | P0 |
| CSP | Current CSP includes `unsafe-inline` for script/style | Remove inline script permission; reduce inline style reliance or document a narrowly justified exception after testing | P1 |
| Observability | No clear structured local log, diagnostics, crash, or support bundle workflow | Add redacted structured logs, rotation, log levels, diagnostic export, and actionable error IDs | P1 |
| Privacy | Replay/custom text/export retention and deletion policy are not explicit | Document local data locations, retention, deletion, export scope, and sensitive-content handling | P0 |

## 8. Phased implementation specification

The phases below are deliberately ordered to minimize irreversible changes. A phase may be split into smaller approved change sets, but its gate must be satisfied before the next phase starts.

### Phase 0 — Release freeze, evidence baseline, and blocker closure

**Priority:** P0  
**Complexity:** L  
**Dependencies:** None  
**Purpose:** Establish an honest baseline and make the project build path deterministic before changing architecture.

#### Objectives

- Freeze unsupported release claims and identify the exact supported baseline.
- Establish one canonical development, test, and package command for each supported platform.
- Fix the Tauri configuration topology so a clean checkout can build the frontend and locate its assets.
- Capture current worktree changes as separate reviewable units; do not fold them into this phase implicitly.
- Define the support matrix, version source of truth, and release artifact naming policy.

#### Expected files and changes

- `Cargo.toml`, `Cargo.lock`, `crates/app/tauri.conf.json`, `frontend/package.json`, and package-lock metadata: choose one version source and one documented invocation.
- `README.md`, `INSTALL.md`, `CONTRIBUTING.md`: replace unsupported commands and stale release claims with verified commands.
- `ARCHITECTURE.md` (new): record the current and target boundaries.
- `scripts/` (new or existing): add small, deterministic wrappers for check/build/package; do not hide failures.
- `.github/workflows/ci.yml`: only baseline command/path corrections needed to reproduce the local checks; the full release workflow belongs to Phase 6.
- `RELEASE_CHECKLIST.md` and release notes: mark claims as verified, provisional, or removed.

#### Risks and mitigations

- **Risk:** A path fix works only from one working directory. **Mitigation:** invoke it from a clean checkout in CI and a local smoke script.
- **Risk:** Version changes break packaging metadata. **Mitigation:** create a version consistency check before changing release automation.
- **Risk:** Existing uncommitted work is overwritten. **Mitigation:** do not edit overlapping feature files; review status and diff before every patch.

#### Validation checklist

- Clean checkout can run the documented development command.
- Frontend check/build pass from the repository root.
- Tauri no-bundle build locates assets and produces a binary.
- Version values are consistent or the intentional source of truth is documented.
- A support matrix distinguishes tested from untested platforms and package formats.
- No existing user modifications are lost.

#### Rollback

Revert only the phase commit. Keep the evidence report and baseline logs. Do not revert user worktree changes or use destructive worktree commands.

#### Gate G0

No subsequent phase starts until the canonical build path, version policy, and honest baseline are approved.

---

### Phase 1 — Apache-2.0 compatibility, provenance, and supply-chain inventory

**Priority:** P0  
**Complexity:** L  
**Dependencies:** Phase 0  
**Purpose:** Remove legal ambiguity before more assets, features, or release artifacts are added.

#### Objectives

- Make all distributed project-owned material traceable to a license and contributor/source.
- Remove or replace GPL/LGPL/AGPL and unknown material from the distributable path.
- Preserve third-party notices rather than relabeling dependencies.
- Turn the inventory into a repeatable CI gate.

#### Expected files and changes

- `PKGBUILD`: correct stale license metadata.
- `resources/themes/**`: remove the imported GPL catalog/importer; retain only original Racoon themes with provenance metadata.
- `crates/app/icons/**`: attest or replace icon source; regenerate size variants from a source asset.
- `resources/courses/**`, `resources/quotes/**`, `resources/words/**`: add source/license metadata and remove uncleared content.
- Pre-existing external-theme importer: remove; no replacement importer is part of Phase 1.
- `THIRD_PARTY_NOTICES.md`, `licenses/` or `docs/legal/`: add human-readable and machine-readable inventories.
- `Cargo.toml`, `frontend/package.json`, lockfiles, and CI configuration: add license policy tooling and exception review.

#### Risks and mitigations

- **Risk:** Recreated themes are still derivative or retain confusing third-party branding. **Mitigation:** use original palettes, remove upstream names/descriptions, and record authorship.
- **Risk:** A transitive dependency has an unrecognized license. **Mitigation:** fail closed on unknown licenses and review exceptions explicitly.
- **Risk:** Asset removal changes the user experience. **Mitigation:** ship original replacement assets and add snapshot/content-count tests.

#### Validation checklist

- No GPL/LGPL/AGPL/unknown item is in the release resource set without a documented legal exception.
- Every asset and content catalog has an owner, source, license, and checksum or generated-source record.
- CI produces a dependency license report and SBOM.
- Package metadata, README, and release artifacts all state Apache-2.0 correctly.
- The release candidate contains only cleared theme/content/icon files.

#### Rollback

Restore a removed asset only in a non-distributable branch while its license is reviewed. Do not restore it to release builds as a rollback shortcut. Revert manifest-only changes if the inventory tooling is defective, but retain the collected provenance records.

#### Gate G1

Maintainer/legal sign-off on the inventory and a passing automated license gate are required before foundation changes are released.

---

### Phase 2 — Runtime foundation, state machines, and lifecycle correctness

**Priority:** P0/P1  
**Complexity:** XL  
**Dependencies:** G1  
**Purpose:** Make typing sessions, completion, time, errors, startup, and shutdown deterministic before restructuring layers.

#### Objectives

- Make the in-memory session lifecycle explicit, closed against invalid transitions, and retry-safe before restart.
- Remove panic-driven user-data paths that are reachable from normal runtime or IPC handling.
- Establish authoritative back-end scoring, monotonic input timing, bounded validation, and atomic completion persistence.
- Preserve behavior with regression tests while making only the small command/service/repository extractions needed for Foundation.
- Do not introduce durable recovery, stable session identifiers, injected clocks/IDs/randomness, or a new application crate in this phase.

#### Expected files and changes

- `crates/core/src/engine.rs` and typing/replay modules: explicit in-memory transitions, idempotent finalization, monotonic elapsed time, and regression tests.
- `crates/app/src/commands/**`, `crates/app/src/session_service.rs`, `crates/app/src/validation.rs`, and `main.rs`: grouped adapters, a private completion service, bounded request validation, stable error envelopes, and backend-authoritative results.
- `crates/data/src/repository/**`: repository-owned streak SQL and persistence validation; no production SQL remains in the app command/service layer.
- `frontend/src/lib/api/ipc.ts`, `frontend/src/lib/types/**`, and the affected Svelte consumers: explicit endpoint response types and lifecycle-aware completion handling.
- Unit and integration regression tests around empty/bounded/Unicode input, direct lifecycle misuse, rapid/repeated completion, failed persistence, rollback, and repository mapping.

#### Risks and mitigations

- **Risk:** Correctness fixes change scores or timing. **Mitigation:** define behavior with golden tests and version any intentional scoring change.
- **Risk:** New state transitions expose frontend races. **Mitigation:** keep the engine claim under its mutex and return authoritative state; Phase 2 deferred session/request correlation until the complete Phase 3A identity protocol was approved.
- **Risk:** More extraction becomes an architecture rewrite. **Mitigation:** use only grouped adapters, one private session service, and repository-owned SQL; defer ports/application crate work to Phase 3B.

#### Validation checklist

- Exactly one completion event and one persistence transaction per in-memory session.
- Duplicate completion/retry is safe and returns the existing in-memory result.
- Time mode follows the documented current policy (duration expiry or supplied-text exhaustion), with generated time-mode text sized conservatively.
- Invalid mode/language/text/duration requests return typed errors.
- Startup directory and migration failures are recoverable; raw-error redaction/logging remains security/observability work and is not claimed complete here.
- Workspace unit/integration tests, frontend checks, and regression tests pass; this phase does not claim property-test coverage.

#### Rollback

Use feature flags only for user-visible scoring or persistence migrations, not to preserve two conflicting state machines indefinitely. Revert the phase commit if invariants cannot be demonstrated; keep database schema changes out of this phase unless backward-compatible and separately approved.

#### Gate G2

A reviewer must sign off the lifecycle diagram, in-process completion/retry tests, validation limits, atomic transaction evidence, and error contract. G2 does not authorize Phase 3A or Phase 3B automatically.

#### Explicitly deferred Trusted Core work

- durable crash recovery and restart-safe idempotency;
- deterministic injected clocks, IDs, and randomness;
- a versioned typed mode/config algebra.

---

### Phase 3A — Durable session identity

**Status:** Complete in the current working tree; Phase 3A is closed after explicit approval. Phase 3B.1 and Phase 3B.2 are complete; later Phase 3B milestones remain gated.

**Priority:** P0/P1  
**Complexity:** M  
**Dependencies:** G2  
**Purpose:** Replace process-local timestamp handles with a stable, backend-authoritative identity that survives persistence and supports replay/synchronization keys.

#### Delivered design

- `racoon_domain::SessionId` generates canonical UUIDv7 values. UUIDv7 provides standard UUID interoperability, time ordering, uniqueness, timestamp-based debugging, and a compact text form suitable for SQLite/cloud synchronization. ULID was rejected because UUIDv7 offers the same ordering properties with broader standard-library and database ecosystem support.
- `CoreEngine` stores the identity immutably for the full `running → awaiting_persistence → persisting → persisted` lifecycle. A replacement after `persisted` receives a new identity.
- `V005__session_identity.sql` adds `tests.session_id` without replacing the existing integer relational key, deterministically backfills existing rows, creates a unique index, and guards required/immutable values. Replay and personal-best foreign keys remain unchanged.
- Start commands generate identities on the backend and return them in the explicit session response. Input and abort commands accept the value only as a correlation token and reject mismatches; the frontend cannot choose or replace an identity.

#### Validation and rollback

- Domain, core, application, repository, migration, and frontend regression tests cover UUIDv7 generation/order, lifecycle retention, stale identity rejection, persistence round trips, legacy backfill, uniqueness, and immutability.
- The migration is additive and data-preserving. Existing integer IDs and replay references remain valid; a pre-migration database backup plus forward-fix remains the operational recovery path.

#### Explicit non-goals

- Durable in-progress crash recovery, restart-safe idempotency, generated/versioned universal contracts, and broader application-layer use-case migration remain Phase 3B.3 or later work. Phase 3B.2 supplies only deterministic runtime provider seams; it does not add a ledger, recovery policy, or persistence policy.

---

### Phase 3B — Application architecture and Trusted Core contract hardening

**Status:** Phase 3B.1 and Phase 3B.2 complete in the current working tree; remaining milestones require explicit approval after the Phase 3B.2 handoff.

**Priority:** P1  
**Complexity:** XL  
**Dependencies:** G2  
**Purpose:** Separate transport, use cases, domain policy, persistence, resources, and presentation state.

#### Objectives

- Complete command thinness beyond the Phase 2 grouping/private-session-service boundary.
- Introduce application services/use cases with typed ports.
- Move remaining analytics and use-case policy out of transport-facing modules; production SQL is already repository-owned after Phase 2.
- Establish a single source of truth for IPC contracts.
- Split monolithic frontend state without changing product behavior.

#### Phase 3B.1 delivered baseline

- Added the infrastructure-free `crates/application` workspace member. It depends only on the domain, core, and value-serialization crates.
- Added transport-neutral session start/completion contracts and business-oriented ports for identity, monotonic time, mode construction, and completion persistence.
- Added a transport-neutral `SessionKernel` for session start, input, abort, lifecycle correlation, and completion snapshot handoff; wired the existing UUIDv7, monotonic clock, resource, and SQLite implementations through those ports without adding deterministic providers, recovery, or changing runtime behavior. Content-specific preparation remains adapter-owned.

#### Phase 3B.2 delivered baseline

- Added application-owned provider ports for UTC wall time and runtime random values, while retaining the Phase 3B.1 identity and monotonic-clock ports.
- Injected those providers into `SessionKernel`; completion timestamps are captured once from the wall-clock port and retained across the existing in-memory persistence retry path.
- Moved session-path word and quote selection to resource-adapter methods that accept the random-value port. Production UUIDv7, monotonic/UTC clocks, and runtime random values remain adapter implementations; fixed providers are test-only.
- Preserved current selection policy, session lifecycle, persistence transaction, and IPC behavior. No recovery, replay, durable ledger, exactly-once restart behavior, service locator, or persistence-policy change was introduced.

#### Phase 3B.3.1 delivered baseline — accepted after independent re-review

- Added application-owned durable session states distinct from the in-memory engine lifecycle, including explicit terminal, interrupted, and quarantined states.
- Added an exhaustive durable transition matrix with explicit valid, idempotent, invalid, and terminal-forbidden outcomes; no terminal state can reopen.
- Added immutable version-1 completion-intent DTOs containing completion effects and an explicitly tagged `time`/`wpm`/`accuracy` policy snapshot; current mutable settings are not consulted by the contract.
- Added independent canonicalization version 1, recursive canonical JSON, stable lowercase SHA-256 fingerprints, strict typed stored-envelope validation at every owned nested schema, an 8 MiB exact canonical bound, signed-zero normalization, finite-number rejection, redacted diagnostics, pure recovery classification/claim policy, ledger/finalizer ports, and readiness-state transitions with deterministic regression tests.
- Finalization is bound to the requested session identity and expected immutable-intent fingerprint; its business result contains no local `TestId` or repository receipt. The ledger explicitly claims `AwaitingPersistence → FinalizationPending`; recovery candidate listings contain metadata only and full intents are validated separately before future finalization.
- Added no migrations, SQLite adapters, startup scanning, command gating, finalization effects, per-key journaling, or persistence-policy changes.

#### Phase 3B.3.1a stored-intent header amendment — accepted after focused review

- Discovery for the future ledger adapter found that metadata-only candidate listing could not construct `CompletionIntentMetadata` from untrusted stored header columns without loading a full payload.
- Added an application-owned stored-header input/classification API. It distinguishes a missing row from malformed fields, uses strict `SessionId::parse`, requires the persisted fingerprint itself to be lowercase SHA-256, preserves deterministic unsupported-version precedence, and returns only metadata—never payload or replay content.
- Header classification proves only that the indexed header is syntactically usable. `CompletionIntent::from_stored_payload` remains the separate strict payload/schema/canonicalization/fingerprint verification boundary.
- This is a minimal contract amendment only: no migration, SQLite schema, data adapter, runtime recovery, finalization effect, startup scan, or command gating was added. Its focused acceptance unblocks Phase 3B.3.2, which remains not started pending separate implementation approval.

#### Phase 3B.3.2 session ledger schema and adapter — accepted after independent re-review

- Added V006 `session_ledger`: one durable lifecycle row per session with bounded sanitized descriptor metadata, UTC timestamps, redacted reason codes, deterministic `(created_at, session_id)` scan ordering, and terminal update/delete/replacement protection. Historical `tests` rows are not backfilled.
- Added V007 `session_completion_intents`: one immutable foreign-keyed canonical payload per ledger session, lowercase SHA-256 format checks, an exact 8 MiB bound, payload-length agreement, and update/delete/replacement rejection. SQL stores opaque bytes only.
- Added `racoon-data → racoon-application` `SessionRecoveryLedger` implementation. Metadata-only candidate scans load ledger and intent headers only and delegate header interpretation to `CompletionIntentMetadata::from_stored_header`; explicit loading separately reads bytes and delegates full validation to `CompletionIntent::from_stored_payload`.
- Application-owned recursive descriptor validation allows only bounded configuration metadata and rejects normalized raw-typing/replay field names at every nesting depth; the data adapter repeats the validation defensively before V006 writes.
- Conditional lifecycle operations use the existing `Database::with_transaction` `IMMEDIATE` helper, so a claim atomically changes `awaiting_persistence → finalization_pending` before another in-process claimant can observe the prior state. Structured SQLite `DatabaseBusy`/`DatabaseLocked` failures map to retryable port failures, verified with separate file-backed connections.
- V006/V007 are additive forward migrations. Running an old binary after migration is unsupported; no down migration or automatic downgrade exists. Backup restore or a forward fix is the recovery path. Historical V001–V005 fixtures are created through Refinery itself and upgraded through the production runner. Corrupt rows are never deleted automatically; representable header/state corruption is surfaced for quarantine, while an invalid ledger identity returns a permanent integrity failure because it cannot form an application `RecoveryCandidate`.
- No startup scan, runtime recovery, completion effects, finalizer, command gating, Tauri wiring, or frontend/IPC change was added. Phase 3B.3.3 was subsequently implemented and accepted after independent re-review; Phase 3B.3.4 is unblocked but remains not started pending separate approval.

#### Phase 3B.3.3 completion finalization ledger — accepted after independent re-review

- Added V008 `session_finalizations`: one finalization record per session with an immutable V007-fingerprint association and `pending`, `committed`, and terminal `quarantined` states. A composite foreign key prevents mismatched completion-intent fingerprints, and triggers reject identity updates, deletion, replacement, reopening committed records, and invalid timestamp/reason shapes.
- Added the application-owned `FinalizationLedger` business port and `racoon-data` SQLite adapter. Claims require the V006 record to be `finalization_pending` and a matching V007 intent header, then atomically create a pending V008 record using the existing `IMMEDIATE` transaction helper. Commit and quarantine mutate V008 only; quarantine revalidates the expected, V008, and current V007 fingerprints, with mismatch/missing/corrupt durable metadata taking precedence over any caller-supplied reason. No session-finalized transition or completion effect is performed.
- V008 is additive and forward-only, does not backfill finalizations for existing sessions/intents, and has no automatic downgrade path. Historical V001–V007 Refinery fixtures upgrade through the production runner to V008; corruption is surfaced as business corruption/quarantine outcomes without deleting records.
- No `SessionFinalizer` implementation, test/replay/statistic/lesson effect, startup recovery, command gating, Tauri wiring, or frontend/IPC change was added. Phase 3B.3.4 is unblocked but remains not started pending separate approval.

#### Phase 3B.3.4 restart-safe exactly-once finalizer — accepted after independent re-review

- Added an unwired `SqliteSessionFinalizer` implementation of the application-owned `SessionFinalizer` port. One `IMMEDIATE` transaction strictly validates the V006/V007/V008 durable inputs and matching immutable fingerprint; applies the existing test, replay, personal-best, daily-statistics, streak, daily-goal, and lesson effects from the V007 intent; then changes V008 to `committed` and V006 to `finalized`.
- The durable idempotency identity is `SessionId + CompletionIntentFingerprint`. A successful retry proves V006 `finalized`, V008 `committed`, matching fingerprints, the complete immutable test result, and the exact replay projection before returning `AlreadyFinalized`; conflicting or inconsistent durable records do not mutate effects. Daily-goal evaluation preserves the current zero and fractional-minute behavior, and the injected rollback seam is available only through racoon-data's non-default `test-support` feature.
- Terminal proof compares deterministic empty tags. The history `is_pb` flag remains a validated boolean but is not directly compared to V007 because it is a historical PB-derived effect whose relationship can legitimately change after later sessions finalize.
- The adapter remains unwired from normal completion commands. Phase 3B.3.5 now invokes it only from startup recovery after a metadata-only ledger scan and durable V008 claim; it adds no finalizer call to live completion, frontend recovery flow, active-session resume, or migration. Phase 3B.3.6 adds only a test-only crash campaign around the accepted adapter.

#### Phase 3B.3.5 startup recovery coordinator — accepted after independent re-review

- Added an application-owned coordinator over `SessionRecoveryLedger`, `FinalizationLedger`, and `SessionFinalizer`; it has no SQLite, Tauri, concrete repository, payload, replay, or connection dependency.
- Startup transitions atomically through `NotStarted → Recovering → Ready|Blocked`. A bounded retry policy retries only typed retryable port failures through an injected sleeper; scan/global permanent failures, retry exhaustion, and unresolved conflicts block readiness.
- The initial scan remains header-only and deterministic in accepted ledger order. Only candidates classified as eligible load the strict V007 payload, establish/verify a V008 claim, and invoke the finalizer. Running sessions become interrupted; invalid/missing/unsupported rows are quarantined without destructive repair; terminal rows are reported without reopening.
- Tauri startup composes the existing SQLite adapters before command handling. The shared readiness gate rejects session lifecycle, custom-text mutation, and settings-write commands until recovery is `Ready`; read-only reporting remains available. Normal live completion wiring remains unchanged.
- Reports are process-local, bounded, and contain only session ID, durable state, bounded action/reason, attempts, and aggregate counts. No V009, active-session resume, periodic worker, or recovery UI was added. Phase 3B.3.6 adds only a test-only process-crash campaign; normal completion wiring remains unchanged.

#### Phase 3B.3.6 process-crash recovery fault campaign — accepted after independent re-review

- Added `racoon-data`'s non-default `crash-test-support` feature, which extends the existing non-default `test-support` feature. It is not enabled by default or by racoon-app. The default data/app/workspace build exposes neither a crash checkpoint type nor a crash-capable finalizer constructor.
- The `process_crash_recovery` integration-test target uses a real independent child process rather than returned errors or panic unwinding. The child receives only a temporary SQLite path, a bounded scenario/checkpoint enum name, and a bounded marker path; after synchronizing the marker it calls `std::process::abort()`. No production executable, Tauri command, environment setting, or live-completion path can invoke this control.
- Parent tests reopen the same file after child death, take an immediate durable snapshot, run the accepted `StartupRecoveryCoordinator`, and take a final snapshot. The matrix covers committed `running`, V007 intent persistence, V006 finalization-pending, V008 pending-claim, every standard pre-commit effect/terminal checkpoint, one late lesson checkpoint, and the post-commit/before-caller-success ambiguity. A dedicated 1,000-ms `CompletionPolicySnapshot::time(0.01)` fixture performs the real `daily_goal_met: false → true` write before `AfterDailyGoalUpdate`; the pre-commit abort removes it, recovery reapplies it once, and a second reopen leaves the entire snapshot unchanged. Pre-commit cases prove that tests, replay, PB, daily statistics, streak, daily goal, lesson effects, V008 commit, and V006 finalization are absent or unchanged until recovery; valid states then converge exactly once. The post-commit case proves terminal evidence remains exactly once after restart.
- Normal CI runs the default-feature workspace suite. The non-default `test-support` rollback suite and `crash-test-support` default process-crash campaign are explicit final-acceptance commands; the ignored extended campaign performs eight ordinary pre-commit boundaries ten times, the dedicated real daily-goal mutation ten times, and the post-commit ambiguity twenty-five times: `8 × 10 + 1 × 10 + 25 = 115` child crashes. Run it with `cargo test -p racoon-data --features crash-test-support --test process_crash_recovery -- --ignored`.
- This is a process-crash recovery campaign: it models abrupt application termination, lost in-memory state, and SQLite/WAL reopening. It does not claim physical power removal, disk-controller cache loss, kernel panic, filesystem corruption, or storage-device failure coverage. No migration, V009, active-session resume, per-key journal, frontend recovery work, periodic recovery, destructive repair, or normal live-completion rewiring was added.
- Phase 3B.3 recovery architecture is accepted and complete. Phase 3B.3.7 final acceptance is complete. Later Phase 3B milestones remain not started and require separate approval.

#### Phase 3B.3.7 final acceptance — complete

- Final independent acceptance confirms the coherent V006–V008 recovery protocol, metadata-only candidate scan, immutable-intent authority, V008 claim convergence, one-transaction finalization, terminal proof, startup readiness gating, command mutation guards, row-local quarantine isolation, global-failure blocking, and process-crash convergence.
- Validation includes the file-backed mixed-candidate matrix, repeated same-file two-coordinator convergence, real SQLite lock/retry evidence, error-return rollback failpoints, the default child-process crash campaign, and the 115-child extended campaign. Crash support remains non-default and production-isolated.
- Phase 3B.3 recovery architecture accepted and complete. Phase 3B.3.7 final acceptance complete. Later Phase 3B milestones remain not started and require separate approval.

#### Expected files and changes

- `Cargo.toml`: add the `crates/application` workspace member and dependency wiring (completed in Phase 3B.1).
- New `crates/application/src/**`: the session kernel, start/input/abort/completion contracts, and ports are established in Phase 3B.1; broader use cases such as dashboard, settings, and export remain later milestones.
- `crates/app/src/commands.rs`: command registration/adaptation only.
- `crates/data/src/**`: repository and unit-of-work implementations, with no domain policy leakage.
- `crates/resources/src/**`: validated catalog ports and source-aware errors.
- `frontend/src/lib/api/**`, `frontend/src/lib/stores/**`, feature components: typed API client, session store, settings store, analytics store, and view-local state.
- `ARCHITECTURE.md` and ADRs: document dependency direction, ownership, and rejected alternatives.

#### Risks and mitigations

- **Risk:** A mechanical move preserves bad abstractions. **Mitigation:** move one use case at a time, delete duplicate logic, and require contract tests.
- **Risk:** New crate boundaries slow development. **Mitigation:** keep the application layer small and avoid abstracting one-off pure functions.
- **Risk:** Generated contracts become a build burden. **Mitigation:** choose one tool, commit generated output where appropriate, and make CI verify freshness.

#### Validation checklist

- No Tauri command contains SQL or multi-repository business orchestration.
- Domain/core tests run without Tauri or SQLite.
- IPC responses and errors are typed and versioned.
- Frontend has explicit state ownership and handles loading/error/retry paths.
- Contract, integration, and end-to-end smoke tests cover the critical journeys.

#### Rollback

Refactor behind compatible application service interfaces. Keep adapters temporarily only where a complete move would increase risk, but track every compatibility shim with an owner and removal issue. Revert a use-case migration independently rather than reverting all architecture work.

#### Gate G3

Architecture review confirms dependency direction, command thinness, contract ownership, and test coverage of the migrated use cases.

---

### Phase 4 — Database integrity, migration safety, and data lifecycle

**Priority:** P0/P1  
**Complexity:** XL  
**Dependencies:** G2 and the relevant Phase 3B persistence ports  
**Purpose:** Prevent data loss, partial saves, incorrect aggregates, and unrecoverable migrations.

#### Objectives

- Make SQLite configuration and transaction boundaries explicit.
- Enforce foreign keys and integrity constraints on every connection.
- Add indexes based on measured query patterns and long-history tests.
- Replace rollback-by-git with backup/restore and forward-compatible migration operations.
- Define export, deletion, retention, and restore semantics.

#### Expected files and changes

- `crates/data/src/db.rs` and repository modules: connection initialization, pragmas, lock ownership, transaction/unit-of-work API, error mapping, and backup hooks.
- `crates/data/migrations/**`: forward migrations only, compatibility columns/tables where needed, data backfills, and migration metadata.
- `crates/data/src/repository/**`: atomic completion workflow, complete aggregate queries, pagination/cursor semantics, and explicit ordering.
- `crates/data/tests/**`: migration upgrade tests from every supported version, rollback/restore tests on copies, FK/cascade tests, corruption/error tests, and long-history metric tests.
- `crates/app`/application export/import code: versioned schema, bounded input, validation-before-write, dry-run, conflict policy, and atomic replacement.
- `docs/`: data model, backup, restore, deletion, and migration runbook.

#### Risks and mitigations

- **Risk:** Existing users have databases with undocumented states. **Mitigation:** fixture databases from released versions, preflight checks, backup before migration, and a recovery path.
- **Risk:** A transaction holds the database lock too long. **Mitigation:** keep work outside the transaction, measure duration, and index queries.
- **Risk:** Import corrupts or overwrites valuable data. **Mitigation:** validate into a temporary database, require explicit replacement/merge choice, and retain an automatic backup.

#### Validation checklist

- `PRAGMA foreign_keys` is enabled and tested on every connection.
- Completion-related rows are all-or-nothing and retry-safe.
- Migration tests cover every supported release and a failed migration leaves a restorable backup.
- Metrics are correct beyond the old record caps.
- Export/import schemas are versioned and validated before mutation.
- Backup, restore, delete, and privacy documentation matches behavior.

#### Rollback

Before a migration, create and verify a backup. On failure, restore the backup or ship a forward fix; never ask users to check out an older application and expect that to undo a schema change. Keep schema changes separately commit-able from query changes where possible.

#### Gate G4

A release candidate must pass migration matrix, backup/restore, transaction, FK, and long-history tests with evidence retained as CI artifacts.

---

### Phase 5 — Security, privacy, and least-privilege hardening

**Priority:** P0/P1  
**Complexity:** L  
**Dependencies:** G3 and G4  
**Purpose:** Make the desktop trust boundary explicit and safe before expanding import, themes, or extension capabilities.

#### Objectives

- Minimize Tauri capabilities and registered command exposure.
- Validate and bound all IPC, import, filesystem, and content inputs.
- Remove unnecessary shell/plugin surface.
- Establish redacted logging, privacy controls, and a threat model.
- Ensure user-provided content cannot become executable code or unsafe CSS.

#### Expected files and changes

- `crates/app/capabilities/*.json`, `crates/app/tauri.conf.json`, `crates/app/build.rs`: least-privilege capabilities, command allowlists, window/webview restrictions, and documented exceptions.
- `frontend/package.json`/lockfile: remove unused shell plugin or add it only with a reviewed use case and capability.
- `crates/app/src/**`, application/data boundaries: input limits, path handling, safe export/import, redacted errors/logs, and controlled file dialogs.
- `resources/themes/**` and theme loader: structured tokens, sanitization/allowlist if any CSS remains, and no dynamic executable behavior.
- `docs/security/THREAT_MODEL.md`, `SECURITY.md`, privacy/data handling documentation.

Tauri capabilities are the correct control point for narrowing frontend access, but they do not make unsafe Rust logic or unsafe dependencies safe by themselves; the Rust command implementation and supply chain still require review. The official Tauri capability and permission model should be treated as a design input, not as a substitute for application authorization.

#### Risks and mitigations

- **Risk:** Removing permissions breaks legitimate platform behavior. **Mitigation:** add permissions one use case at a time, with tests and justification.
- **Risk:** Sanitization is incomplete for custom CSS or imports. **Mitigation:** prefer structured data; if raw CSS is retained, use a constrained grammar/allowlist and test hostile inputs.
- **Risk:** Diagnostic logs expose typed text or quotes. **Mitigation:** redact content and make verbose logging opt-in with retention limits.

#### Validation checklist

- Capability manifest exposes only required commands and resources.
- Security tests cover malformed IPC, oversized input, path traversal, malicious import, CSS injection, error leakage, and repeated requests.
- Threat model lists assets, trust boundaries, abuse cases, mitigations, and residual risk.
- Data deletion and export behavior is demonstrably complete.
- No known high-severity dependency advisory is accepted without a documented decision; the GLib advisory line must be upgraded, isolated, or explicitly supported with rationale and a remediation issue. The RustSec advisory describes the affected `glib` range and patched line; it is a release input, not a reason to ignore the rest of the graph.

#### Rollback

Use capability/configuration changes that can be reverted independently. If a permission is required unexpectedly, fail closed and restore only the minimum previous permission while the threat model is updated; do not re-enable broad access.

#### Gate G5

Security review, threat-model sign-off, dependency policy pass, and privacy verification are required before release automation is promoted.

---

### Phase 6 — CI/CD, packaging, reproducible releases, and smoke validation

**Priority:** P0  
**Complexity:** XL  
**Dependencies:** G5  
**Purpose:** Produce trustworthy, signed, traceable artifacts for every supported platform.

#### Objectives

- Separate pull-request validation, nightly/platform validation, and release promotion.
- Build from a clean checkout with pinned toolchains and deterministic dependency installation.
- Generate configured installers instead of raw binaries with unsupported claims.
- Attach checksums, SBOM, provenance/attestation, release notes, and signatures where platform distribution supports them.
- Run install/launch/smoke tests on each supported artifact.

#### Expected files and changes

- `.github/workflows/ci.yml`: matrix for format, clippy, unit/integration, frontend check/build, license/audit/SBOM, and package smoke jobs; least-privilege `permissions`; pinned action versions or reviewed update policy.
- New workflow files under `.github/workflows/`: release candidate, artifact build, signing/promotion, and post-release verification, kept separate from PR CI.
- `rust-toolchain.toml`, Node/package-manager version policy, lockfiles: pin supported toolchains and use immutable installs.
- `crates/app/tauri.conf.json`: correct bundle configuration and platform metadata.
- `PKGBUILD`: reproducible source/build steps, Apache metadata, pinned/checksummed sources, and a tested packaging command.
- `com.racoon.typper.json`: source build rather than copying an undeclared prebuilt binary; narrow Flatpak permissions; pinned runtime/tooling.
- `build-appimage.sh` or a replacement packaging workflow: remove stale defaults, correct path variables, pin and verify appimagetool, and make failures explicit.
- `RELEASE_CHECKLIST.md`, `RELEASE_AUDIT.md`, `CHANGELOG.md`: artifact evidence, versioning, migrations, rollback, and support notes.

#### Recommended release flow

```text
Pull request
  → deterministic checks and security/license gates
  → merge to protected branch
  → signed/tagged release candidate
  → build per-platform artifacts
  → checksum/SBOM/provenance/signing
  → install/launch/smoke verification
  → maintainer approval
  → publish release and notes
  → post-release health/support review
```

#### Risks and mitigations

- **Risk:** Signing secrets or permissions are overexposed. **Mitigation:** least-privilege workflow permissions, protected environments, short-lived/OIDC credentials where supported, and no secrets in build logs.
- **Risk:** A package builds but does not launch on a clean machine. **Mitigation:** VM/container smoke tests and artifact inspection, not only compilation.
- **Risk:** “Reproducible” differs across native toolchains. **Mitigation:** document the achievable reproducibility level, pin inputs, capture build metadata, and compare hashes where supported.
- **Risk:** Third-party GitHub actions change behavior. **Mitigation:** pin reviewed SHAs or enforce a controlled update process.

#### Validation checklist

- PR checks pass without network-dependent hidden state after dependencies are cached or fetched deterministically.
- Linux, Windows, and any claimed additional platform artifacts are built by the same documented release process.
- Each artifact installs, launches, reaches the first usable screen, starts a short test, completes it, persists it, and exits cleanly.
- Checksums, SBOM, source revision, version, and license notices are attached.
- Release is draft/promoted only after smoke checks and maintainer approval.
- Rollback is a release-channel action plus database forward-fix/restore guidance, not a promise that an old binary reverses a migration.

#### Rollback

Keep the previous known-good release available. If an artifact is defective, unpublish or mark the release as withdrawn, stop promotion, and publish a fixed version. For user data, use the Phase 4 backup/restore/forward-fix procedure.

#### Gate G6

No “production-ready” label or public release is allowed until artifacts, checksums, SBOM, signatures/attestations, install smoke tests, and release notes all agree.

---

### Phase 7 — Public repository and documentation modernization

**Priority:** P1  
**Complexity:** L  
**Dependencies:** G6, with content drafting allowed earlier  
**Purpose:** Make the repository understandable, trustworthy, and maintainable for users and contributors.

#### Objectives

- Make the GitHub landing page answer what, why, install, build, contribute, status, license, and download questions quickly.
- Remove stale claims and broken documentation links.
- Establish durable documentation ownership; the wiki is supplemental, not the source of truth.
- Make issues, pull requests, security reports, and releases consistent.

#### Expected files and changes

- `README.md`: concise product statement, verified feature list, screenshots with provenance, badges that reflect real workflows, install/download paths, build/test commands, data/privacy notes, architecture link, status, roadmap link, and license.
- `INSTALL.md`: supported platforms, prerequisites, developer setup, package verification, troubleshooting, and clean build commands.
- `CONTRIBUTING.md`: branch/commit/PR policy, test matrix, architecture boundaries, generated files, asset licensing, and local release checks.
- `ARCHITECTURE.md`: target boundaries and important decisions.
- `docs/` structure, for example:

  ```text
  docs/
    development/setup.md
    development/testing.md
    development/debugging.md
    product/supported-features.md
    release/process.md
    release/support-matrix.md
    security/threat-model.md
    security/reporting.md
    legal/third-party-notices.md
    data/backup-restore.md
  ```

- `.github/ISSUE_TEMPLATE/bug.yml`, feature template, security reporting instructions, and `PULL_REQUEST_TEMPLATE.md`.
- Optional `CODEOWNERS`, `CODE_OF_CONDUCT.md`, and Discussions categories once maintainer ownership exists.
- `CHANGELOG.md`/release notes: link changes to versions and verified artifacts; do not list planned work as shipped.

#### Risks and mitigations

- **Risk:** Documentation promises unsupported platforms/features. **Mitigation:** generate the support table from release evidence and require review for claims.
- **Risk:** Screenshots include uncleared content or old UI. **Mitigation:** capture only from a cleared release candidate and record source/version.
- **Risk:** Too many documents diverge. **Mitigation:** keep `ROADMAP.md` authoritative for sequencing and link other docs to it.

#### Validation checklist

- All README links resolve; no reference to missing `ARCHITECTURE.md` remains.
- A new contributor can install dependencies, run checks, and start the app from the documented instructions.
- A user can find a verified download and license within one or two clicks.
- Security reports have a private contact/workflow distinct from public issues.
- Release notes match actual artifact contents and support claims.

#### Rollback

Documentation commits are independently revertible. Preserve the new structure and correct inaccurate pages forward rather than restoring stale release claims.

#### Gate G7

Maintainer review confirms public claims, links, screenshots, contribution path, license notices, and support matrix.

---

### Phase 8 — Product features after foundation approval

**Priority:** P2  
**Complexity:** XL across the complete feature set  
**Dependencies:** G4, G5, G6, and G7  
**Purpose:** Deliver user-facing roadmap value on top of reliable primitives.

Features should be delivered as vertical slices, each with a product brief, design notes, IPC contract, persistence migration if needed, tests, documentation, and release notes.

#### Recommended order

1. **Daily goals and weekly summaries** — uses existing session aggregates; first define metric semantics and avoid capped-history errors.
2. **Onboarding and achievement gallery** — improves activation; must not create noisy or irreversible notifications.
3. **Versioned profile export/import** — implement validation, preview, backup, merge/replace policy, and privacy controls before claiming “full restore.”
4. **Theme catalog and safe customization** — prefer structured tokens and cleared assets; defer arbitrary CSS until the security gate approves it.
5. **Language expansion** — add one language as a complete vertical slice, including resources, UI strings, validation, quotes/courses/word lists, tests, and fallback policy; then repeat.
6. **Keyboard layouts** — model layout as data, validate key mappings, update weak-key and visualization logic, and test non-QWERTY behavior.
7. **Course editor and interchange** — use a versioned content schema and sandboxed import pipeline; Anki/CSV must have explicit mapping and validation rules.

#### Per-feature acceptance criteria

- Product hypothesis and success metric documented.
- Domain/application behavior tested independently of Tauri.
- Persistence migration and rollback/restore story documented if data changes.
- IPC contract and frontend states cover success, loading, validation, empty, and failure cases.
- Security/privacy review for imported, executable, or user-authored content.
- Performance baseline where the feature affects typing latency, startup, or large histories.
- Documentation and changelog updated only after a release artifact demonstrates it.

#### Rollback

Use a feature flag or schema-compatible inactive path for user-visible experiments. If a feature changes persisted data, use a versioned migration and an export/restore path; do not rely on reverting the binary.

#### Gate G8

Each feature has its own release decision. Completing Phase 8 does not automatically authorize plugins or cloud features.

---

### Phase 9 — Deferred and exploratory capabilities

**Priority:** P3  
**Complexity:** L to XXL, to be re-estimated per product brief  
**Dependencies:** all prior gates plus validated demand  
**Purpose:** Prevent high-risk scope from destabilizing the local desktop foundation.

#### Candidates

- WASM extension model with explicit capabilities and resource quotas;
- LAN multiplayer with an explicit privacy/network consent model;
- cloud sync with conflict resolution, encryption, account/recovery, and a data-processing policy;
- mobile companion with a separately supported product boundary;
- adaptive/ML models with local-only data and measurable benefit;
- voice narration and eye-tracking integrations with platform permissions and accessibility review.

#### Native plugin decision

The legacy v1.2 native plugin proposal is removed from the committed near-term plan. Native plugins would share a dangerous process boundary and greatly expand supply-chain, update, permission, crash, and data-exfiltration risk. Reconsider only after a separate threat model, capability design, versioned ABI, signing/trust policy, sandbox strategy, uninstall/rollback behavior, and maintainer capacity plan.

## 9. Release gates and definition of done

### Release gates

| Gate | Required evidence |
|---|---|
| G0 Baseline | Clean documented build path; version/support matrix; no lost worktree changes |
| G1 Legal | Asset/content provenance; license policy pass; notices; no uncleared copyleft material |
| G2 Runtime | Closed in-memory state machine, monotonic timing, bounded validation, atomic completion/retry tests, and startup/error tests; not durable crash recovery |
| G3 Architecture | Thin commands; application/domain/data boundaries; typed IPC contract |
| G4 Data | Migration matrix; backup/restore; FK/transaction/idempotency; long-history metrics |
| G5 Security | Threat model; least-privilege capabilities; hostile-input tests; privacy review; dependency decisions |
| G6 Release | Clean platform artifacts; install/launch smoke tests; checksums; SBOM; signing/attestation; release approval |
| G7 Repository | Accurate README/docs; contribution/security workflows; support matrix and changelog |
| G8 Feature | Feature-specific product, engineering, security, test, documentation, and release evidence |

### Definition of done for any change

- Scope and acceptance criteria are written.
- Dependencies and migration impact are identified.
- Tests are added at the lowest useful layer and at the relevant integration boundary.
- Errors, retries, cancellation, and shutdown behavior are considered.
- Logs and metrics are useful without exposing user content.
- License/provenance is recorded for new dependencies and assets.
- Documentation and changelog claims match the implementation.
- CI is green, and a reviewer validates the intended behavior.
- The change can be reverted or recovered without destructive user-data assumptions.

## 10. Risk register

| ID | Risk | Impact | Mitigation/trigger |
|---|---|---|---|
| R-001 | GPL or unclear third-party assets ship accidentally | Legal exposure; forced release withdrawal | Phase 1 manifest and fail-closed CI license gate |
| R-002 | Tauri path/packaging mismatch | No installable release | Clean-checkout package smoke in G0/G6 |
| R-003 | Partial or duplicated completion persistence | Lost/duplicated history, streaks, PBs | Phase 2 transaction plus in-process retry tests; durable idempotency and backup/restore remain Trusted Core/Phase 4 work |
| R-004 | Frontend-provided score is trusted | Tampered or inconsistent records | Backend authoritative calculation and contract tests |
| R-005 | Migration cannot be reversed | User data loss/support crisis | Backup, fixture matrix, restore and forward-fix runbook |
| R-006 | Broad Tauri capability or shell surface | Local code execution/data exfiltration risk | Least-privilege capabilities and remove unused plugin |
| R-007 | User CSS/import becomes executable or unsafe | XSS-like desktop boundary abuse | Structured theme tokens and hostile-input tests |
| R-008 | Cross-platform path/startup assumptions | Windows or restricted environment failure | Tauri app paths, platform matrix, graceful startup errors |
| R-009 | GTK/GLib dependency maintenance/security debt | Upgrade or platform support risk | Dependency audit, upgrade/containment ADR, supported-platform policy |
| R-010 | Analytics record caps misstate progress | Incorrect product decisions and user feedback | Complete aggregate queries, indexes, long-history fixtures |
| R-011 | Large refactor changes behavior invisibly | Regression and review overload | Vertical migrations, contract tests, reversible commits |
| R-012 | Release secrets or actions are compromised | Supply-chain or artifact compromise | Pinned actions, minimal permissions, protected signing environment |
| R-013 | Feature scope outruns maintainer capacity | Stalled project and growing debt | Gate features behind validated product briefs and capacity estimates |
| R-014 | Content/source provenance cannot be established | Assets must be removed late | Early attestation deadline; original replacements as default |

## 11. Verification strategy

### Fast checks on every change

- Rust formatting and clippy with warnings denied.
- Rust unit tests and focused property tests.
- Frontend type checking, linting if adopted, and production build.
- Contract tests for IPC DTOs and error envelopes.
- License/notice policy for changed dependencies/assets.
- `git diff --check` and generated-file freshness checks.

### Integration checks

- SQLite repository tests against temporary databases.
- Migration upgrade tests from every supported schema version.
- Transaction failure and retry tests.
- Filesystem path and atomic settings-write tests.
- Resource catalog validation and malformed-input tests.

### End-to-end smoke journeys

At minimum, on every release candidate:

1. Install or unpack the artifact.
2. Launch on a clean profile.
3. Start a short test.
4. Type valid, invalid, Unicode, and boundary input.
5. Complete once and retry the completion request.
6. Restart and verify persisted history/settings.
7. Export data and validate the file.
8. Exercise a migration fixture if the release changes schema.
9. Close cleanly and verify no unexpected process remains.

### Performance and reliability targets to establish

Do not invent targets before measurement. Phase 2 did not establish performance baselines; capture these before setting Phase 4/6 regression thresholds:

- first launch and warm launch;
- time-to-first-keystroke;
- per-keystroke processing latency at representative text lengths;
- history/dashboard query latency at 10,000+ sessions;
- export/import duration and peak memory;
- database file growth and migration duration.

After baselines exist, set regression thresholds and run them in CI or scheduled benchmarks.

## 12. GitHub repository design

The public repository should present the project in this order:

1. What Racoon Typper is and who it is for.
2. A current screenshot or short demo from a cleared release.
3. Verified feature highlights.
4. Download/install links and supported platform table.
5. Developer quick start and test commands.
6. Data/privacy and local-storage explanation.
7. Architecture and contribution links.
8. Current status, roadmap, license, security reporting, and release history.

Badges should be limited to reliable facts such as CI, latest release, and license. Do not add badges for coverage, platforms, or package formats until the workflow proves them.

Use GitHub Issues for actionable bugs/features, Discussions for design and user questions once maintainers can moderate them, and the wiki only for supplemental material. The repository files remain authoritative.

## 13. Changelog relative to the legacy roadmaps

This roadmap makes the following changes:

1. Replaces `ROADMAP_v1.1.md` and `ROADMAP_v1.2.md` as the intended source of truth after approval; legacy files are retained for historical comparison until a separate cleanup approval.
2. Reorders work so release blockers, licensing, runtime foundation, architecture, database integrity, security, CI/CD, and documentation precede feature expansion.
3. Separates “implemented,” “present in the working tree,” “planned,” and “release verified.”
4. Removes unsupported completion claims for session recovery, full profile export, extended analytics, and other features until evidence exists.
5. Removes native plugins from the near-term commitment and defers them behind a separate threat model and capability design.
6. Adds explicit acceptance criteria, dependencies, complexity, validation checklists, rollback strategies, and release gates.
7. Adds a complete licensing/provenance workstream for themes, icons, content, scripts, fonts, dependencies, and generated assets.
8. Treats the pre-existing imported GPL theme catalog as a distribution blocker and removes it rather than relicensing it.
9. Adds architectural boundaries between Tauri commands, application services, domain/core, data, resources, and frontend state.
10. Defines and partially completes in-process exactly-once completion, transactionality, authoritative backend scoring, monotonic timing, and typed-error requirements; Phase 3A now closes the stable durable ID gap while crash recovery remains deferred.
11. Adds migration backup/restore and forward-fix procedures instead of relying on code checkout rollback.
12. Adds least-privilege Tauri capabilities, CSP reduction, import/theme threat modeling, privacy controls, and removal of unused shell/plugin surface.
13. Adds reproducible packaging, artifact checksums, SBOM, signing/attestation, platform smoke tests, and release promotion controls.
14. Adds a public repository modernization plan, support matrix, security workflow, contribution templates, and documentation structure.
15. Establishes a definition of done and a risk register so future features cannot bypass the foundation gates.

## 14. Future recommendations after the roadmap

After the foundation and first product slices are stable, consider:

- crash diagnostics or opt-in telemetry with explicit consent and strict redaction;
- accessibility audits for keyboard navigation, contrast, screen readers, reduced motion, and non-QWERTY layouts;
- signed update channels and a documented vulnerability response process;
- benchmark dashboards and performance budgets for startup and keystroke latency;
- reproducible-build comparisons and independent release verification;
- localization workflow with translation ownership, pluralization, and pseudo-localization tests;
- data portability across future versions with schema compatibility guarantees;
- community governance and maintainer succession for a sustainable open-source project;
- a narrowly scoped, sandboxed WASM extension prototype only if user demand is demonstrated;
- cloud/mobile/multiplayer only after a separate product, privacy, operational, and cost model proves they are worth the additional service surface.

## 15. Approval protocol

Before implementation begins, the project owner should approve:

- this roadmap as the source of truth;
- the supported platform and package matrix;
- the legal interpretation and asset replacement policy;
- the version source of truth;
- whether a new `application` crate is preferred over an in-place module boundary;
- the exact scope of Phase 0.

After approval, work proceeds one phase at a time. The implementer must report changed files, tests, release evidence, unresolved risks, and commit identity at the end of each phase, then stop for explicit approval.
