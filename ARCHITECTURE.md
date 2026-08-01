# Racoon Typper Architecture

**Status:** Phase 2 Foundation, Phase 3A durable session identity, Phase 3B.1–3B.2 session architecture, Phase 3B.3.1 recovery contracts, the focused Phase 3B.3.1a stored-header amendment, Phase 3B.3.2, Phase 3B.3.3, and Phase 3B.3.4 are accepted after independent review; Phase 3B.3.5 and Phase 3B.3.6 are accepted after independent re-review; Phase 3B.3 recovery architecture accepted and complete; Phase 3B.3.7 final acceptance complete; later Phase 3B milestones remain not started and require separate approval  
**Last reviewed:** 2026-07-17  
**Scope:** This document describes the verified baseline plus the approved Phase 2 foundation, Phase 3A identity, Phase 3B.1–3B.2 kernel/provider changes, Phase 3B.3.1/3B.3.1a recovery contracts, the accepted Phase 3B.3.2 ledger schema/adapter, the accepted Phase 3B.3.3 finalization ledger, the accepted Phase 3B.3.4 finalizer, the accepted Phase 3B.3.5 startup coordinator, and the accepted test-only Phase 3B.3.6 process-crash campaign. It does not claim physical power-loss durability, remaining Phase 3B use-case migration, or later release/security phases are complete.

## Product boundary

Racoon Typper is a local-first desktop typing trainer. The application runs as a Tauri desktop process with a Rust backend, a Svelte frontend, embedded learning resources, a local SQLite database, and a TOML settings file. Network services, accounts, and cloud synchronization are not part of the current baseline.

The application is intentionally a modular monolith. The current product does not justify microservices, a server database, or a native plugin ABI.

## Verified repository structure

```text
.
├── Cargo.toml                 # workspace and canonical release version
├── crates/
│   ├── domain/                # shared value/data types and contract models
│   ├── core/                  # typing engine, modes, lessons, scoring, analytics
│   ├── data/                  # SQLite connection, migrations, repositories
│   ├── resources/             # embedded course, word, and quote loading
│   ├── application/           # infrastructure-free session kernel and ports
│   └── app/                   # Tauri binary, command adapters, session service, state
├── frontend/
│   ├── src/                   # Svelte UI, IPC wrappers, frontend types
│   └── package.json           # frontend tooling and canonical command entrypoints
├── resources/                 # runtime content and theme resources
├── scripts/                   # repository-level verification and Tauri wrappers
├── ARCHITECTURE.md
├── SUPPORT_MATRIX.md
├── ROADMAP.md
└── LICENSE
```

Phase 2 groups Tauri adapters under `crates/app/src/commands/`, keeps `commands.rs` as a small registration-facing facade, and extracts the cohesive completion workflow into the private `session_service.rs` module. Phase 3A adds the durable `SessionId` value type. Phase 3B.1 adds the standalone infrastructure-free `racoon-application` crate with session request/completion contracts, a transport-neutral `SessionKernel`, and business-oriented ports. Phase 3B.2 adds wall-clock and random-value provider ports while retaining identity and monotonic-clock injection. Phase 3B.3.1 adds pure durable-state vocabulary, versioned immutable completion-intent contracts, canonical SHA-256 fingerprinting, recovery classification, recovery ports, and a readiness-state contract. The Tauri service and resource loader continue to supply production adapters to the existing ports; content-specific preparation and all recovery execution remain outside this milestone.

## Runtime flow

```text
Svelte components
    │ typed invoke() requests; render backend-confirmed session state
    ▼
Tauri command adapters in crates/app/src/commands
    ├── private session service
    │     resource/SQLite adapters for the application kernel
    ├── racoon-application::SessionKernel
    │     lifecycle, input, completion snapshot, and port orchestration
    ├── resource loaders for words, quotes, courses, and themes
    └── data repositories for SQLite persistence
             │
             ├── one IMMEDIATE transaction for a completed test:
             │   history + replay + PB + daily stats + streak + goal + lesson
             ├── data.db (SQLite, WAL, foreign keys, busy timeout)
             └── settings.toml (serialized, atomic file replacement)
```

The dependency direction for the new kernel is deliberately one-way:

```text
racoon-application (session contracts and ports)
        │
        ├── racoon-core
        └── racoon-domain

racoon-resources (resource adapter) ────────────► racoon-application
racoon-app (Tauri/SQLite adapters) ─────────────► racoon-application
```

`racoon-application` has no Tauri, SQLite, filesystem, or embedded-resource
dependency. Its ports describe session capabilities rather than repository
queries. The session kernel depends on four provider capabilities: backend
identity, process-monotonic time, UTC wall time, and random values used by
resource selection. Production implementations remain adapters: UUIDv7 and
the two `AppState` clocks are supplied by `racoon-app`, while resource loaders
retain selection policy and accept the random-value port. Fixed test providers
are confined to application/resource tests; no service locator or production
deterministic implementation is introduced. Recovery contracts are likewise
application-owned and contain no SQLite or startup dependency; later ledger
adapters must implement them from outside this crate.

Phase 3B.2 runtime flow is therefore:

```text
SessionKernel
    │ SessionIdSource / SessionClock / SessionWallClock / SessionRandomSource
    ▼
application provider ports
    ▼
Tauri and resource adapters
    ▼
UUIDv7, process clock, UTC clock, and existing runtime-value behavior
```

Phase 3B.3.1 adds a pure recovery contract layer:

```text
durable candidate DTOs and immutable completion intents
    │ state classification / SHA-256 fingerprint / readiness transitions
    ▼
racoon-application recovery contracts and business ports
    ▼
racoon-data durable-ledger adapters (Phases 3B.3.2–3B.3.3)
```

The durable state model is separate from the in-memory `SessionState`. Running
sessions classify as interrupted after a future restart; valid pending intents
classify as eligible for finalization; terminal states are no-ops; malformed,
unsupported, conflicting, or inconsistent records classify for quarantine.
Phase 3B.3.5 adds an application-owned startup coordinator that first lists
metadata-only candidates in ledger order, loads V007 only for eligible pending
records, claims V008 through the accepted port, and invokes the accepted
`SessionFinalizer`. Its shared readiness gate remains `NotStarted`,
`Recovering`, `Ready`, or `Blocked`; session lifecycle, custom-text mutation,
and settings-write commands reject work until it is `Ready`. Row-local records
that are durably quarantined do not prevent unrelated recovery, while scan,
retry-exhaustion, permanent, and unresolved conflict failures block readiness.
There is no active-session resume, periodic scan, or startup recovery UI.

The frontend is a presentation/client layer. It no longer sends lesson WPM or accuracy for persistence: final scores, lesson completion, and all completion-side effects are derived from the Rust engine and committed before the successful IPC response is returned. It renders the backend `session_state` rather than treating a local completion as durable.

### Lifecycle transition and retry behavior

```text
idle ────────┐
persisted ───┴─ start ──► running ── complete ──► awaiting_persistence
                                                    │
                                                    ├─ claim ─► persisting ── commit ─► persisted
                                                    │                  │
                                                    │                  └─ failure ─► awaiting_persistence
running ── abort ──► idle
persisted ── abort ─► idle

start is rejected from running, awaiting_persistence, and persisting.
abort and reset are rejected from awaiting_persistence and persisting.
```

`CoreEngine` emits a final result once, then holds that immutable result in `awaiting_persistence`. `SessionKernel` claims it as `persisting` before releasing the engine lock, so concurrent key requests cannot initiate a second completion transaction. A successful transaction moves the session to `persisted`; a failed transaction returns it to `awaiting_persistence` with the same result available for a safe retry. The core itself now refuses direct replacement outside `idle`/`persisted`, and reset only operates while `running`, closing the former in-memory escape hatches. Custom-text use counts and lesson-progress initialization occur only after this lifecycle gate passes. Every accepted session receives one backend-generated UUIDv7 `SessionId`; that identity is retained through completion/retry and written with the test record. `process_key` and `abort_session` accept the returned value only as a correlation token and reject a mismatch. This is an in-process exactly-once boundary; crash recovery and restart-safe idempotency remain Phase 3B work.

### Durable session identity (Phase 3A)

`racoon_domain::SessionId` is an immutable value object backed by a canonical UUIDv7 string. UUIDv7 was selected over ULID because it is an IETF-standard UUID variant with broad database/cloud interoperability, while retaining millisecond time ordering, random uniqueness, and timestamp bits useful for debugging. The canonical 36-character form is straightforward to index, serialize over Tauri, and use as a future synchronization key. Generation happens only in `session_service::start_engine_session`; the frontend receives the value after the backend accepts the start and cannot select or replace it.

The integer `tests.id` remains the local relational key for personal-best and replay foreign keys. Migration `V005__session_identity.sql` adds `tests.session_id`, backfills existing rows as deterministic `legacy-test-<id>` values, creates a unique index, and installs insert/immutability guards. New records always persist UUIDv7 values. This additive migration preserves existing history and replay rows without rewriting their foreign keys. A completed test's identity survives application restart through the history row; an in-progress session is still intentionally in-memory until the Phase 3B crash-recovery work is approved.

### Recovery contracts (Phase 3B.3.1)

`racoon-application::DurableSessionState` is intentionally distinct from the
in-memory engine lifecycle. It models `running`, `awaiting_persistence`,
`finalization_pending`, `finalized`, `aborted`, `interrupted`, and
`quarantined`. Pure classification never resumes an active session. It marks
an ordinary durable running candidate interrupted, sends a valid pending
completion intent to a future finalizer, treats terminal states as no-ops, and
quarantines malformed, unsupported, conflicting, or inconsistent records.

The durable transition policy is explicit and exhaustive. Session creation is
`none → running`; completion capture advances through
`running → awaiting_persistence → finalization_pending`; finalization advances
to `finalized`; explicit aborts and recovery quarantine/interruption transitions
are allowed only from non-terminal states. Repeating the current state is
idempotent. Terminal states (`finalized`, `aborted`, `interrupted`, and
`quarantined`) cannot transition to any other state.

`CompletionIntent` is an immutable version-1 contract containing the session
identity, completion timestamp, final statistics, mode configuration, language,
text length, replay frames, lesson identity, and completion-affecting daily-goal
policy inputs. The policy snapshot uses explicit tagged `time`/`wpm`/`accuracy`
targets, so the legacy time-goal numeric setting is not stored under a WPM
name. Its canonical JSON uses independent canonicalization version 1, sorts
objects recursively, preserves array order, normalizes equivalent finite
numbers (`1`, `1.0`, and `1e0` become `1`; signed zero becomes `0`), and is
hashed with lowercase SHA-256. Stored envelopes must be exact canonical bytes,
use known schema fields, pass strict typed re-canonicalization (including
`FinalStats`, `ReplayFrame`, and policy objects), match their fingerprint, and
reject non-finite numbers. Opaque `mode_config` remains a `serde_json::Value`
because this application layer owns no mode-specific schema; it is preserved
and included in the fingerprint. The application enforces an 8 MiB completion
envelope bound and redacts payload contents from debug/error output.

Recovery candidate listings are metadata-only: session identity, state, intent
presence/version, and fingerprint are sufficient for deterministic startup
classification. The focused Phase 3B.3.1a amendment adds an application-owned
stored-header factory for untrusted header columns. It validates the strict
session identity, signed version representation, lowercase fingerprint form,
and missing/invalid field combinations without loading payload bytes; it does
not prove that the payload is valid. Full completion intents are loaded
separately through `CompletionIntent::from_stored_payload`, which performs
strict payload/schema/canonicalization/fingerprint validation, and must match
candidate metadata before finalization. This amendment exists solely to let a
future ledger adapter preserve the metadata-only boundary. Phase 3B.3.2 now
implements that adapter with V006/V007 additive SQLite storage, while
retaining the same two-stage boundary: candidate scans join only intent
headers; explicit loading reads canonical bytes and invokes
`CompletionIntent::from_stored_payload`. The ledger port explicitly
claims `AwaitingPersistence → FinalizationPending`; identical retries report
`AlreadyPending`, differing fingerprints report a business conflict, missing
sessions report `NotFound`, and terminal records never reopen. The finalizer
port returns only business outcomes (`NewlyFinalized`, `AlreadyFinalized`,
`NotFound`, conflict, and quarantine) while retryable/permanent failures remain
application failure classes; it exposes no local `TestId` or repository
receipt. V006–V008 adapters write durable recovery/finalization metadata.
Phase 3B.3.4's separate, unwired atomic finalizer strictly validates the full
V007 payload and matching V007/V008 fingerprints, writes the existing test,
replay, personal-best, daily-statistics, streak, daily-goal, and
lesson-completion effects, then marks V008 `committed` and V006 `finalized` in
the same `IMMEDIATE` transaction. Its durable idempotency identity is
`SessionId + CompletionIntentFingerprint`. Its terminal retry proof validates
the complete immutable test payload and projected replay sequence; mismatches
remain quarantined without rewriting terminal data. Daily-goal comparison
preserves zero and fractional-minute semantics, and rollback failpoints are
available only through racoon-data's non-default test-support feature. Phase
3B.3.5 composes that finalizer only during startup recovery; normal live
completion remains on its existing path. It performs no completion effects of
its own, and does not add active-session resume or a periodic worker.

Phase 3B.3.6 adds a test-only file-backed process-crash campaign behind
racoon-data's non-default `crash-test-support` feature (which extends
`test-support`). Its integration-test executable is both parent and bounded
child helper: the child opens the same SQLite file independently, writes a
checkpoint-name marker, synchronizes it, and calls `std::process::abort()`
without Rust unwinding. Production builds expose neither the checkpoint enum
nor a crash-capable finalizer constructor, and no application binary, Tauri
command, or normal environment setting can trigger the seam.

The campaign snapshots the database before recovery and after the accepted
`StartupRecoveryCoordinator` runs. It covers committed V006 `running`, V007
`awaiting_persistence`, V006 `finalization_pending`, V008 pending claims, all
meaningful pre-commit finalizer effect/terminal boundaries, one late lesson
boundary, and the precise post-commit/before-caller-success window. Pre-commit
aborts must leave no partial effect or terminal marker; post-commit aborts must
converge without duplicate effects. A dedicated 1,000-ms
`CompletionPolicySnapshot::time(0.01)` fixture crosses the time-goal threshold,
so its real `daily_goal_met: false → true` update is aborted before commit,
reapplied once by recovery, and unchanged by a second reopen. Standard and
lesson fixtures verify test, replay, PB, daily, streak, daily-goal, and lesson
evidence as applicable. The campaign simulates abrupt application-process death
and SQLite/WAL reopening; it does not claim physical power-loss,
disk-controller-cache-loss, kernel, or filesystem-failure coverage. Phase
3B.3 recovery architecture is accepted and complete. Phase 3B.3.7 final
acceptance is complete. Later Phase 3B milestones remain not started and
require separate approval.

The history row's `tags` are deterministically empty in both completion paths
and are included in terminal proof. `is_pb` is a historical derivative of the
personal-best comparison against other sessions, not an immutable V007 input;
terminal proof validates its SQLite boolean shape but does not recompute a
later-changing personal-best relationship.

### Transaction boundaries

- Completion uses one SQLite `IMMEDIATE` transaction for test history, replay frames, personal bests, daily statistics, daily streak, daily goal state, and optional lesson completion. The completion wall-clock timestamp is captured once, so history and daily aggregation use the same UTC date.
- The completion transaction writes the immutable backend-issued `session_id` in the history row before replay, personal-best, aggregate, and lesson side effects commit. The unique index and SQLite guards prevent duplicate or replaced identities at the persistence boundary.
- An accepted custom-text start uses one transaction to read the text and increment its use count; a lifecycle-rejected request makes no persistence change.
- An accepted lesson start uses one transaction to initialize its progress row after resource validation; a lifecycle-rejected request makes no persistence change.
- Settings read-modify-write work is serialized in-process and replaces `settings.toml` atomically after syncing the temporary file.

## Current crate responsibilities

### `racoon-domain`

Shared data models, settings, themes, keyboard/layout types, lesson types, statistics, and test records. It also owns the immutable UUIDv7/legacy-compatible `SessionId` value object. It must remain independent of Tauri, SQLite, filesystem paths, and UI concerns.

### `racoon-core`

Typing/session engine, input processing, test modes, lessons, weak-key/adaptive logic, replay data, sound decisions, and analytics algorithms. It must remain runnable without a database or desktop runtime.

### `racoon-data`

SQLite connection and migration setup plus repository implementations. SQL and persistence-specific mapping belong here. Migration V005 owns the additive session identity column, deterministic legacy backfill, unique index, and immutability/required-value guards. Every opened connection now enables foreign keys, WAL, `synchronous=NORMAL`, and a five-second busy timeout. `Database::with_connection` and `Database::with_transaction` are the production access seams; the legacy `conn()` accessor remains for diagnostic/test code only.

### `racoon-resources`

Embedded content loading and validation for words, quotes, and courses. Resource provenance and distribution licensing are tracked by Phase 1; malformed optional content must not be silently treated as valid content.

### `racoon-application`

Infrastructure-free application kernel for the session vertical slice. It owns
the transport-neutral `SessionStartRequest` and completion snapshot contracts,
the `SessionKernel` orchestration, and ports for session identity, monotonic
time, wall time, random values, mode construction, and completion persistence.
Phase 3B.3.1 additionally owns the durable recovery state vocabulary and
exhaustive transition policy, immutable completion-intent/fingerprint contract,
metadata-only recovery candidates, pure recovery decisions, readiness
transitions, and business-oriented recovery/finalizer ports. Phase 3B.3.5 adds
the application-only coordinator, bounded retry/sleeper port, bounded report,
and mutex-protected process readiness gate; it retains no SQLite, Tauri,
payload, replay, or connection type. Phase 3B.3.3 adds
the business-only `FinalizationLedger` contract, bound to a stored intent and
expected fingerprint; strict stored-envelope validation re-canonicalizes every
application-owned nested object, applies the versioned numeric rule, normalizes
signed zero, and rejects non-finite values. It depends only on
`racoon-domain` and `racoon-core`; concrete providers and adapters remain
outside the crate. Finalization effects are accepted in the infrastructure-side
finalizer; broader use-case extraction remains later Phase 3B work. The
process-crash campaign is test-only data-layer evidence and does not add a
production recovery or persistence dependency to racoon-application.

### `racoon-app`

Tauri application entrypoint, command registration, application state, platform integration, and transport adaptation. `commands/` groups thin adapters by session, content, reporting, preferences, and diagnostics; `session_service.rs` supplies the Phase 3B.1–3B.2 ports with UUIDv7, clock, resource, and SQLite adapters while `SessionKernel` owns session lifecycle/input orchestration. It creates session identities and validates the identity token on input/abort commands. `AppState` provides the production monotonic and UTC clocks and retains sound cooldown state. Fully migrated use cases, versioned IPC contracts, and frontend store decomposition remain later Phase 3B work.

### `frontend`

Svelte 5/Vite presentation layer. IPC wrappers and TypeScript models live under `frontend/src/lib`. The frontend owns display and interaction state, while backend-confirmed session lifecycle and all persisted/domain authority remain in Rust.

## Version policy

The canonical product release version is the `version` value under `[workspace.package]` in the root `Cargo.toml`. The following files mirror that value because their tools require local metadata:

- `crates/app/tauri.conf.json`;
- `frontend/package.json` and the root entry in `frontend/package-lock.json`;
- `PKGBUILD`.

Run `npm run check:version --prefix frontend` to verify the mirrors. A release tag must be `v<canonical-version>`.

`racoon_domain::API_VERSION` is a separate API-contract compatibility identifier. It must not be used as the product release version or silently changed as part of a release-version update.

## Canonical commands

Run these commands from the repository root:

```bash
# Install the locked frontend toolchain
npm ci --prefix frontend

# Foundation validation
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm run check --prefix frontend
npm run build --prefix frontend
npm run check:version --prefix frontend
npm run tauri:build:binary --prefix frontend

# Native package builds on a Linux packaging host
npm run tauri:build:ci --prefix frontend -- --bundles deb
npm run tauri:build:ci --prefix frontend -- --bundles rpm

# License and diff policy (`cargo-deny` 0.20.2 is a CI prerequisite)
cargo deny check licenses
npm run license:check --prefix frontend
git diff --check

# Desktop development (Tauri runs from crates/app)
npm run tauri:dev --prefix frontend

# Production Tauri build with all configured platform bundles
npm run tauri:build --prefix frontend
```

The wrapper in `scripts/tauri.mjs` deliberately runs the CLI with `crates/app` as its working directory. This keeps the Rust manifest, Tauri configuration, asset paths, and pre-build commands deterministic on supported host shells.

## Build topology

The Tauri configuration is stored in `crates/app/tauri.conf.json`. Therefore, paths under `build.frontendDist` and `bundle.icon` are interpreted relative to `crates/app`. Tauri executes its configured pre-build command from the workspace's `crates` directory, so the frontend build command uses `npm --prefix ../frontend`.

The frontend output is `frontend/dist`. It is generated and ignored; it is not a source-controlled release input. Tauri embeds that directory into the application binary or bundle.

## Persistence locations and lifecycle

The application now resolves data and configuration locations through Tauri's platform path resolver using the bundle identifier. On Linux startup, it performs a non-destructive one-way copy from the verified Phase 0 locations when the new destination file does not yet exist:

- legacy data: `${XDG_DATA_HOME:-$HOME/.local/share}/racoon-typper/data.db` plus WAL/SHM companions;
- legacy settings: `${XDG_CONFIG_HOME:-$HOME/.config}/racoon-typper/settings.toml`.

The legacy files are retained, and an existing destination always wins. Directory creation and database migration failures stop startup cleanly with a process error rather than an `expect` panic. SQLite is closed by normal Rust drop during shutdown; no asynchronous replay buffer exists because replay persistence completes synchronously inside the completion transaction.

## Security boundary

The frontend can invoke only commands registered by the Tauri application, and the capability manifest defines the window permissions. The backend, not the frontend, owns session identity, timestamps, typing state, final scores, completion, lesson completion, persistence, and retry state. Phase 2 validates bounded input at the adapter boundary and in repositories where persistence owns the invariant; Phase 3A additionally validates the UUIDv7/legacy identity token and compares it with the engine's immutable current identity before processing input or aborting. Dedicated command responses now use named serializable types rather than endpoint-level `serde_json::Value` wrappers.

Existing dynamic fields such as mode configuration and a scalar settings update remain bounded and backend-validated compatibility forms. A generated/versioned universal IPC contract, minimum capability/CSP review, import/export policy, and detailed threat model remain deferred to their approved Phase 3B and security-phase work.

## Known limitations after Phase 3B.2

The following are intentionally not hidden by this document:

- deterministic runtime provider seams and test providers now exist, but production behavior still comes from the existing runtime adapters;
- Phase 3B.3.1 recovery contracts, the focused Phase 3B.3.1a stored-header amendment, Phase 3B.3.2, Phase 3B.3.3, and Phase 3B.3.4 are accepted after independent review. Phase 3B.3.5 and Phase 3B.3.6 are accepted after independent re-review. V008 binds one finalization record to the immutable V007 fingerprint through a composite foreign key; `pending → committed|quarantined` is the only mutable state path; and update/delete/replacement/reopen attacks are rejected by schema triggers. Quarantine revalidates expected, V008, and current V007 fingerprints, so durable mismatch/missing/corruption classifications take precedence over a caller-supplied reason. Phase 3B.3.4's infrastructure-side `SessionFinalizer` uses one `IMMEDIATE` transaction to strictly load V007, validate V006/V008 state and fingerprints, apply the existing completion effects from immutable intent data, then commit V008 and V006 terminal markers. Phase 3B.3.5 invokes that port only from its startup coordinator after a metadata-only scan and V008 claim; normal live completion remains unwired. The coordinator blocks unsafe state-changing commands until `Ready`. Phase 3B.3.6 adds only a non-default-feature, independent-child process-crash campaign; its focused remediation proves a real daily-goal false-to-true write is rolled back before commit and reapplied once. It adds no V009, resume flow, production crash API, or physical-power-loss claim. Phase 3B.3 recovery architecture is accepted and complete; Phase 3B.3.7 final acceptance is complete; later Phase 3B milestones remain not started and require separate approval;
- Phase 3B.1–3B.3.1 establishes the standalone application layer, session kernel, provider ports, recovery contracts, and test seams; later Phase 3B still owns broader use-case migration, request/config contract ownership, generated/versioned universal contracts, and frontend store decomposition;
- migration backup/restore, preflight integrity recovery, and long-history query redesign remain database lifecycle work;
- structured persistent logging and operational observability are not yet implemented;
- package signing, SBOMs, reproducible artifacts, and cross-platform smoke tests are not yet release-complete;
- asset/content provenance and the Apache-2.0 compatibility inventory are implemented in `licenses/` and enforced by the Phase 1 policy job, pending maintainer/legal review;
- uncommitted worktree changes are not part of the verified release baseline.

## Architecture change policy

Changes that cross crate boundaries, alter IPC contracts, modify persistent schema, add a dependency, add an asset, or change a release artifact require a short architecture or decision note. The implementation must preserve dependency direction, typed errors, deterministic tests, bounded inputs, and a documented rollback path.
