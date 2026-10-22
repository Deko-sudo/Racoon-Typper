// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

use racoon_application::{GetAnalyticsSnapshot, SessionWallClock};
use racoon_data::db::Database;
use racoon_data::repository::{
    PersonalBestsRepository, SqliteAnalyticsReportingPort, SqlitePersonalBestsRepository,
    SqliteTestRepository, TestRepository,
};
use racoon_domain::{SessionId, TestRecord};

struct Clock;
impl SessionWallClock for Clock {
    fn utc_now(&self) -> chrono::DateTime<chrono::Utc> {
        "2026-07-16T12:00:00Z".parse().unwrap()
    }
}

fn make_record(n: usize) -> TestRecord {
    let created_at = (chrono::DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z").unwrap()
        + chrono::Duration::seconds(n as i64))
    .to_rfc3339();
    TestRecord {
        session_id: SessionId::from(format!("achievement-test-{n:06}")),
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

fn save(db: &Database, n: usize, wpm: f64, accuracy: f64) {
    let mut record = make_record(n);
    record.wpm = wpm;
    record.accuracy = accuracy;
    db.with_transaction(|conn| {
        let id = SqliteTestRepository::new(conn).save_test(record)?;
        SqlitePersonalBestsRepository::new(conn)
            .check_and_update("time", "{}", wpm, accuracy, id)?;
        Ok(())
    })
    .unwrap();
}

fn combined(db: &Database) -> Vec<bool> {
    let port = SqliteAnalyticsReportingPort::new(db);
    GetAnalyticsSnapshot::new(&port, &Clock)
        .execute()
        .unwrap()
        .achievements()
        .iter()
        .filter(|a| a.id == "balanced_50_95" || a.id == "elite_80_95")
        .map(|a| a.unlocked)
        .collect()
}

#[test]
fn separate_speed_and_accuracy_records_do_not_unlock_combined_achievements() {
    let db = Database::open_in_memory().unwrap();
    save(&db, 0, 100.0, 80.0);
    save(&db, 1, 30.0, 100.0);
    assert_eq!(combined(&db), vec![false, false]);
}

#[test]
fn combined_achievement_thresholds_are_inclusive_for_one_completed_test() {
    let db = Database::open_in_memory().unwrap();
    save(&db, 0, 80.0, 94.999);
    assert_eq!(combined(&db), vec![false, false]);
    save(&db, 1, 50.0, 95.0);
    assert_eq!(combined(&db), vec![true, false]);
    save(&db, 2, 80.0, 95.0);
    assert_eq!(combined(&db), vec![true, true]);
}

#[test]
fn qualifying_record_outside_recent_history_window_still_counts() {
    let db = Database::open_in_memory().unwrap();
    save(&db, 0, 80.0, 95.0);
    for n in 1..=120 {
        save(&db, n, 30.0, 80.0);
    }
    assert_eq!(combined(&db), vec![true, true]);
}
