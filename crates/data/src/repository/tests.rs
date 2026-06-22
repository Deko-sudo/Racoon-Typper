//! TestRepository — save_test, get_history, get_by_id, get_heatmaps.

use rusqlite::{params, Connection};

use crate::error::DbError;
use crate::models::TestRow;
use racoon_domain::{TestDetail, TestRecord, TestSummary};

/// Trait для тестового репозитория.
pub trait TestRepository {
    fn save_test(&self, record: TestRecord) -> Result<i64, DbError>;
    fn get_history(
        &self,
        limit: usize,
        offset: usize,
        mode_filter: Option<&str>,
    ) -> Result<Vec<TestSummary>, DbError>;
    fn get_by_id(&self, id: i64) -> Result<TestDetail, DbError>;
    fn get_count(&self, mode_filter: Option<&str>) -> Result<i64, DbError>;
    fn get_recent_heatmaps(
        &self,
        recent_count: usize,
        language: Option<&str>,
    ) -> Result<Vec<serde_json::Value>, DbError>;
}

/// SQLite реализация TestRepository.
pub struct SqliteTestRepository<'a> {
    conn: &'a Connection,
}

impl<'a> SqliteTestRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }
}

const SELECT_ALL_COLS: &str = "id, created_at, mode_type, mode_config, language, text_length,
    duration_ms, wpm, raw_wpm, accuracy, raw_accuracy, consistency,
    correct_chars, incorrect_chars, backspaces, char_stats, heatmap_data,
    graph_data, is_pb, tags";

impl<'a> TestRepository for SqliteTestRepository<'a> {
    fn save_test(&self, record: TestRecord) -> Result<i64, DbError> {
        let mode_config_json = serde_json::to_string(&record.mode_config)
            .map_err(|e| DbError::Write(format!("mode_config serialization: {}", e)))?;
        let char_stats_json = serde_json::to_string(&record.char_stats)
            .map_err(|e| DbError::Write(format!("char_stats serialization: {}", e)))?;
        let heatmap_json = serde_json::to_string(&record.heatmap_data)
            .map_err(|e| DbError::Write(format!("heatmap serialization: {}", e)))?;
        let graph_json = record
            .graph_data
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap_or_default());

        self.conn
            .execute(
                "INSERT INTO tests (
                    created_at, mode_type, mode_config, language, text_length,
                    duration_ms, wpm, raw_wpm, accuracy, raw_accuracy, consistency,
                    correct_chars, incorrect_chars, backspaces, char_stats, heatmap_data,
                    graph_data, is_pb, tags
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)",
                params![
                    record.created_at,
                    record.mode_type,
                    mode_config_json,
                    record.language,
                    record.text_length as i64,
                    record.duration_ms as i64,
                    record.wpm,
                    record.raw_wpm,
                    record.accuracy,
                    record.raw_accuracy,
                    record.consistency,
                    record.correct_chars as i64,
                    record.incorrect_chars as i64,
                    record.backspaces as i64,
                    char_stats_json,
                    heatmap_json,
                    graph_json,
                    record.is_pb,
                    record.tags,
                ],
            )
            .map_err(|e| DbError::Write(e.to_string()))?;

        Ok(self.conn.last_insert_rowid())
    }

    fn get_history(
        &self,
        limit: usize,
        offset: usize,
        mode_filter: Option<&str>,
    ) -> Result<Vec<TestSummary>, DbError> {
        let mut stmt = if mode_filter.is_some() {
            self.conn
                .prepare(&format!(
                    "SELECT {} FROM tests WHERE mode_type = ?1 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3",
                    SELECT_ALL_COLS
                ))
                .map_err(|e| DbError::Query(e.to_string()))?
        } else {
            self.conn
                .prepare(&format!(
                    "SELECT {} FROM tests ORDER BY created_at DESC LIMIT ?1 OFFSET ?2",
                    SELECT_ALL_COLS
                ))
                .map_err(|e| DbError::Query(e.to_string()))?
        };

        let rows: Vec<Result<TestRow, rusqlite::Error>> = match mode_filter {
            Some(mode) => stmt
                .query_map(params![mode, limit as i64, offset as i64], map_test_row)
                .map_err(|e| DbError::Query(e.to_string()))?
                .collect(),
            None => stmt
                .query_map(params![limit as i64, offset as i64], map_test_row)
                .map_err(|e| DbError::Query(e.to_string()))?
                .collect(),
        };

        let mut summaries = Vec::new();
        for row in rows {
            let test_row = row.map_err(|e| DbError::Query(e.to_string()))?;
            summaries.push(TestSummary::from(test_row));
        }
        Ok(summaries)
    }

    fn get_by_id(&self, id: i64) -> Result<TestDetail, DbError> {
        let mut stmt = self
            .conn
            .prepare(&format!(
                "SELECT {} FROM tests WHERE id = ?1",
                SELECT_ALL_COLS
            ))
            .map_err(|e| DbError::Query(e.to_string()))?;

        let row = stmt
            .query_row(params![id], map_test_row)
            .map_err(|e| DbError::NotFound(format!("Test id={}: {}", id, e)))?;

        Ok(TestDetail::from(row))
    }

    fn get_count(&self, mode_filter: Option<&str>) -> Result<i64, DbError> {
        let count: i64 = match mode_filter {
            Some(mode) => self
                .conn
                .query_row(
                    "SELECT COUNT(*) FROM tests WHERE mode_type = ?1",
                    params![mode],
                    |row| row.get(0),
                )
                .map_err(|e| DbError::Query(e.to_string()))?,
            None => self
                .conn
                .query_row("SELECT COUNT(*) FROM tests", [], |row| row.get(0))
                .map_err(|e| DbError::Query(e.to_string()))?,
        };
        Ok(count)
    }

    fn get_recent_heatmaps(
        &self,
        recent_count: usize,
        language: Option<&str>,
    ) -> Result<Vec<serde_json::Value>, DbError> {
        let mut stmt = if language.is_some() {
            self.conn
                .prepare(
                    "SELECT heatmap_data FROM tests WHERE language = ?1 ORDER BY created_at DESC LIMIT ?2",
                )
                .map_err(|e| DbError::Query(e.to_string()))?
        } else {
            self.conn
                .prepare("SELECT heatmap_data FROM tests ORDER BY created_at DESC LIMIT ?1")
                .map_err(|e| DbError::Query(e.to_string()))?
        };

        let rows: Vec<Result<String, rusqlite::Error>> = match language {
            Some(lang) => stmt
                .query_map(params![lang, recent_count as i64], |row| row.get(0))
                .map_err(|e| DbError::Query(e.to_string()))?
                .collect(),
            None => stmt
                .query_map(params![recent_count as i64], |row| row.get(0))
                .map_err(|e| DbError::Query(e.to_string()))?
                .collect(),
        };

        let mut heatmaps = Vec::new();
        for row in rows {
            let json_str = row.map_err(|e| DbError::Query(e.to_string()))?;
            let val: serde_json::Value =
                serde_json::from_str(&json_str).unwrap_or(serde_json::Value::Null);
            heatmaps.push(val);
        }
        Ok(heatmaps)
    }
}

/// Маппинг строки → TestRow.
fn map_test_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TestRow> {
    Ok(TestRow {
        id: row.get(0)?,
        created_at: row.get(1)?,
        mode_type: row.get(2)?,
        mode_config: row.get(3)?,
        language: row.get(4)?,
        text_length: row.get(5)?,
        duration_ms: row.get(6)?,
        wpm: row.get(7)?,
        raw_wpm: row.get(8)?,
        accuracy: row.get(9)?,
        raw_accuracy: row.get(10)?,
        consistency: row.get(11)?,
        correct_chars: row.get(12)?,
        incorrect_chars: row.get(13)?,
        backspaces: row.get(14)?,
        char_stats: row.get(15)?,
        heatmap_data: row.get(16)?,
        graph_data: row.get(17)?,
        is_pb: row.get(18)?,
        tags: row.get(19)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use racoon_domain::keyboard::CharStatsMap;

    fn make_test_record(wpm: f64, accuracy: f64) -> TestRecord {
        TestRecord {
            created_at: "2026-06-21T22:00:00Z".to_string(),
            mode_type: "time".to_string(),
            mode_config: serde_json::json!({"duration": 30, "language": "en"}),
            language: "en".to_string(),
            text_length: 100,
            duration_ms: 30000,
            wpm,
            raw_wpm: wpm + 2.0,
            accuracy,
            raw_accuracy: accuracy - 5.0,
            consistency: None,
            correct_chars: 95,
            incorrect_chars: 5,
            backspaces: 2,
            char_stats: serde_json::to_value(&CharStatsMap::new()).unwrap(),
            heatmap_data: serde_json::json!({"a": {"total_attempts": 10, "correct": 9, "incorrect": 1, "avg_wpm_at_key": 40.0}}),
            graph_data: None,
            is_pb: false,
            tags: "".to_string(),
        }
    }

    #[test]
    fn save_and_query_test() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        let repo = SqliteTestRepository::new(&conn);

        let id = repo.save_test(make_test_record(45.0, 95.0)).unwrap();
        assert!(id > 0);

        let history = repo.get_history(10, 0, None).unwrap();
        assert_eq!(history.len(), 1);
        assert!((history[0].wpm - 45.0).abs() < 0.01);
    }

    #[test]
    fn get_by_id_returns_detail() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        let repo = SqliteTestRepository::new(&conn);

        let id = repo.save_test(make_test_record(50.0, 90.0)).unwrap();
        let detail = repo.get_by_id(id).unwrap();
        assert_eq!(detail.id, id);
        assert!((detail.wpm - 50.0).abs() < 0.01);
        assert_eq!(detail.correct_chars, 95);
    }

    #[test]
    fn get_by_id_not_found() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        let repo = SqliteTestRepository::new(&conn);

        let result = repo.get_by_id(999);
        assert!(result.is_err());
    }

    #[test]
    fn get_history_with_mode_filter() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        let repo = SqliteTestRepository::new(&conn);

        repo.save_test(make_test_record(40.0, 95.0)).unwrap();
        let mut record2 = make_test_record(50.0, 90.0);
        record2.mode_type = "words".to_string();
        repo.save_test(record2).unwrap();

        let all = repo.get_history(10, 0, None).unwrap();
        assert_eq!(all.len(), 2);

        let time_only = repo.get_history(10, 0, Some("time")).unwrap();
        assert_eq!(time_only.len(), 1);
        assert_eq!(time_only[0].mode_type, "time");
    }

    #[test]
    fn get_count_total_and_filtered() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        let repo = SqliteTestRepository::new(&conn);

        repo.save_test(make_test_record(40.0, 95.0)).unwrap();
        repo.save_test(make_test_record(50.0, 90.0)).unwrap();

        assert_eq!(repo.get_count(None).unwrap(), 2);
        assert_eq!(repo.get_count(Some("time")).unwrap(), 2);
        assert_eq!(repo.get_count(Some("words")).unwrap(), 0);
    }

    #[test]
    fn get_recent_heatmaps_all_and_filtered() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        let repo = SqliteTestRepository::new(&conn);

        repo.save_test(make_test_record(40.0, 95.0)).unwrap();
        repo.save_test(make_test_record(50.0, 90.0)).unwrap();

        let heatmaps = repo.get_recent_heatmaps(10, None).unwrap();
        assert_eq!(heatmaps.len(), 2);

        let en_heatmaps = repo.get_recent_heatmaps(10, Some("en")).unwrap();
        assert_eq!(en_heatmaps.len(), 2);

        let ru_heatmaps = repo.get_recent_heatmaps(10, Some("ru")).unwrap();
        assert_eq!(ru_heatmaps.len(), 0);
    }

    #[test]
    fn pagination_works() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        let repo = SqliteTestRepository::new(&conn);

        for i in 0..15 {
            repo.save_test(make_test_record(40.0 + i as f64, 95.0))
                .unwrap();
        }

        let page1 = repo.get_history(10, 0, None).unwrap();
        let page2 = repo.get_history(10, 10, None).unwrap();
        assert_eq!(page1.len(), 10);
        assert_eq!(page2.len(), 5);
    }

    #[test]
    fn save_multiple_preserve_desc_order() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        let repo = SqliteTestRepository::new(&conn);

        repo.save_test(make_test_record(30.0, 95.0)).unwrap();
        repo.save_test(make_test_record(40.0, 95.0)).unwrap();
        repo.save_test(make_test_record(50.0, 95.0)).unwrap();

        let history = repo.get_history(10, 0, None).unwrap();
        assert_eq!(history.len(), 3);
        // DESC: last inserted first
        assert!((history[0].wpm - 50.0).abs() < 0.01);
        assert!((history[2].wpm - 30.0).abs() < 0.01);
    }
}
