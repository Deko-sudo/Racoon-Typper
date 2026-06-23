//! LessonRepository — полная реализация для Sprint 9.
//! Прогресс уроков, модулей, курсов.

use rusqlite::{params, Connection};

use crate::error::DbError;

/// Прогресс одного урока.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LessonProgressRecord {
    pub id: i64,
    pub lesson_id: String,
    pub module_id: String,
    pub language: String,
    pub difficulty: String,
    pub status: String,
    pub best_wpm: f64,
    pub best_accuracy: f64,
    pub attempts: i64,
    pub last_attempt_at: Option<String>,
    pub completed_at: Option<String>,
}

/// Прогресс модуля (агрегация).
#[derive(Debug, Clone, serde::Serialize)]
pub struct ModuleProgress {
    pub module_id: String,
    pub total_lessons: i64,
    pub completed_lessons: i64,
    pub in_progress_lessons: i64,
    pub not_started_lessons: i64,
    pub best_wpm: f64,
    pub best_accuracy: f64,
}

/// Прогресс курса (агрегация по модулям).
#[derive(Debug, Clone, serde::Serialize)]
pub struct CourseProgress {
    pub language: String,
    pub total_modules: i64,
    pub total_lessons: i64,
    pub completed_lessons: i64,
    pub overall_progress: f64,
    pub modules: Vec<ModuleProgress>,
}

pub trait LessonRepository {
    fn create_progress(
        &self,
        lesson_id: &str,
        module_id: &str,
        language: &str,
        difficulty: &str,
    ) -> Result<i64, DbError>;
    fn get_progress(&self, language: &str) -> Result<Vec<LessonProgressRecord>, DbError>;
    fn get_lesson_progress(&self, lesson_id: &str)
        -> Result<Option<LessonProgressRecord>, DbError>;
    fn update_progress(&self, lesson_id: &str, wpm: f64, accuracy: f64) -> Result<(), DbError>;
    fn complete_lesson(&self, lesson_id: &str, wpm: f64, accuracy: f64) -> Result<(), DbError>;
    fn get_course_progress(&self, language: &str) -> Result<CourseProgress, DbError>;
}

pub struct SqliteLessonRepository<'a> {
    conn: &'a Connection,
}

impl<'a> SqliteLessonRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }
}

impl<'a> LessonRepository for SqliteLessonRepository<'a> {
    fn create_progress(
        &self,
        lesson_id: &str,
        module_id: &str,
        language: &str,
        difficulty: &str,
    ) -> Result<i64, DbError> {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO lesson_progress
                    (lesson_id, module_id, language, difficulty, status, best_wpm, best_accuracy, attempts, last_attempt_at, completed_at)
                 VALUES (?1, ?2, ?3, ?4, 'not_started', 0.0, 0.0, 0, NULL, NULL)",
                params![lesson_id, module_id, language, difficulty],
            )
            .map_err(|e| DbError::Write(e.to_string()))?;

        let id: i64 = self
            .conn
            .query_row(
                "SELECT id FROM lesson_progress WHERE lesson_id = ?1",
                params![lesson_id],
                |row| row.get(0),
            )
            .map_err(|e| DbError::Query(e.to_string()))?;
        Ok(id)
    }

    fn get_progress(&self, language: &str) -> Result<Vec<LessonProgressRecord>, DbError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, lesson_id, module_id, language, difficulty, status,
                        best_wpm, best_accuracy, attempts, last_attempt_at, completed_at
                 FROM lesson_progress WHERE language = ?1 ORDER BY module_id, lesson_id",
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        let rows = stmt
            .query_map(params![language], |row| {
                Ok(LessonProgressRecord {
                    id: row.get(0)?,
                    lesson_id: row.get(1)?,
                    module_id: row.get(2)?,
                    language: row.get(3)?,
                    difficulty: row.get(4)?,
                    status: row.get(5)?,
                    best_wpm: row.get(6)?,
                    best_accuracy: row.get(7)?,
                    attempts: row.get(8)?,
                    last_attempt_at: row.get(9)?,
                    completed_at: row.get(10)?,
                })
            })
            .map_err(|e| DbError::Query(e.to_string()))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| DbError::Query(e.to_string()))?);
        }
        Ok(result)
    }

    fn get_lesson_progress(
        &self,
        lesson_id: &str,
    ) -> Result<Option<LessonProgressRecord>, DbError> {
        let result = self.conn.query_row(
            "SELECT id, lesson_id, module_id, language, difficulty, status,
                    best_wpm, best_accuracy, attempts, last_attempt_at, completed_at
             FROM lesson_progress WHERE lesson_id = ?1",
            params![lesson_id],
            |row| {
                Ok(LessonProgressRecord {
                    id: row.get(0)?,
                    lesson_id: row.get(1)?,
                    module_id: row.get(2)?,
                    language: row.get(3)?,
                    difficulty: row.get(4)?,
                    status: row.get(5)?,
                    best_wpm: row.get(6)?,
                    best_accuracy: row.get(7)?,
                    attempts: row.get(8)?,
                    last_attempt_at: row.get(9)?,
                    completed_at: row.get(10)?,
                })
            },
        );

        match result {
            Ok(record) => Ok(Some(record)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::Query(e.to_string())),
        }
    }

    fn update_progress(&self, lesson_id: &str, wpm: f64, accuracy: f64) -> Result<(), DbError> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn
            .execute(
                "UPDATE lesson_progress
                 SET best_wpm = CASE WHEN ?1 > best_wpm THEN ?1 ELSE best_wpm END,
                     best_accuracy = CASE WHEN ?2 > best_accuracy THEN ?2 ELSE best_accuracy END,
                     attempts = attempts + 1,
                     last_attempt_at = ?3,
                     status = CASE WHEN status = 'not_started' THEN 'in_progress' ELSE status END
                 WHERE lesson_id = ?4",
                params![wpm, accuracy, now, lesson_id],
            )
            .map_err(|e| DbError::Write(e.to_string()))?;
        Ok(())
    }

    fn complete_lesson(&self, lesson_id: &str, wpm: f64, accuracy: f64) -> Result<(), DbError> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn
            .execute(
                "UPDATE lesson_progress
                 SET best_wpm = CASE WHEN ?1 > best_wpm THEN ?1 ELSE best_wpm END,
                     best_accuracy = CASE WHEN ?2 > best_accuracy THEN ?2 ELSE best_accuracy END,
                     attempts = attempts + 1,
                     last_attempt_at = ?3,
                     completed_at = ?3,
                     status = 'completed'
                 WHERE lesson_id = ?4",
                params![wpm, accuracy, now, lesson_id],
            )
            .map_err(|e| DbError::Write(e.to_string()))?;
        Ok(())
    }

    fn get_course_progress(&self, language: &str) -> Result<CourseProgress, DbError> {
        let records = self.get_progress(language)?;

        if records.is_empty() {
            return Ok(CourseProgress {
                language: language.to_string(),
                total_modules: 0,
                total_lessons: 0,
                completed_lessons: 0,
                overall_progress: 0.0,
                modules: Vec::new(),
            });
        }

        // Group by module_id
        use std::collections::HashMap;
        let mut modules_map: HashMap<String, Vec<&LessonProgressRecord>> = HashMap::new();
        for r in &records {
            modules_map.entry(r.module_id.clone()).or_default().push(r);
        }

        let mut modules = Vec::new();
        let mut total_lessons = 0i64;
        let mut completed_lessons = 0i64;

        for (module_id, lessons) in &modules_map {
            let total = lessons.len() as i64;
            let completed = lessons.iter().filter(|l| l.status == "completed").count() as i64;
            let in_progress = lessons.iter().filter(|l| l.status == "in_progress").count() as i64;
            let not_started = lessons.iter().filter(|l| l.status == "not_started").count() as i64;
            let best_wpm = lessons.iter().map(|l| l.best_wpm).fold(0.0_f64, f64::max);
            let best_accuracy = lessons
                .iter()
                .map(|l| l.best_accuracy)
                .fold(0.0_f64, f64::max);

            modules.push(ModuleProgress {
                module_id: module_id.clone(),
                total_lessons: total,
                completed_lessons: completed,
                in_progress_lessons: in_progress,
                not_started_lessons: not_started,
                best_wpm,
                best_accuracy,
            });

            total_lessons += total;
            completed_lessons += completed;
        }

        modules.sort_by(|a, b| a.module_id.cmp(&b.module_id));

        let overall_progress = if total_lessons > 0 {
            (completed_lessons as f64 / total_lessons as f64) * 100.0
        } else {
            0.0
        };

        Ok(CourseProgress {
            language: language.to_string(),
            total_modules: modules_map.len() as i64,
            total_lessons,
            completed_lessons,
            overall_progress,
            modules,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> SqliteLessonRepository<'static> {
        let conn = Box::leak(Box::new(rusqlite::Connection::open_in_memory().unwrap()));
        crate::db::run_migrations(conn);
        SqliteLessonRepository::new(conn)
    }

    #[test]
    fn create_progress_new() {
        let repo = setup();
        let id = repo
            .create_progress("en_m1_l1", "en_m1", "en", "beginner")
            .unwrap();
        assert!(id > 0);
    }

    #[test]
    fn create_progress_idempotent() {
        let repo = setup();
        let id1 = repo
            .create_progress("en_m1_l1", "en_m1", "en", "beginner")
            .unwrap();
        let id2 = repo
            .create_progress("en_m1_l1", "en_m1", "en", "beginner")
            .unwrap();
        assert_eq!(id1, id2, "Second create should not insert duplicate");
    }

    #[test]
    fn get_progress_empty() {
        let repo = setup();
        let progress = repo.get_progress("en").unwrap();
        assert!(progress.is_empty());
    }

    #[test]
    fn get_progress_after_create() {
        let repo = setup();
        repo.create_progress("en_m1_l1", "en_m1", "en", "beginner")
            .unwrap();
        repo.create_progress("en_m1_l2", "en_m1", "en", "beginner")
            .unwrap();
        let progress = repo.get_progress("en").unwrap();
        assert_eq!(progress.len(), 2);
    }

    #[test]
    fn get_progress_filters_by_language() {
        let repo = setup();
        repo.create_progress("en_m1_l1", "en_m1", "en", "beginner")
            .unwrap();
        repo.create_progress("ru_m1_l1", "ru_m1", "ru", "beginner")
            .unwrap();
        let en = repo.get_progress("en").unwrap();
        let ru = repo.get_progress("ru").unwrap();
        assert_eq!(en.len(), 1);
        assert_eq!(ru.len(), 1);
    }

    #[test]
    fn get_lesson_progress_existing() {
        let repo = setup();
        repo.create_progress("en_m1_l1", "en_m1", "en", "beginner")
            .unwrap();
        let p = repo.get_lesson_progress("en_m1_l1").unwrap();
        assert!(p.is_some());
        assert_eq!(p.unwrap().status, "not_started");
    }

    #[test]
    fn get_lesson_progress_nonexistent() {
        let repo = setup();
        let p = repo.get_lesson_progress("nonexistent").unwrap();
        assert!(p.is_none());
    }

    #[test]
    fn update_progress_sets_in_progress() {
        let repo = setup();
        repo.create_progress("en_m1_l1", "en_m1", "en", "beginner")
            .unwrap();
        repo.update_progress("en_m1_l1", 30.0, 95.0).unwrap();
        let p = repo.get_lesson_progress("en_m1_l1").unwrap().unwrap();
        assert_eq!(p.status, "in_progress");
        assert!((p.best_wpm - 30.0).abs() < 0.01);
        assert!((p.best_accuracy - 95.0).abs() < 0.01);
        assert_eq!(p.attempts, 1);
    }

    #[test]
    fn update_progress_keeps_best_wpm() {
        let repo = setup();
        repo.create_progress("en_m1_l1", "en_m1", "en", "beginner")
            .unwrap();
        repo.update_progress("en_m1_l1", 40.0, 95.0).unwrap();
        repo.update_progress("en_m1_l1", 35.0, 90.0).unwrap();
        let p = repo.get_lesson_progress("en_m1_l1").unwrap().unwrap();
        assert!((p.best_wpm - 40.0).abs() < 0.01, "Best WPM should stay 40");
        assert!(
            (p.best_accuracy - 95.0).abs() < 0.01,
            "Best accuracy should stay 95"
        );
        assert_eq!(p.attempts, 2);
    }

    #[test]
    fn complete_lesson_sets_completed() {
        let repo = setup();
        repo.create_progress("en_m1_l1", "en_m1", "en", "beginner")
            .unwrap();
        repo.complete_lesson("en_m1_l1", 45.0, 98.0).unwrap();
        let p = repo.get_lesson_progress("en_m1_l1").unwrap().unwrap();
        assert_eq!(p.status, "completed");
        assert!(p.completed_at.is_some());
        assert!((p.best_wpm - 45.0).abs() < 0.01);
        assert!((p.best_accuracy - 98.0).abs() < 0.01);
    }

    #[test]
    fn complete_lesson_increments_attempts() {
        let repo = setup();
        repo.create_progress("en_m1_l1", "en_m1", "en", "beginner")
            .unwrap();
        repo.update_progress("en_m1_l1", 30.0, 90.0).unwrap();
        repo.complete_lesson("en_m1_l1", 40.0, 95.0).unwrap();
        let p = repo.get_lesson_progress("en_m1_l1").unwrap().unwrap();
        assert_eq!(p.attempts, 2);
    }

    #[test]
    fn get_course_progress_empty() {
        let repo = setup();
        let cp = repo.get_course_progress("en").unwrap();
        assert_eq!(cp.total_lessons, 0);
        assert!((cp.overall_progress).abs() < 0.01);
    }

    #[test]
    fn get_course_progress_with_data() {
        let repo = setup();
        // Module 1: 3 lessons
        repo.create_progress("en_m1_l1", "en_m1", "en", "beginner")
            .unwrap();
        repo.create_progress("en_m1_l2", "en_m1", "en", "beginner")
            .unwrap();
        repo.create_progress("en_m1_l3", "en_m1", "en", "beginner")
            .unwrap();
        // Module 2: 2 lessons
        repo.create_progress("en_m2_l1", "en_m2", "en", "intermediate")
            .unwrap();
        repo.create_progress("en_m2_l2", "en_m2", "en", "intermediate")
            .unwrap();

        // Complete 2 lessons in module 1
        repo.complete_lesson("en_m1_l1", 40.0, 95.0).unwrap();
        repo.complete_lesson("en_m1_l2", 45.0, 98.0).unwrap();

        let cp = repo.get_course_progress("en").unwrap();
        assert_eq!(cp.total_modules, 2);
        assert_eq!(cp.total_lessons, 5);
        assert_eq!(cp.completed_lessons, 2);
        assert!((cp.overall_progress - 40.0).abs() < 0.01); // 2/5 = 40%
    }

    #[test]
    fn get_course_progress_module_stats() {
        let repo = setup();
        repo.create_progress("en_m1_l1", "en_m1", "en", "beginner")
            .unwrap();
        repo.create_progress("en_m1_l2", "en_m1", "en", "beginner")
            .unwrap();
        repo.complete_lesson("en_m1_l1", 40.0, 95.0).unwrap();
        repo.update_progress("en_m1_l2", 35.0, 90.0).unwrap();

        let cp = repo.get_course_progress("en").unwrap();
        assert_eq!(cp.modules.len(), 1);
        let m = &cp.modules[0];
        assert_eq!(m.total_lessons, 2);
        assert_eq!(m.completed_lessons, 1);
        assert_eq!(m.in_progress_lessons, 1);
        assert_eq!(m.not_started_lessons, 0);
    }

    #[test]
    fn update_progress_nonexistent_no_error() {
        let repo = setup();
        // UPDATE on nonexistent row = 0 rows affected, not an error
        repo.update_progress("nonexistent", 30.0, 95.0).unwrap();
    }

    #[test]
    fn complete_lesson_nonexistent_no_error() {
        let repo = setup();
        repo.complete_lesson("nonexistent", 30.0, 95.0).unwrap();
    }

    #[test]
    fn last_attempt_at_set_on_update() {
        let repo = setup();
        repo.create_progress("en_m1_l1", "en_m1", "en", "beginner")
            .unwrap();
        repo.update_progress("en_m1_l1", 30.0, 95.0).unwrap();
        let p = repo.get_lesson_progress("en_m1_l1").unwrap().unwrap();
        assert!(p.last_attempt_at.is_some());
    }

    #[test]
    fn multiple_modules_progress() {
        let repo = setup();
        for i in 1..=3 {
            repo.create_progress(&format!("en_m1_l{}", i), "en_m1", "en", "beginner")
                .unwrap();
        }
        for i in 1..=2 {
            repo.create_progress(&format!("en_m2_l{}", i), "en_m2", "en", "intermediate")
                .unwrap();
        }
        repo.complete_lesson("en_m1_l1", 40.0, 95.0).unwrap();
        let cp = repo.get_course_progress("en").unwrap();
        assert_eq!(cp.modules.len(), 2);
        assert_eq!(cp.modules[0].module_id, "en_m1");
        assert_eq!(cp.modules[1].module_id, "en_m2");
    }
}
