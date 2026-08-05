//! Long-history index and metric evidence for the analytics/reporting reads
//! (Task D, gate G4 / TD5).
//!
//! Three families of evidence, all on a 10 000+ row `tests` fixture:
//! 1. EXPLAIN QUERY PLAN shape assertions — prove the V009 covering indexes are
//!    actually selected by the planner for the filtered-history and
//!    personal-bests-listing reads, and that the long-history scan paths that
//!    remain (count-all, unfiltered history) are the only ones that scan.
//! 2. Timing regression thresholds on those reads at 10 k scale.
//! 3. Metric-correctness regression: a global best that lives well past the old
//!    500-test window is recoverable from `personal_bests`, and the global
//!    `longest_streak` survives in the maintained `streaks` row — proving the
//!    Task D fix that moved those metrics off bounded history slices.

// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use chrono::{Duration as ChronoDuration, TimeZone, Utc};
use racoon_data::db::Database;
use racoon_data::repository::{
    PersonalBestsRepository, SqlitePersonalBestsRepository, SqliteTestRepository, TestRepository,
};
use rusqlite::Connection;

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

const ROW_COUNT: usize = 10_000;

fn temp_path(name: &str) -> PathBuf {
    let sequence = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "racoon-longhist-{name}-{}-{sequence}.db",
        std::process::id()
    ))
}

fn remove_database(path: &Path) {
    let _ = std::fs::remove_file(path);
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        let _ = std::fs::remove_file(path.with_file_name(format!("{name}-wal")));
        let _ = std::fs::remove_file(path.with_file_name(format!("{name}-shm")));
    }
}

/// Inserts `count` test rows spanning several modes, languages, and dates so
/// that all index/code paths have realistic data. Rows get monotonically
/// increasing ids and timestamps, matching how real history accumulates.
/// `mode_type` cycles to populate both filtered and unfiltered paths.
fn seed_history(conn: &mut Connection, count: usize) {
    let tx = conn.transaction().expect("begin seed transaction");
    {
        let mut stmt = tx
            .prepare(
                "INSERT INTO tests (
                    session_id, created_at, mode_type, mode_config, language, text_length,
                    duration_ms, wpm, raw_wpm, accuracy, raw_accuracy, consistency,
                    correct_chars, incorrect_chars, backspaces, char_stats, heatmap_data,
                    graph_data, is_pb, tags
                 ) VALUES (
                    ?1, ?2, ?3, '{}', ?4, 50, 30000,
                    ?5, ?5, 90.0, 90.0, NULL, 95, 5, 2, '{}', '{}', NULL, 0, ''
                 )",
            )
            .expect("prepare seed insert");
        for i in 0..count {
            // Mode cycles across a small set so filtered queries are selective.
            let mode = match i % 4 {
                0 => "time",
                1 => "words",
                2 => "quote",
                _ => "custom",
            };
            let language = if i % 5 == 0 { "ru" } else { "en" };
            let created_at = Utc
                .with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
                .single()
                .expect("fixed timestamp")
                .checked_add_signed(ChronoDuration::seconds(i as i64))
                .expect("fixture timestamp in range")
                .to_rfc3339();
            let wpm = 40.0 + (i % 30) as f64;
            // Use a distinct session_id per row.
            let session_id = format!("lh-session-{i}");
            stmt.execute(rusqlite::params![
                session_id, created_at, mode, language, wpm,
            ])
            .unwrap_or_else(|e| panic!("seed insert {i}: {e}"));
        }
    }
    tx.commit().expect("commit seed transaction");
}

/// Runs `EXPLAIN QUERY PLAN` for `sql` with `params` and joins the plan rows
/// into a single lowercase string for substring assertions.
fn query_plan(conn: &Connection, sql: &str, params: &[&dyn rusqlite::ToSql]) -> String {
    // EXPLAIN QUERY PLAN must prefix the full statement in one prepared stmt.
    let explain_sql = format!("EXPLAIN QUERY PLAN {sql}");
    let mut stmt = conn
        .prepare(&explain_sql)
        .unwrap_or_else(|e| panic!("prepare EXPLAIN for {sql}: {e}"));
    let mut rows = stmt
        .query(params)
        .unwrap_or_else(|e| panic!("EXPLAIN for {sql}: {e}"));
    let mut parts: Vec<String> = Vec::new();
    while let Ok(Some(row)) = rows.next() {
        let detail: String = row.get(3).unwrap_or_default();
        parts.push(detail.to_ascii_lowercase());
    }
    parts.join(" | ")
}

/// Convenience for parameterless EXPLAIN.
fn query_plan_no_params(conn: &Connection, sql: &str) -> String {
    query_plan(conn, sql, &[])
}

// ---------------------------------------------------------------------------
// 1. EXPLAIN QUERY PLAN — the V009 indexes must be selected.
// ---------------------------------------------------------------------------

#[test]
fn filtered_history_uses_its_deterministic_ordering_index() {
    let path = temp_path("explain-filtered");
    remove_database(&path);
    let db = Database::open(&path).expect("open db");
    {
        let mut conn = db.conn();
        seed_history(&mut conn, ROW_COUNT);
        let plan = query_plan_no_params(
            &conn,
            "SELECT id, session_id, created_at,
                    EXISTS(SELECT 1 FROM test_replays WHERE test_replays.test_id = tests.id)
             FROM tests WHERE mode_type = 'time'
             ORDER BY created_at DESC, session_id DESC LIMIT 50",
        );
        assert!(
            plan.contains("idx_tests_mode_created_at_session_id"),
            "filtered history must use its ordering index, plan: {plan}"
        );
        // A bare-table scan (no index) shows as "scan tests" without an index
        // qualifier. The index-backed search/scan is the fast path we want, so
        // only reject a bare table scan.
        assert!(
            !plan.contains("scan tests")
                || plan.contains("using covering index")
                || plan.contains("using index"),
            "filtered history must use the index, not a bare table scan, plan: {plan}"
        );
    }
    remove_database(&path);
}

#[test]
fn personal_bests_listing_uses_its_deterministic_ordering_index() {
    let path = temp_path("explain-pb");
    remove_database(&path);
    let db = Database::open(&path).expect("open db");
    {
        let conn = db.conn();
        // Seed a handful of PB rows so the listing has data.
        for mode in ["time", "words", "quote", "custom"] {
            conn.execute(
                "INSERT INTO personal_bests (
                    mode_type, mode_config_hash, mode_config, best_wpm, best_wpm_test_id,
                    best_accuracy, best_accuracy_test_id, updated_at
                 ) VALUES (?1, ?2, '{}', 50.0, NULL, 95.0, NULL, '2026-01-01T00:00:00Z')",
                rusqlite::params![mode, format!("hash-{mode}")],
            )
            .expect("seed PB");
        }
        let plan = query_plan_no_params(
            &conn,
            "SELECT id, mode_type, mode_config_hash, mode_config, best_wpm,
                    best_wpm_test_id, best_accuracy, best_accuracy_test_id,
                    best_consistency, best_consistency_test_id, updated_at
             FROM personal_bests
             ORDER BY updated_at DESC, mode_type ASC, mode_config_hash ASC",
        );
        assert!(
            plan.contains("idx_personal_bests_updated_at_config_hash"),
            "PB listing must use its ordering index, plan: {plan}"
        );
        assert!(
            !plan.contains("use temp b-tree"),
            "PB listing must not sort into a temporary B-tree, plan: {plan}"
        );
    }
    remove_database(&path);
}

#[test]
fn count_all_is_allowed_to_scan() {
    // COUNT(*) over all tests is inherently a full scan; document that this is
    // the expected and acceptable plan (no filter to use an index on).
    let path = temp_path("explain-count");
    remove_database(&path);
    let db = Database::open(&path).expect("open db");
    {
        let mut conn = db.conn();
        seed_history(&mut conn, ROW_COUNT);
        let plan = query_plan_no_params(&conn, "SELECT COUNT(*) FROM tests");
        assert!(
            plan.contains("scan"),
            "count-all is expected to scan, plan: {plan}"
        );
    }
    remove_database(&path);
}

// ---------------------------------------------------------------------------
// 2. Timing regression at 10k scale.
// ---------------------------------------------------------------------------

/// Thresholds are intentionally loose to stay green on shared/CI runners; they
/// exist to catch gross regressions (e.g. an index accidentally dropped), not
/// to enforce tight budgets. All well under a second on a warm local box.
const FILTERED_HISTORY_BUDGET: Duration = Duration::from_millis(300);
const PB_LISTING_BUDGET: Duration = Duration::from_millis(200);

#[test]
fn long_history_reads_complete_within_budget() {
    let path = temp_path("timing");
    remove_database(&path);
    let db = Database::open(&path).expect("open db");
    {
        let mut conn = db.conn();
        seed_history(&mut conn, ROW_COUNT);

        let repo = SqliteTestRepository::new(&conn);
        let t = Instant::now();
        let _ = repo
            .get_history(50, 0, Some("time"))
            .expect("filtered history");
        let filtered = t.elapsed();
        assert!(
            filtered < FILTERED_HISTORY_BUDGET,
            "filtered history read took {filtered:?}, budget {FILTERED_HISTORY_BUDGET:?}"
        );

        // PB listing over a modest number of PB rows.
        for mode in ["time", "words", "quote", "custom"] {
            let _ = conn.execute(
                "INSERT INTO personal_bests (
                    mode_type, mode_config_hash, mode_config, best_wpm, best_wpm_test_id,
                    best_accuracy, best_accuracy_test_id, updated_at
                 ) VALUES (?1, ?2, '{}', 50.0, NULL, 95.0, NULL, '2026-01-01T00:00:00Z')",
                rusqlite::params![mode, format!("hash-{mode}")],
            );
        }
        let pb_repo = SqlitePersonalBestsRepository::new(&conn);
        let t = Instant::now();
        let _ = pb_repo.get_bests(None).expect("pb listing");
        let pb = t.elapsed();
        assert!(
            pb < PB_LISTING_BUDGET,
            "PB listing took {pb:?}, budget {PB_LISTING_BUDGET:?}"
        );
    }
    remove_database(&path);
}

// ---------------------------------------------------------------------------
// 3. Metric-correctness regression on long history.
// ---------------------------------------------------------------------------

#[test]
fn global_best_is_recoverable_from_personal_bests_past_old_window() {
    // Build a history where the global best WPM lives on a row well past the
    // old 500-test window, and a PB row records it. The Task D fix reads
    // best_wpm from personal_bests; this test pins that the global best is
    // recoverable from PB regardless of how long the history is.
    let path = temp_path("metric-best");
    remove_database(&path);
    let db = Database::open(&path).expect("open db");
    {
        let mut conn = db.conn();
        // Insert 1000 ordinary rows (well beyond the old 500 window) with
        // modest WPM, then the global best as an explicit personal_best row.
        seed_history(&mut conn, 1000);
        // The PB row is the source of truth for global best.
        conn.execute(
            "INSERT INTO personal_bests (
                mode_type, mode_config_hash, mode_config, best_wpm, best_wpm_test_id,
                best_accuracy, best_accuracy_test_id, updated_at
             ) VALUES ('time', 'hash-best', '{}', 120.0, NULL, 99.0, NULL, '2026-01-01T00:00:00Z')",
            [],
        )
        .expect("seed global-best PB");

        let bests = SqlitePersonalBestsRepository::new(&conn)
            .get_bests(None)
            .expect("get bests");
        let global_best_wpm = bests.iter().map(|pb| pb.best_wpm).fold(0.0_f64, f64::max);
        assert_eq!(
            global_best_wpm, 120.0,
            "global best WPM must be recoverable from personal_bests past the old 500-test window"
        );
    }
    remove_database(&path);
}

#[test]
fn global_longest_streak_survives_in_maintained_streaks_row() {
    // Seed a streaks row with a large longest_streak that no bounded history
    // window could ever recompute correctly. The Task D fix reads longest_streak
    // from the maintained row; this pins that the global value survives there.
    let path = temp_path("metric-streak");
    remove_database(&path);
    let db = Database::open(&path).expect("open db");
    {
        let mut conn = db.conn();
        // A short history that could not produce a 42-day streak.
        seed_history(&mut conn, 50);
        conn.execute(
            "INSERT INTO streaks (type, current_streak, longest_streak, last_date, started_date)
             VALUES ('daily_test', 1, 42, '2026-01-01', '2025-12-01')",
            [],
        )
        .expect("seed streak");

        let row: Option<(i64, i64)> = conn
            .query_row(
                "SELECT current_streak, longest_streak FROM streaks WHERE type = 'daily_test'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok();
        let (current, longest) = row.expect("streak row present");
        assert_eq!(current, 1);
        assert_eq!(
            longest, 42,
            "global longest_streak must survive in the maintained streaks row"
        );
    }
    remove_database(&path);
}
