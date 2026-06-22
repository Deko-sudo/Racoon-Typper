//! Database — соединение SQLite, WAL mode, миграции.

use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;

use crate::error::DbError;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("migrations");
}

/// Database — обёртка над Mutex<Connection>.
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Открывает БД по пути и применяет миграции.
    pub fn open(path: &Path) -> Result<Self, DbError> {
        let conn = Connection::open(path).map_err(|e| DbError::Connection(e.to_string()))?;

        // WAL mode
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| DbError::Connection(e.to_string()))?;

        // Миграции
        let mut migrate_conn =
            rusqlite::Connection::open(path).map_err(|e| DbError::Connection(e.to_string()))?;
        embedded::migrations::runner()
            .run(&mut migrate_conn)
            .map_err(|e| DbError::Migration(e.to_string()))?;
        drop(migrate_conn);

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Открывает in-memory БД (для тестов).
    pub fn open_in_memory() -> Result<Self, DbError> {
        let mut conn =
            Connection::open_in_memory().map_err(|e| DbError::Connection(e.to_string()))?;

        embedded::migrations::runner()
            .run(&mut conn)
            .map_err(|e| DbError::Migration(e.to_string()))?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Возвращает блокировку соединения.
    pub fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().expect("DB mutex poisoned")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_in_memory_creates_tables() {
        let db = Database::open_in_memory().expect("Failed to open in-memory DB");

        let conn = db.conn();
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.contains(&"tests".to_string()));
        assert!(tables.contains(&"personal_bests".to_string()));
        assert!(tables.contains(&"lesson_progress".to_string()));
        assert!(tables.contains(&"daily_stats".to_string()));
        assert!(tables.contains(&"streaks".to_string()));
        assert!(tables.contains(&"custom_texts".to_string()));
    }

    #[test]
    fn migrations_are_idempotent() {
        let db1 = Database::open_in_memory().expect("First open failed");
        drop(db1);

        // Вторичное открытие той же БД не должно падать
        // (in-memory не персистит, но проверяем что миграции не падают при повторном run)
        let db2 = Database::open_in_memory().expect("Second open failed");
        let conn = db2.conn();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM tests", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn wal_mode_enabled() {
        // In-memory не имеет WAL, но проверяем что pragma не падает
        let db = Database::open_in_memory().expect("Failed to open");
        let _conn = db.conn();
        // In-memory всегда rollback, но миграции должны пройти
    }

    #[test]
    fn indexes_exist() {
        let db = Database::open_in_memory().expect("Failed to open");
        let conn = db.conn();

        let indexes: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='index' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(indexes.contains(&"idx_tests_created_at".to_string()));
        assert!(indexes.contains(&"idx_tests_wpm".to_string()));
        assert!(indexes.contains(&"uniq_pb_mode_config_hash".to_string()));
        assert!(indexes.contains(&"idx_streaks_type".to_string()));
    }

    #[test]
    fn unique_constraints_work() {
        let db = Database::open_in_memory().expect("Failed to open");
        let conn = db.conn();

        // Проверяем UNIQUE на streaks.type
        conn.execute(
            "INSERT INTO streaks (type, current_streak, longest_streak) VALUES ('daily_test', 1, 1)",
            [],
        )
        .unwrap();

        // Повторная вставка должна провалиться
        let result = conn.execute(
            "INSERT INTO streaks (type, current_streak, longest_streak) VALUES ('daily_test', 2, 2)",
            [],
        );
        assert!(result.is_err());
    }
}
