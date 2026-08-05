<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- Copyright 2026 Racoon Typper Contributors -->

# ADR 0002: Long-history query semantics

- Status: Accepted
- Date: 2026-08-05
- Scope: Tasks D–E / Gate G4

## Context

History pages must remain deterministic as records grow. An ordering on a timestamp alone is not total: multiple completed tests or personal-best updates can share the same timestamp, causing offset pages to repeat or omit rows across requests.

Several legacy dashboard and achievement reads previously derived all-time values from bounded history pages. A page cap is appropriate for a list or a deliberately recent metric, but it is not correct for an all-time record or streak.

## Decision

- Test history is paginated with its existing bounded offset contract and is ordered by `created_at DESC, session_id DESC`.
- Personal-best lists are ordered by `updated_at DESC, mode_type ASC, mode_config_hash ASC`.
- The SQLite indexes introduced by V009 match those final ordering clauses. They are ordering indexes, not covering indexes: history projections contain wide `tests` rows.
- Dashboard and achievement global best/streak values use maintained `personal_bests` and `streaks` projections. The completion finalizer updates those projections in the same database transaction as the test record.
- `daily_stats` remains sparse and is queried by explicit date ranges for dashboard/progress aggregates.
- Consistency remains a recent-rhythm metric based on the most recent 100 sessions. It is intentionally not an all-time statistic; an all-time coefficient of variation would hide current learning changes.
- Legacy `export_data` still returns one bounded 1,000-row page. Versioned full export/import and its conflict/restore semantics are deferred to Task F.

## Verification

`crates/data/tests/long_history.rs` seeds 10,000 records for production ordering/index planner paths, an acceptable full-count scan, and bounded read timing. It separately seeds 100,001 records to prove global personal-best and streak projections remain available beyond former dashboard and achievement caps. Application reporting-contract tests verify that dashboard and achievement use those complete projections while insight consistency retains its bounded recent sample. Repository regressions cover timestamp tie-break pagination. Migration-matrix tests verify V1..V8 upgrades through the production Refinery runner to the V009 index set.

## Consequences

Existing IPC pagination remains backward compatible. Offset pagination is intentionally bounded; if measured product usage requires deep-history navigation beyond the configured offset, a versioned cursor contract must be designed separately rather than silently changing the IPC surface.
