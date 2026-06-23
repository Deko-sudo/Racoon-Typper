//! Migration + large DB integration tests.

use racoon_data::db::Database;
use racoon_data::repository::{
    DailyStatsRepository, SqliteDailyStatsRepository, SqliteTestRepository, TestRepository,
};
use racoon_domain::TestRecord;

fn make_record(n: i64) -> TestRecord {
    TestRecord {
        created_at: format!("2026-06-{:02}T12:00:00Z", (n % 30) + 1),
        mode_type: "time".to_string(),
        mode_config: serde_json::json!({"duration": 30}),
        language: "en".to_string(),
        text_length: 50,
        duration_ms: 30000,
        wpm: 30.0 + (n % 50) as f64,
        raw_wpm: 35.0 + (n % 50) as f64,
        accuracy: 80.0 + (n % 20) as f64,
        raw_accuracy: 75.0 + (n % 20) as f64,
        consistency: None,
        correct_chars: 95,
        incorrect_chars: 5,
        backspaces: 2,
        char_stats: serde_json::json!({}),
        heatmap_data: serde_json::json!({}),
        graph_data: None,
        is_pb: false,
        tags: "".to_string(),
    }
}

// ── Migration tests ──

#[test]
fn migration_v1_to_v2_adds_language_column() {
    let db = Database::open_in_memory().unwrap();
    let conn = db.conn();
    // Check that lesson_progress has language column
    let result: rusqlite::Result<String> =
        conn.query_row("SELECT language FROM lesson_progress LIMIT 1", [], |row| {
            row.get(0)
        });
    // Should fail with no rows, not with column-not-found
    match result {
        Err(rusqlite::Error::QueryReturnedNoRows) => {} // OK — column exists
        Ok(_) => {}                                     // OK — data exists
        Err(e) => panic!("language column missing: {}", e),
    }
}

#[test]
fn migration_v3_adds_replays_table() {
    let db = Database::open_in_memory().unwrap();
    let conn = db.conn();
    // Check test_replays table exists
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM test_replays", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn migration_idempotent_safe() {
    // Opening DB twice should not fail
    let path = std::env::temp_dir().join("racoon_test_mig_idempotent.db");
    let _ = std::fs::remove_file(&path);
    let _db1 = Database::open(&path).unwrap();
    let db2 = Database::open(&path).unwrap();
    // Should not panic or error
    drop(db2.conn());
    let _ = std::fs::remove_file(&path);
}

#[test]
fn migration_all_tables_exist() {
    let db = Database::open_in_memory().unwrap();
    let conn = db.conn();
    for table in &[
        "tests",
        "personal_bests",
        "lesson_progress",
        "daily_stats",
        "streaks",
        "custom_texts",
        "test_replays",
    ] {
        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                rusqlite::params![table],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(exists, 1, "Table {} not found", table);
    }
}

#[test]
fn migration_all_indexes_exist() {
    let db = Database::open_in_memory().unwrap();
    let conn = db.conn();
    for idx in &[
        "idx_tests_created_at",
        "idx_tests_mode_config",
        "idx_tests_wpm",
        "uniq_pb_mode_config_hash",
        "idx_lesson_progress_lesson_id",
        "idx_daily_stats_date",
        "idx_streaks_type",
        "idx_replays_test_id",
        "idx_replays_frame",
    ] {
        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name=?1",
                rusqlite::params![idx],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(exists, 1, "Index {} not found", idx);
    }
}

#[test]
fn migration_existing_data_preserved() {
    let path = std::env::temp_dir().join("racoon_test_mig_preserve.db");
    let _ = std::fs::remove_file(&path);

    // Create DB and add data
    {
        let db = Database::open(&path).unwrap();
        let conn = db.conn();
        let repo = SqliteTestRepository::new(&conn);
        repo.save_test(make_record(1)).unwrap();
        repo.save_test(make_record(2)).unwrap();
    }

    // Reopen — should not lose data
    {
        let db = Database::open(&path).unwrap();
        let conn = db.conn();
        let repo = SqliteTestRepository::new(&conn);
        let count = repo.get_count(None).unwrap();
        assert_eq!(count, 2);
    }

    let _ = std::fs::remove_file(&path);
}

#[test]
fn migration_wal_mode_preserved() {
    let path = std::env::temp_dir().join("racoon_test_mig_wal.db");
    let _ = std::fs::remove_file(&path);
    let db = Database::open(&path).unwrap();
    let conn = db.conn();
    let mode: String = conn
        .query_row("PRAGMA journal_mode", [], |row| row.get(0))
        .unwrap();
    assert_eq!(mode, "wal");
    let _ = std::fs::remove_file(&path);
}

// ── Large DB tests ──

#[test]
fn large_db_1000_tests() {
    let db = Database::open_in_memory().unwrap();
    let conn = db.conn();
    let repo = SqliteTestRepository::new(&conn);

    let start = std::time::Instant::now();
    for i in 0..1000 {
        repo.save_test(make_record(i)).unwrap();
    }
    let insert_time = start.elapsed();

    let start = std::time::Instant::now();
    let count = repo.get_count(None).unwrap();
    let count_time = start.elapsed();

    assert_eq!(count, 1000);

    let start = std::time::Instant::now();
    let history = repo.get_history(100, 0, None).unwrap();
    let history_time = start.elapsed();

    assert_eq!(history.len(), 100);

    // Performance assertions (generous — CI may be slow)
    assert!(insert_time.as_secs() < 10, "Insert took {:?}", insert_time);
    assert!(count_time.as_millis() < 500, "Count took {:?}", count_time);
    assert!(
        history_time.as_millis() < 500,
        "History took {:?}",
        history_time
    );
}

#[test]
fn large_db_5000_tests() {
    let db = Database::open_in_memory().unwrap();
    let conn = db.conn();
    let repo = SqliteTestRepository::new(&conn);

    for i in 0..5000 {
        repo.save_test(make_record(i)).unwrap();
    }

    let start = std::time::Instant::now();
    let count = repo.get_count(None).unwrap();
    let count_time = start.elapsed();
    assert_eq!(count, 5000);
    assert!(
        count_time.as_millis() < 1000,
        "Count 5000 took {:?}",
        count_time
    );

    let start = std::time::Instant::now();
    let history = repo.get_history(50, 0, None).unwrap();
    let history_time = start.elapsed();
    assert_eq!(history.len(), 50);
    assert!(
        history_time.as_millis() < 500,
        "History 50 from 5000 took {:?}",
        history_time
    );
}

#[test]
fn large_db_10000_tests() {
    let db = Database::open_in_memory().unwrap();
    let conn = db.conn();
    let repo = SqliteTestRepository::new(&conn);

    for i in 0..10000 {
        repo.save_test(make_record(i)).unwrap();
    }

    let start = std::time::Instant::now();
    let count = repo.get_count(None).unwrap();
    let count_time = start.elapsed();
    assert_eq!(count, 10000);
    assert!(
        count_time.as_millis() < 2000,
        "Count 10000 took {:?}",
        count_time
    );

    let start = std::time::Instant::now();
    let history = repo.get_history(100, 0, None).unwrap();
    let history_time = start.elapsed();
    assert_eq!(history.len(), 100);
    assert!(
        history_time.as_millis() < 1000,
        "History 100 from 10000 took {:?}",
        history_time
    );
}

#[test]
fn large_db_daily_stats_aggregation() {
    let db = Database::open_in_memory().unwrap();
    let conn = db.conn();
    let daily_repo = SqliteDailyStatsRepository::new(&conn);

    // Simulate 30 days of tests
    for day in 0..30 {
        let date = format!("2026-06-{:02}", day + 1);
        for _ in 0..10 {
            daily_repo
                .update_after_test(&date, 30000, 100, 40.0, 95.0)
                .unwrap();
        }
    }

    let start = std::time::Instant::now();
    let range = daily_repo.get_range("2026-06-01", "2026-06-30").unwrap();
    let range_time = start.elapsed();

    assert_eq!(range.len(), 30);
    assert!(
        range_time.as_millis() < 500,
        "Range aggregation took {:?}",
        range_time
    );

    // Each day should have 10 tests
    for day in &range {
        assert_eq!(day.total_tests, 10);
    }
}

#[test]
fn large_db_streak_calculation() {
    use racoon_core::StreakEngine;

    // Simulate 100 days of daily practice
    let dates: Vec<String> = (0..100)
        .map(|i| format!("2026-{:02}-{:02}", (i / 30) + 1, (i % 30) + 1))
        .collect();

    let start = std::time::Instant::now();
    let (current, longest) = StreakEngine::streak_from_dates(&dates);
    let calc_time = start.elapsed();

    assert!(longest > 0);
    assert!(
        calc_time.as_millis() < 100,
        "Streak calc took {:?}",
        calc_time
    );
}

// ── Settings persistence tests ──

#[test]
fn settings_persistence_roundtrip() {
    use racoon_data::repository::SettingsStore;
    let path = std::env::temp_dir().join("racoon_test_settings_persist.toml");
    let _ = std::fs::remove_file(&path);

    let store = SettingsStore::new(path.clone());

    // Save settings
    store
        .set("theme", toml::Value::String("racoon_dark".to_string()))
        .unwrap();
    store.set("font_size", toml::Value::Integer(28)).unwrap();
    store
        .set("show_keyboard_trainer", toml::Value::Boolean(false))
        .unwrap();

    // Reload
    let store2 = SettingsStore::new(path.clone());
    let settings = store2.load().unwrap();

    assert_eq!(settings.theme, "racoon_dark");
    assert_eq!(settings.font_size, 28);
    assert!(!settings.show_keyboard_trainer);

    let _ = std::fs::remove_file(&path);
}

#[test]
fn settings_persistence_new_fields_default() {
    use racoon_data::repository::SettingsStore;
    let path = std::env::temp_dir().join("racoon_test_settings_new_fields.toml");
    let _ = std::fs::remove_file(&path);

    let store = SettingsStore::new(path.clone());
    let settings = store.load().unwrap();

    // New fields should default to true
    assert!(settings.show_keyboard_trainer);
    assert!(settings.show_hand_guide);
    assert!(settings.show_layout_warnings);
    assert!(settings.show_capslock_warnings);

    let _ = std::fs::remove_file(&path);
}

#[test]
fn settings_persistence_update_single_field() {
    use racoon_data::repository::SettingsStore;
    let path = std::env::temp_dir().join("racoon_test_settings_update.toml");
    let _ = std::fs::remove_file(&path);

    let store = SettingsStore::new(path.clone());
    store.load().unwrap(); // create defaults

    store.set("font_size", toml::Value::Integer(32)).unwrap();
    store
        .set("caret_style", toml::Value::String("block".to_string()))
        .unwrap();

    let store2 = SettingsStore::new(path.clone());
    let settings = store2.load().unwrap();
    assert_eq!(settings.font_size, 32);
    assert_eq!(settings.caret_style, "block");
    // Other fields should be defaults
    assert_eq!(settings.theme, "serika_dark");

    let _ = std::fs::remove_file(&path);
}

// ── Lesson progress upgrade tests ──

#[test]
fn lesson_progress_upgrade_after_completion() {
    use racoon_data::repository::{LessonRepository, SqliteLessonRepository};
    let conn = Box::leak(Box::new(rusqlite::Connection::open_in_memory().unwrap()));
    racoon_data::db::run_migrations(conn);
    let repo = SqliteLessonRepository::new(conn);

    // Create progress
    repo.create_progress("en_m1_l1", "en_m1", "en", "beginner")
        .unwrap();
    let p = repo.get_lesson_progress("en_m1_l1").unwrap().unwrap();
    assert_eq!(p.status, "not_started");

    // Update with attempt
    repo.update_progress("en_m1_l1", 30.0, 85.0).unwrap();
    let p = repo.get_lesson_progress("en_m1_l1").unwrap().unwrap();
    assert_eq!(p.status, "in_progress");
    assert_eq!(p.attempts, 1);

    // Complete lesson
    repo.complete_lesson("en_m1_l1", 45.0, 95.0).unwrap();
    let p = repo.get_lesson_progress("en_m1_l1").unwrap().unwrap();
    assert_eq!(p.status, "completed");
    assert_eq!(p.attempts, 2);
    assert!((p.best_wpm - 45.0).abs() < 0.01);
    assert!((p.best_accuracy - 95.0).abs() < 0.01);
}

#[test]
fn lesson_progress_course_progress_after_multiple_completions() {
    use racoon_data::repository::{LessonRepository, SqliteLessonRepository};
    let conn = Box::leak(Box::new(rusqlite::Connection::open_in_memory().unwrap()));
    racoon_data::db::run_migrations(conn);
    let repo = SqliteLessonRepository::new(conn);

    // Create 5 lessons across 2 modules
    for i in 1..=3 {
        repo.create_progress(&format!("en_m1_l{}", i), "en_m1", "en", "beginner")
            .unwrap();
    }
    for i in 1..=2 {
        repo.create_progress(&format!("en_m2_l{}", i), "en_m2", "en", "intermediate")
            .unwrap();
    }

    // Complete 3 of 5
    repo.complete_lesson("en_m1_l1", 40.0, 95.0).unwrap();
    repo.complete_lesson("en_m1_l2", 45.0, 92.0).unwrap();
    repo.complete_lesson("en_m2_l1", 50.0, 90.0).unwrap();

    let cp = repo.get_course_progress("en").unwrap();
    assert_eq!(cp.total_lessons, 5);
    assert_eq!(cp.completed_lessons, 3);
    assert!((cp.overall_progress - 60.0).abs() < 0.01);
}

#[test]
fn dashboard_aggregation_accuracy() {
    let db = Database::open_in_memory().unwrap();
    let conn = db.conn();
    let test_repo = SqliteTestRepository::new(&conn);
    let daily_repo = SqliteDailyStatsRepository::new(&conn);

    // Save 10 tests on same day
    for i in 0..10 {
        let record = make_record(i);
        test_repo.save_test(record).unwrap();
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        daily_repo
            .update_after_test(&today, 30000, 100, 30.0 + i as f64, 85.0 + i as f64)
            .unwrap();
    }

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let stats = daily_repo.get_day(&today).unwrap().unwrap();
    assert_eq!(stats.total_tests, 10);
    // avg_wpm = (30+31+...+39)/10 = 34.5
    assert!((stats.avg_wpm - 34.5).abs() < 1.0);
    // best_wpm = 39
    assert!((stats.best_wpm - 39.0).abs() < 0.01);
}

#[test]
fn analytics_export_csv_contains_all_fields() {
    let export = racoon_core::analytics::export_csv(&[
        vec![
            "Date".to_string(),
            "Mode".to_string(),
            "WPM".to_string(),
            "Accuracy".to_string(),
        ],
        vec![
            "2026-06-01".to_string(),
            "time".to_string(),
            "40.0".to_string(),
            "95.0".to_string(),
        ],
        vec![
            "2026-06-02".to_string(),
            "words".to_string(),
            "45.0".to_string(),
            "90.0".to_string(),
        ],
    ]);

    assert!(export.contains("Date,Mode,WPM,Accuracy"));
    assert!(export.contains("2026-06-01,time,40.0,95.0"));
    assert!(export.contains("2026-06-02,words,45.0,90.0"));
}

#[test]
fn analytics_export_json_valid_structure() {
    let data = serde_json::json!({
        "tests": [{"date": "2026-06-01", "wpm": 40.0}],
    });
    let json = racoon_core::analytics::export_json(&data);
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed["tests"].is_array());
    assert_eq!(parsed["tests"][0]["wpm"], 40.0);
}
