//! LessonRepository — stub для Sprint 4. Реализация в Sprint 5.

use rusqlite::Connection;

use crate::error::DbError;

pub trait LessonRepository {
    fn get_progress(&self, language: &str) -> Result<Vec<racoon_domain::LessonStatus>, DbError>;
    fn save_progress(
        &self,
        lesson_id: &str,
        status: &str,
        wpm: f64,
        accuracy: f64,
    ) -> Result<(), DbError>;
}

pub struct SqliteLessonRepository<'a> {
    #[allow(dead_code)]
    conn: &'a Connection,
}

impl<'a> SqliteLessonRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }
}

impl<'a> LessonRepository for SqliteLessonRepository<'a> {
    fn get_progress(&self, _language: &str) -> Result<Vec<racoon_domain::LessonStatus>, DbError> {
        Ok(Vec::new()) // stub
    }

    fn save_progress(
        &self,
        _lesson_id: &str,
        _status: &str,
        _wpm: f64,
        _accuracy: f64,
    ) -> Result<(), DbError> {
        Ok(()) // stub
    }
}
