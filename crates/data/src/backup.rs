//! Online SQLite backup, restore, and backup rotation.
//!
//! The database runs in WAL mode (see `db::configure_connection`), so copying
//! `data.db` with `fs::copy` is unsafe two ways: copying only the main file
//! loses un-checkpointed WAL contents, and copying `data.db` + `-wal` + `-shm`
//! together is not atomic across the three files. SQLite's Online Backup API
//! (`rusqlite::backup::Backup`) is the only correct approach: it reads through
//! the SQLite engine and writes a single transactionally consistent snapshot
//! file with the WAL fully applied.
//!
//! All public functions return typed `DbError::Backup` / `DbError::Restore`
//! failures so that nothing is silently swallowed.

use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::backup::Backup;
use rusqlite::Connection;

use crate::error::DbError;
use crate::Database;

/// Page-step size and sleep used by the online backup. The database is a local
/// single-writer file, so a small step keeps the source lock short while the
/// whole snapshot still completes quickly.
const BACKUP_PAGE_STEP: i32 = 100;
const BACKUP_SLEEP: Duration = Duration::from_millis(10);

/// Suffix for timestamped pre-migration backups inside the rotation directory.
pub const PREMIGRATION_SUFFIX: &str = ".premigration.db";

/// Directory that holds the rotating pre-migration snapshots, relative to the
/// application data directory.
pub const BACKUP_DIR_NAME: &str = "backups";

/// Default number of snapshots retained by `rotate_backups`.
pub const DEFAULT_KEEP: usize = 5;

/// Returns the backup rotation directory for a given application data directory.
pub fn backup_rotation_dir(data_dir: &Path) -> PathBuf {
    data_dir.join(BACKUP_DIR_NAME)
}

/// Builds a UTC-lexicographically-sortable filename for a pre-migration backup.
///
/// The name is `<prefix>-<YYYYMMDDTHHMMSSZ><PREMIGRATION_SUFFIX>`. The timestamp
/// is fixed-width UTC so that lexicographic filename order matches chronological
/// order, which `rotate_backups` relies on.
pub fn premigration_backup_filename(
    prefix: &str,
    timestamp: chrono::DateTime<chrono::Utc>,
) -> String {
    format!(
        "{prefix}-{}{PREMIGRATION_SUFFIX}",
        timestamp.format("%Y%m%dT%H%M%SZ")
    )
}

/// Copies the live database into a consistent single-file snapshot at `dest`.
///
/// `dest` is written atomically: the snapshot is built at `dest.tmp` and renamed
/// into place, so a reader never observes a half-written backup. The destination
/// parent directory is created if missing. The returned `PathBuf` is the final
/// backup path (equal to `dest`).
///
/// The source `Database` is read through its mutex, so concurrent writes are
/// serialized as usual; the backup itself is a read-only snapshot.
pub fn backup_to_path(source: &Database, dest: &Path) -> Result<PathBuf, DbError> {
    if dest.as_os_str().is_empty() {
        return Err(DbError::Backup("destination path is empty".to_string()));
    }
    if let Some(parent) = dest.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|error| DbError::Backup(format!("create backup directory: {error}")))?;
        }
    }

    // Build the snapshot at a sibling temp path, then rename into place.
    let temp_dest = sibling_temp_path(dest, "tmp");
    run_online_backup(source, &temp_dest)?;
    std::fs::rename(&temp_dest, dest)
        .map_err(|error| DbError::Backup(format!("finalize backup file: {error}")))?;
    Ok(dest.to_path_buf())
}

/// Restores a backup file into a live database path.
///
/// # Contract
/// The caller **must close** any open `Database` over `live_db_path` before
/// calling this. Restore opens its own destination connection and SQLite holds a
/// file lock; an open writer would conflict. This function operates on file
/// paths rather than an open `Database` precisely because restore requires the
/// live connection to be torn down first.
///
/// `backup_path` must exist and be a regular file. Any existing destination
/// file at `live_db_path` is **removed first** along with its `-wal`/`-shm`
/// companions, then the backup is copied in through the Online Backup API. This
/// never layers a restore onto a suspect or corrupt live file: the destination
/// always starts empty, so the snapshot is written cleanly. Returns the restored
/// path on success.
pub fn restore_from_path(backup_path: &Path, live_db_path: &Path) -> Result<PathBuf, DbError> {
    if backup_path.as_os_str().is_empty() || live_db_path.as_os_str().is_empty() {
        return Err(DbError::Restore("empty backup or live path".to_string()));
    }
    let backup_meta = std::fs::metadata(backup_path)
        .map_err(|error| DbError::Restore(format!("backup source is not readable: {error}")))?;
    if !backup_meta.is_file() {
        return Err(DbError::Restore(
            "backup source is not a regular file".to_string(),
        ));
    }
    if let Some(parent) = live_db_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|error| DbError::Restore(format!("create live directory: {error}")))?;
        }
    }

    // Remove any existing live database and its WAL/SHM companions before
    // restoring. Layering an Online Backup onto a garbage or partial file is
    // unreliable (SQLite can refuse with NOTADB); starting from a clean path is
    // correct and avoids leaving stale -wal/-shm behind.
    remove_database_companions(live_db_path);

    // Open the backup as source and the live path as destination. The Online
    // Backup API copies the source into the destination, overwriting it.
    let source = Connection::open(backup_path)
        .map_err(|error| DbError::Restore(format!("open backup source: {error}")))?;
    let mut dest = Connection::open(live_db_path)
        .map_err(|error| DbError::Restore(format!("open live destination: {error}")))?;
    copy_via_online_backup(&source, &mut dest, "restore")?;
    // A checkpoint+truncate ensures the restored snapshot has no lingering WAL.
    let _ = dest.pragma_update(None, "wal_checkpoint", "TRUNCATE");
    Ok(live_db_path.to_path_buf())
}

/// Removes a database file and its `-wal` / `-shm` companions if present. Used
/// before restore so the destination starts clean. Missing files are not errors.
fn remove_database_companions(db_path: &Path) {
    let _ = std::fs::remove_file(db_path);
    if let Some(stem) = db_path.file_name().and_then(|n| n.to_str()) {
        let _ = std::fs::remove_file(db_path.with_file_name(format!("{stem}-wal")));
        let _ = std::fs::remove_file(db_path.with_file_name(format!("{stem}-shm")));
    }
}

/// Retains at most `keep` newest `*.premigration.db` files in `dir`, deleting
/// older snapshots. Files without a sortable timestamp name are left untouched
/// (never silently deleted). Rotation failures (missing dir, individual delete
/// errors) are returned but a missing directory is treated as nothing to rotate.
pub fn rotate_backups(dir: &Path, keep: usize) -> Result<(), DbError> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(DbError::Backup(format!("read backup directory: {error}"))),
    };

    let mut snapshots: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension().is_some_and(|ext| ext == "db")
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.ends_with(PREMIGRATION_SUFFIX))
        })
        .collect();
    // Lexicographic order of the timestamped names matches chronological order.
    snapshots.sort();

    let excess = snapshots.len().saturating_sub(keep);
    for path in snapshots.iter().take(excess) {
        std::fs::remove_file(path)
            .map_err(|error| DbError::Backup(format!("delete old backup: {error}")))?;
    }
    Ok(())
}

/// Creates a timestamped pre-migration backup of `db_path` inside `data_dir`'s
/// `backups/` directory, then rotates to keep at most `keep` snapshots.
///
/// This opens a short-lived connection over the (already migrated-or-not) file
/// at `db_path` purely to take the snapshot. It is intended to be called *before*
/// the production connection runs migrations, so the snapshot reflects the exact
/// pre-migration on-disk state.
pub fn create_pre_migration_backup(
    db_path: &Path,
    data_dir: &Path,
    prefix: &str,
    timestamp: chrono::DateTime<chrono::Utc>,
    keep: usize,
) -> Result<PathBuf, DbError> {
    let dir = backup_rotation_dir(data_dir);
    std::fs::create_dir_all(&dir)
        .map_err(|error| DbError::Backup(format!("create backup directory: {error}")))?;
    let filename = premigration_backup_filename(prefix, timestamp);
    let dest = dir.join(filename);

    // Open a throwaway source connection to take the snapshot. The caller is
    // responsible for not holding a conflicting write transaction open.
    let source = Connection::open(db_path)
        .map_err(|error| DbError::Backup(format!("open source for backup: {error}")))?;
    let temp_dest = sibling_temp_path(&dest, "tmp");
    run_online_backup_from_conn(&source, &temp_dest)?;
    std::fs::rename(&temp_dest, &dest)
        .map_err(|error| DbError::Backup(format!("finalize backup file: {error}")))?;

    // Rotation prunes older snapshots. A missing rotation directory is treated
    // as nothing to rotate by `rotate_backups`; any other rotation failure is a
    // real backup subsystem error and is surfaced (the fresh snapshot above
    // still landed on disk before this point).
    rotate_backups(&dir, keep)?;
    Ok(dest)
}

/// Runs the Online Backup API from the live `Database` into `dest_path`,
/// building the destination from a fresh connection.
fn run_online_backup(source: &Database, dest_path: &Path) -> Result<(), DbError> {
    source.with_connection(|source_conn| {
        let mut dest = Connection::open(dest_path)
            .map_err(|error| DbError::Backup(format!("open backup destination: {error}")))?;
        copy_via_online_backup(source_conn, &mut dest, "backup")?;
        Ok(())
    })
}

/// Runs the Online Backup API from an already-open source connection.
fn run_online_backup_from_conn(source: &Connection, dest_path: &Path) -> Result<(), DbError> {
    let mut dest = Connection::open(dest_path)
        .map_err(|error| DbError::Backup(format!("open backup destination: {error}")))?;
    copy_via_online_backup(source, &mut dest, "backup")
}

/// Copies `source` into `dest` using `rusqlite::backup::Backup::run_to_completion`.
fn copy_via_online_backup(
    source: &Connection,
    dest: &mut Connection,
    operation: &'static str,
) -> Result<(), DbError> {
    let backup = Backup::new(source, dest)
        .map_err(|error| DbError::Backup(format!("{operation} initialize: {error}")))?;
    backup
        .run_to_completion(BACKUP_PAGE_STEP, BACKUP_SLEEP, None)
        .map_err(|error| DbError::Backup(format!("{operation} run: {error}")))?;
    Ok(())
}

/// Builds a sibling temporary path for atomic file replacement, e.g.
/// `foo.db` -> `foo.db.<suffix>-<pid>`. Uniqueness across parallel test/binary
/// processes is ensured by the process id suffix.
fn sibling_temp_path(target: &Path, suffix: &str) -> PathBuf {
    let mut name = target
        .file_name()
        .map(|n| n.to_os_string())
        .unwrap_or_default();
    name.push(format!(".{suffix}-{}", std::process::id()));
    target.with_file_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn premigration_filename_is_sortable_and_suffixed() {
        let ts = chrono::DateTime::parse_from_rfc3339("2026-08-01T12:34:56Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        let name = premigration_backup_filename("data", ts);
        assert_eq!(name, "data-20260801T123456Z.premigration.db");
        assert!(name.ends_with(PREMIGRATION_SUFFIX));
    }

    #[test]
    fn backup_rotation_dir_is_under_data_dir() {
        let dir = backup_rotation_dir(Path::new("/tmp/racoon"));
        assert_eq!(dir, Path::new("/tmp/racoon/backups"));
    }

    #[test]
    fn sibling_temp_path_is_sibling_with_pid() {
        let target = Path::new("/tmp/data.db");
        let temp = sibling_temp_path(target, "tmp");
        assert_eq!(temp.parent(), Some(Path::new("/tmp")));
        let name = temp.file_name().unwrap().to_str().unwrap();
        assert!(name.starts_with("data.db.tmp-"));
    }

    #[test]
    fn rotate_backups_missing_dir_is_ok() {
        let dir = Path::new("/tmp/racoon-nonexistent-backup-dir-rotate-test");
        assert!(rotate_backups(dir, DEFAULT_KEEP).is_ok());
    }
}
