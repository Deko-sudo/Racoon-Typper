//! Ошибки Database Layer.

use std::fmt;

use rusqlite::ErrorCode;

#[derive(Debug)]
pub enum DbError {
    Connection(String),
    Migration(String),
    Query(String),
    Write(String),
    Transaction(String),
    /// Structured SQLite failure retained until a storage adapter maps it to
    /// its infrastructure-neutral port result.
    Sqlite {
        code: ErrorCode,
        extended_code: i32,
        operation: &'static str,
    },
    Integrity(String),
    Validation(String),
    LockPoisoned,
    NotFound(String),
    /// A pre-migration or on-demand online backup failed. Backups use SQLite's
    /// Online Backup API, so this covers source/destination connection failures,
    /// the backup step itself, and the atomic file-rename of the snapshot.
    Backup(String),
    /// A restore-from-backup failed (missing source, copy/backup step failure,
    /// or path validation). Restore always operates on file paths while the live
    /// `Database` connection is closed.
    Restore(String),
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbError::Connection(msg) => write!(f, "DB connection error: {}", msg),
            DbError::Migration(msg) => write!(f, "DB migration error: {}", msg),
            DbError::Query(msg) => write!(f, "DB query error: {}", msg),
            DbError::Write(msg) => write!(f, "DB write error: {}", msg),
            DbError::Transaction(msg) => write!(f, "DB transaction error: {}", msg),
            DbError::Sqlite {
                code,
                extended_code,
                operation,
            } => write!(
                f,
                "SQLite {operation} failure: {code:?} (extended code {extended_code})"
            ),
            DbError::Integrity(msg) => write!(f, "DB integrity error: {}", msg),
            DbError::Validation(msg) => write!(f, "Validation error: {}", msg),
            DbError::LockPoisoned => write!(f, "Database state is unavailable after a panic"),
            DbError::NotFound(msg) => write!(f, "Not found: {}", msg),
            DbError::Backup(msg) => write!(f, "DB backup error: {}", msg),
            DbError::Restore(msg) => write!(f, "DB restore error: {}", msg),
        }
    }
}

impl std::error::Error for DbError {}

impl DbError {
    /// Retains SQLite's stable primary and extended error codes without
    /// exposing them through application-facing ports.
    pub(crate) fn from_sqlite(operation: &'static str, error: rusqlite::Error) -> Self {
        match error {
            rusqlite::Error::SqliteFailure(error, _) => Self::Sqlite {
                code: error.code,
                extended_code: error.extended_code,
                operation,
            },
            error => Self::Query(error.to_string()),
        }
    }
}

impl From<rusqlite::Error> for DbError {
    fn from(e: rusqlite::Error) -> Self {
        match e {
            rusqlite::Error::SqliteFailure(error, _) => DbError::Sqlite {
                code: error.code,
                extended_code: error.extended_code,
                operation: "database operation",
            },
            _ => DbError::Query(e.to_string()),
        }
    }
}
