# PERFORMANCE REPORT — Racoon Typper v0.1.0

**Date**: 2026-06-22

---

## Binary Metrics

| Metric | Value |
|--------|-------|
| Release binary | 15 MB (unstripped) |
| Estimated stripped | ~10 MB |
| Frontend bundle | 57 KB JS + 9 KB CSS (gzip: 21 KB + 2 KB) |
| SQLite DB (empty) | ~12 KB |
| Settings file | ~200 bytes |
| Total install footprint | ~15 MB + webkit2gtk (system) |

## Build Times

| Target | Time |
|--------|------|
| cargo check (debug) | ~1s (incremental) |
| cargo build (debug) | ~3s (incremental) |
| cargo build --release | ~45s (clean) |
| cargo test --workspace | ~0.5s |
| npm run build (vite) | ~0.4s |
| cargo tauri dev (first) | ~5min (compile 455 crates) |
| cargo tauri dev (incremental) | ~3s |

## Runtime Performance

| Operation | Latency |
|-----------|---------|
| start_test (IPC) | <1ms |
| process_key (IPC) | <1ms |
| get_stats_history (IPC) | <2ms (50 records) |
| save_test (SQLite) | <1ms |
| check_and_update PB | <1ms |
| Word pack generation (50 words) | <0.1ms |
| Quote loading (20 quotes) | <0.1ms (embedded, include_str!) |
| Theme CSS loading | <0.1ms (embedded, include_str!) |

## Memory

| Component | Estimated |
|-----------|-----------|
| Rust process baseline | ~20 MB |
| WebView (webkit2gtk) | ~50-100 MB |
| SQLite connection | ~1 MB |
| Word packs (embedded) | ~50 KB |
| Quote packs (embedded) | ~10 KB |
| Total RSS | ~80-120 MB |

## Heaviest Modules

| Module | Est. binary size | Reason |
|--------|-----------------|--------|
| webkit2gtk-sys | ~3 MB | FFI bindings to libwebkit2gtk |
| libsqlite3-sys (bundled) | ~2 MB | Static SQLite compilation |
| tauri runtime (tao, wry) | ~2 MB | Window management |
| gtk/glib bindings | ~2 MB | GTK3 FFI |
| serde + serde_json | ~1 MB | Serialization framework |
| html5ever | ~1 MB | HTML parser (WebView) |

## Optimization Recommendations

### Immediate (Sprint 9)

1. Add `strip = true` to release profile — saves ~5 MB
```toml
[profile.release]
strip = true
```

2. Fix process_key lock pattern — reduces lock contention

### Short-term (v0.2)

3. Lazy text generation for Time mode — currently generates 200 words upfront for 120s mode (~2KB). Negligible, but true lazy generation would be cleaner.

4. SQLite prepared statement caching — rusqlite prepares statements per query. Consider statement caching for high-frequency queries.

### Long-term (post-v1.0)

5. LTO (Link-Time Optimization) — may reduce binary ~10-15%
```toml
[profile.release]
lto = true
codegen-units = 1
```

6. Consider `opt-level = "z"` for size optimization (may reduce ~20% but slower runtime)

## Frontend Performance

| Metric | Value |
|--------|-------|
| Bundle size (uncompressed) | 67 KB |
| Bundle size (gzip) | 23 KB |
| Components | 9 + 1 orchestrator |
| Initial render | <100ms (SPA, no SSR) |
| Keystroke → render | <16ms (IPC round-trip + Svelte reactivity) |

**Verdict**: Frontend performance is excellent for a typing app. No optimization needed.