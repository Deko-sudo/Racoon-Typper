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

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use rusqlite::backup::Backup;
use rusqlite::{Connection, OpenFlags};

use crate::error::DbError;
use crate::Database;

mod restore_protocol;
#[cfg(test)]
mod restore_protocol_tests;
pub(crate) use restore_protocol::recover as recover_pending_restore;

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
/// `dest` is written atomically: the snapshot is built in a private sibling
/// directory and renamed into place, so a reader never observes a half-written
/// backup. Existing temporary paths are never reused. The destination
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

    source.with_connection(|conn| publish_backup(conn, dest))
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
/// A private sibling copy is migrated and schema-checked before replacement.
/// All writers, including external processes, must remain closed throughout.
/// The original main/WAL/SHM/journal files are retained in a sibling
/// `restore-rollback-*` directory.
/// Companions are detached before rename and restored on a pre-swap error.
/// A durable pending/committed/aborted journal recovers unfinished swaps before
/// SQLite opens. Failed recovery keeps the connection closed. Five completed
/// snapshot directories are retained (best-effort cleanup).
pub fn restore_from_path(backup_path: &Path, live_db_path: &Path) -> Result<PathBuf, DbError> {
    restore_with_steps(backup_path, live_db_path, &mut |_| Ok(()))
}

fn restore_with_steps(
    backup_path: &Path,
    live_db_path: &Path,
    step: &mut impl FnMut(&'static str) -> Result<(), DbError>,
) -> Result<PathBuf, DbError> {
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
    if backup_path == live_db_path {
        return Err(DbError::Restore(
            "backup source and live destination must differ".to_string(),
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if fs::metadata(live_db_path)
            .is_ok_and(|live| backup_meta.dev() == live.dev() && backup_meta.ino() == live.ino())
        {
            return Err(restore_fail("backup source aliases the live database"));
        }
    }
    if let Some(parent) = live_db_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|error| DbError::Restore(format!("create live directory: {error}")))?;
        }
    }

    recover_pending_restore(live_db_path)?;
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let stage = sibling_temp_path(live_db_path, &format!("restore-rollback-{sequence}"));
    let mut builder = fs::DirBuilder::new();
    builder.recursive(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(&stage).map_err(restore_fail)?;
    let temp_path = stage.join("replacement.db");
    let result = (|| -> Result<(), DbError> {
        let source = Connection::open_with_flags(backup_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(restore_fail)?;
        let mut dest = Connection::open(&temp_path).map_err(restore_fail)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&temp_path, fs::Permissions::from_mode(0o600))
                .map_err(restore_fail)?;
        }
        copy_via_online_backup(&source, &mut dest, "restore").map_err(restore_error)?;
        validate_sqlite_snapshot(&dest)?;
        validate_schema(&mut dest)?;
        dest.pragma_update(None, "journal_mode", "DELETE")
            .map_err(restore_fail)?;
        dest.close().map_err(|(_, error)| restore_fail(error))?;
        drop(source);
        sync_file(&temp_path)?;
        restore_protocol::publish(&stage, live_db_path, step)
    })();
    if result.is_err() && matches!(stage.join("pending").try_exists(), Ok(false)) {
        let _ = fs::remove_dir_all(&stage);
    }
    result?;
    let _ = restore_protocol::prune(live_db_path);
    Ok(live_db_path.to_path_buf())
}

fn restore_fail(error: impl std::fmt::Display) -> DbError {
    DbError::Restore(error.to_string())
}
fn restore_companion(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(suffix);
    PathBuf::from(name)
}
fn sync_file(path: &Path) -> Result<(), DbError> {
    // Windows FlushFileBuffers requires GENERIC_WRITE. Unix also calls this
    // helper for directories, so retain the read-only handle on those systems.
    #[cfg(windows)]
    let file = File::options().write(true).open(path);
    #[cfg(not(windows))]
    let file = File::open(path);
    file.and_then(|file| file.sync_all()).map_err(restore_fail)
}

/// Token boundaries, quoted identifiers and literal bytes are significant.
/// Only whitespace/comments and unquoted ASCII case are normalized.
fn schema_tokens(sql: &str) -> Result<Vec<String>, DbError> {
    let mut chars = sql.chars().peekable();
    let mut tokens = Vec::new();
    while let Some(ch) = chars.next() {
        if matches!(ch, ' ' | '\t' | '\n' | '\r' | '\u{c}') {
            continue;
        }
        if ch == '-' && chars.next_if_eq(&'-').is_some() {
            for ch in chars.by_ref() {
                if ch == '\n' {
                    break;
                }
            }
            continue;
        }
        if ch == '/' && chars.next_if_eq(&'*').is_some() {
            let mut closed = false;
            while let Some(ch) = chars.next() {
                if ch == '*' && chars.next_if_eq(&'/').is_some() {
                    closed = true;
                    break;
                }
            }
            if !closed {
                return Err(restore_fail("unclosed schema comment"));
            }
            continue;
        }
        let mut token = ch.to_string();
        if matches!(ch, '\'' | '"' | '`' | '[') {
            let end = if ch == '[' { ']' } else { ch };
            loop {
                let next = chars
                    .next()
                    .ok_or_else(|| restore_fail("unclosed schema quote"))?;
                token.push(next);
                if next == end {
                    if ch != '[' && chars.next_if_eq(&end).is_some() {
                        token.push(end);
                    } else {
                        break;
                    }
                }
            }
        } else if ch.is_alphanumeric() || ch == '_' {
            while let Some(next) = chars.next_if(|c| c.is_alphanumeric() || matches!(c, '_' | '$'))
            {
                token.push(next);
            }
            token.make_ascii_lowercase();
        }
        tokens.push(token);
    }
    Ok(tokens)
}

type Schema = BTreeMap<(String, String, String), Vec<String>>;
fn schema(conn: &Connection) -> Result<Schema, DbError> {
    let mut query = conn
        .prepare("SELECT type, name, tbl_name, sql FROM sqlite_schema WHERE sql IS NOT NULL")
        .map_err(restore_fail)?;
    let rows = query
        .query_map([], |row| {
            Ok((
                (row.get(0)?, row.get(1)?, row.get(2)?),
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(restore_fail)?;
    let mut objects = BTreeMap::new();
    for row in rows {
        let (key, sql) = row.map_err(restore_fail)?;
        objects.insert(key, schema_tokens(&sql)?);
    }
    Ok(objects)
}
fn validate_schema(conn: &mut Connection) -> Result<(), DbError> {
    let mut reference = Connection::open_in_memory().map_err(restore_fail)?;
    crate::db::run_migrations(&mut reference)?;
    let version: i64 = conn
        .query_row(
            "SELECT MAX(version) FROM refinery_schema_history",
            [],
            |row| row.get(0),
        )
        .map_err(restore_fail)?;
    if !(1..=9).contains(&version) {
        return Err(restore_fail("unsupported backup migration version"));
    }
    conn.pragma_update(None, "foreign_keys", "ON")
        .map_err(restore_fail)?;
    crate::db::run_migrations(conn)?;
    let mut expected = schema(&reference)?;
    let mut names: BTreeSet<String> = expected
        .keys()
        .flat_map(|(_, name, table)| [name.clone(), table.clone()])
        .collect();
    for (kind, name, _) in expected.keys() {
        if kind == "table" {
            let mut query = reference
                .prepare("SELECT name FROM pragma_table_info(?1)")
                .map_err(restore_fail)?;
            for column in query
                .query_map([name], |row| row.get::<_, String>(0))
                .map_err(restore_fail)?
            {
                names.insert(column.map_err(restore_fail)?);
            }
        }
    }
    let mut actual = schema(conn)?;
    for objects in [&mut expected, &mut actual] {
        for token in objects.values_mut().flatten() {
            // Single-quoted literals and unknown quoted text stay byte-exact.
            let identifier = token
                .strip_prefix('"')
                .and_then(|text| text.strip_suffix('"'))
                .or_else(|| {
                    token
                        .strip_prefix('`')
                        .and_then(|text| text.strip_suffix('`'))
                })
                .or_else(|| {
                    token
                        .strip_prefix('[')
                        .and_then(|text| text.strip_suffix(']'))
                });
            if let Some(identifier) = identifier {
                let name = identifier.to_ascii_lowercase();
                if names.contains(&name) {
                    *token = name;
                }
            }
        }
    }
    if actual != expected {
        return Err(restore_fail("backup differs from application schema"));
    }
    let violations: i64 = conn
        .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
            row.get(0)
        })
        .map_err(restore_fail)?;
    if violations != 0 {
        return Err(restore_fail("backup contains foreign-key violations"));
    }
    let id = format!("restore-probe-{}", std::process::id());
    for recursive in [false, true] {
        conn.pragma_update(None, "recursive_triggers", recursive)
            .map_err(restore_fail)?;
        conn.execute_batch("SAVEPOINT restore_guard")
            .map_err(restore_fail)?;
        let probe = (|| -> Result<(), DbError> {
            conn.execute("INSERT INTO session_ledger(session_id,state,mode_type,mode_descriptor,language,created_at,updated_at) VALUES (?1,'running','time','{}','en','2026-01-01T00:00:00Z','2026-01-01T00:00:00Z')", [&id]).map_err(restore_fail)?;
            conn.execute(
                "UPDATE session_ledger SET state='finalized' WHERE session_id=?1",
                [&id],
            )
            .map_err(restore_fail)?;
            match conn.execute(
                "UPDATE session_ledger SET state='running' WHERE session_id=?1",
                [&id],
            ) {
                Err(rusqlite::Error::SqliteFailure(error, _)) if error.extended_code == 1811 => {
                    Ok(())
                }
                _ => Err(restore_fail("terminal state guard did not reject update")),
            }
        })();
        conn.execute_batch("ROLLBACK TO restore_guard; RELEASE restore_guard")
            .map_err(restore_fail)?;
        probe?;
    }
    validate_sqlite_snapshot(conn)
}

fn restore_error(error: DbError) -> DbError {
    match error {
        DbError::Backup(message) => DbError::Restore(message),
        other => other,
    }
}

fn validate_sqlite_snapshot(conn: &Connection) -> Result<(), DbError> {
    let result: String = conn
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(|error| DbError::Restore(format!("validate restored snapshot: {error}")))?;
    if result == "ok" {
        Ok(())
    } else {
        Err(DbError::Restore(format!(
            "restored snapshot integrity check failed: {result}"
        )))
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
    publish_backup(&source, &dest)?;

    // Rotation prunes older snapshots. A missing rotation directory is treated
    // as nothing to rotate by `rotate_backups`; any other rotation failure is a
    // real backup subsystem error and is surfaced (the fresh snapshot above
    // still landed on disk before this point).
    rotate_backups(&dir, keep)?;
    Ok(dest)
}

/// Builds one backup in an exclusively created private directory and removes
/// partial snapshots on failure. Caller holds the source lock where applicable.
fn publish_backup(source: &Connection, destination: &Path) -> Result<PathBuf, DbError> {
    let source_path: String = source.query_row(
        "SELECT file FROM pragma_database_list WHERE name = 'main'",
        [],
        |row| row.get(0),
    )?;
    if !source_path.is_empty() {
        let source_path = fs::canonicalize(source_path)
            .map_err(|error| DbError::Backup(format!("resolve backup source: {error}")))?;
        if fs::canonicalize(destination).is_ok_and(|path| path == source_path) {
            return Err(DbError::Backup(
                "backup destination aliases the live database".into(),
            ));
        }
    }
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let stage = sibling_temp_path(destination, &format!("backup-{sequence}"));
    let mut builder = fs::DirBuilder::new();
    builder.recursive(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder
        .create(&stage)
        .map_err(|error| DbError::Backup(format!("create backup staging directory: {error}")))?;
    let result = (|| {
        let temporary_path = stage.join("snapshot.db");
        let mut options = fs::OpenOptions::new();
        options.create_new(true).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        drop(
            options
                .open(&temporary_path)
                .map_err(|error| DbError::Backup(format!("create backup snapshot: {error}")))?,
        );
        let mut snapshot = Connection::open(&temporary_path)
            .map_err(|error| DbError::Backup(format!("open backup snapshot: {error}")))?;
        copy_via_online_backup(source, &mut snapshot, "backup")?;
        snapshot
            .close()
            .map_err(|(_, error)| DbError::Backup(format!("close backup snapshot: {error}")))?;
        fs::rename(&temporary_path, destination)
            .map_err(|error| DbError::Backup(format!("finalize backup file: {error}")))?;
        Ok(destination.to_path_buf())
    })();
    let _ = fs::remove_dir_all(stage);
    result
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
    fn sync_file_flushes_existing_bytes_without_creating_or_truncating_files() {
        let root = std::env::temp_dir().join(format!("restore-sync-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let path = root.join("snapshot.db");
        fs::write(&path, b"preserve all snapshot bytes").unwrap();
        let result = sync_file(&path);
        let bytes = fs::read(&path).unwrap();
        let missing = root.join("missing.db");
        let missing_result = sync_file(&missing);
        let created = missing.exists();
        fs::remove_dir_all(root).unwrap();
        assert!(result.is_ok(), "existing file sync failed: {result:?}");
        assert_eq!(bytes, b"preserve all snapshot bytes");
        assert!(missing_result.is_err());
        assert!(!created);
    }

    #[test]
    fn schema_comparison_preserves_literals_and_quote_boundaries() {
        assert_eq!(
            schema_tokens("SELECT/*comment*/'finalized'").unwrap(),
            schema_tokens("select 'finalized'").unwrap()
        );
        for different in [
            "SELECT 'final ized'",
            "SELECT 'FINALIZED'",
            "SELECT \"finalized\"",
            "SELECT 'final' 'ized'",
            "SELECT 'final''ized'",
            "SELECT\u{a0}'finalized'",
        ] {
            assert_ne!(
                schema_tokens("SELECT 'finalized'").unwrap(),
                schema_tokens(different).unwrap()
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn restore_keeps_the_published_database_private() {
        use std::os::unix::fs::PermissionsExt;
        let root = std::env::temp_dir().join(format!("restore-private-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let source = root.join("source.db");
        let live = root.join("live.db");
        backup_to_path(&Database::open_in_memory().unwrap(), &source).unwrap();
        fs::copy(&source, &live).unwrap();
        fs::set_permissions(&live, fs::Permissions::from_mode(0o600)).unwrap();
        restore_from_path(&source, &live).unwrap();
        let mode = fs::metadata(&live).unwrap().permissions().mode() & 0o777;
        fs::remove_dir_all(root).unwrap();
        assert_eq!(mode, 0o600);
    }

    #[cfg(unix)]
    #[test]
    fn restore_rejects_a_hard_link_to_the_live_database() {
        let root = std::env::temp_dir().join(format!("restore-alias-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let source = root.join("source.db");
        let live = root.join("live.db");
        backup_to_path(&Database::open_in_memory().unwrap(), &live).unwrap();
        fs::hard_link(&live, &source).unwrap();
        let bytes = fs::read(&live).unwrap();
        let result = restore_from_path(&source, &live);
        let unchanged = fs::read(&live).unwrap() == bytes;
        fs::remove_dir_all(root).unwrap();
        assert!(result.is_err(), "restore accepted a live-file alias");
        assert!(unchanged, "rejected alias changed the live file");
    }

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
