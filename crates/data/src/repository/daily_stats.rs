//! DailyStatsRepository — агрегация статистики по дням.

use rusqlite::{params, Connection};

use crate::error::DbError;

/// Статистика за один день.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DailyStats {
    pub date: String,
    pub total_tests: i64,
    pub total_time_ms: i64,
    pub total_chars: i64,
    pub best_wpm: f64,
    pub avg_wpm: f64,
    pub avg_accuracy: f64,
    pub lessons_completed: i64,
}

pub trait DailyStatsRepository {
    fn update_after_test(
        &self,
        date: &str,
        duration_ms: i64,
        total_chars: i64,
        wpm: f64,
        accuracy: f64,
    ) -> Result<(), DbError>;
    fn get_day(&self, date: &str) -> Result<Option<DailyStats>, DbError>;
    fn get_range(&self, from: &str, to: &str) -> Result<Vec<DailyStats>, DbError>;
    fn get_last_30_days(&self) -> Result<Vec<DailyStats>, DbError>;
}

pub struct SqliteDailyStatsRepository<'a> {
    conn: &'a Connection,
}

impl<'a> SqliteDailyStatsRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }
}

impl<'a> DailyStatsRepository for SqliteDailyStatsRepository<'a> {
    fn update_after_test(
        &self,
        date: &str,
        duration_ms: i64,
        total_chars: i64,
        wpm: f64,
        accuracy: f64,
    ) -> Result<(), DbError> {
        // Try to update existing row first
        let updated = self
            .conn
            .execute(
                "UPDATE daily_stats SET
                    total_tests = total_tests + 1,
                    total_time_ms = total_time_ms + ?2,
                    total_chars = total_chars + ?3,
                    best_wpm = CASE WHEN ?4 > best_wpm THEN ?4 ELSE best_wpm END,
                    avg_wpm = (avg_wpm * total_tests + ?4) / (total_tests + 1),
                    avg_accuracy = (avg_accuracy * total_tests + ?5) / (total_tests + 1)
                 WHERE date = ?1",
                params![date, duration_ms, total_chars, wpm, accuracy],
            )
            .map_err(|e| DbError::Write(e.to_string()))?;

        if updated == 0 {
            // No existing row — insert new
            self.conn
                .execute(
                    "INSERT INTO daily_stats (date, total_tests, total_time_ms, total_chars, best_wpm, avg_wpm, avg_accuracy, lessons_completed, daily_goal_met)
                     VALUES (?1, 1, ?2, ?3, ?4, ?4, ?5, 0, 0)",
                    params![date, duration_ms, total_chars, wpm, accuracy],
                )
                .map_err(|e| DbError::Write(e.to_string()))?;
        }
        Ok(())
    }

    fn get_day(&self, date: &str) -> Result<Option<DailyStats>, DbError> {
        let result = self.conn.query_row(
            "SELECT date, total_tests, total_time_ms, total_chars, best_wpm, avg_wpm, avg_accuracy, lessons_completed
             FROM daily_stats WHERE date = ?1",
            params![date],
            |row| {
                Ok(DailyStats {
                    date: row.get(0)?,
                    total_tests: row.get(1)?,
                    total_time_ms: row.get(2)?,
                    total_chars: row.get(3)?,
                    best_wpm: row.get(4)?,
                    avg_wpm: row.get(5)?,
                    avg_accuracy: row.get(6)?,
                    lessons_completed: row.get(7)?,
                })
            },
        );
        match result {
            Ok(s) => Ok(Some(s)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::Query(e.to_string())),
        }
    }

    fn get_range(&self, from: &str, to: &str) -> Result<Vec<DailyStats>, DbError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT date, total_tests, total_time_ms, total_chars, best_wpm, avg_wpm, avg_accuracy, lessons_completed
                 FROM daily_stats WHERE date >= ?1 AND date <= ?2 ORDER BY date",
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        let rows = stmt
            .query_map(params![from, to], |row| {
                Ok(DailyStats {
                    date: row.get(0)?,
                    total_tests: row.get(1)?,
                    total_time_ms: row.get(2)?,
                    total_chars: row.get(3)?,
                    best_wpm: row.get(4)?,
                    avg_wpm: row.get(5)?,
                    avg_accuracy: row.get(6)?,
                    lessons_completed: row.get(7)?,
                })
            })
            .map_err(|e| DbError::Query(e.to_string()))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| DbError::Query(e.to_string()))?);
        }
        Ok(result)
    }

    fn get_last_30_days(&self) -> Result<Vec<DailyStats>, DbError> {
        let now = chrono::Utc::now();
        let from = (now - chrono::Duration::days(30))
            .format("%Y-%m-%d")
            .to_string();
        let to = now.format("%Y-%m-%d").to_string();
        self.get_range(&from, &to)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> SqliteDailyStatsRepository<'static> {
        let conn = Box::leak(Box::new(rusqlite::Connection::open_in_memory().unwrap()));
        crate::db::run_migrations(conn);
        SqliteDailyStatsRepository::new(conn)
    }

    fn today() -> String {
        chrono::Utc::now().format("%Y-%m-%d").to_string()
    }

    #[test]
    fn get_day_empty() {
        let repo = setup();
        let s = repo.get_day("2026-01-01").unwrap();
        assert!(s.is_none());
    }

    #[test]
    fn update_creates_day() {
        let repo = setup();
        let date = today();
        repo.update_after_test(&date, 30000, 100, 40.0, 95.0)
            .unwrap();
        let s = repo.get_day(&date).unwrap().unwrap();
        assert_eq!(s.total_tests, 1);
        assert!((s.best_wpm - 40.0).abs() < 0.01);
        assert!((s.avg_wpm - 40.0).abs() < 0.01);
        assert!((s.avg_accuracy - 95.0).abs() < 0.01);
    }

    #[test]
    fn update_multiple_tests_same_day() {
        let repo = setup();
        let date = today();
        repo.update_after_test(&date, 30000, 100, 40.0, 90.0)
            .unwrap();
        repo.update_after_test(&date, 30000, 100, 50.0, 95.0)
            .unwrap();
        let s = repo.get_day(&date).unwrap().unwrap();
        assert_eq!(s.total_tests, 2);
        assert!((s.best_wpm - 50.0).abs() < 0.01);
        // avg_wpm: (40+50)/2 = 45, but SQL uses total_tests after increment
        // SQLite: total_tests becomes 2, then avg = (40 * (2-1) + 50) / 2 = 45
        // But if total_tests is read AFTER increment, (2-1)=1, so (40*1+50)/2=45
        assert!((s.avg_wpm - 45.0).abs() < 0.01, "avg_wpm was {}", s.avg_wpm);
    }

    #[test]
    fn update_tracks_total_time() {
        let repo = setup();
        let date = today();
        repo.update_after_test(&date, 15000, 50, 30.0, 90.0)
            .unwrap();
        repo.update_after_test(&date, 30000, 100, 40.0, 95.0)
            .unwrap();
        let s = repo.get_day(&date).unwrap().unwrap();
        assert_eq!(s.total_time_ms, 45000);
        assert_eq!(s.total_chars, 150);
    }

    #[test]
    fn update_best_wpm_keeps_max() {
        let repo = setup();
        let date = today();
        repo.update_after_test(&date, 30000, 100, 50.0, 90.0)
            .unwrap();
        repo.update_after_test(&date, 30000, 100, 30.0, 95.0)
            .unwrap();
        let s = repo.get_day(&date).unwrap().unwrap();
        assert!((s.best_wpm - 50.0).abs() < 0.01);
    }

    #[test]
    fn get_range_multiple_days() {
        let repo = setup();
        repo.update_after_test("2026-01-01", 30000, 100, 40.0, 90.0)
            .unwrap();
        repo.update_after_test("2026-01-02", 30000, 100, 45.0, 92.0)
            .unwrap();
        repo.update_after_test("2026-01-03", 30000, 100, 50.0, 95.0)
            .unwrap();
        let range = repo.get_range("2026-01-01", "2026-01-03").unwrap();
        assert_eq!(range.len(), 3);
        assert_eq!(range[0].date, "2026-01-01");
        assert_eq!(range[2].date, "2026-01-03");
    }

    #[test]
    fn get_range_filters_by_date() {
        let repo = setup();
        repo.update_after_test("2026-01-01", 30000, 100, 40.0, 90.0)
            .unwrap();
        repo.update_after_test("2026-01-05", 30000, 100, 50.0, 95.0)
            .unwrap();
        let range = repo.get_range("2026-01-02", "2026-01-04").unwrap();
        assert_eq!(range.len(), 0);
    }

    #[test]
    fn get_range_orders_by_date() {
        let repo = setup();
        repo.update_after_test("2026-03-01", 30000, 100, 40.0, 90.0)
            .unwrap();
        repo.update_after_test("2026-01-01", 30000, 100, 30.0, 85.0)
            .unwrap();
        repo.update_after_test("2026-02-01", 30000, 100, 35.0, 88.0)
            .unwrap();
        let range = repo.get_range("2026-01-01", "2026-03-01").unwrap();
        assert_eq!(range[0].date, "2026-01-01");
        assert_eq!(range[1].date, "2026-02-01");
        assert_eq!(range[2].date, "2026-03-01");
    }

    #[test]
    fn get_last_30_days_returns_recent() {
        let repo = setup();
        let now = chrono::Utc::now().format("%Y-%m-%d").to_string();
        repo.update_after_test(&now, 30000, 100, 40.0, 90.0)
            .unwrap();
        let range = repo.get_last_30_days().unwrap();
        assert!(!range.is_empty());
        assert_eq!(range[0].date, now);
    }

    #[test]
    fn update_idempotent_date() {
        let repo = setup();
        let date = "2026-06-01";
        repo.update_after_test(date, 30000, 100, 40.0, 90.0)
            .unwrap();
        repo.update_after_test(date, 30000, 100, 45.0, 92.0)
            .unwrap();
        repo.update_after_test(date, 30000, 100, 35.0, 88.0)
            .unwrap();
        let s = repo.get_day(date).unwrap().unwrap();
        assert_eq!(s.total_tests, 3);
    }

    #[test]
    fn avg_wpm_recomputed_correctly() {
        let repo = setup();
        let date = "2026-06-01";
        // 3 tests: 30, 40, 50 → avg = 40
        repo.update_after_test(date, 30000, 100, 30.0, 90.0)
            .unwrap();
        repo.update_after_test(date, 30000, 100, 40.0, 90.0)
            .unwrap();
        repo.update_after_test(date, 30000, 100, 50.0, 90.0)
            .unwrap();
        let s = repo.get_day(date).unwrap().unwrap();
        assert!((s.avg_wpm - 40.0).abs() < 0.1, "avg_wpm was {}", s.avg_wpm);
    }
}
