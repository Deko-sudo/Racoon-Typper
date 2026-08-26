// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

//! Performance baselines for the SQLite reporting adapters.
//!
//! Seeds a 10k-record in-memory database and asserts generous regression
//! thresholds for every reporting port read path. Thresholds are intentionally
//! loose so slow CI runners do not flake; the goal is to catch accidental
//! full scans or lost indexes, not to measure absolute speed.

use std::time::{Duration, Instant};

use racoon_application::{
    AchievementInputQuery, AnalyticsReportingPort, HistoryFilter, HistoryQuery,
    HistoryReportingPort, InclusiveDateRange, InsightInputQuery, OffsetPagination,
    PersonalBestReportingPort, ProgressReportingPort, ReplayQuery, ReportingDay, ReportingLanguage,
    ANALYTICS_HISTORY_LIMIT,
};
use racoon_data::db::Database;
use racoon_data::repository::{
    DailyStatsRepository, PersonalBestsRepository, ReplayFrame, ReplayRepository,
    SqliteAnalyticsReportingPort, SqliteHistoryReportingPort, SqlitePersonalBestReportingPort,
    SqlitePersonalBestsRepository, SqliteProgressReportingPort, SqliteReplayRepository,
    SqliteStreakRepository, SqliteTestRepository, StreakRecord, StreakRepository, TestRepository,
};
use racoon_domain::{SessionId, TestRecord};

const RECORD_COUNT: usize = 10_000;
const REPLAY_FRAMES: usize = 600;

fn reporting_day(day: &str) -> ReportingDay {
    ReportingDay::parse_iso(day).expect("valid reporting day")
}

fn make_record(n: usize) -> TestRecord {
    let created_at = (chrono::DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z").unwrap()
        + chrono::Duration::seconds(n as i64))
    .to_rfc3339();
    TestRecord {
        session_id: SessionId::from(format!("perf-test-{n:06}")),
        created_at,
        mode_type: if n.is_multiple_of(2) { "time" } else { "words" }.to_string(),
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

fn seed_tests(database: &Database) -> Duration {
    let start = Instant::now();
    database
        .with_transaction(|conn| {
            let repo = SqliteTestRepository::new(conn);
            for n in 0..RECORD_COUNT {
                repo.save_test(make_record(n))?;
            }
            Ok(())
        })
        .expect("seed tests");
    start.elapsed()
}

fn seed_projections(database: &Database) {
    database
        .with_transaction(|conn| {
            let daily = racoon_data::repository::SqliteDailyStatsRepository::new(conn);
            for day_offset in 0..365 {
                let date = (chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
                    + chrono::Duration::days(day_offset))
                .format("%Y-%m-%d")
                .to_string();
                daily.update_after_test(&date, 30000, 100, 45.0, 95.0)?;
            }
            SqliteStreakRepository::new(conn).upsert(&StreakRecord {
                streak_type: "daily_test".into(),
                current_streak: 12,
                longest_streak: 40,
                last_date: Some("2026-12-31".into()),
                started_date: Some("2026-01-01".into()),
            })?;
            let test_id = SqliteTestRepository::new(conn)
                .get_id_by_session_id(&SessionId::from("perf-test-000000"))?;
            SqlitePersonalBestsRepository::new(conn)
                .check_and_update("time", "{}", 79.0, 99.0, test_id)?;
            SqlitePersonalBestsRepository::new(conn)
                .check_and_update("words", "{}", 78.0, 98.0, test_id)?;
            Ok(())
        })
        .expect("seed projections");
}

#[test]
fn history_port_page_and_count_at_10k() {
    let database = Database::open_in_memory().unwrap();
    let seed_time = seed_tests(&database);
    assert!(
        seed_time.as_secs() < 30,
        "seeding {RECORD_COUNT} tests took {seed_time:?}"
    );

    let port = SqliteHistoryReportingPort::new(&database);

    let start = Instant::now();
    let query = HistoryQuery::new(
        HistoryFilter::default(),
        OffsetPagination::new(100, 0).unwrap(),
    );
    let page = port.list_history(&query).unwrap();
    let unfiltered_page_time = start.elapsed();

    assert_eq!(page.total(), RECORD_COUNT as u64);
    assert_eq!(page.items().len(), 100);
    assert!(
        unfiltered_page_time < Duration::from_millis(1500),
        "history page took {unfiltered_page_time:?}"
    );

    let start = Instant::now();
    let filter = HistoryFilter::new(Some("time".parse().unwrap()), None);
    let query = HistoryQuery::new(filter, OffsetPagination::new(100, 0).unwrap());
    let filtered = port.list_history(&query).unwrap();
    let filtered_page_time = start.elapsed();

    assert_eq!(filtered.total(), (RECORD_COUNT / 2) as u64);
    assert!(
        filtered_page_time < Duration::from_millis(1500),
        "filtered history page took {filtered_page_time:?}"
    );

    let range =
        InclusiveDateRange::new(reporting_day("2026-01-01"), reporting_day("2026-01-01")).unwrap();
    let filter = HistoryFilter::new(None, Some(range));
    let query = HistoryQuery::new(filter, OffsetPagination::new(100, 0).unwrap());
    let start = Instant::now();
    let ranged = port.list_history(&query).unwrap();
    let ranged_page_time = start.elapsed();

    assert!(!ranged.items().is_empty());
    assert!(
        ranged_page_time < Duration::from_millis(1500),
        "date-ranged history page took {ranged_page_time:?}"
    );
}

#[test]
fn progress_port_surfaces_at_10k() {
    let database = Database::open_in_memory().unwrap();
    let _seed_time = seed_tests(&database);
    seed_projections(&database);

    let port = SqliteProgressReportingPort::new(&database);

    let start = Instant::now();
    let count = port.count_tests().unwrap();
    let count_time = start.elapsed();

    assert_eq!(count, RECORD_COUNT as u64);
    assert!(
        count_time < Duration::from_millis(1000),
        "count_tests took {count_time:?}"
    );

    let range =
        InclusiveDateRange::new(reporting_day("2026-01-01"), reporting_day("2026-12-31")).unwrap();
    let start = Instant::now();
    let points = port.load_daily_statistics(range).unwrap();
    let daily_time = start.elapsed();

    assert_eq!(points.len(), 365);
    assert!(
        daily_time < Duration::from_millis(500),
        "daily statistics year load took {daily_time:?}"
    );

    let start = Instant::now();
    let streak = port
        .load_streak_report(reporting_day("2026-12-31"))
        .unwrap();
    let streak_time = start.elapsed();

    assert_eq!(streak.current_streak(), 12);
    assert_eq!(streak.longest_streak(), 40);
    assert!(
        streak_time < Duration::from_millis(200),
        "streak report took {streak_time:?}"
    );
}

#[test]
fn personal_bests_and_analytics_ports_at_10k() {
    let database = Database::open_in_memory().unwrap();
    seed_tests(&database);
    seed_projections(&database);

    let bests_port = SqlitePersonalBestReportingPort::new(&database);
    let start = Instant::now();
    let entries = bests_port.list_personal_bests(None).unwrap();
    let bests_time = start.elapsed();

    assert_eq!(entries.len(), 2);
    assert!(
        bests_time < Duration::from_millis(500),
        "personal bests listing took {bests_time:?}"
    );

    let analytics_port = SqliteAnalyticsReportingPort::new(&database);

    let start = Instant::now();
    let achievement_query =
        AchievementInputQuery::new(vec![ReportingLanguage::parse("en").unwrap()]).unwrap();
    let inputs = analytics_port
        .load_achievement_inputs(&achievement_query)
        .unwrap();
    let achievement_time = start.elapsed();

    assert_eq!(inputs.total_tests(), RECORD_COUNT as u64);
    assert!(
        achievement_time < Duration::from_millis(2000),
        "achievement inputs took {achievement_time:?}"
    );

    let range =
        InclusiveDateRange::new(reporting_day("2026-01-01"), reporting_day("2026-12-31")).unwrap();
    let start = Instant::now();
    let insight_query = InsightInputQuery::new(range, ANALYTICS_HISTORY_LIMIT);
    let insights = analytics_port.load_insight_inputs(&insight_query).unwrap();
    let insight_time = start.elapsed();

    assert_eq!(insights.recent_wpm().len(), ANALYTICS_HISTORY_LIMIT);
    assert_eq!(insights.daily_statistics().len(), 365);
    assert!(
        insight_time < Duration::from_millis(2000),
        "insight inputs took {insight_time:?}"
    );
}

#[test]
fn replay_pagination_baseline() {
    let database = Database::open_in_memory().unwrap();
    database
        .with_transaction(|conn| {
            let repo = SqliteTestRepository::new(conn);
            let test_id = repo.save_test(make_record(0))?;
            let frames: Vec<ReplayFrame> = (0..REPLAY_FRAMES)
                .map(|i| ReplayFrame {
                    id: 0,
                    test_id,
                    frame_index: i as i64,
                    timestamp_ms: (i * 80) as i64,
                    position: i as i64,
                    expected_char: char::from_u32(0x61 + (i % 26) as u32).unwrap().to_string(),
                    typed_char: Some(char::from_u32(0x61 + (i % 26) as u32).unwrap().to_string()),
                    correct: true,
                })
                .collect();
            SqliteReplayRepository::new(conn).save_replay(test_id, &frames)?;
            Ok(())
        })
        .expect("seed replay");

    let port = SqliteHistoryReportingPort::new(&database);
    let session_id = SessionId::from("perf-test-000000");

    let start = Instant::now();
    let mut total_frames = 0usize;
    let mut offset = 0usize;
    loop {
        let query = ReplayQuery::new(
            session_id.clone(),
            OffsetPagination::new(100, offset).unwrap(),
        );
        let page = port
            .list_replay_frames(&query)
            .unwrap()
            .expect("replay exists");
        total_frames += page.frames().len();
        if !page.has_more() {
            break;
        }
        offset += 100;
    }
    let replay_time = start.elapsed();

    assert_eq!(total_frames, REPLAY_FRAMES);
    assert!(
        replay_time < Duration::from_millis(1000),
        "{total_frames} replay frames over paginated fetches took {replay_time:?}"
    );
}
