# Threat Model — Racoon Typper

**Status:** Baseline inventory for Task G; it records the current implementation, not a claim that the listed residual risks are closed.

**Scope and assumptions:** Racoon Typper 1.1.0 is a local-first Tauri desktop application. It has no account, backend service, telemetry, or application-controlled network API. The primary adversary is therefore a malicious or compromised renderer, a person or process with access to the local user account or its files, or a malicious file/content supplied to the user. This document does not treat a fully compromised operating system or a malicious installed application binary as preventable by application code.

**Evidence inspected:** `crates/app/src/main.rs`, `crates/app/src/commands/`, `crates/app/src/paths.rs`, `crates/app/src/error.rs`, `crates/app/src/validation.rs`, `crates/data/src/backup.rs`, `crates/data/src/profile_transfer.rs`, `crates/app/tauri.conf.json`, `crates/app/capabilities/main.json`, packaging manifests/scripts, and the tests cited below. The command and packaging inventory is current at Task G; it must be revisited when either surface changes.

## Assets

| Asset | Sensitivity / integrity need | Current location or flow |
|---|---|---|
| Typed practice text, custom texts, test history, replays, and derived analytics | May contain private user content; must not be disclosed or silently corrupted | SQLite `data.db`; custom-text/session and profile-transfer IPC |
| Session/recovery/finalization records | Integrity and availability; they determine durable session completion and recovery | SQLite tables managed by `racoon-data` |
| Settings | Integrity; controls local application behavior | `settings.toml` in the platform-resolved config directory |
| Portable profile JSON and SQLite backups | Confidentiality and integrity; may disclose history/custom text and can overwrite or merge local portable data | Returned through IPC; storage is chosen by the caller; pre-migration snapshots are under the data directory's `backups/` subdirectory |
| Bundled frontend, resources, licenses, and desktop metadata | Integrity; a modified bundle can alter renderer behavior or distribution claims | Tauri bundle resources, `resources/`, package manifests/scripts |
| Diagnostic output | Confidentiality; paths and error strings can disclose local information | Current process stderr only; no application log file exists |

## Trust boundaries and entry points

| Boundary | Untrusted side | Trusted side | Current boundary evidence |
|---|---|---|---|
| Webview ↔ Rust | Renderer-originated Tauri `invoke` arguments | Tauri command adapters, core engine, SQLite/filesystem | The local `main` window has 31 explicitly permitted application commands. `capability_audit.rs` verifies exact equality among the handler, frontend wrappers, build-time manifest, and capability. |
| IPC adapter ↔ persistence/runtime | Command inputs, including text and JSON | `AppState`, `CoreEngine`, `racoon-data` | Command-specific validation plus startup-recovery and engine-state guards are present, but contracts are still hand-maintained. |
| Local filesystem ↔ app | Existing database/settings, legacy XDG paths, backups, package inputs | Path resolver, SQLite, settings store | `paths.rs` uses Tauri's app data/config resolver and performs a one-way Linux legacy copy only when the destination is absent. |
| User-supplied content ↔ stored/rendered content | Custom text, test text, profile JSON, query/filter strings | Validation, SQLite repositories, IPC responses | Inputs are accepted as data, not executable code. This is not a sanitizer or access-control boundary for another local process. |
| Build/release environment ↔ installed artifact | Source checkout, dependencies, AppImage download, Flatpak source directory | CI/build scripts, Tauri bundler, package manifests | Artifact integrity and clean-install execution are not yet fully verified; see Packaging below. |

### Current IPC inventory

The `tauri::generate_handler!` registration in `crates/app/src/main.rs` exposes only the commands wrapped by `frontend/src/lib/api/ipc.ts` to the local `main` window:

- Session: `start_test`, `process_key`, `abort_session`, `start_custom_text_test`, `start_lesson`.
- Reporting/replay: `get_stats_history`, `get_personal_bests`, `get_dashboard_stats`, `get_progress_history`, `get_achievements`, `get_insights`, `get_consistency`, `export_data`, `get_replay`.
- Custom content/course/weak keys: `get_custom_texts`, `save_custom_text`, `update_custom_text`, `delete_custom_text`, `search_custom_texts`, `get_course`, `get_lesson_progress`, `analyze_weak_keys`, `generate_weak_keys_training`.
- Preferences/themes/sound: `get_settings`, `set_setting`, `get_themes`, `get_theme_css`, `get_sound_event`.
- Portable profile transfer: `export_profile`, `preview_profile_import`, `import_profile`.

`crates/app/build.rs` generates Tauri 2 application-command permissions from the exact 31-command set. `crates/app/capabilities/main.json` grants each generated `allow-*` permission only to the local `main` window. It grants no `core:*` permission: the frontend imports only `invoke`, and neither the Rust application nor frontend uses Tauri window, webview, event, app, path, resource, image, menu, or tray IPC APIs. The capability has no remote URL association; the bundled local content is the sole permitted origin. `crates/app/tests/capability_audit.rs` is the regression control for unexpected or missing handler/frontend/manifest/capability coverage.

## Abuse cases, controls, and evidence

| Surface / abuse case | Current controls and directly relevant evidence | Residual risk / required follow-up |
|---|---|---|
| A compromised or unintended renderer invokes a mutating command with malformed, oversized, or state-inconsistent arguments. | Central validation bounds page limits, offsets, key fields, duration, word count, language, settings keys, and direct test text (`crates/app/src/validation.rs`). Mutations that affect recovery are gated by `AppState::require_startup_recovery_ready`; profile import also rejects a running/finalizing engine. Unit tests cover unsafe/bounded validation and active-session import rejection. Tauri 2 application-command permissions are generated and audited against the 31 registered frontend commands. | Not every command has a versioned request DTO or contract test. An authorized renderer can still invoke each command it is permitted to use. Task J must make hostile-input coverage systematic. |
| IPC errors disclose typed text, paths, or database details to the renderer. | The data layer has tests that debug recovery/session payloads redact custom text (`crates/application/src/session.rs` and `crates/application/src/recovery.rs`). Some SQLite failures are collapsed to `"SQLite operation failed"` at the app boundary. | `AppError` serializes many underlying error strings, including settings, DB, migration, backup, and restore messages. No comprehensive IPC error-redaction test proves that user content and paths cannot escape. Treat error strings as potentially sensitive until Task I/J adds and verifies a redaction policy. |
| An untrusted local file or path replaces, mixes with, or corrupts active data/settings. | Tauri resolves app-managed data/config paths. The Linux legacy migration copies only missing destinations through a sibling temporary file and rename; tests prove existing destination preservation and one-time copy. SQLite uses its engine/backup API rather than manual live-WAL copying. | Local user/process access is outside the app's access-control model. The legacy source paths are derived from `HOME`/XDG variables, and no documented permission/ownership hardening or adversarial symlink test exists. Whole-file restore is deliberately not exposed through IPC/UI. |
| A custom text or direct test text causes resource exhaustion or is interpreted as active content. | Stored custom text is validated for nonblank name/text and a 10,000-character maximum in the repository; direct test text shares that maximum. Theme CSS comes only from three compiled-in names after theme-name validation; there is no arbitrary CSS import command. | Custom text is intentionally returned to the renderer and may be rendered there. The threat model does not claim HTML/CSS sanitization, CSP coverage for future content paths, or an XSS test suite. Task J should add adversarial content regression tests before new import/rendering features. |
| A hostile portable profile JSON exhausts memory, creates invalid state, or partially overwrites portable data. | `profile_transfer.rs` rejects unknown fields, unsupported format/version, documents over 64 MiB, collections over 100,000 rows, duplicate identities, invalid timestamps/scalars/JSON, invalid custom texts, and invalid references before a write transaction. Preview is no-write; merge/replace import is one SQLite transaction. `crates/data/tests/profile_transfer.rs` verifies malformed/oversize/unsupported/custom-text rejection, no-write preview, rollback on invalid import, and merge/replace behavior. | The document is passed as a single IPC `String`, so the 64 MiB limit is checked after IPC deserialization/allocation. The UI/file-picker/persistence/confirmation workflow does not exist. Preview can become stale before import. The profile intentionally excludes settings, replays, raw backups, and recovery ledgers; it is not a full system backup. |
| A restore file corrupts or replaces the live SQLite database. | `restore_from_path` requires a separate regular source file, builds and integrity-checks a sibling temporary SQLite database using SQLite's Online Backup API, then renames it into place and removes stale WAL/SHM companions. `crates/data/tests/backup_restore.rs` verifies invalid input leaves the live database intact and backup round trips preserve data/WAL state. | The caller must close every live `Database` first. No restore IPC/UI closes and reopens the managed connection, and no platform/manual recovery smoke exists. Manual copying of an active WAL-mode database remains unsafe. Do not expose whole-file restore until lifecycle coordination and recovery tests exist. |
| Logs or stderr leak user content, absolute paths, or raw errors; diagnostics grow without bound. | The only observed application diagnostics are two `eprintln!` calls in `main.rs`: a pre-migration-backup warning and a startup failure. The backup warning comment states an intent to avoid typed content, but the error string itself is not redacted by a structured logging component. | There is no opt-in logger, redaction layer, bounded rotation/retention, or test proving logging output excludes sensitive payloads. Task I owns this gap; this model does not claim logging controls that have not been implemented. |
| A tampered or misleading package/install artifact executes code or omits required notices. | Tauri declares deb/rpm/AppImage/NSIS bundle targets and embeds license/provenance resources. CI pins GitHub Action revisions and performs workspace/frontend/license checks; the license policy checks Tauri/Flatpak/AppImage/PKGBUILD notice declarations. | No release checksum/provenance/signing/attestation gate is complete, and clean-install launch smoke is pending. `build-appimage.sh` downloads the moving `continuous` appimagetool without a pinned checksum. The Flatpak manifest consumes the checkout as a `dir` source, enables build-time network, and grants X11, Wayland, IPC, DRI, and XDG data/config filesystem access. Tasks K–P must reduce and verify this release surface. |

## Decisions and scope boundaries

- This document does **not** add a new dependency decision. The Linux GLib/GTK3 risk remains governed by [ADR 0001](../adr/0001-glib-gtk3-advisory.md), which records a bounded documented acceptance for the transitive `glib` 0.18/gtk3-rs chain and its explicit revisit triggers.
- The threat model's packaging and renderer boundaries include that dependency exposure. `cargo tree -i glib@0.18.5` currently resolves the chain through Tauri/wry/webkit2gtk/gtk, while the Windows and macOS target queries have no `webkit2gtk` result. This does not make the renderer or package chain risk-free; it only scopes this particular advisory as ADR 0001 describes.
- No application network API is in scope today. Any future remote sync, telemetry, update service, external content fetch, file picker, arbitrary theme import, or restore IPC changes the trust boundaries and requires a threat-model update before implementation.

## Residual-risk register

| ID | Residual risk | Status / owner |
|---|---|---|
| TM-1 | IPC request/config contracts remain unversioned and hand-maintained. The Task H capability control limits the local main window to the audited command set, but does not type or version those contracts. | Open — Task R |
| TM-2 | Raw error-string and stderr disclosure; no bounded redacted application logging. | Open — Task I and Task J |
| TM-3 | Portable profile JSON is allocated across IPC before its data-layer size check; no file workflow or user-confirmation UI exists. | Open — Task F follow-up / Task J |
| TM-4 | Whole-file restore lacks app lifecycle coordination and platform/manual recovery evidence. | Open — Task F follow-up |
| TM-5 | Package provenance/checksums/signing and clean-install runtime evidence are incomplete; AppImage tool download and Flatpak manifest require redesign. | Open — Tasks K–P |
| TM-6 | The Linux Tauri renderer depends transitively on the GLib/GTK3 advisory chain. | Monitored — ADR 0001 / `TECH_DEBT.md` DD4 |

## Maintenance and verification

Update this model with the implementation change when an IPC command, capability, filesystem path, imported content type, diagnostic sink, backup/restore flow, bundle target, or transitive-renderer dependency decision changes. The release status is tracked in [RELEASE_CHECKLIST.md](../../RELEASE_CHECKLIST.md); current technical debt is tracked in [TECH_DEBT.md](../../TECH_DEBT.md). The data-transfer operating constraints remain in the [Profile Transfer and Recovery Runbook](../data/profile-transfer.md).
