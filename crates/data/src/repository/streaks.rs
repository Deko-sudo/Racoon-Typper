//! Streak persistence. Streak calculation remains a core/application policy;
//! this repository owns only SQLite mapping and upsert behavior.

use rusqlite::{params, Connection, OptionalExtension};

use crate::error::DbError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreakRecord {
    pub streak_type: String,
    pub current_streak: i64,
    pub longest_streak: i64,
    pub last_date: Option<String>,
    pub started_date: Option<String>,
}

pub trait StreakRepository {
    fn get(&self, streak_type: &str) -> Result<Option<StreakRecord>, DbError>;
    fn upsert(&self, record: &StreakRecord) -> Result<(), DbError>;
}

pub struct SqliteStreakRepository<'a> {
    conn: &'a Connection,
}

impl<'a> SqliteStreakRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }
}

impl StreakRepository for SqliteStreakRepository<'_> {
    fn get(&self, streak_type: &str) -> Result<Option<StreakRecord>, DbError> {
        self.conn
            .query_row(
                "SELECT type, current_streak, longest_streak, last_date, started_date
                 FROM streaks WHERE type = ?1",
                params![streak_type],
                |row| {
                    Ok(StreakRecord {
                        streak_type: row.get(0)?,
                        current_streak: row.get(1)?,
                        longest_streak: row.get(2)?,
                        last_date: row.get(3)?,
                        started_date: row.get(4)?,
                    })
                },
            )
            .optional()
            .map_err(|error| DbError::Query(error.to_string()))
    }

    fn upsert(&self, record: &StreakRecord) -> Result<(), DbError> {
        self.conn
            .execute(
                "INSERT INTO streaks (
                    type, current_streak, longest_streak, last_date, started_date
                 ) VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(type) DO UPDATE SET
                    current_streak = excluded.current_streak,
                    longest_streak = excluded.longest_streak,
                    last_date = excluded.last_date,
                    started_date = excluded.started_date",
                params![
                    record.streak_type,
                    record.current_streak,
                    record.longest_streak,
                    record.last_date,
                    record.started_date,
                ],
            )
            .map_err(|error| DbError::Write(error.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    #[test]
    fn loads_missing_streak_as_none() {
        let database = Database::open_in_memory().unwrap();
        let conn = database.conn();

        assert!(SqliteStreakRepository::new(&conn)
            .get("daily_test")
            .unwrap()
            .is_none());
    }

    #[test]
    fn upsert_creates_and_replaces_a_streak() {
        let database = Database::open_in_memory().unwrap();
        let conn = database.conn();
        let repository = SqliteStreakRepository::new(&conn);

        repository
            .upsert(&StreakRecord {
                streak_type: "daily_test".to_string(),
                current_streak: 1,
                longest_streak: 1,
                last_date: Some("2026-07-10".to_string()),
                started_date: Some("2026-07-10".to_string()),
            })
            .unwrap();
        repository
            .upsert(&StreakRecord {
                streak_type: "daily_test".to_string(),
                current_streak: 2,
                longest_streak: 2,
                last_date: Some("2026-07-11".to_string()),
                started_date: Some("2026-07-10".to_string()),
            })
            .unwrap();

        assert_eq!(
            repository.get("daily_test").unwrap(),
            Some(StreakRecord {
                streak_type: "daily_test".to_string(),
                current_streak: 2,
                longest_streak: 2,
                last_date: Some("2026-07-11".to_string()),
                started_date: Some("2026-07-10".to_string()),
            })
        );
    }
}
