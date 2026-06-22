# AUDIT REPORT — Racoon Typper v0.1.0

**Date**: 2026-06-22
**Auditor**: Automated + manual
**Scope**: Full codebase audit before Sprint 9

---

## 1. Dependency Audit

### cargo audit

**Result**: 0 vulnerabilities, 17 warnings (unmaintained crates)

| Crate | Version | Warning | Source |
|-------|---------|---------|--------|
| gtk, atk, gdk, gdk-sys, gtk-sys, atk-sys, gdkx11, gdkx11-sys, gdkwayland-sys, gtk3-macros | 0.18.x | unmaintained (GTK3 bindings) | Tauri 2.x dependency |
| glib | 0.18.5 | unsound (RUSTSEC-2024-0429) | Tauri 2.x dependency |
| proc-macro-error | 1.0.4 | unmaintained | Tauri build chain |
| unic-* (5 crates) | 0.9.0 | unmaintained | Tauri/html5ever dependency |

**Verdict**: All warnings come from Tauri 2.x transitive dependencies. Cannot fix without Tauri update. No action needed for v0.1.x. Monitor Tauri 2.x releases for GTK3 → GTK4 migration.

### Outdated Dependencies

| Crate | Current | Latest | Action |
|-------|---------|--------|--------|
| rusqlite | 0.31.0 | 0.40.x | Defer — breaking API changes, not critical for MVP |
| refinery | 0.8.16 | 0.9.x | Defer — minor API changes |
| toml | 0.8.23 | 1.1.x | Defer — major version, stable on 0.8 |
| chrono | 0.4.45 | latest | OK — up to date |
| serde | 1.0.228 | latest | OK — up to date |
| tauri | 2.11.3 | 2.x | OK — latest minor |

### Licenses

All workspace crates: MIT. Transitive dependencies: standard open-source (MIT, Apache-2.0, MPL-2.0). No GPL contamination.

---

## 2. Code Quality Audit

### Dead Code

| Location | Type | Status |
|----------|------|--------|
| stats.rs: `type_text()` | unused test helper | REMOVED |
| custom_texts.rs: `make_text()` | unused test helper | REMOVED |
| typing.rs: `current_expected()` | unused pub fn | Keep — API for future use |
| typing.rs: `typed_at()` | unused pub fn | Keep — API for future use |
| engine.rs: `is_active()`, `current_text()`, `caret_position()` | unused outside tests | Keep — public API |
| stats.rs: `elapsed_minutes()`, `net_wpm()`, `net_accuracy()`, `build_char_stats()`, `count_first_correct()`, `count_first_attempts()` | unused outside tests | Keep — public API, used in finalize() internally |

### Unused IPC Commands

| Command | Used in frontend? | Status |
|---------|-------------------|--------|
| get_app_info | No | Keep — diagnostic command |
| get_custom_text | No | Keep — individual text retrieval, future use |

### Clippy

**Result**: 0 warnings with `-D warnings`. Clean.

### Fmt

**Result**: 0 diffs. Clean.

---

## 3. Binary Audit

### Size

| Metric | Value |
|--------|-------|
| Binary size (release) | 15 MB |
| Stripped (estimated) | ~8-10 MB |
| Dependencies compiled | 455 crates |
| Build time (release) | ~45s |

### Heaviest Dependencies (estimated)

| Dependency | Estimated size | Reason |
|------------|---------------|--------|
| webkit2gtk bindings | ~3 MB | Tauri WebView |
| rusqlite (bundled SQLite) | ~2 MB | Static SQLite |
| tauri runtime | ~2 MB | Window management |
| serde/serde_json | ~1 MB | Serialization |
| gtk/glib bindings | ~2 MB | GTK3 bindings |

**Recommendations**: Binary size is acceptable for desktop app. Consider `strip = true` in release profile for ~30% reduction.

---

## 4. Frontend Audit

### Component Sizes

| Component | Lines | Status |
|-----------|-------|--------|
| App.svelte | 329 | OK — orchestrator |
| TestView.svelte | 111 | OK |
| SettingsView.svelte | 86 | OK |
| CustomTextsView.svelte | 85 | OK |
| KeyboardHeatmap.svelte | 68 | OK |
| ModeSelector.svelte | 60 | OK |
| HistoryView.svelte | 40 | OK |
| BestsView.svelte | 38 | OK |
| ResultOverlay.svelte | 35 | OK |
| NavigationBar.svelte | 21 | OK |

**Verdict**: All components under 300 lines. App.svelte at 329 — acceptable as orchestrator.

### TypeScript Types

| File | Lines | Status |
|------|-------|--------|
| types/index.ts | 103 | Complete — mirrors domain types |
| api/ipc.ts | 90 | Complete — all 19 commands wrapped |

### Duplications

- App.svelte still holds all state — no separate stores. Acceptable for MVP, consider Svelte stores for Sprint 9.
- IPC types in frontend mirror domain types in Rust — manual sync required. Consider ts-rs post-MVP.

---

## 5. Database Audit

### Schema

6 tables, 9 indices, 3 UNIQUE constraints. All correct per ARCHITECTURE.md.

### WAL Mode

```sql
PRAGMA journal_mode=WAL
```
Configured in `db.rs:open()`. Correct.

### Repository Layer

| Repository | Trait | Impl | Status |
|------------|-------|------|--------|
| TestRepository | ✅ | ✅ SqliteTestRepository | Full |
| PersonalBestsRepository | ✅ | ✅ SqlitePersonalBestsRepository | Full |
| CustomTextRepository | ✅ | ✅ SqliteCustomTextRepository | Full |
| LessonRepository | ✅ | ✅ SqliteLessonRepository | STUB — returns empty |
| SettingsStore | ✅ | ✅ TomlSettingsStore | Full |

### Race Conditions

**process_key** has 3 locks on `engine_state` and 1 on `app_state.db`:
- Lock 1: `engine_state.lock()` — process key
- Lock 2: `engine_state.lock()` — read mode info (inside if test_complete)
- Lock 3: `app_state.db.lock()` — save test
- Lock 4: `engine_state.lock()` — read mode config for PB

**Risk**: Lock 2 and 4 re-lock engine_state while lock 1 is still held. This compiles because lock 1's guard is dropped, but it's inefficient.

**Recommendation**: Cache mode_type and mode_config before the if block. Refactor in Sprint 9.

---

## 6. IPC Audit

### Commands

19 commands total. All return `Result<T, AppError>`. No `String` errors.

### Payload Sizes

| Command | Payload | Status |
|---------|---------|--------|
| process_key | EngineOutput (small) | OK |
| get_stats_history | Vec<TestSummary> (up to 50 items) | OK |
| get_custom_texts | Vec<CustomText> (up to 50 items) | OK |
| start_test | TestSessionResponse (text up to ~2KB) | OK |

### Duplications

- `json_to_toml_value()` used in 2 places — acceptable, single function.
- Mode type string conversion repeated 3x in process_key — refactor candidate.

---

## 7. Summary

### Health Score: 8.5/10

| Category | Score | Notes |
|----------|-------|-------|
| Dependencies | 7/10 | Unmaintained GTK3 bindings (Tauri), outdated rusqlite/refinery |
| Code Quality | 9/10 | 0 warnings, dead code removed, clean clippy |
| Binary | 8/10 | 15 MB, acceptable, could strip |
| Frontend | 9/10 | All components < 330 lines, well-typed |
| Database | 8/10 | Correct schema, WAL, stub LessonRepository, minor lock inefficiency |
| IPC | 9/10 | Typed errors, no String, minor duplications |
| Tests | 9/10 | 155 tests, 9 e2e, good coverage |

### Action Items

1. **Sprint 9**: Fix process_key double-lock pattern
2. **Sprint 9**: Implement LessonRepository (replace stub)
3. **Post-Sprint 9**: Add `strip = true` to release profile
4. **Post-v1.0**: Update rusqlite 0.31 → 0.40 (breaking)
5. **Post-v1.0**: Update refinery 0.8 → 0.9
6. **Post-v1.0**: Consider ts-rs for Rust → TypeScript type generation