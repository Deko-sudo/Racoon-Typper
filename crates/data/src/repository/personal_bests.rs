//! PersonalBestsRepository — check_and_update, get_bests.

use rusqlite::{params, Connection};

use crate::error::DbError;
use crate::models::{config_hash, PersonalBestRow};
use racoon_domain::PersonalBest;

/// Результат проверки рекорда.
#[derive(Debug, Clone)]
pub struct PbUpdate {
    pub metric: String, // "wpm" | "accuracy"
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
}

impl<'a> PersonalBestsRepository for SqlitePersonalBestsRepository<'a> {
    fn get_bests(&self, mode_filter: Option<&str>) -> Result<Vec<PersonalBest>, DbError> {
        let mut stmt = if mode_filter.is_some() {
            self.conn
                .prepare(
                    "SELECT id, mode_type, mode_config_hash, mode_config, best_wpm,
                     best_wpm_test_id, best_accuracy, best_accuracy_test_id,
                     best_consistency, best_consistency_test_id, updated_at
                     FROM personal_bests WHERE mode_type = ?1 ORDER BY updated_at DESC",
                )
                .map_err(|e| DbError::Query(e.to_string()))?
        } else {
            self.conn
                .prepare(
                    "SELECT id, mode_type, mode_config_hash, mode_config, best_wpm,
                     best_wpm_test_id, best_accuracy, best_accuracy_test_id,
                     best_consistency, best_consistency_test_id, updated_at
                     FROM personal_bests ORDER BY updated_at DESC",
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
        let hash = config_hash(mode_type, mode_config);
        let now = chrono::Utc::now().to_rfc3339();

        let mut updates = Vec::new();

        // Проверяем существующую запись
        let existing: Option<(f64, f64, Option<f64>)> = self
            .conn
            .query_row(
                "SELECT best_wpm, best_accuracy, best_consistency FROM personal_bests
                 WHERE mode_type = ?1 AND mode_config_hash = ?2",
                params![mode_type, hash],
                |row| Ok((row.get(0)?, row.get(1)?, row.get::<_, Option<f64>>(2)?)),
            )
            .ok();

        match existing {
            Some((prev_wpm, prev_acc, _)) => {
                // Обновляем если превзойдён
                let mut need_update = false;
                let mut new_best_wpm = prev_wpm;
                let mut new_best_wpm_test_id: Option<i64> = None;

                if wpm > prev_wpm {
                    updates.push(PbUpdate {
                        metric: "wpm".to_string(),
                        previous: Some(prev_wpm),
                        new: wpm,
                        test_id,
                    });
                    new_best_wpm = wpm;
                    new_best_wpm_test_id = Some(test_id);
                    need_update = true;
                }

                if accuracy > prev_acc {
                    updates.push(PbUpdate {
                        metric: "accuracy".to_string(),
                        previous: Some(prev_acc),
                        new: accuracy,
                        test_id,
                    });
                    need_update = true;
                }

                if need_update {
                    self.conn
                        .execute(
                            "UPDATE personal_bests SET best_wpm = ?1, best_wpm_test_id = ?2,
                         best_accuracy = ?3, best_accuracy_test_id = ?4, updated_at = ?5
                         WHERE mode_type = ?6 AND mode_config_hash = ?7",
                            params![
                                new_best_wpm,
                                new_best_wpm_test_id,
                                accuracy.max(prev_acc),
                                if accuracy > prev_acc {
                                    Some(test_id)
                                } else {
                                    None
                                },
                                now,
                                mode_type,
                                hash,
                            ],
                        )
                        .map_err(|e| DbError::Write(e.to_string()))?;
                }
            }
            None => {
                // Первая запись — все метрики = рекорды
                self.conn
                    .execute(
                        "INSERT INTO personal_bests (mode_type, mode_config_hash, mode_config,
                     best_wpm, best_wpm_test_id, best_accuracy, best_accuracy_test_id,
                     best_consistency, best_consistency_test_id, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, NULL, ?8)",
                        params![
                            mode_type,
                            hash,
                            mode_config,
                            wpm,
                            test_id,
                            accuracy,
                            test_id,
                            now,
                        ],
                    )
                    .map_err(|e| DbError::Write(e.to_string()))?;

                updates.push(PbUpdate {
                    metric: "wpm".to_string(),
                    previous: None,
                    new: wpm,
                    test_id,
                });
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
    use racoon_domain::TestRecord;

    fn save_test_with_repo(conn: &Connection, wpm: f64, acc: f64) -> i64 {
        let test_repo = SqliteTestRepository::new(conn);
        let record = TestRecord {
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
            consistency: None,
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
}
