// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

use racoon_data::{backup, Database};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

fn root() -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!(
        "restore-journal-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    root
}
fn db_with_text(path: &Path, name: &str) {
    let db = Database::open(path).unwrap();
    db.with_connection(|conn| {
        conn.execute("INSERT INTO custom_texts(name,text,created_at) VALUES(?1,'body','2026-01-01T00:00:00Z')", [name])?; Ok(())
    }).unwrap();
}
fn text(path: &Path) -> String {
    let db = Database::open(path).unwrap();
    db.with_connection(|conn| {
        conn.query_row("SELECT name FROM custom_texts", [], |row| row.get(0))
            .map_err(Into::into)
    })
    .unwrap()
}
fn pending(root: &Path) -> PathBuf {
    let stage = root.join("live.db.restore-rollback-fixture");
    fs::create_dir(&stage).unwrap();
    fs::write(
        stage.join("pending"),
        b"{\"version\":1,\"sizes\":[0,null,null,null]}",
    )
    .unwrap();
    stage
}

#[test]
fn open_recovers_a_pending_swap_before_sqlite_or_the_backup_hook_runs() {
    let root = root();
    let live = root.join("live.db");
    db_with_text(&live, "old");
    let stage = pending(&root);
    fs::copy(&live, stage.join("original.db")).unwrap();
    fs::write(
        stage.join("pending"),
        format!(
            "{{\"version\":1,\"sizes\":[{},null,null,null]}}",
            fs::metadata(&live).unwrap().len()
        ),
    )
    .unwrap();
    fs::remove_file(&live).unwrap();
    db_with_text(&root.join("replacement.db"), "new");
    fs::rename(root.join("replacement.db"), &live).unwrap();
    let db = Database::open_with_pre_migration(&live, |path| {
        let conn = rusqlite::Connection::open(path).unwrap();
        let value: String = conn
            .query_row("SELECT name FROM custom_texts", [], |row| row.get(0))
            .unwrap();
        assert_eq!(value, "old", "backup hook saw an unresolved replacement");
    })
    .unwrap();
    drop(db);
    assert_eq!(text(&live), "old");
    assert!(stage.join("aborted").is_file());
    assert_eq!(text(&live), "old");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn incomplete_pending_snapshot_blocks_open_without_modifying_live_bytes() {
    let root = root();
    let live = root.join("live.db");
    db_with_text(&live, "new");
    pending(&root);
    let before = fs::read(&live).unwrap();
    assert!(matches!(
        Database::open(&live),
        Err(racoon_data::DbError::RestoreBlocked)
    ));
    assert_eq!(fs::read(&live).unwrap(), before);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn completed_restore_retention_keeps_five_and_leaves_legacy_directories() {
    let root = root();
    let live = root.join("live.db");
    let source = root.join("source.db");
    db_with_text(&live, "old");
    db_with_text(&source, "new");
    let legacy = root.join("live.db.restore-rollback-legacy");
    fs::create_dir(&legacy).unwrap();
    fs::write(legacy.join("original.db"), b"manual recovery copy").unwrap();
    for _ in 0..7 {
        backup::restore_from_path(&source, &live).unwrap();
    }
    let completed = fs::read_dir(&root)
        .unwrap()
        .flatten()
        .filter(|e| e.path().join("committed").is_file())
        .count();
    assert_eq!(completed, 5);
    assert!(legacy.join("original.db").is_file());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn truncated_snapshot_and_ambiguous_pending_intents_block_before_any_write() {
    for case in [
        "truncated",
        "two-pending",
        "bad-version",
        "two-terminal-markers",
    ] {
        let root = root();
        let live = root.join("live.db");
        db_with_text(&live, "retained");
        let bytes = fs::read(&live).unwrap();
        let stage = pending(&root);
        fs::write(stage.join("original.db"), &bytes).unwrap();
        fs::write(
            stage.join("pending"),
            format!(
                "{{\"version\":1,\"sizes\":[{},null,null,null]}}",
                bytes.len()
            ),
        )
        .unwrap();
        match case {
            "truncated" => fs::write(stage.join("original.db"), b"short").unwrap(),
            "two-pending" => {
                let other = root.join("live.db.restore-rollback-second");
                fs::create_dir(&other).unwrap();
                fs::copy(stage.join("pending"), other.join("pending")).unwrap();
                fs::copy(stage.join("original.db"), other.join("original.db")).unwrap();
            }
            "bad-version" => fs::write(
                stage.join("pending"),
                b"{\"version\":2,\"sizes\":[null,null,null,null]}",
            )
            .unwrap(),
            _ => {
                fs::write(stage.join("committed"), b"racoon-restore-v1\n").unwrap();
                fs::write(stage.join("aborted"), b"racoon-restore-v1\n").unwrap();
            }
        }
        assert!(
            matches!(
                Database::open(&live),
                Err(racoon_data::DbError::RestoreBlocked)
            ),
            "{case}"
        );
        assert_eq!(fs::read(&live).unwrap(), bytes, "{case}");
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn reopen_recovers_pending_before_accepting_a_connection() {
    let root = root();
    let live = root.join("live.db");
    db_with_text(&live, "old");
    let db = Database::open(&live).unwrap();
    db.close().unwrap();
    let stage = pending(&root);
    fs::copy(&live, stage.join("original.db")).unwrap();
    fs::write(
        stage.join("pending"),
        format!(
            "{{\"version\":1,\"sizes\":[{},null,null,null]}}",
            fs::metadata(&live).unwrap().len()
        ),
    )
    .unwrap();
    fs::remove_file(&live).unwrap();
    fs::write(&live, b"partial replacement").unwrap();
    db.reopen(&live).unwrap();
    let name: String = db
        .with_connection(|conn| {
            conn.query_row("SELECT name FROM custom_texts", [], |r| r.get(0))
                .map_err(Into::into)
        })
        .unwrap();
    assert_eq!(name, "old");
    drop(db);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn partially_deleted_retired_snapshot_does_not_block_the_live_profile() {
    let root = root();
    let live = root.join("live.db");
    db_with_text(&live, "retained");
    let retired = root.join(".restore-trash-live.db.restore-rollback-retired");
    fs::create_dir(&retired).unwrap();
    fs::write(retired.join("pending"), b"partial deletion").unwrap();
    assert_eq!(text(&live), "retained");
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn symlinked_snapshot_and_journal_are_rejected_without_following_them() {
    for link in ["pending", "original.db"] {
        let root = root();
        let live = root.join("live.db");
        db_with_text(&live, "retained");
        let stage = pending(&root);
        let bytes = fs::read(&live).unwrap();
        fs::write(stage.join("original.db"), &bytes).unwrap();
        fs::write(
            stage.join("pending"),
            format!(
                "{{\"version\":1,\"sizes\":[{},null,null,null]}}",
                bytes.len()
            ),
        )
        .unwrap();
        let outside = root.join("outside");
        fs::write(&outside, b"untouched").unwrap();
        fs::remove_file(stage.join(link)).unwrap();
        std::os::unix::fs::symlink(&outside, stage.join(link)).unwrap();
        assert!(matches!(
            Database::open(&live),
            Err(racoon_data::DbError::RestoreBlocked)
        ));
        assert_eq!(fs::read(&live).unwrap(), bytes);
        assert_eq!(fs::read(&outside).unwrap(), b"untouched");
        fs::remove_dir_all(root).unwrap();
    }
}

#[cfg(unix)]
#[test]
fn rollback_copies_and_journal_remain_private() {
    use std::os::unix::fs::PermissionsExt;
    let root = root();
    let live = root.join("live.db");
    let source = root.join("source.db");
    db_with_text(&live, "old");
    db_with_text(&source, "new");
    backup::restore_from_path(&source, &live).unwrap();
    let stage = fs::read_dir(&root)
        .unwrap()
        .flatten()
        .find(|e| e.path().join("committed").is_file())
        .unwrap()
        .path();
    assert_eq!(
        fs::metadata(&stage).unwrap().permissions().mode() & 0o777,
        0o700
    );
    for name in ["original.db", "pending", "committed"] {
        assert_eq!(
            fs::metadata(stage.join(name)).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    fs::remove_dir_all(root).unwrap();
}
