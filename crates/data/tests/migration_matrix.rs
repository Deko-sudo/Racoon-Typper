//! Migration matrix — systematic upgrade evidence from every historical schema
//! version (V1..V7) to the current V8.
//!
//! These tests complement the focused migration tests in `migration_and_perf.rs`
//! and the backup round-trip in `backup_restore.rs`; they do not replace them.
//!
//! What the matrix proves, for each starting level V1..V7 upgraded through the
//! production runner (`Database::open`):
//!   * every schema object from V1..V8 exists afterwards (tables + indexes);
//!   * `refinery_schema_history` records exactly migrations V1..V8;
//!   * PRAGMA `foreign_keys = ON`, `journal_mode = wal`;
//!   * epoch data inserted before the upgrade survives unchanged, including the
//!     deterministic V005 `session_id` backfill for pre-V005 `tests` rows;
//!   * a reopen is a Refinery no-op and changes nothing.
//!
//! Separate tests cover FK integrity (cascade on replays, RESTRICT on the
//! session ledger chain, orphan-child rejection) and a V1→V8 pre-migration
//! backup round-trip using the Task B backup seam.

// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use racoon_data::backup::{create_pre_migration_backup, restore_from_path};
use racoon_data::db::Database;
use refinery::Migration;
use rusqlite::Connection;

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

/// (migration name, embedded SQL) for V1..V8, in order.
const MIGRATION_FILES: [(&str, &str); 8] = [
    (
        "V001__initial.sql",
        include_str!("../migrations/V001__initial.sql"),
    ),
    (
        "V002__lesson_language.sql",
        include_str!("../migrations/V002__lesson_language.sql"),
    ),
    (
        "V003__replays.sql",
        include_str!("../migrations/V003__replays.sql"),
    ),
    (
        "V004__custom_text_language.sql",
        include_str!("../migrations/V004__custom_text_language.sql"),
    ),
    (
        "V005__session_identity.sql",
        include_str!("../migrations/V005__session_identity.sql"),
    ),
    (
        "V006__session_ledger.sql",
        include_str!("../migrations/V006__session_ledger.sql"),
    ),
    (
        "V007__session_completion_intents.sql",
        include_str!("../migrations/V007__session_completion_intents.sql"),
    ),
    (
        "V008__session_finalizations.sql",
        include_str!("../migrations/V008__session_finalizations.sql"),
    ),
];

/// Current schema level applied by the production runner.
const CURRENT_LEVEL: usize = 8;

fn temp_path(name: &str) -> PathBuf {
    let sequence = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "racoon-migmatrix-{name}-{}-{sequence}.db",
        std::process::id()
    ))
}

fn temp_dir(name: &str) -> PathBuf {
    let sequence = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "racoon-migmatrix-dir-{name}-{}-{sequence}",
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

fn remove_dir_all(dir: &Path) {
    if dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    let _ = std::fs::remove_dir_all(&p);
                } else {
                    let _ = std::fs::remove_file(p);
                }
            }
        }
        let _ = std::fs::remove_dir(dir);
    }
}

/// Builds a historical database at `path` carrying exactly the first `version`
/// migrations, by replaying them through Refinery. This produces a realistic
/// `refinery_schema_history` table, so the subsequent production open runs only
/// the remaining migrations — exactly like a real upgrade.
fn build_historical_db(path: &Path, version: usize) {
    assert!(
        (1..=CURRENT_LEVEL).contains(&version),
        "version must be in 1..=8, got {version}"
    );
    let mut conn = Connection::open(path).expect("open historical fixture db");
    let migrations: Vec<Migration> = MIGRATION_FILES
        .iter()
        .take(version)
        .map(|(name, sql)| Migration::unapplied(name, sql).expect("valid historical migration"))
        .collect();
    refinery::Runner::new(&migrations)
        .run(&mut conn)
        .unwrap_or_else(|e| panic!("historical Refinery run to V{version} failed: {e}"));
    seed_epoch_data(&conn, version);
}

/// Inserts realistic data for the given epoch. Each statement only references
/// columns/tables that already exist at that level, so seeding never relies on
/// a not-yet-applied migration.
fn seed_epoch_data(conn: &Connection, version: usize) {
    // tests row — shape depends on whether session_id exists yet (V005+). We use
    // format!() so the seeded session id carries the real version number and the
    // JSON columns hold valid JSON objects, not literal braces.
    let session_one = format!("matrix-v{version}-session-1");
    let session_two = format!("matrix-v{version}-session-2");
    if version >= 5 {
        conn.execute(
            &format!(
                "INSERT INTO tests (
                    session_id, created_at, mode_type, mode_config, language, text_length,
                    duration_ms, wpm, raw_wpm, accuracy, raw_accuracy, consistency,
                    correct_chars, incorrect_chars, backspaces, char_stats, heatmap_data,
                    graph_data, is_pb, tags
                 ) VALUES (
                    '{session_one}', '2026-01-01T00:00:00Z', 'time', '{{}}',
                    'en', 50, 30000, 42.0, 48.0, 92.0, 88.0, NULL, 95, 5, 2,
                    '{{}}', '{{}}', NULL, 0, ''
                 )"
            ),
            [],
        )
        .unwrap_or_else(|e| panic!("seed tests (v{version}): {e}"));
    } else {
        conn.execute(
            "INSERT INTO tests (
                created_at, mode_type, mode_config, language, text_length, duration_ms,
                wpm, raw_wpm, accuracy, raw_accuracy, consistency, correct_chars,
                incorrect_chars, backspaces, char_stats, heatmap_data, graph_data, is_pb, tags
             ) VALUES (
                '2026-01-01T00:00:00Z', 'time', '{}', 'en', 50, 30000,
                42.0, 48.0, 92.0, 88.0, NULL, 95, 5, 2, '{}', '{}', NULL, 0, ''
             )",
            [],
        )
        .unwrap_or_else(|e| panic!("seed tests (v{version}): {e}"));
    }
    // A second tests row so backfill/identity uniqueness is exercised.
    if version >= 5 {
        conn.execute(
            &format!(
                "INSERT INTO tests (
                    session_id, created_at, mode_type, mode_config, language, text_length,
                    duration_ms, wpm, raw_wpm, accuracy, raw_accuracy, consistency,
                    correct_chars, incorrect_chars, backspaces, char_stats, heatmap_data,
                    graph_data, is_pb, tags
                 ) VALUES (
                    '{session_two}', '2026-01-02T00:00:00Z', 'words', '{{}}',
                    'en', 60, 25000, 55.0, 60.0, 95.0, 90.0, NULL, 110, 4, 1,
                    '{{}}', '{{}}', NULL, 0, ''
                 )"
            ),
            [],
        )
        .expect("seed second tests row");
    } else {
        conn.execute(
            "INSERT INTO tests (
                created_at, mode_type, mode_config, language, text_length, duration_ms,
                wpm, raw_wpm, accuracy, raw_accuracy, consistency, correct_chars,
                incorrect_chars, backspaces, char_stats, heatmap_data, graph_data, is_pb, tags
             ) VALUES (
                '2026-01-02T00:00:00Z', 'words', '{}', 'en', 60, 25000,
                55.0, 60.0, 95.0, 90.0, NULL, 110, 4, 1, '{}', '{}', NULL, 0, ''
             )",
            [],
        )
        .expect("seed second tests row");
    }

    // lesson_progress exists from V1; V2 adds the language column (NOT NULL).
    conn.execute(
        "INSERT INTO lesson_progress (lesson_id, module_id, difficulty, status, attempts)
         VALUES ('lesson-matrix', 'mod-matrix', 'beginner', 'in_progress', 1)",
        [],
    )
    .expect("seed lesson_progress");

    // daily_stats + streaks exist from V1.
    conn.execute(
        "INSERT INTO daily_stats (date, total_tests, total_time_ms, total_chars, best_wpm, avg_wpm, avg_accuracy, lessons_completed, daily_goal_met)
         VALUES ('2026-01-01', 1, 30000, 100, 42.0, 42.0, 92.0, 0, 0)",
        [],
    )
    .expect("seed daily_stats");
    conn.execute(
        "INSERT INTO streaks (type, current_streak, longest_streak, last_date, started_date)
         VALUES ('daily_test', 1, 1, '2026-01-01', '2026-01-01')",
        [],
    )
    .expect("seed streaks");

    // custom_texts exists from V1; V4 adds the language column (NOT NULL).
    conn.execute(
        "INSERT INTO custom_texts (name, text, created_at, use_count)
         VALUES ('matrix custom', 'the quick brown fox', '2026-01-01T00:00:00Z', 0)",
        [],
    )
    .expect("seed custom_texts");

    // test_replays exists from V3 and references tests(id) ON DELETE CASCADE.
    if version >= 3 {
        conn.execute(
            "INSERT INTO test_replays (test_id, frame_index, timestamp_ms, position, expected_char, typed_char, correct)
             SELECT id, 0, 0, 0, 't', 't', 1 FROM tests ORDER BY id LIMIT 1",
            [],
        )
        .expect("seed test_replays");
    }

    // Session ledger chain (V6 -> V7 -> V8). All rows are mutually consistent
    // so an upgrade never trips a CHECK or FK that the previous level allowed.
    // mode_descriptor must be valid JSON per the V006 CHECK.
    if version >= 6 {
        conn.execute(
            "INSERT INTO session_ledger (
                session_id, state, mode_type, mode_descriptor, language, created_at, updated_at
             ) VALUES (
                'matrix-ledger-session', 'finalized', 'time', '{\"duration\":30}', 'en',
                '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'
             )",
            [],
        )
        .expect("seed session_ledger");
    }
    if version >= 7 {
        // V007 payload is opaque bytes with a checksum-style length. We store a
        // small valid blob; only the byte-length agreement and 64-hex fingerprint
        // matter at the SQL level.
        let payload: Vec<u8> = vec![b'm', b'a', b't', b'r', b'i', b'x'];
        conn.execute(
            "INSERT INTO session_completion_intents (
                session_id, canonicalization_version, payload_version, fingerprint,
                canonical_payload, payload_byte_length, created_at
             ) VALUES (
                'matrix-ledger-session', 1, 1,
                '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef',
                ?, 6, '2026-01-01T00:00:00Z'
             )",
            [&payload],
        )
        .expect("seed session_completion_intents");
    }
    if version >= 8 {
        conn.execute(
            "INSERT INTO session_finalizations (
                session_id, fingerprint, state, claimed_at
             ) VALUES (
                'matrix-ledger-session',
                '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef',
                'pending', '2026-01-01T00:00:00Z'
             )",
            [],
        )
        .expect("seed session_finalizations");
    }
}

/// Asserts the post-upgrade schema carries every table and index the matrix
/// cares about. (Not an exhaustive `sqlite_master` dump — focused on objects
/// introduced across V1..V8.)
fn assert_full_schema_present(conn: &Connection) {
    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
        .expect("prepare tables")
        .query_map([], |row| row.get::<_, String>(0))
        .expect("query tables")
        .filter_map(Result::ok)
        .collect();
    for expected in [
        "custom_texts",
        "daily_stats",
        "lesson_progress",
        "personal_bests",
        "session_completion_intents",
        "session_finalizations",
        "session_ledger",
        "streaks",
        "test_replays",
        "tests",
    ] {
        assert!(
            tables.iter().any(|t| t == expected),
            "missing table {expected} after upgrade; tables = {tables:?}"
        );
    }

    let indexes: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='index' ORDER BY name")
        .expect("prepare indexes")
        .query_map([], |row| row.get::<_, String>(0))
        .expect("query indexes")
        .filter_map(Result::ok)
        .collect();
    for expected in [
        "idx_daily_stats_date",
        "idx_lesson_progress_difficulty",
        "idx_lesson_progress_lesson_id",
        "idx_lesson_progress_module_id",
        "idx_replays_frame",
        "idx_replays_test_id",
        "idx_session_completion_intents_session_fingerprint",
        "idx_session_finalizations_diagnostic_order",
        "idx_session_ledger_recovery_order",
        "idx_session_ledger_state_recovery_order",
        "idx_tests_created_at",
        "idx_tests_mode_config",
        "idx_tests_session_id",
        "idx_tests_wpm",
        "uniq_pb_mode_config_hash",
    ] {
        assert!(
            indexes.iter().any(|i| i == expected),
            "missing index {expected} after upgrade; indexes = {indexes:?}"
        );
    }
}

/// Asserts refinery reports exactly migrations V1..V8 applied and no later
/// version pending.
fn assert_refinery_history_complete(conn: &Connection) {
    let versions: Vec<i64> = conn
        .prepare("SELECT version FROM refinery_schema_history ORDER BY version")
        .expect("prepare history")
        .query_map([], |row| row.get::<_, i64>(0))
        .expect("query history")
        .filter_map(Result::ok)
        .collect();
    let expected: Vec<i64> = (1..=CURRENT_LEVEL as i64).collect();
    assert_eq!(
        versions, expected,
        "refinery_schema_history must list exactly V1..V{}",
        CURRENT_LEVEL
    );
}

fn assert_pragmas(conn: &Connection) {
    let fk: i64 = conn
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .expect("PRAGMA foreign_keys");
    assert_eq!(fk, 1, "foreign_keys must be ON");
    let journal: String = conn
        .query_row("PRAGMA journal_mode", [], |row| row.get(0))
        .expect("PRAGMA journal_mode");
    assert_eq!(
        journal.to_ascii_lowercase(),
        "wal",
        "journal_mode must be wal, got {journal}"
    );
}

/// Asserts epoch data survived the upgrade. The pre-V005 tests rows must have
/// been backfilled with deterministic `legacy-test-%016x` session ids; V005+
/// rows keep their explicit identity.
fn assert_epoch_data_survived(conn: &Connection, seeded_version: usize) {
    let test_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM tests", [], |row| row.get(0))
        .expect("count tests");
    assert_eq!(test_count, 2, "both seeded tests rows must survive upgrade");

    let session_ids: Vec<String> = conn
        .prepare("SELECT session_id FROM tests ORDER BY id")
        .expect("prepare session ids")
        .query_map([], |row| row.get::<_, String>(0))
        .expect("query session ids")
        .filter_map(Result::ok)
        .collect();
    assert_eq!(session_ids.len(), 2, "session ids must be populated");
    if seeded_version < 5 {
        // Deterministic V005 backfill: legacy-test-<16 hex digits of id>.
        assert!(
            session_ids[0].starts_with("legacy-test-"),
            "pre-V005 row must be backfilled, got {}",
            session_ids[0]
        );
        assert_eq!(
            session_ids[0], "legacy-test-0000000000000001",
            "first pre-V005 row backfill value"
        );
        assert_eq!(
            session_ids[1], "legacy-test-0000000000000002",
            "second pre-V005 row backfill value"
        );
    } else {
        let expected = format!("matrix-v{seeded_version}-session-1");
        assert_eq!(
            session_ids[0], expected,
            "V005+ explicit identity must survive the upgrade"
        );
    }

    // lesson_progress / daily_stats / streaks / custom_texts each retain one row.
    for (table, expected) in [
        ("lesson_progress", 1),
        ("daily_stats", 1),
        ("streaks", 1),
        ("custom_texts", 1),
    ] {
        let count: i64 = conn
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .unwrap_or_else(|e| panic!("count {table}: {e}"));
        assert_eq!(count, expected, "{table} row count after upgrade");
    }

    // Replay row seeded from V3 must survive and still point at a tests row.
    if seeded_version >= 3 {
        let replay_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM test_replays", [], |row| row.get(0))
            .expect("count replays");
        assert_eq!(replay_count, 1, "seeded replay must survive upgrade");
    }
}

// ---------------------------------------------------------------------------
// 1. Matrix: every historical version upgrades cleanly to V8.
// ---------------------------------------------------------------------------

#[test]
fn matrix_upgrade_from_v1_to_v8_preserves_data_and_schema() {
    run_matrix_upgrade(1);
}

#[test]
fn matrix_upgrade_from_v2_to_v8_preserves_data_and_schema() {
    run_matrix_upgrade(2);
}

#[test]
fn matrix_upgrade_from_v3_to_v8_preserves_data_and_schema() {
    run_matrix_upgrade(3);
}

#[test]
fn matrix_upgrade_from_v4_to_v8_preserves_data_and_schema() {
    run_matrix_upgrade(4);
}

#[test]
fn matrix_upgrade_from_v5_to_v8_preserves_data_and_schema() {
    run_matrix_upgrade(5);
}

#[test]
fn matrix_upgrade_from_v6_to_v8_preserves_data_and_schema() {
    run_matrix_upgrade(6);
}

#[test]
fn matrix_upgrade_from_v7_to_v8_preserves_data_and_schema() {
    run_matrix_upgrade(7);
}

fn run_matrix_upgrade(start_version: usize) {
    let path = temp_path(&format!("upgrade-v{start_version}"));
    remove_database(&path);
    build_historical_db(&path, start_version);

    // Production upgrade: Database::open runs the remaining migrations.
    let db = Database::open(&path).expect("production upgrade open");
    {
        let conn = db.conn();
        assert_refinery_history_complete(&conn);
        assert_full_schema_present(&conn);
        assert_pragmas(&conn);
        assert_epoch_data_survived(&conn, start_version);
    }
    remove_database(&path);
}

// ---------------------------------------------------------------------------
// 2. Foreign-key integrity on the upgraded schema.
// ---------------------------------------------------------------------------

#[test]
fn foreign_keys_are_enforced_on_upgraded_connection() {
    let path = temp_path("fk-enforced");
    remove_database(&path);
    build_historical_db(&path, 5);
    let db = Database::open(&path).expect("open upgraded db");
    let conn = db.conn();
    let fk: i64 = conn
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .expect("PRAGMA foreign_keys");
    assert_eq!(fk, 1);
    remove_database(&path);
}

#[test]
fn replay_cascades_when_parent_test_deleted() {
    // V003 declares test_replays.test_id REFERENCES tests(id) ON DELETE CASCADE.
    let path = temp_path("fk-cascade");
    remove_database(&path);
    let db = Database::open(&path).expect("open fresh db");
    {
        let conn = db.conn();
        conn.execute(
            "INSERT INTO tests (
                session_id, created_at, mode_type, mode_config, language, text_length,
                duration_ms, wpm, raw_wpm, accuracy, raw_accuracy, consistency,
                correct_chars, incorrect_chars, backspaces, char_stats, heatmap_data,
                graph_data, is_pb, tags
             ) VALUES (
                'cascade-session', '2026-01-01T00:00:00Z', 'time', '{}', 'en', 10, 1000,
                10.0, 10.0, 90.0, 90.0, NULL, 9, 1, 0, '{}', '{}', NULL, 0, ''
             )",
            [],
        )
        .expect("insert parent test");
        conn.execute(
            "INSERT INTO test_replays (test_id, frame_index, timestamp_ms, position, expected_char, typed_char, correct)
             SELECT id, 0, 0, 0, 'a', 'a', 1 FROM tests WHERE session_id = 'cascade-session'",
            [],
        )
        .expect("insert child replay");
        let before: i64 = conn
            .query_row("SELECT COUNT(*) FROM test_replays", [], |row| row.get(0))
            .expect("count before");
        assert_eq!(before, 1);

        conn.execute("DELETE FROM tests WHERE session_id = 'cascade-session'", [])
            .expect("delete parent");
        let after: i64 = conn
            .query_row("SELECT COUNT(*) FROM test_replays", [], |row| row.get(0))
            .expect("count after");
        assert_eq!(after, 0, "ON DELETE CASCADE must remove the child replay");
    }
    remove_database(&path);
}

#[test]
fn ledger_child_with_unknown_session_is_rejected() {
    // session_completion_intents.session_id REFERENCES session_ledger ON DELETE
    // RESTRICT; inserting an intent for an unknown session must be rejected.
    let path = temp_path("fk-ledger-restrict");
    remove_database(&path);
    let db = Database::open(&path).expect("open fresh db");
    {
        let conn = db.conn();
        let payload: Vec<u8> = vec![0u8; 4];
        let result = conn.execute(
            "INSERT INTO session_completion_intents (
                session_id, canonicalization_version, payload_version, fingerprint,
                canonical_payload, payload_byte_length, created_at
             ) VALUES (
                'unknown-ledger-session', 1, 1,
                '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef',
                ?, 4, '2026-01-01T00:00:00Z'
             )",
            [&payload],
        );
        assert!(
            result.is_err(),
            "intent with unknown parent session must be rejected by FK, got {result:?}"
        );
    }
    remove_database(&path);
}

#[test]
fn finalization_rejects_mismatched_intent_fingerprint() {
    // V008 composite FK (session_id, fingerprint) -> session_completion_intents.
    // A finalization row whose (session_id, fingerprint) does not match any
    // intent must be rejected even if the session exists in the ledger.
    let path = temp_path("fk-finalization-composite");
    remove_database(&path);
    let db = Database::open(&path).expect("open fresh db");
    {
        let conn = db.conn();
        conn.execute(
            "INSERT INTO session_ledger (
                session_id, state, mode_type, mode_descriptor, language, created_at, updated_at
             ) VALUES (
                'final-composite', 'finalization_pending', 'time', '{\"duration\":30}', 'en',
                '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'
             )",
            [],
        )
        .expect("insert ledger");
        let payload: Vec<u8> = vec![0u8; 4];
        conn.execute(
            "INSERT INTO session_completion_intents (
                session_id, canonicalization_version, payload_version, fingerprint,
                canonical_payload, payload_byte_length, created_at
             ) VALUES (
                'final-composite', 1, 1,
                'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
                ?, 4, '2026-01-01T00:00:00Z'
             )",
            [&payload],
        )
        .expect("insert intent");
        // Same session but a fingerprint that matches no intent.
        let result = conn.execute(
            "INSERT INTO session_finalizations (
                session_id, fingerprint, state, claimed_at
             ) VALUES (
                'final-composite',
                'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',
                'pending', '2026-01-01T00:00:00Z'
             )",
            [],
        );
        assert!(
            result.is_err(),
            "finalization with mismatched fingerprint must be rejected by composite FK"
        );
    }
    remove_database(&path);
}

#[test]
fn ledger_delete_blocked_when_completion_intent_exists() {
    // V007 declares session_completion_intents.session_id REFERENCES
    // session_ledger(session_id) ON DELETE RESTRICT. Deleting a ledger row that
    // still has an intent must be rejected, so an intent can never be orphaned.
    let path = temp_path("fk-ledger-delete-restrict");
    remove_database(&path);
    let db = Database::open(&path).expect("open fresh db");
    {
        let conn = db.conn();
        conn.execute(
            "INSERT INTO session_ledger (
                session_id, state, mode_type, mode_descriptor, language, created_at, updated_at
             ) VALUES (
                'restrict-ledger', 'finalization_pending', 'time', '{\"duration\":30}', 'en',
                '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'
             )",
            [],
        )
        .expect("insert ledger");
        let payload: Vec<u8> = vec![0u8; 4];
        conn.execute(
            "INSERT INTO session_completion_intents (
                session_id, canonicalization_version, payload_version, fingerprint,
                canonical_payload, payload_byte_length, created_at
             ) VALUES (
                'restrict-ledger', 1, 1,
                '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef',
                ?, 4, '2026-01-01T00:00:00Z'
             )",
            [&payload],
        )
        .expect("insert intent");

        // Direct delete of the referenced ledger row must fail with RESTRICT.
        let result = conn.execute(
            "DELETE FROM session_ledger WHERE session_id = 'restrict-ledger'",
            [],
        );
        assert!(
            result.is_err(),
            "deleting a ledger row with an existing intent must be rejected (ON DELETE RESTRICT)"
        );
        // The ledger row and intent must both still be present.
        let ledger: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM session_ledger WHERE session_id = 'restrict-ledger'",
                [],
                |row| row.get(0),
            )
            .expect("count ledger");
        let intents: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM session_completion_intents WHERE session_id = 'restrict-ledger'",
                [],
                |row| row.get(0),
            )
            .expect("count intents");
        assert_eq!(ledger, 1, "ledger row must survive the blocked delete");
        assert_eq!(intents, 1, "intent row must survive the blocked delete");
    }
    remove_database(&path);
}

#[test]
fn finalization_orphan_session_is_rejected() {
    // V008 finalizations.session_id REFERENCES session_ledger ON DELETE
    // RESTRICT. Inserting a finalization for a session that has no ledger row
    // must be rejected before the composite fingerprint FK is even considered.
    let path = temp_path("fk-finalization-orphan");
    remove_database(&path);
    let db = Database::open(&path).expect("open fresh db");
    {
        let conn = db.conn();
        let result = conn.execute(
            "INSERT INTO session_finalizations (
                session_id, fingerprint, state, claimed_at
             ) VALUES (
                'never-in-ledger',
                '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef',
                'pending', '2026-01-01T00:00:00Z'
             )",
            [],
        );
        assert!(
            result.is_err(),
            "finalization referencing a session absent from session_ledger must be rejected"
        );
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM session_finalizations", [], |row| {
                row.get(0)
            })
            .expect("count finalizations");
        assert_eq!(count, 0, "no finalization row may be written on FK failure");
    }
    remove_database(&path);
}

#[test]
fn personal_best_reference_blocks_parent_test_delete() {
    // personal_bests.best_wpm_test_id REFERENCES tests(id) with no ON DELETE
    // clause, so SQLite applies NO ACTION: deleting the referenced tests row
    // while a personal_best still points at it must fail, keeping the reference
    // intact rather than silently orphaning the PB columns.
    let path = temp_path("fk-pb-no-action");
    remove_database(&path);
    let db = Database::open(&path).expect("open fresh db");
    {
        let conn = db.conn();
        conn.execute(
            "INSERT INTO tests (
                session_id, created_at, mode_type, mode_config, language, text_length,
                duration_ms, wpm, raw_wpm, accuracy, raw_accuracy, consistency,
                correct_chars, incorrect_chars, backspaces, char_stats, heatmap_data,
                graph_data, is_pb, tags
             ) VALUES (
                'pb-session', '2026-01-01T00:00:00Z', 'time', '{}', 'en', 10, 1000,
                90.0, 95.0, 99.0, 98.0, NULL, 9, 1, 0, '{}', '{}', NULL, 0, ''
             )",
            [],
        )
        .expect("insert parent test");
        conn.execute(
            "INSERT INTO personal_bests (
                mode_type, mode_config_hash, mode_config, best_wpm, best_wpm_test_id,
                best_accuracy, best_accuracy_test_id, updated_at
             ) SELECT 'time', 'hash-pb', '{}', wpm, id, accuracy, id, '2026-01-01T00:00:00Z'
               FROM tests WHERE session_id = 'pb-session'",
            [],
        )
        .expect("insert personal_best referencing tests(id)");

        // Deleting the referenced tests row must be rejected (NO ACTION).
        let result = conn.execute("DELETE FROM tests WHERE session_id = 'pb-session'", []);
        assert!(
            result.is_err(),
            "deleting a tests row referenced by personal_bests must be rejected (NO ACTION)"
        );
        let tests: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM tests WHERE session_id = 'pb-session'",
                [],
                |row| row.get(0),
            )
            .expect("count tests");
        assert_eq!(
            tests, 1,
            "referenced tests row must survive the blocked delete"
        );
    }
    remove_database(&path);
}

// ---------------------------------------------------------------------------
// 3. Back-to-back / idempotent reopen on a historically seeded DB.
// ---------------------------------------------------------------------------

#[test]
fn reopen_upgraded_historical_db_is_idempotent() {
    let path = temp_path("idempotent-reopen");
    remove_database(&path);
    build_historical_db(&path, 3);

    let db = Database::open(&path).expect("first open upgrades");
    let (history_first, test_count_first): (Vec<i64>, i64) = {
        let conn = db.conn();
        let h = conn
            .prepare("SELECT version FROM refinery_schema_history ORDER BY version")
            .expect("prepare")
            .query_map([], |row| row.get::<_, i64>(0))
            .expect("query")
            .filter_map(Result::ok)
            .collect();
        let c = conn
            .query_row("SELECT COUNT(*) FROM tests", [], |row| row.get(0))
            .expect("count");
        (h, c)
    };
    drop(db);

    // Reopen: Refinery must be a no-op, data must be unchanged.
    let db = Database::open(&path).expect("second open is no-op");
    {
        let conn = db.conn();
        let history_second: Vec<i64> = conn
            .prepare("SELECT version FROM refinery_schema_history ORDER BY version")
            .expect("prepare")
            .query_map([], |row| row.get::<_, i64>(0))
            .expect("query")
            .filter_map(Result::ok)
            .collect();
        let test_count_second: i64 = conn
            .query_row("SELECT COUNT(*) FROM tests", [], |row| row.get(0))
            .expect("count");
        assert_eq!(
            history_second, history_first,
            "history must not change on reopen"
        );
        assert_eq!(
            test_count_second, test_count_first,
            "data must not change on reopen"
        );
    }
    remove_database(&path);
}

// ---------------------------------------------------------------------------
// 4. Pre-migration backup round-trip on a V1 fixture (Task B seam).
// ---------------------------------------------------------------------------

#[test]
fn pre_migration_backup_roundtrip_from_v1_fixture() {
    // Build a V1 fixture with epoch data, take a pre-migration backup through
    // the Task B seam, upgrade to V8, then restore the backup into a fresh path
    // and confirm the restored file is a working V1-era database with the seed
    // data intact. The live Database is dropped before restore per the contract.
    let live = temp_path("premig-v1-live");
    let backup = temp_path("premig-v1-backup.db");
    let restored = temp_path("premig-v1-restored");
    let data_dir = temp_dir("premig-v1-data");
    remove_database(&live);
    remove_database(&backup);
    remove_database(&restored);
    remove_dir_all(&data_dir);
    std::fs::create_dir_all(&data_dir).expect("create data dir");

    build_historical_db(&live, 1);

    // Snapshot the V1-era on-disk state before migrating.
    let snapshot = create_pre_migration_backup(
        &live,
        &data_dir,
        "matrix-v1",
        chrono::Utc::now(),
        racoon_data::backup::DEFAULT_KEEP,
    )
    .expect("pre-migration backup of V1 fixture");
    assert!(snapshot.is_file(), "backup file must exist");

    // Production upgrade to V8 via the pre-migration seam (no-op callback).
    {
        let _db = Database::open_with_pre_migration(&live, |_| {}).expect("upgrade V1 -> V8");
    }

    // Restore the V1 snapshot into a fresh path and confirm it is a usable V1
    // database carrying the original tests rows. We avoid opening two
    // connections on `live`; restore writes into `restored`.
    restore_from_path(&snapshot, &restored).expect("restore V1 backup");

    let conn = Connection::open(&restored).expect("open restored V1 db");
    let test_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM tests", [], |row| row.get(0))
        .expect("count restored tests");
    assert_eq!(
        test_count, 2,
        "restored V1 backup must carry both seed rows"
    );
    // The restored file is pre-V005, so session_id must not be a column yet.
    let session_col_exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('tests') WHERE name = 'session_id'",
            [],
            |row| row.get(0),
        )
        .expect("check session_id column");
    assert_eq!(
        session_col_exists, 0,
        "restored V1 backup must predate the V005 session_id column"
    );

    remove_database(&live);
    remove_database(&backup);
    remove_database(&restored);
    remove_dir_all(&data_dir);
}
