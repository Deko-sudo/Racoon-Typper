//! Online backup / restore round-trip integration tests.
//!
//! These tests prove the safety properties of the backup feature against a real
//! WAL-mode SQLite file: a `fs::copy` of `data.db` would fail them.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use racoon_data::backup::{
    backup_rotation_dir, backup_to_path, create_pre_migration_backup, premigration_backup_filename,
    restore_from_path, rotate_backups, DEFAULT_KEEP,
};
use racoon_data::db::Database;
use racoon_data::repository::{SqliteTestRepository, TestRepository};
use racoon_domain::{SessionId, TestRecord};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

fn temp_path(name: &str) -> PathBuf {
    let sequence = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "racoon-backup-{name}-{}-{sequence}.db",
        std::process::id()
    ))
}

fn temp_dir(name: &str) -> PathBuf {
    let sequence = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "racoon-backup-dir-{name}-{}-{sequence}",
        std::process::id()
    ))
}

fn remove_database(path: &Path) {
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(path.with_extension("db-wal"));
    let _ = std::fs::remove_file(path.with_extension("db-shm"));
}

fn remove_dir_all(dir: &Path) {
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let _ = std::fs::remove_file(entry.path());
        }
        let _ = std::fs::remove_dir(dir);
    }
}

fn make_record(n: i64) -> TestRecord {
    TestRecord {
        session_id: SessionId::from(format!("backup-test-{n}")),
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

fn count_tests(path: &Path) -> i64 {
    let db = Database::open(path).expect("reopen database");
    let conn = db.conn();
    SqliteTestRepository::new(&conn)
        .get_count(None)
        .expect("count tests")
}

fn integrity_ok(path: &Path) -> bool {
    let conn = rusqlite::Connection::open(path).expect("open for integrity check");
    let result: String = conn
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .expect("integrity check query");
    result == "ok"
}

#[test]
fn backup_roundtrip_preserves_all_data() {
    let live = temp_path("roundtrip-live");
    let backup = temp_path("roundtrip-backup.db");
    remove_database(&live);
    std::fs::remove_file(&backup).ok();
    {
        let db = Database::open(&live).expect("open live");
        // Save via a transaction, then drop any guard before backup: backup_to_path
        // acquires the connection mutex internally, so a held `db.conn()` guard
        // would self-deadlock.
        for n in 1..=3 {
            db.with_transaction(|tx| {
                SqliteTestRepository::new(tx)
                    .save_test(make_record(n))
                    .map(|_| ())
            })
            .expect("save row");
        }
        backup_to_path(&db, &backup).expect("backup");
    }
    assert!(integrity_ok(&backup), "backup must pass integrity check");
    assert_eq!(count_tests(&backup), 3, "backup must contain all rows");

    // Corrupt the live database: truncate to garbage so a restore is
    // unambiguous. The corrupted file must NOT be openable as a database
    // (SQLite returns FILEESTDB / "not a database"); that refusal is what makes
    // restore meaningful here.
    std::fs::write(&live, b"not a database").expect("corrupt live");
    assert!(
        Database::open(&live).is_err(),
        "corrupted live must refuse to open before restore"
    );

    restore_from_path(&backup, &live).expect("restore");
    assert!(
        integrity_ok(&live),
        "restored live must pass integrity check"
    );
    assert_eq!(count_tests(&live), 3, "restored live must contain all rows");

    remove_database(&live);
    remove_database(&backup);
}

#[test]
fn backup_captures_uncheckpointed_wal_state() {
    // The central safety property: a snapshot taken via the Online Backup API
    // includes WAL contents that an fs::copy of only data.db would drop. We
    // force WAL to retain data by inserting rows and taking the backup without
    // an explicit checkpoint, then restore into a fresh file and confirm the
    // rows survived.
    let live = temp_path("wal-live");
    let backup = temp_path("wal-backup.db");
    remove_database(&live);
    std::fs::remove_file(&backup).ok();
    {
        let db = Database::open(&live).expect("open live");
        for n in 1..=5 {
            db.with_transaction(|tx| {
                SqliteTestRepository::new(tx)
                    .save_test(make_record(n))
                    .map(|_| ())
            })
            .expect("save row");
        }
        // WAL-mode writes may still be in the -wal file at this point; do not
        // checkpoint before backing up. backup_to_path takes the mutex itself,
        // so no guard must be held here.
        backup_to_path(&db, &backup).expect("backup with uncheckpointed WAL");
    }

    // Restore into a brand-new path and verify every WAL-resident row survived.
    let restored = temp_path("wal-restored");
    remove_database(&restored);
    restore_from_path(&backup, &restored).expect("restore");
    assert_eq!(count_tests(&restored), 5, "WAL-resident rows must survive");

    remove_database(&live);
    remove_database(&backup);
    remove_database(&restored);
}

#[test]
fn backup_is_transactionally_consistent() {
    // A rolled-back transaction must not appear in the backup: the Online
    // Backup API snapshots a consistent database view, not a raw byte stream.
    //
    // We commit one row, then run a transaction that inserts a second row but
    // returns an error so the whole transaction rolls back. The snapshot is
    // taken after the rollback. Do not interleave `db.conn()` with
    // `db.with_transaction()`: both acquire the same Mutex<Connection>, which
    // would self-deadlock.
    let live = temp_path("consistent-live");
    let backup = temp_path("consistent-backup.db");
    remove_database(&live);
    std::fs::remove_file(&backup).ok();
    {
        let db = Database::open(&live).expect("open live");

        // Commit one row through a transaction.
        db.with_transaction(|tx| {
            SqliteTestRepository::new(tx)
                .save_test(make_record(1))
                .map(|_| ())
        })
        .expect("commit row 1");

        // A transaction that inserts a second row but rolls back.
        let rolled_back: Result<(), racoon_data::DbError> = db.with_transaction(|tx| {
            SqliteTestRepository::new(tx).save_test(make_record(2))?;
            // Force a rollback: the inserted row must not survive.
            Err(racoon_data::DbError::Write("forced rollback".to_string()))
        });
        assert!(rolled_back.is_err(), "transaction must roll back");

        // Snapshot is taken after the rollback; no held guard remains.
        backup_to_path(&db, &backup).expect("backup");
    }
    assert_eq!(
        count_tests(&backup),
        1,
        "rolled-back row must not leak into the snapshot"
    );

    remove_database(&live);
    remove_database(&backup);
}

#[test]
fn restore_rejects_missing_backup() {
    let missing = temp_path("does-not-exist.db");
    let live = temp_path("restore-missing-live");
    remove_database(&live);
    let result = restore_from_path(&missing, &live);
    assert!(
        matches!(result, Err(racoon_data::DbError::Restore(_))),
        "missing backup must return Restore error, got {result:?}"
    );
    // The live path must not have been created as a side effect.
    assert!(!live.exists(), "live must not be created on missing backup");
    remove_database(&live);
}

#[test]
fn restore_rejects_non_file_backup() {
    let dir_backup = temp_dir("restore-dir-backup");
    let live = temp_path("restore-dir-live");
    remove_database(&live);
    let _ = std::fs::create_dir_all(&dir_backup);
    let result = restore_from_path(&dir_backup, &live);
    assert!(
        matches!(result, Err(racoon_data::DbError::Restore(_))),
        "directory-as-backup must return Restore error, got {result:?}"
    );
    remove_dir_all(&dir_backup);
    remove_database(&live);
}

#[test]
fn rotate_backups_keeps_latest_n() {
    let dir = temp_dir("rotate");
    remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create rotation dir");
    // Create 7 timestamped snapshots in chronological order. Names are
    // lexicographically sortable, matching rotate_backups' assumption.
    let base = chrono::DateTime::parse_from_rfc3339("2026-08-01T00:00:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc);
    for i in 0..7 {
        let ts = base + chrono::Duration::seconds(i);
        let name = premigration_backup_filename("data", ts);
        std::fs::write(dir.join(name), b"snapshot").expect("write snapshot");
    }
    // Also drop a non-matching file to prove it is left untouched.
    std::fs::write(dir.join("stranger.txt"), b"x").expect("write stranger");

    rotate_backups(&dir, DEFAULT_KEEP).expect("rotate");

    let mut remaining: Vec<String> = std::fs::read_dir(&dir)
        .expect("read dir")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    remaining.sort();
    let premigration_count = remaining
        .iter()
        .filter(|n| n.ends_with(".premigration.db"))
        .count();
    assert_eq!(
        premigration_count, DEFAULT_KEEP,
        "exactly DEFAULT_KEEP snapshots must remain"
    );
    // The newest 5 (timestamps 2..=6) must be the ones retained; the oldest two
    // (T000000Z and T000001Z) must be gone. Match the full timestamp to avoid a
    // loose substring catching T000002..T000006.
    assert!(
        !remaining
            .iter()
            .any(|n| n.contains("20260801T000000Z") || n.contains("20260801T000001Z")),
        "oldest snapshots must be pruned: {remaining:?}"
    );
    assert!(
        remaining.iter().any(|n| n == "stranger.txt"),
        "non-matching files must be preserved"
    );

    remove_dir_all(&dir);
}

#[test]
fn create_pre_migration_backup_then_upgrade_preserves_history() {
    // Start from an old-schema database (V001 only), back it up, then open with
    // the production runner which upgrades to the current schema. The pre-
    // migration backup must be readable and contain the original rows, proving
    // the value of taking the snapshot before migrating.
    use refinery::Runner;
    let live = temp_path("premig-live");
    let data_dir = temp_dir("premig-data");
    remove_database(&live);
    remove_dir_all(&data_dir);
    std::fs::create_dir_all(&data_dir).expect("create data dir");

    // Build a V001-only historical database through Refinery so its schema
    // history is realistic.
    {
        let mut conn = rusqlite::Connection::open(&live).expect("historical open");
        let v001_sql = include_str!("../migrations/V001__initial.sql");
        let migrations = vec![refinery::Migration::unapplied("V001__initial", v001_sql).unwrap()];
        Runner::new(&migrations)
            .run(&mut conn)
            .expect("historical V001 migration");
        // Insert a row using the V001 tests schema (no session_id column yet).
        conn.execute(
            "INSERT INTO tests (created_at, mode_type, mode_config, language, text_length, duration_ms, wpm, raw_wpm, accuracy, raw_accuracy, consistency, correct_chars, incorrect_chars, backspaces, char_stats, heatmap_data, graph_data, is_pb, tags)
             VALUES ('2026-06-01T12:00:00Z','time','{}','en',50,30000,30.0,35.0,80.0,75.0,NULL,95,5,2,'{}','{}',NULL,0,'')",
            [],
        )
        .expect("insert historical row");
    }

    // Take the pre-migration backup.
    let ts = chrono::Utc::now();
    let backup_path = create_pre_migration_backup(&live, &data_dir, "data", ts, DEFAULT_KEEP)
        .expect("pre-migration backup");
    assert!(backup_path.is_file(), "backup file must exist");
    assert!(integrity_ok(&backup_path), "backup must be valid");

    // Now run the full production migration path (V002..V008) over the live DB.
    {
        let db = Database::open(&live).expect("production upgrade");
        let conn = db.conn();
        // V005 backfills a legacy session_id; the row must still be present.
        let count = SqliteTestRepository::new(&conn)
            .get_count(None)
            .expect("count after upgrade");
        assert_eq!(count, 1, "upgraded DB must retain the historical row");
    }

    // The pre-migration backup is itself a valid V001 database we could restore
    // in a recovery scenario. Confirm it still reads as having one row.
    let backup_check = temp_path("premig-restored");
    remove_database(&backup_check);
    restore_from_path(&backup_path, &backup_check).expect("restore the pre-migration backup");
    // Reopen restored backup with V001-only expectation: open runs all
    // migrations, but the row count must be preserved through the round trip.
    let db = Database::open(&backup_check).expect("open restored backup");
    let conn = db.conn();
    let count = SqliteTestRepository::new(&conn)
        .get_count(None)
        .expect("count restored backup");
    assert_eq!(count, 1, "pre-migration backup must preserve the row");

    remove_database(&live);
    remove_database(&backup_check);
    remove_dir_all(&data_dir);
}

#[test]
fn backup_rotation_dir_under_data_dir() {
    // Light guard that the documented location contract holds.
    let dir = backup_rotation_dir(std::path::Path::new("/var/lib/racoon-typper"));
    assert_eq!(dir, std::path::Path::new("/var/lib/racoon-typper/backups"));
}
