# TECH DEBT — Racoon Typper

**Last updated**: 2026-06-22 (Sprint 8.5)

## Active Debt (v0.1.x)

| ID | Debt | Priority | When | Effort |
|----|------|----------|------|--------|
| TD1 | process_key double-locks engine_state (3x) | Medium | Sprint 9 | S |
| TD2 | LessonRepository is a stub returning empty | High | Sprint 9 | M |
| TD3 | App.svelte holds all state (no Svelte stores) | Low | Post-Sprint 9 | M |
| TD4 | Frontend TS types manually mirror Rust domain | Low | Post-v1.0 | M (ts-rs) |
| TD5 | release profile missing `strip = true` | Low | Sprint 9 | S |
| TD6 | No Tauri events (only invoke-based) | Low | Post-v1.0 | L |
| TD7 | daily_stats/streaks tables exist but no logic | Low | v0.2 | M |
| TD8 | consistency/graph_data fields always NULL | Low | v0.2 | M |
| TD9 | text_length always 0 in save_test | Medium | Sprint 9 | S |

## Dependency Debt

| ID | Debt | Priority | When |
|----|------|----------|------|
| DD1 | rusqlite 0.31 (latest 0.40) — major API changes | Low | Post-v1.0 |
| DD2 | refinery 0.8 (latest 0.9) — minor API changes | Low | Post-v1.0 |
| DD3 | toml 0.8 (latest 1.1) — major version | Low | Post-v1.0 |
| DD4 | GTK3 bindings unmaintained (Tauri transitive) | None | Wait for Tauri update |

## Resolved Debt

| ID | Debt | Resolved in |
|----|------|-------------|
| TD-OLD1 | type_text() unused test helper | Sprint 8.5 — removed |
| TD-OLD2 | make_text() unused test helper | Sprint 8.5 — removed |
| TD-OLD3 | Result<T, String> in all commands | Sprint 8 — AppError |
| TD-OLD4 | No git commits | Sprint 8 — first commit |
| TD-OLD5 | App.svelte 330-line monolith | Sprint 7 — split to 8 components |
| TD-OLD6 | Hardcoded mode_type="time" | Sprint 6 — TestMode trait |