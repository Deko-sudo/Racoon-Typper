//! CustomTextRepository — CRUD для пользовательских текстов.

use rusqlite::{params, Connection};

use crate::error::DbError;

/// Пользовательский текст.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CustomText {
    pub id: i64,
    pub name: String,
    pub text: String,
    pub created_at: String,
    pub last_used_at: Option<String>,
    pub use_count: i64,
}

/// Trait для репозитория пользовательских текстов.
pub trait CustomTextRepository {
    fn get_all(&self, limit: usize) -> Result<Vec<CustomText>, DbError>;
    fn get_by_id(&self, id: i64) -> Result<CustomText, DbError>;
    fn save(&self, name: &str, text: &str) -> Result<i64, DbError>;
    fn update(&self, id: i64, name: &str, text: &str) -> Result<(), DbError>;
    fn delete(&self, id: i64) -> Result<(), DbError>;
    fn increment_use(&self, id: i64) -> Result<(), DbError>;
    fn search(&self, query: &str, limit: usize) -> Result<Vec<CustomText>, DbError>;
}

/// SQLite реализация.
pub struct SqliteCustomTextRepository<'a> {
    conn: &'a Connection,
}

impl<'a> SqliteCustomTextRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }
}

/// Максимальная длина пользовательского текста (10000 символов).
pub const MAX_CUSTOM_TEXT_LENGTH: usize = 10_000;

/// Максимальная длина названия (200 символов).
pub const MAX_NAME_LENGTH: usize = 200;

/// Валидация текста.
pub fn validate_text(name: &str, text: &str) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Name is empty".to_string());
    }
    if name.len() > MAX_NAME_LENGTH {
        return Err(format!("Name too long (max {} chars)", MAX_NAME_LENGTH));
    }
    if text.trim().is_empty() {
        return Err("Text is empty".to_string());
    }
    if text.len() > MAX_CUSTOM_TEXT_LENGTH {
        return Err(format!(
            "Text too long (max {} chars)",
            MAX_CUSTOM_TEXT_LENGTH
        ));
    }
    Ok(())
}

fn map_custom_text_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<CustomText> {
    Ok(CustomText {
        id: row.get(0)?,
        name: row.get(1)?,
        text: row.get(2)?,
        created_at: row.get(3)?,
        last_used_at: row.get(4)?,
        use_count: row.get(5)?,
    })
}

impl<'a> CustomTextRepository for SqliteCustomTextRepository<'a> {
    fn get_all(&self, limit: usize) -> Result<Vec<CustomText>, DbError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, text, created_at, last_used_at, use_count
                 FROM custom_texts ORDER BY last_used_at DESC NULLS LAST LIMIT ?1",
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        let rows: Vec<Result<CustomText, rusqlite::Error>> = stmt
            .query_map(params![limit as i64], map_custom_text_row)
            .map_err(|e| DbError::Query(e.to_string()))?
            .collect();

        let mut texts = Vec::new();
        for row in rows {
            texts.push(row.map_err(|e| DbError::Query(e.to_string()))?);
        }
        Ok(texts)
    }

    fn get_by_id(&self, id: i64) -> Result<CustomText, DbError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, text, created_at, last_used_at, use_count
                 FROM custom_texts WHERE id = ?1",
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        let row = stmt
            .query_row(params![id], map_custom_text_row)
            .map_err(|e| DbError::NotFound(format!("CustomText id={}: {}", id, e)))?;

        Ok(row)
    }

    fn save(&self, name: &str, text: &str) -> Result<i64, DbError> {
        validate_text(name, text).map_err(DbError::Write)?;

        let now = chrono::Utc::now().to_rfc3339();
        self.conn
            .execute(
                "INSERT INTO custom_texts (name, text, created_at, use_count) VALUES (?1, ?2, ?3, 0)",
                params![name, text, now],
            )
            .map_err(|e| DbError::Write(e.to_string()))?;

        Ok(self.conn.last_insert_rowid())
    }

    fn update(&self, id: i64, name: &str, text: &str) -> Result<(), DbError> {
        validate_text(name, text).map_err(DbError::Write)?;

        let affected = self
            .conn
            .execute(
                "UPDATE custom_texts SET name = ?1, text = ?2 WHERE id = ?3",
                params![name, text, id],
            )
            .map_err(|e| DbError::Write(e.to_string()))?;

        if affected == 0 {
            return Err(DbError::NotFound(format!("CustomText id={}", id)));
        }
        Ok(())
    }

    fn delete(&self, id: i64) -> Result<(), DbError> {
        let affected = self
            .conn
            .execute("DELETE FROM custom_texts WHERE id = ?1", params![id])
            .map_err(|e| DbError::Write(e.to_string()))?;

        if affected == 0 {
            return Err(DbError::NotFound(format!("CustomText id={}", id)));
        }
        Ok(())
    }

    fn increment_use(&self, id: i64) -> Result<(), DbError> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn
            .execute(
                "UPDATE custom_texts SET use_count = use_count + 1, last_used_at = ?1 WHERE id = ?2",
                params![now, id],
            )
            .map_err(|e| DbError::Write(e.to_string()))?;
        Ok(())
    }

    fn search(&self, query: &str, limit: usize) -> Result<Vec<CustomText>, DbError> {
        let pattern = format!("%{}%", query);
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, text, created_at, last_used_at, use_count
                 FROM custom_texts WHERE name LIKE ?1 OR text LIKE ?2
                 ORDER BY last_used_at DESC NULLS LAST LIMIT ?3",
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        let rows: Vec<Result<CustomText, rusqlite::Error>> = stmt
            .query_map(params![pattern, pattern, limit as i64], map_custom_text_row)
            .map_err(|e| DbError::Query(e.to_string()))?
            .collect();

        let mut texts = Vec::new();
        for row in rows {
            texts.push(row.map_err(|e| DbError::Query(e.to_string()))?);
        }
        Ok(texts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    fn make_text<'a>(name: &'a str, text: &'a str) -> (&'a str, &'a str) {
        (name, text)
    }

    #[test]
    fn save_and_get_all() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        let repo = SqliteCustomTextRepository::new(&conn);

        let id = repo.save("Test 1", "Hello world").unwrap();
        assert!(id > 0);

        let texts = repo.get_all(10).unwrap();
        assert_eq!(texts.len(), 1);
        assert_eq!(texts[0].name, "Test 1");
        assert_eq!(texts[0].text, "Hello world");
        assert_eq!(texts[0].use_count, 0);
    }

    #[test]
    fn get_by_id() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        let repo = SqliteCustomTextRepository::new(&conn);

        let id = repo.save("My Text", "The quick brown fox").unwrap();
        let text = repo.get_by_id(id).unwrap();
        assert_eq!(text.id, id);
        assert_eq!(text.name, "My Text");
    }

    #[test]
    fn get_by_id_not_found() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        let repo = SqliteCustomTextRepository::new(&conn);

        let result = repo.get_by_id(999);
        assert!(result.is_err());
    }

    #[test]
    fn update_text() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        let repo = SqliteCustomTextRepository::new(&conn);

        let id = repo.save("Original", "Original text").unwrap();
        repo.update(id, "Updated", "Updated text").unwrap();

        let text = repo.get_by_id(id).unwrap();
        assert_eq!(text.name, "Updated");
        assert_eq!(text.text, "Updated text");
    }

    #[test]
    fn update_nonexistent_fails() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        let repo = SqliteCustomTextRepository::new(&conn);

        let result = repo.update(999, "Name", "Text");
        assert!(result.is_err());
    }

    #[test]
    fn delete_text() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        let repo = SqliteCustomTextRepository::new(&conn);

        let id = repo.save("To Delete", "Delete me").unwrap();
        repo.delete(id).unwrap();

        let result = repo.get_by_id(id);
        assert!(result.is_err());

        let texts = repo.get_all(10).unwrap();
        assert_eq!(texts.len(), 0);
    }

    #[test]
    fn delete_nonexistent_fails() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        let repo = SqliteCustomTextRepository::new(&conn);

        let result = repo.delete(999);
        assert!(result.is_err());
    }

    #[test]
    fn increment_use_count() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        let repo = SqliteCustomTextRepository::new(&conn);

        let id = repo.save("Counter", "Test text").unwrap();
        assert_eq!(repo.get_by_id(id).unwrap().use_count, 0);
        assert!(repo.get_by_id(id).unwrap().last_used_at.is_none());

        repo.increment_use(id).unwrap();
        repo.increment_use(id).unwrap();
        repo.increment_use(id).unwrap();

        let text = repo.get_by_id(id).unwrap();
        assert_eq!(text.use_count, 3);
        assert!(text.last_used_at.is_some());
    }

    #[test]
    fn search_by_name() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        let repo = SqliteCustomTextRepository::new(&conn);

        repo.save("Python tutorial", "print hello").unwrap();
        repo.save("Rust guide", "fn main()").unwrap();
        repo.save("Python advanced", "asyncio").unwrap();

        let results = repo.search("Python", 10).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn search_by_text_content() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        let repo = SqliteCustomTextRepository::new(&conn);

        repo.save("Text 1", "The quick brown fox").unwrap();
        repo.save("Text 2", "lazy dog sleeps").unwrap();

        let results = repo.search("quick", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Text 1");
    }

    #[test]
    fn order_by_last_used() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        let repo = SqliteCustomTextRepository::new(&conn);

        let _id1 = repo.save("First", "text1").unwrap();
        let id2 = repo.save("Second", "text2").unwrap();
        let id3 = repo.save("Third", "text3").unwrap();

        // Используем Second и Third
        repo.increment_use(id2).unwrap();
        repo.increment_use(id3).unwrap();
        repo.increment_use(id3).unwrap();

        let texts = repo.get_all(10).unwrap();
        // Third (used 2x) → Second (used 1x) → First (never used)
        assert_eq!(texts[0].name, "Third");
        assert_eq!(texts[1].name, "Second");
        assert_eq!(texts[2].name, "First");
    }

    #[test]
    fn validate_empty_name_rejected() {
        assert!(validate_text("", "hello").is_err());
        assert!(validate_text("   ", "hello").is_err());
    }

    #[test]
    fn validate_empty_text_rejected() {
        assert!(validate_text("Name", "").is_err());
        assert!(validate_text("Name", "   ").is_err());
    }

    #[test]
    fn validate_too_long_name_rejected() {
        let long_name = "a".repeat(MAX_NAME_LENGTH + 1);
        assert!(validate_text(&long_name, "text").is_err());
    }

    #[test]
    fn validate_too_long_text_rejected() {
        let long_text = "a".repeat(MAX_CUSTOM_TEXT_LENGTH + 1);
        assert!(validate_text("Name", &long_text).is_err());
    }

    #[test]
    fn validate_valid_text_accepted() {
        assert!(validate_text("My Text", "Hello world").is_ok());
        assert!(validate_text("A", "x").is_ok());
    }

    #[test]
    fn save_invalid_name_rejected() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        let repo = SqliteCustomTextRepository::new(&conn);

        let result = repo.save("", "hello");
        assert!(result.is_err());
    }

    #[test]
    fn save_invalid_text_rejected() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        let repo = SqliteCustomTextRepository::new(&conn);

        let result = repo.save("Name", "");
        assert!(result.is_err());
    }

    #[test]
    fn multiple_texts_preserve_order() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        let repo = SqliteCustomTextRepository::new(&conn);

        for i in 0..5 {
            repo.save(&format!("Text {}", i), &format!("content {}", i))
                .unwrap();
        }

        let texts = repo.get_all(10).unwrap();
        assert_eq!(texts.len(), 5);
    }
}
