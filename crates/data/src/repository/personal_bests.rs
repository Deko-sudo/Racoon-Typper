//! PersonalBestsRepository — check_and_update, get_bests.

use rusqlite::{params, Connection, OptionalExtension};

use crate::error::DbError;
use crate::models::{config_hash, PersonalBestRow};
use racoon_domain::PersonalBest;

/// Результат проверки рекорда.
#[derive(Debug, Clone)]
pub struct PbUpdate {
    pub metric: String, // "wpm" | "accuracy" | "consistency"
    pub previous: Option<f64>,
    pub new: f64,
    pub test_id: i64,
}

/// Trait для репозитория личных рекордов.
pub trait PersonalBestsRepository {
    fn get_bests(&self, mode_filter: Option<&str>) -> Result<Vec<PersonalBest>, DbError>;
    fn check_and_update(
        &self,
        mode_type: &str,
        mode_config: &str,
        wpm: f64,
        accuracy: f64,
        test_id: i64,
    ) -> Result<Vec<PbUpdate>, DbError>;
}

/// SQLite реализация.
pub struct SqlitePersonalBestsRepository<'a> {
    conn: &'a Connection,
}

impl<'a> SqlitePersonalBestsRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Applies the existing personal-best policy with an explicitly supplied
    /// timestamp. This is intentionally infrastructure-private: the atomic
    /// recovery finalizer supplies the immutable completion timestamp, while
    /// the legacy repository trait retains its wall-clock convenience method.
    pub(crate) fn check_and_update_at(
        &self,
        mode_type: &str,
        mode_config: &str,
        wpm: f64,
        accuracy: f64,
        test_id: i64,
        updated_at: &str,
    ) -> Result<Vec<PbUpdate>, DbError> {
        let hash = config_hash(mode_type, mode_config);
        let consistency = self
            .conn
            .query_row(
                "SELECT consistency FROM tests WHERE id = ?1",
                params![test_id],
                |row| row.get::<_, Option<f64>>(0),
            )
            .optional()
            .map_err(|e| DbError::Query(e.to_string()))?
            .flatten();

        let mut updates = Vec::new();
        let existing: Option<(f64, f64, Option<f64>)> = self
            .conn
            .query_row(
                "SELECT best_wpm, best_accuracy, best_consistency FROM personal_bests
                 WHERE mode_type = ?1 AND mode_config_hash = ?2",
                params![mode_type, hash],
                |row| Ok((row.get(0)?, row.get(1)?, row.get::<_, Option<f64>>(2)?)),
            )
            .optional()
            .map_err(|e| DbError::Query(e.to_string()))?;

        match existing {
            Some((prev_wpm, prev_acc, prev_consistency)) => {
                if wpm > prev_wpm {
                    updates.push(PbUpdate {
                        metric: "wpm".to_string(),
                        previous: Some(prev_wpm),
                        new: wpm,
                        test_id,
                    });
                }
                if accuracy > prev_acc {
                    updates.push(PbUpdate {
                        metric: "accuracy".to_string(),
                        previous: Some(prev_acc),
                        new: accuracy,
                        test_id,
                    });
                }
                if let Some(value) = consistency {
                    if prev_consistency.is_none_or(|previous| value > previous) {
                        updates.push(PbUpdate {
                            metric: "consistency".to_string(),
                            previous: prev_consistency,
                            new: value,
                            test_id,
                        });
                    }
                }
                if !updates.is_empty() {
                    self.conn.execute(
                        "UPDATE personal_bests SET
                            best_wpm = CASE WHEN ?1 > best_wpm THEN ?1 ELSE best_wpm END,
                            best_wpm_test_id = CASE WHEN ?1 > best_wpm THEN ?4 ELSE best_wpm_test_id END,
                            best_accuracy = CASE WHEN ?2 > best_accuracy THEN ?2 ELSE best_accuracy END,
                            best_accuracy_test_id = CASE WHEN ?2 > best_accuracy THEN ?4 ELSE best_accuracy_test_id END,
                            best_consistency = CASE WHEN ?3 IS NOT NULL AND (best_consistency IS NULL OR ?3 > best_consistency) THEN ?3 ELSE best_consistency END,
                            best_consistency_test_id = CASE WHEN ?3 IS NOT NULL AND (best_consistency IS NULL OR ?3 > best_consistency) THEN ?4 ELSE best_consistency_test_id END,
                            updated_at = ?5
                         WHERE mode_type = ?6 AND mode_config_hash = ?7",
                        params![wpm, accuracy, consistency, test_id, updated_at, mode_type, hash],
                    ).map_err(|e| DbError::Write(e.to_string()))?;
                }
            }
            None => {
                self.conn
                    .execute(
                        "INSERT INTO personal_bests (mode_type, mode_config_hash, mode_config,
                 best_wpm, best_wpm_test_id, best_accuracy, best_accuracy_test_id,
                 best_consistency, best_consistency_test_id, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                        params![
                            mode_type,
                            hash,
                            mode_config,
                            wpm,
                            test_id,
                            accuracy,
                            test_id,
                            consistency,
                            consistency.map(|_| test_id),
                            updated_at
                        ],
                    )
                    .map_err(|e| DbError::Write(e.to_string()))?;
                updates.push(PbUpdate {
                    metric: "wpm".to_string(),
                    previous: None,
                    new: wpm,
                    test_id,
                });
                if let Some(value) = consistency {
                    updates.push(PbUpdate {
                        metric: "consistency".to_string(),
                        previous: None,
                        new: value,
                        test_id,
                    });
                }
                updates.push(PbUpdate {
                    metric: "accuracy".to_string(),
                    previous: None,
                    new: accuracy,
                    test_id,
                });
            }
        }
        Ok(updates)
    }
}

impl<'a> PersonalBestsRepository for SqlitePersonalBestsRepository<'a> {
    fn get_bests(&self, mode_filter: Option<&str>) -> Result<Vec<PersonalBest>, DbError> {
        let mut stmt = if mode_filter.is_some() {
            self.conn
                .prepare(
                    "SELECT id, mode_type, mode_config_hash, mode_config, best_wpm,
                     best_wpm_test_id, best_accuracy, best_accuracy_test_id,
                     best_consistency, best_consistency_test_id, updated_at
                     FROM personal_bests WHERE mode_type = ?1
                     ORDER BY updated_at DESC, mode_config_hash ASC",
                )
                .map_err(|e| DbError::Query(e.to_string()))?
        } else {
            self.conn
                .prepare(
                    "SELECT id, mode_type, mode_config_hash, mode_config, best_wpm,
                     best_wpm_test_id, best_accuracy, best_accuracy_test_id,
                     best_consistency, best_consistency_test_id, updated_at
                     FROM personal_bests
                     ORDER BY updated_at DESC, mode_type ASC, mode_config_hash ASC",
                )
                .map_err(|e| DbError::Query(e.to_string()))?
        };

        let rows: Vec<Result<PersonalBestRow, rusqlite::Error>> = match mode_filter {
            Some(mode) => stmt
                .query_map(params![mode], map_pb_row)
                .map_err(|e| DbError::Query(e.to_string()))?
                .collect(),
            None => stmt
                .query_map([], map_pb_row)
                .map_err(|e| DbError::Query(e.to_string()))?
                .collect(),
        };

        let mut bests = Vec::new();
        for row in rows {
            let pb_row = row.map_err(|e| DbError::Query(e.to_string()))?;
            bests.push(PersonalBest::from(pb_row));
        }
        Ok(bests)
    }

    fn check_and_update(
        &self,
        mode_type: &str,
        mode_config: &str,
        wpm: f64,
        accuracy: f64,
        test_id: i64,
    ) -> Result<Vec<PbUpdate>, DbError> {
        self.check_and_update_at(
            mode_type,
            mode_config,
            wpm,
            accuracy,
            test_id,
            &chrono::Utc::now().to_rfc3339(),
        )
    }
}

fn map_pb_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<PersonalBestRow> {
    Ok(PersonalBestRow {
        id: row.get(0)?,
        mode_type: row.get(1)?,
        mode_config_hash: row.get(2)?,
        mode_config: row.get(3)?,
        best_wpm: row.get(4)?,
        best_wpm_test_id: row.get(5)?,
        best_accuracy: row.get(6)?,
        best_accuracy_test_id: row.get(7)?,
        best_consistency: row.get(8)?,
        best_consistency_test_id: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::repository::tests::{SqliteTestRepository, TestRepository};
    use racoon_domain::{SessionId, TestRecord};

    fn save_test_with_repo(conn: &Connection, wpm: f64, acc: f64) -> i64 {
        save_test_with_consistency(conn, wpm, acc, None)
    }

    fn save_test_with_consistency(
        conn: &Connection,
        wpm: f64,
        acc: f64,
        consistency: Option<f64>,
    ) -> i64 {
        let test_repo = SqliteTestRepository::new(conn);
        let record = TestRecord {
            session_id: SessionId::from(format!("personal-best-{wpm}-{acc}")),
            created_at: chrono::Utc::now().to_rfc3339(),
            mode_type: "time".to_string(),
            mode_config: serde_json::json!({"duration": 30}),
            language: "en".to_string(),
            text_length: 100,
            duration_ms: 30000,
            wpm,
            raw_wpm: wpm + 2.0,
            accuracy: acc,
            raw_accuracy: acc - 5.0,
            consistency,
            correct_chars: 95,
            incorrect_chars: 5,
            backspaces: 2,
            char_stats: serde_json::json!({}),
            heatmap_data: serde_json::json!({}),
            graph_data: None,
            is_pb: false,
            tags: "".to_string(),
        };
        test_repo.save_test(record).unwrap()
    }

    #[test]
    fn first_test_creates_pb() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        let pb_repo = SqlitePersonalBestsRepository::new(&conn);

        let test_id = save_test_with_repo(&conn, 45.0, 95.0);
        let updates = pb_repo
            .check_and_update("time", r#"{"duration":30}"#, 45.0, 95.0, test_id)
            .unwrap();

        assert_eq!(updates.len(), 2); // wpm + accuracy
        assert_eq!(updates[0].metric, "wpm");
        assert!(updates[0].previous.is_none());
    }

    #[test]
    fn better_wpm_updates_pb() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        let pb_repo = SqlitePersonalBestsRepository::new(&conn);

        let id1 = save_test_with_repo(&conn, 40.0, 95.0);
        pb_repo
            .check_and_update("time", r#"{"duration":30}"#, 40.0, 95.0, id1)
            .unwrap();

        let id2 = save_test_with_repo(&conn, 50.0, 90.0);
        let updates = pb_repo
            .check_and_update("time", r#"{"duration":30}"#, 50.0, 90.0, id2)
            .unwrap();

        assert_eq!(updates.len(), 1); // только wpm
        assert_eq!(updates[0].metric, "wpm");
        assert_eq!(updates[0].previous, Some(40.0));
        assert_eq!(updates[0].new, 50.0);
    }

    #[test]
    fn worse_wpm_no_update() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        let pb_repo = SqlitePersonalBestsRepository::new(&conn);

        let id1 = save_test_with_repo(&conn, 50.0, 95.0);
        pb_repo
            .check_and_update("time", r#"{"duration":30}"#, 50.0, 95.0, id1)
            .unwrap();

        let id2 = save_test_with_repo(&conn, 40.0, 90.0);
        let updates = pb_repo
            .check_and_update("time", r#"{"duration":30}"#, 40.0, 90.0, id2)
            .unwrap();

        assert!(updates.is_empty());
    }

    #[test]
    fn get_bests() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        let pb_repo = SqlitePersonalBestsRepository::new(&conn);

        let id1 = save_test_with_repo(&conn, 45.0, 95.0);
        pb_repo
            .check_and_update("time", r#"{"duration":30}"#, 45.0, 95.0, id1)
            .unwrap();

        let bests = pb_repo.get_bests(None).unwrap();
        assert_eq!(bests.len(), 1);
        assert!((bests[0].best_wpm - 45.0).abs() < 0.01);
    }

    #[test]
    fn get_bests_with_filter() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        let pb_repo = SqlitePersonalBestsRepository::new(&conn);

        let id1 = save_test_with_repo(&conn, 45.0, 95.0);
        pb_repo
            .check_and_update("time", r#"{"duration":30}"#, 45.0, 95.0, id1)
            .unwrap();

        let time_bests = pb_repo.get_bests(Some("time")).unwrap();
        assert_eq!(time_bests.len(), 1);

        let words_bests = pb_repo.get_bests(Some("words")).unwrap();
        assert_eq!(words_bests.len(), 0);
    }

    #[test]
    fn get_bests_has_a_stable_dimension_tie_breaker() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        let pb_repo = SqlitePersonalBestsRepository::new(&conn);

        for (mode, hash) in [
            ("time", "hash-time"),
            ("custom", "hash-custom"),
            ("quote", "hash-quote"),
        ] {
            conn.execute(
                "INSERT INTO personal_bests (
                    mode_type, mode_config_hash, mode_config, best_wpm, best_accuracy, updated_at
                 ) VALUES (?1, ?2, '{}', 50.0, 95.0, '2026-01-01T00:00:00Z')",
                params![mode, hash],
            )
            .unwrap();
        }

        let bests = pb_repo.get_bests(None).unwrap();
        let modes: Vec<&str> = bests.iter().map(|best| best.mode_type.as_str()).collect();

        assert_eq!(modes, ["custom", "quote", "time"]);
    }

    #[test]
    fn different_configs_separate_pb() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        let pb_repo = SqlitePersonalBestsRepository::new(&conn);

        let id1 = save_test_with_repo(&conn, 45.0, 95.0);
        pb_repo
            .check_and_update("time", r#"{"duration":30}"#, 45.0, 95.0, id1)
            .unwrap();

        let id2 = save_test_with_repo(&conn, 50.0, 90.0);
        pb_repo
            .check_and_update("time", r#"{"duration":60}"#, 50.0, 90.0, id2)
            .unwrap();

        let bests = pb_repo.get_bests(None).unwrap();
        assert_eq!(bests.len(), 2); // разные config → разные PB
    }

    #[test]
    fn consistency_personal_best_is_populated_and_updated() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        let pb_repo = SqlitePersonalBestsRepository::new(&conn);

        let first_id = save_test_with_consistency(&conn, 45.0, 95.0, Some(72.0));
        pb_repo
            .check_and_update("time", r#"{"duration":30}"#, 45.0, 95.0, first_id)
            .unwrap();
        let second_id = save_test_with_consistency(&conn, 44.0, 94.0, Some(88.0));
        let updates = pb_repo
            .check_and_update("time", r#"{"duration":30}"#, 44.0, 94.0, second_id)
            .unwrap();

        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].metric, "consistency");
        let best = pb_repo.get_bests(None).unwrap().pop().unwrap();
        assert_eq!(best.best_consistency, Some(88.0));
        assert_eq!(best.best_consistency_test_id, Some(second_id));
    }
}
