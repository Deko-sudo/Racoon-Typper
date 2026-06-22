//! Ошибки Database Layer.

use std::fmt;

#[derive(Debug)]
pub enum DbError {
    Connection(String),
    Migration(String),
    Query(String),
    Write(String),
    NotFound(String),
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbError::Connection(msg) => write!(f, "DB connection error: {}", msg),
            DbError::Migration(msg) => write!(f, "DB migration error: {}", msg),
            DbError::Query(msg) => write!(f, "DB query error: {}", msg),
            DbError::Write(msg) => write!(f, "DB write error: {}", msg),
            DbError::NotFound(msg) => write!(f, "Not found: {}", msg),
        }
    }
}

impl std::error::Error for DbError {}

impl From<rusqlite::Error> for DbError {
    fn from(e: rusqlite::Error) -> Self {
        match e {
            rusqlite::Error::SqliteFailure(_, ref msg) => {
                if msg
                    .as_ref()
                    .map(|m| m.contains("no such row"))
                    .unwrap_or(false)
                {
                    DbError::NotFound(e.to_string())
                } else {
                    DbError::Query(e.to_string())
                }
            }
            _ => DbError::Query(e.to_string()),
        }
    }
}
