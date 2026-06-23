//! Database validation + stress tests for v1.0.0 release.

use racoon_data::repository::ReplayFrame;
use racoon_data::repository::{
    CustomTextRepository, DailyStatsRepository, LessonRepository, PersonalBestsRepository,
    ReplayRepository, SettingsStore, SqliteCustomTextRepository, SqliteDailyStatsRepository,
    SqliteLessonRepository, SqlitePersonalBestsRepository, SqliteReplayRepository,
    SqliteTestRepository, TestRepository,
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

// ── Database validation tests ──

#[test]
fn v001_to_v003_all_tables_present() {
    let db = racoon_data::db::Database::open_in_memory().unwrap();
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
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                rusqlite::params![table],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "Missing table: {}", table);
    }
}

#[test]
fn v002_language_column_exists() {
    let db = racoon_data::db::Database::open_in_memory().unwrap();
    let conn = db.conn();
    let _: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM lesson_progress WHERE language = 'en'",
            [],
            |row| row.get(0),
        )
        .unwrap();
}

#[test]
fn v003_replays_table_structure() {
    let db = racoon_data::db::Database::open_in_memory().unwrap();
    let conn = db.conn();
    // Check all columns exist
    let _: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM test_replays WHERE test_id = 0 AND frame_index = 0 AND timestamp_ms = 0 AND position = 0 AND expected_char = '' AND correct = 1",
            [],
            |row| row.get(0),
        )
        .unwrap();
}

#[test]
fn history_preserved_across_reopen() {
    let path = std::env::temp_dir().join("racoon_v1_validation_history.db");
    let _ = std::fs::remove_file(&path);
    {
        let db = racoon_data::db::Database::open(&path).unwrap();
        let conn = db.conn();
        let repo = SqliteTestRepository::new(&conn);
        for i in 0..50 {
            repo.save_test(make_record(i)).unwrap();
        }
    }
    {
        let db = racoon_data::db::Database::open(&path).unwrap();
        let conn = db.conn();
        let repo = SqliteTestRepository::new(&conn);
        assert_eq!(repo.get_count(None).unwrap(), 50);
    }
    let _ = std::fs::remove_file(&path);
}

#[test]
fn personal_bests_preserved_across_reopen() {
    let path = std::env::temp_dir().join("racoon_v1_validation_pb.db");
    let _ = std::fs::remove_file(&path);
    {
        let db = racoon_data::db::Database::open(&path).unwrap();
        let conn = db.conn();
        let test_repo = SqliteTestRepository::new(&conn);
        let pb_repo = SqlitePersonalBestsRepository::new(&conn);
        let test_id = test_repo.save_test(make_record(1)).unwrap();
        pb_repo
            .check_and_update("time", "{}", 40.0, 90.0, test_id)
            .unwrap();
    }
    {
        let db = racoon_data::db::Database::open(&path).unwrap();
        let conn = db.conn();
        let pb_repo = SqlitePersonalBestsRepository::new(&conn);
        let bests = pb_repo.get_bests(None).unwrap();
        assert!(!bests.is_empty());
    }
    let _ = std::fs::remove_file(&path);
}

#[test]
fn custom_texts_preserved_across_reopen() {
    let path = std::env::temp_dir().join("racoon_v1_validation_ct.db");
    let _ = std::fs::remove_file(&path);
    {
        let db = racoon_data::db::Database::open(&path).unwrap();
        let conn = db.conn();
        let repo = SqliteCustomTextRepository::new(&conn);
        repo.save("My Text", "Hello world this is a test").unwrap();
    }
    {
        let db = racoon_data::db::Database::open(&path).unwrap();
        let conn = db.conn();
        let repo = SqliteCustomTextRepository::new(&conn);
        let texts = repo.get_all(100).unwrap();
        assert!(!texts.is_empty());
        assert_eq!(texts[0].name, "My Text");
    }
    let _ = std::fs::remove_file(&path);
}

#[test]
fn lesson_progress_preserved_across_reopen() {
    let path = std::env::temp_dir().join("racoon_v1_validation_lp.db");
    let _ = std::fs::remove_file(&path);
    {
        let db = racoon_data::db::Database::open(&path).unwrap();
        let conn = db.conn();
        let repo = SqliteLessonRepository::new(&conn);
        repo.create_progress("en_m1_l1", "en_m1", "en", "beginner")
            .unwrap();
        repo.complete_lesson("en_m1_l1", 45.0, 95.0).unwrap();
    }
    {
        let db = racoon_data::db::Database::open(&path).unwrap();
        let conn = db.conn();
        let repo = SqliteLessonRepository::new(&conn);
        let p = repo.get_lesson_progress("en_m1_l1").unwrap().unwrap();
        assert_eq!(p.status, "completed");
        assert!((p.best_wpm - 45.0).abs() < 0.01);
    }
    let _ = std::fs::remove_file(&path);
}

#[test]
fn replay_preserved_across_reopen() {
    let path = std::env::temp_dir().join("racoon_v1_validation_replay.db");
    let _ = std::fs::remove_file(&path);
    {
        let db = racoon_data::db::Database::open(&path).unwrap();
        let conn = db.conn();
        let test_repo = SqliteTestRepository::new(&conn);
        let test_id = test_repo.save_test(make_record(1)).unwrap();
        let replay_repo = SqliteReplayRepository::new(&conn);
        let frames = vec![
            ReplayFrame {
                id: 0,
                test_id,
                frame_index: 0,
                timestamp_ms: 0,
                position: 0,
                expected_char: "h".to_string(),
                typed_char: Some("h".to_string()),
                correct: true,
            },
            ReplayFrame {
                id: 0,
                test_id,
                frame_index: 1,
                timestamp_ms: 100,
                position: 1,
                expected_char: "i".to_string(),
                typed_char: Some("i".to_string()),
                correct: true,
            },
        ];
        replay_repo.save_replay(test_id, &frames).unwrap();
    }
    {
        let db = racoon_data::db::Database::open(&path).unwrap();
        let conn = db.conn();
        let replay_repo = SqliteReplayRepository::new(&conn);
        let loaded = replay_repo.load_replay(1).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].expected_char, "h");
    }
    let _ = std::fs::remove_file(&path);
}

#[test]
fn daily_stats_preserved_across_reopen() {
    let path = std::env::temp_dir().join("racoon_v1_validation_ds.db");
    let _ = std::fs::remove_file(&path);
    {
        let db = racoon_data::db::Database::open(&path).unwrap();
        let conn = db.conn();
        let repo = SqliteDailyStatsRepository::new(&conn);
        repo.update_after_test("2026-06-01", 30000, 100, 40.0, 90.0)
            .unwrap();
        repo.update_after_test("2026-06-01", 30000, 100, 50.0, 95.0)
            .unwrap();
    }
    {
        let db = racoon_data::db::Database::open(&path).unwrap();
        let conn = db.conn();
        let repo = SqliteDailyStatsRepository::new(&conn);
        let s = repo.get_day("2026-06-01").unwrap().unwrap();
        assert_eq!(s.total_tests, 2);
        assert!((s.best_wpm - 50.0).abs() < 0.01);
    }
    let _ = std::fs::remove_file(&path);
}

#[test]
fn settings_preserved_across_reopen() {
    let path = std::env::temp_dir().join("racoon_v1_validation_settings.toml");
    let _ = std::fs::remove_file(&path);
    {
        let store = SettingsStore::new(path.clone());
        store
            .set("theme", toml::Value::String("racoon_dark".to_string()))
            .unwrap();
        store.set("font_size", toml::Value::Integer(28)).unwrap();
    }
    {
        let store = SettingsStore::new(path.clone());
        let s = store.load().unwrap();
        assert_eq!(s.theme, "racoon_dark");
        assert_eq!(s.font_size, 28);
    }
    let _ = std::fs::remove_file(&path);
}

#[test]
fn all_data_preserved_full_roundtrip() {
    let path = std::env::temp_dir().join("racoon_v1_validation_full.db");
    let _ = std::fs::remove_file(&path);
    {
        let db = racoon_data::db::Database::open(&path).unwrap();
        let conn = db.conn();
        let test_repo = SqliteTestRepository::new(&conn);
        let pb_repo = SqlitePersonalBestsRepository::new(&conn);
        let ct_repo = SqliteCustomTextRepository::new(&conn);
        let lesson_repo = SqliteLessonRepository::new(&conn);
        let daily_repo = SqliteDailyStatsRepository::new(&conn);
        let replay_repo = SqliteReplayRepository::new(&conn);

        let test_id = test_repo.save_test(make_record(1)).unwrap();
        pb_repo
            .check_and_update("time", "{}", 40.0, 90.0, test_id)
            .unwrap();
        ct_repo.save("Test", "Hello world").unwrap();
        lesson_repo
            .create_progress("en_m1_l1", "en_m1", "en", "beginner")
            .unwrap();
        lesson_repo.complete_lesson("en_m1_l1", 45.0, 95.0).unwrap();
        daily_repo
            .update_after_test("2026-06-01", 30000, 100, 40.0, 90.0)
            .unwrap();
        let frames = vec![ReplayFrame {
            id: 0,
            test_id,
            frame_index: 0,
            timestamp_ms: 0,
            position: 0,
            expected_char: "h".to_string(),
            typed_char: Some("h".to_string()),
            correct: true,
        }];
        replay_repo.save_replay(test_id, &frames).unwrap();
    }
    {
        let db = racoon_data::db::Database::open(&path).unwrap();
        let conn = db.conn();
        assert_eq!(SqliteTestRepository::new(&conn).get_count(None).unwrap(), 1);
        assert!(!SqlitePersonalBestsRepository::new(&conn)
            .get_bests(None)
            .unwrap()
            .is_empty());
        assert!(!SqliteCustomTextRepository::new(&conn)
            .get_all(100)
            .unwrap()
            .is_empty());
        assert_eq!(
            SqliteLessonRepository::new(&conn)
                .get_lesson_progress("en_m1_l1")
                .unwrap()
                .unwrap()
                .status,
            "completed"
        );
        assert!(SqliteDailyStatsRepository::new(&conn)
            .get_day("2026-06-01")
            .unwrap()
            .is_some());
        assert!(SqliteReplayRepository::new(&conn).has_replay(1).unwrap());
    }
    let _ = std::fs::remove_file(&path);
}

// ── Stress tests ──

#[test]
fn stress_50k_tests_performance() {
    let db = racoon_data::db::Database::open_in_memory().unwrap();
    let conn = db.conn();
    let repo = SqliteTestRepository::new(&conn);

    let start = std::time::Instant::now();
    for i in 0..50000 {
        repo.save_test(make_record(i)).unwrap();
    }
    let insert_time = start.elapsed();
    assert!(
        insert_time.as_secs() < 30,
        "Insert 50k took {:?}",
        insert_time
    );

    let start = std::time::Instant::now();
    let count = repo.get_count(None).unwrap();
    let count_time = start.elapsed();
    assert_eq!(count, 50000);
    assert!(
        count_time.as_millis() < 2000,
        "Count 50k took {:?}",
        count_time
    );

    let start = std::time::Instant::now();
    let history = repo.get_history(100, 0, None).unwrap();
    let history_time = start.elapsed();
    assert_eq!(history.len(), 100);
    assert!(
        history_time.as_millis() < 1000,
        "History 100 from 50k took {:?}",
        history_time
    );

    // Export serialization
    let start = std::time::Instant::now();
    let rows: Vec<Vec<String>> = history
        .iter()
        .map(|t| {
            vec![
                t.created_at.clone(),
                t.wpm.to_string(),
                t.accuracy.to_string(),
            ]
        })
        .collect();
    let _csv = racoon_core::analytics::export_csv(&rows);
    let export_time = start.elapsed();
    assert!(
        export_time.as_millis() < 500,
        "Export 100 from 50k took {:?}",
        export_time
    );
}

#[test]
fn stress_10k_tests_replay_loading() {
    let db = racoon_data::db::Database::open_in_memory().unwrap();
    let conn = db.conn();
    let test_repo = SqliteTestRepository::new(&conn);
    let replay_repo = SqliteReplayRepository::new(&conn);

    // Save 100 tests with replays (100 frames each)
    for i in 0..100 {
        let test_id = test_repo.save_test(make_record(i)).unwrap();
        let frames: Vec<ReplayFrame> = (0..100)
            .map(|f| ReplayFrame {
                id: 0,
                test_id,
                frame_index: f,
                timestamp_ms: f * 100,
                position: f as i64,
                expected_char: "a".to_string(),
                typed_char: Some("a".to_string()),
                correct: true,
            })
            .collect();
        replay_repo.save_replay(test_id, &frames).unwrap();
    }

    let start = std::time::Instant::now();
    let loaded = replay_repo.load_replay(50).unwrap();
    let load_time = start.elapsed();
    assert_eq!(loaded.len(), 100);
    assert!(
        load_time.as_millis() < 200,
        "Replay load 100 frames took {:?}",
        load_time
    );
}

#[test]
fn stress_10k_daily_stats_aggregation() {
    let db = racoon_data::db::Database::open_in_memory().unwrap();
    let conn = db.conn();
    let daily_repo = SqliteDailyStatsRepository::new(&conn);

    // 90 days x 100 tests = 9000 daily stats updates
    for day in 0..90 {
        let date = format!("2026-{:02}-{:02}", (day / 30) + 1, (day % 28) + 1);
        for _ in 0..100 {
            daily_repo
                .update_after_test(&date, 30000, 100, 40.0, 90.0)
                .unwrap();
        }
    }

    let start = std::time::Instant::now();
    let range = daily_repo.get_range("2026-01-01", "2026-03-31").unwrap();
    let range_time = start.elapsed();
    assert!(!range.is_empty());
    assert!(
        range_time.as_millis() < 500,
        "Range agg 90 days took {:?}",
        range_time
    );
}

#[test]
fn stress_streak_calculation_365_days() {
    let dates: Vec<String> = (0..365)
        .map(|i| format!("2026-{:02}-{:02}", (i / 30) + 1, (i % 28) + 1))
        .collect();

    let start = std::time::Instant::now();
    let (_current, longest) = racoon_core::StreakEngine::streak_from_dates(&dates);
    let calc_time = start.elapsed();
    assert!(longest > 0);
    assert!(
        calc_time.as_millis() < 200,
        "Streak 365 days took {:?}",
        calc_time
    );
}

#[test]
fn stress_achievements_check() {
    let start = std::time::Instant::now();
    let achievements = racoon_core::analytics::check_achievements(100, 80.0, 98.0, 30, 30, 20);
    let check_time = start.elapsed();
    assert_eq!(achievements.len(), 15);
    assert!(
        check_time.as_millis() < 10,
        "Achievements check took {:?}",
        check_time
    );
}

#[test]
fn stress_consistency_1000_samples() {
    let samples: Vec<f64> = (0..1000).map(|i| 30.0 + (i as f64 % 20.0)).collect();
    let start = std::time::Instant::now();
    let report = racoon_core::consistency::calc_consistency(&samples);
    let calc_time = start.elapsed();
    assert!(report.score > 0.0);
    assert!(
        calc_time.as_millis() < 10,
        "Consistency 1000 samples took {:?}",
        calc_time
    );
}

#[test]
fn stress_burst_detection_10000_intervals() {
    let intervals: Vec<_> = (0..10000)
        .map(|i| racoon_core::burst::KeystrokeInterval {
            interval_ms: if i % 5 == 0 { 500 } else { 100 },
            is_correct: i % 10 != 0,
        })
        .collect();
    let start = std::time::Instant::now();
    let report = racoon_core::burst::detect_bursts(&intervals, 200);
    let detect_time = start.elapsed();
    assert!(report.burst_count > 0);
    assert!(
        detect_time.as_millis() < 10,
        "Burst 10k intervals took {:?}",
        detect_time
    );
}
