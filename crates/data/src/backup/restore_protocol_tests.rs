// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

use super::*;
use std::process::Command;

struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn fixture() -> Fixture {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!(
        "restore-process-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    let db = Database::open(&root.join("live.db")).unwrap();
    drop(db);
    let source = Database::open(&root.join("source.db")).unwrap();
    source.with_connection(|conn| {
        conn.execute("INSERT INTO custom_texts(name,text,created_at) VALUES('new','body','2026-01-01T00:00:00Z')",[])?;Ok(())
    }).unwrap();
    drop(source);
    child(&root, "seed");
    Fixture(root)
}
fn child(root: &Path, mode: &str) {
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "backup::restore_protocol_tests::restore_process_child",
            "--nocapture",
        ])
        .env("RACOON_RESTORE_TEST_ROOT", root)
        .env("RACOON_RESTORE_TEST_MODE", mode)
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(77),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
fn raw_files(live: &Path) -> Vec<Option<Vec<u8>>> {
    ["", "-wal", "-shm", "-journal"]
        .iter()
        .map(|suffix| fs::read(restore_companion(live, suffix)).ok())
        .collect()
}
fn assert_history(live: &Path, expected: &str) {
    let db = Database::open(live).unwrap();
    db.with_connection(|conn| {
        let name: String = conn.query_row("SELECT name FROM custom_texts", [], |row| row.get(0))?;
        assert_eq!(name, expected);
        let integrity: String = conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
        assert_eq!(integrity, "ok");
        let violations: i64 =
            conn.query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
                row.get(0)
            })?;
        assert_eq!(violations, 0);
        Ok(())
    })
    .unwrap();
}

#[test]
fn restore_process_child() {
    let Ok(root) = std::env::var("RACOON_RESTORE_TEST_ROOT") else {
        return;
    };
    let root = PathBuf::from(root);
    let live = root.join("live.db");
    let mode = std::env::var("RACOON_RESTORE_TEST_MODE").unwrap();
    if mode == "seed" {
        let conn = Connection::open(&live).unwrap();
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA wal_autocheckpoint=0; INSERT INTO custom_texts(name,text,created_at) VALUES('old','body','2026-01-01T00:00:00Z')").unwrap();
        // Abrupt exit deliberately retains a real WAL with a committed row
        // absent from the main file. No connection destructor checkpoints it.
        std::process::exit(77);
    }
    let (action, point) = mode.split_once(':').unwrap();
    let mut stop = |step| {
        if step == point {
            std::process::exit(77);
        }
        Ok(())
    };
    if action == "restore" {
        restore_with_steps(&root.join("source.db"), &live, &mut stop).unwrap();
    } else {
        restore_protocol::recover_with_steps(&live, &mut stop).unwrap();
    }
    panic!("crash seam was not reached: {mode}");
}

#[test]
fn restore_process_crashes_recover_at_every_snapshot_and_publication_boundary() {
    for point in [
        "snapshot-main",
        "snapshot-wal",
        "snapshot-shm",
        "snapshot-journal",
        "pending",
        "detach-wal",
        "detach-shm",
        "detach-journal",
        "swapped",
        "published",
        "committed",
    ] {
        let fixture = fixture();
        let live = fixture.0.join("live.db");
        let raw = raw_files(&live);
        assert!(raw[1].as_ref().is_some_and(|wal| !wal.is_empty()));
        let main = Connection::open_with_flags(
            format!("file:{}?immutable=1", live.display()),
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
        )
        .unwrap();
        let main_count: i64 = main
            .query_row("SELECT COUNT(*) FROM custom_texts", [], |row| row.get(0))
            .unwrap();
        assert_eq!(main_count, 0, "fixture history must reside in WAL");
        drop(main);
        child(&fixture.0, &format!("restore:{point}"));
        let expected = if point == "committed" { "new" } else { "old" };
        assert_history(&live, expected);
        assert_history(&live, expected);
    }
}

#[test]
fn recovery_itself_is_restartable_after_every_file_and_terminal_marker() {
    for point in [
        "rollback-main",
        "rollback-wal",
        "rollback-shm",
        "rollback-journal",
        "aborted",
    ] {
        let fixture = fixture();
        let live = fixture.0.join("live.db");
        child(&fixture.0, "restore:swapped");
        child(&fixture.0, &format!("recover:{point}"));
        assert_history(&live, "old");
        assert_history(&live, "old");
    }
}

#[test]
fn injected_errors_before_commit_restore_exact_main_and_companion_bytes() {
    for point in [
        "snapshot-main",
        "snapshot-wal",
        "snapshot-shm",
        "snapshot-journal",
        "pending",
        "detach-wal",
        "detach-shm",
        "detach-journal",
        "swapped",
        "published",
    ] {
        let fixture = fixture();
        let live = fixture.0.join("live.db");
        let original = raw_files(&live);
        let result = restore_with_steps(&fixture.0.join("source.db"), &live, &mut |step| {
            if step == point {
                Err(restore_fail("injected operation failure"))
            } else {
                Ok(())
            }
        });
        assert!(
            matches!(result, Err(DbError::Restore(_))),
            "{point}: {result:?}"
        );
        assert_eq!(raw_files(&live), original, "{point}");
        assert_history(&live, "old");
    }
}

#[test]
fn rollback_failure_blocks_until_startup_completes_recovery() {
    let fixture = fixture();
    let live = fixture.0.join("live.db");
    let result = restore_with_steps(&fixture.0.join("source.db"), &live, &mut |step| {
        if step == "swapped" || step == "rollback-wal" {
            Err(restore_fail("injected IO failure"))
        } else {
            Ok(())
        }
    });
    assert!(matches!(result, Err(DbError::RestoreBlocked)));
    assert_history(&live, "old");
}

#[test]
fn a_committed_snapshot_never_overwrites_subsequent_user_data_on_open() {
    let fixture = fixture();
    let live = fixture.0.join("live.db");
    restore_from_path(&fixture.0.join("source.db"), &live).unwrap();
    let db = Database::open(&live).unwrap();
    db.with_connection(|conn| {
        conn.execute("UPDATE custom_texts SET name='later'", [])?;
        Ok(())
    })
    .unwrap();
    drop(db);
    assert_history(&live, "later");
    assert_history(&live, "later");
}

#[test]
fn pruning_refuses_any_pending_restore_before_deleting_completed_copies() {
    let fixture = fixture();
    let live = fixture.0.join("live.db");
    for index in 0..7 {
        let stage = fixture
            .0
            .join(format!("live.db.restore-rollback-completed-{index}"));
        fs::create_dir(&stage).unwrap();
        fs::write(stage.join("original.db"), b"snapshot").unwrap();
        fs::write(
            stage.join("pending"),
            br#"{"version":1,"sizes":[8,null,null,null]}"#,
        )
        .unwrap();
        fs::write(stage.join("committed"), b"racoon-restore-v1\n").unwrap();
    }
    let pending = fixture.0.join("live.db.restore-rollback-pending");
    fs::create_dir(&pending).unwrap();
    fs::write(
        pending.join("pending"),
        br#"{"version":1,"sizes":[null,null,null,null]}"#,
    )
    .unwrap();
    assert!(matches!(
        restore_protocol::prune(&live),
        Err(DbError::RestoreBlocked)
    ));
    let retained = fs::read_dir(&fixture.0)
        .unwrap()
        .flatten()
        .filter(|e| e.path().join("committed").is_file())
        .count();
    assert_eq!(retained, 7);
    assert!(pending.join("pending").is_file());
}
