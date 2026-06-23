//! ReplayRepository — сохранение/загрузка replay данных.

use rusqlite::{params, Connection};

use crate::error::DbError;

/// Один кадр replay.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReplayFrame {
    pub id: i64,
    pub test_id: i64,
    pub frame_index: i64,
    pub timestamp_ms: i64,
    pub position: i64,
    pub expected_char: String,
    pub typed_char: Option<String>,
    pub correct: bool,
}

pub trait ReplayRepository {
    fn save_replay(&self, test_id: i64, frames: &[ReplayFrame]) -> Result<(), DbError>;
    fn load_replay(&self, test_id: i64) -> Result<Vec<ReplayFrame>, DbError>;
    fn delete_replay(&self, test_id: i64) -> Result<(), DbError>;
    fn has_replay(&self, test_id: i64) -> Result<bool, DbError>;
}

pub struct SqliteReplayRepository<'a> {
    conn: &'a Connection,
}

impl<'a> SqliteReplayRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }
}

impl<'a> ReplayRepository for SqliteReplayRepository<'a> {
    fn save_replay(&self, test_id: i64, frames: &[ReplayFrame]) -> Result<(), DbError> {
        for frame in frames {
            self.conn
                .execute(
                    "INSERT INTO test_replays (test_id, frame_index, timestamp_ms, position, expected_char, typed_char, correct)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        test_id,
                        frame.frame_index,
                        frame.timestamp_ms,
                        frame.position,
                        frame.expected_char,
                        frame.typed_char,
                        frame.correct
                    ],
                )
                .map_err(|e| DbError::Write(e.to_string()))?;
        }
        Ok(())
    }

    fn load_replay(&self, test_id: i64) -> Result<Vec<ReplayFrame>, DbError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, test_id, frame_index, timestamp_ms, position, expected_char, typed_char, correct
                 FROM test_replays WHERE test_id = ?1 ORDER BY frame_index",
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        let rows = stmt
            .query_map(params![test_id], |row| {
                Ok(ReplayFrame {
                    id: row.get(0)?,
                    test_id: row.get(1)?,
                    frame_index: row.get(2)?,
                    timestamp_ms: row.get(3)?,
                    position: row.get(4)?,
                    expected_char: row.get(5)?,
                    typed_char: row.get(6)?,
                    correct: row.get(7)?,
                })
            })
            .map_err(|e| DbError::Query(e.to_string()))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| DbError::Query(e.to_string()))?);
        }
        Ok(result)
    }

    fn delete_replay(&self, test_id: i64) -> Result<(), DbError> {
        self.conn
            .execute(
                "DELETE FROM test_replays WHERE test_id = ?1",
                params![test_id],
            )
            .map_err(|e| DbError::Write(e.to_string()))?;
        Ok(())
    }

    fn has_replay(&self, test_id: i64) -> Result<bool, DbError> {
        let count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM test_replays WHERE test_id = ?1",
                params![test_id],
                |row| row.get(0),
            )
            .map_err(|e| DbError::Query(e.to_string()))?;
        Ok(count > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> SqliteReplayRepository<'static> {
        let conn = Box::leak(Box::new(rusqlite::Connection::open_in_memory().unwrap()));
        crate::db::run_migrations(conn);
        // Insert a test record for FK
        conn.execute(
            "INSERT INTO tests (created_at, mode_type, mode_config, language, text_length, duration_ms, wpm, raw_wpm, accuracy, raw_accuracy, consistency, correct_chars, incorrect_chars, backspaces, char_stats, heatmap_data, graph_data, is_pb, tags)
             VALUES ('2026-01-01T00:00:00Z', 'time', '{}', 'en', 50, 30000, 40.0, 45.0, 90.0, 85.0, NULL, 45, 5, 2, '{}', '{}', NULL, 0, '')",
            [],
        ).unwrap();
        SqliteReplayRepository::new(conn)
    }

    fn make_frames(test_id: i64) -> Vec<ReplayFrame> {
        vec![
            ReplayFrame {
                id: 0,
                test_id,
                frame_index: 0,
                timestamp_ms: 0,
                position: 0,
                expected_char: "h".to_string(),
                typed_char: Some("h".to_string()),
                correct: true,
            },
            ReplayFrame {
                id: 0,
                test_id,
                frame_index: 1,
                timestamp_ms: 100,
                position: 1,
                expected_char: "e".to_string(),
                typed_char: Some("e".to_string()),
                correct: true,
            },
            ReplayFrame {
                id: 0,
                test_id,
                frame_index: 2,
                timestamp_ms: 200,
                position: 2,
                expected_char: "l".to_string(),
                typed_char: Some("x".to_string()),
                correct: false,
            },
        ]
    }

    #[test]
    fn save_and_load_replay() {
        let repo = setup();
        let frames = make_frames(1);
        repo.save_replay(1, &frames).unwrap();
        let loaded = repo.load_replay(1).unwrap();
        assert_eq!(loaded.len(), 3);
        assert_eq!(loaded[0].expected_char, "h");
        assert_eq!(loaded[2].typed_char.as_deref(), Some("x"));
        assert!(!loaded[2].correct);
    }

    #[test]
    fn load_replay_nonexistent() {
        let repo = setup();
        let loaded = repo.load_replay(999).unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    fn has_replay() {
        let repo = setup();
        assert!(!repo.has_replay(1).unwrap());
        repo.save_replay(1, &make_frames(1)).unwrap();
        assert!(repo.has_replay(1).unwrap());
    }

    #[test]
    fn delete_replay() {
        let repo = setup();
        repo.save_replay(1, &make_frames(1)).unwrap();
        assert!(repo.has_replay(1).unwrap());
        repo.delete_replay(1).unwrap();
        assert!(!repo.has_replay(1).unwrap());
    }

    #[test]
    fn delete_nonexistent_replay() {
        let repo = setup();
        repo.delete_replay(999).unwrap(); // no error
    }

    #[test]
    fn replay_frames_ordered() {
        let repo = setup();
        let mut frames = make_frames(1);
        // Reverse order on save
        frames.reverse();
        repo.save_replay(1, &frames).unwrap();
        let loaded = repo.load_replay(1).unwrap();
        // Should be ordered by frame_index
        assert_eq!(loaded[0].frame_index, 0);
        assert_eq!(loaded[1].frame_index, 1);
        assert_eq!(loaded[2].frame_index, 2);
    }

    #[test]
    fn replay_preserves_correctness() {
        let repo = setup();
        repo.save_replay(1, &make_frames(1)).unwrap();
        let loaded = repo.load_replay(1).unwrap();
        assert!(loaded[0].correct);
        assert!(loaded[1].correct);
        assert!(!loaded[2].correct);
    }

    #[test]
    fn replay_preserves_timestamps() {
        let repo = setup();
        repo.save_replay(1, &make_frames(1)).unwrap();
        let loaded = repo.load_replay(1).unwrap();
        assert_eq!(loaded[0].timestamp_ms, 0);
        assert_eq!(loaded[1].timestamp_ms, 100);
        assert_eq!(loaded[2].timestamp_ms, 200);
    }

    #[test]
    fn replay_multiple_tests() {
        let repo = setup();
        // Insert second test
        repo.conn.execute(
            "INSERT INTO tests (created_at, mode_type, mode_config, language, text_length, duration_ms, wpm, raw_wpm, accuracy, raw_accuracy, consistency, correct_chars, incorrect_chars, backspaces, char_stats, heatmap_data, graph_data, is_pb, tags)
             VALUES ('2026-01-02T00:00:00Z', 'words', '{}', 'ru', 100, 60000, 50.0, 55.0, 92.0, 87.0, NULL, 90, 10, 5, '{}', '{}', NULL, 0, '')",
            [],
        ).unwrap();
        repo.save_replay(1, &make_frames(1)).unwrap();
        repo.save_replay(2, &make_frames(2)).unwrap();
        let r1 = repo.load_replay(1).unwrap();
        let r2 = repo.load_replay(2).unwrap();
        assert_eq!(r1.len(), 3);
        assert_eq!(r2.len(), 3);
        assert_eq!(r1[0].test_id, 1);
        assert_eq!(r2[0].test_id, 2);
    }

    #[test]
    fn replay_null_typed_char() {
        let repo = setup();
        let frames = vec![ReplayFrame {
            id: 0,
            test_id: 1,
            frame_index: 0,
            timestamp_ms: 0,
            position: 0,
            expected_char: "a".to_string(),
            typed_char: None,
            correct: false,
        }];
        repo.save_replay(1, &frames).unwrap();
        let loaded = repo.load_replay(1).unwrap();
        assert!(loaded[0].typed_char.is_none());
    }
}
