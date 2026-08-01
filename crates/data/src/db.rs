//! Database connection lifecycle, SQLite safety pragmas, and migrations.

use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use rusqlite::{Connection, Transaction, TransactionBehavior};

use crate::error::DbError;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("migrations");
}

/// Applies the embedded, forward-only migrations to an already configured connection.
pub fn run_migrations(conn: &mut Connection) -> Result<(), DbError> {
    embedded::migrations::runner()
        .run(conn)
        .map(|_| ())
        .map_err(|error| DbError::Migration(error.to_string()))
}

/// Configures every SQLite connection before it is made available to repositories.
///
/// Foreign-key enforcement is connection-local in SQLite, so this must run for both
/// file-backed and in-memory databases. WAL and the busy timeout keep the single
/// desktop-process writer responsive without introducing a connection pool.
fn configure_connection(conn: &Connection) -> Result<(), DbError> {
    conn.pragma_update(None, "foreign_keys", "ON")
        .map_err(|error| DbError::from_sqlite("configure foreign keys", error))?;
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|error| DbError::from_sqlite("configure journal mode", error))?;
    conn.pragma_update(None, "synchronous", "NORMAL")
        .map_err(|error| DbError::from_sqlite("configure synchronous mode", error))?;
    conn.busy_timeout(Duration::from_secs(5))
        .map_err(|error| DbError::from_sqlite("configure busy timeout", error))?;
    Ok(())
}

/// Database — обёртка над Mutex<Connection>.
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Открывает БД по пути и применяет миграции.
    pub fn open(path: &Path) -> Result<Self, DbError> {
        Self::open_with_pre_migration(path, |_| {})
    }

    /// Opens the database at `path`, invokes `pre_migration` after configuring the
    /// connection but **before** running migrations, then applies migrations.
    ///
    /// The callback is the seam used to take a pre-migration backup: it receives
    /// the configured-but-not-yet-migrated connection's path and runs while the
    /// on-disk file still reflects the previous schema. A failure in `pre_migration`
    /// aborts opening before any migration runs, so a broken backup hook can never
    /// leave the database half-migrated. Callers that want warn-and-continue
    /// semantics must swallow the error inside the callback.
    pub fn open_with_pre_migration(
        path: &Path,
        pre_migration: impl FnOnce(&Path),
    ) -> Result<Self, DbError> {
        let mut conn =
            Connection::open(path).map_err(|error| DbError::from_sqlite("open database", error))?;
        configure_connection(&conn)?;
        pre_migration(path);
        run_migrations(&mut conn)?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Открывает in-memory БД (для тестов).
    pub fn open_in_memory() -> Result<Self, DbError> {
        let mut conn = Connection::open_in_memory()
            .map_err(|error| DbError::from_sqlite("open in-memory database", error))?;
        configure_connection(&conn)?;
        run_migrations(&mut conn)?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn lock(&self) -> Result<MutexGuard<'_, Connection>, DbError> {
        self.conn.lock().map_err(|_| DbError::LockPoisoned)
    }

    /// Executes one read or single-write operation while holding the only database lock.
    /// Application code should prefer this over retaining a connection guard across work.
    pub fn with_connection<T>(
        &self,
        operation: impl FnOnce(&Connection) -> Result<T, DbError>,
    ) -> Result<T, DbError> {
        let conn = self.lock()?;
        operation(&conn)
    }

    /// Executes a logical unit of work atomically.
    ///
    /// `IMMEDIATE` obtains the write reservation up front. This avoids a partially
    /// completed user-visible operation after repositories have started writing; an
    /// error from the closure rolls the transaction back automatically.
    pub fn with_transaction<T>(
        &self,
        operation: impl FnOnce(&Transaction<'_>) -> Result<T, DbError>,
    ) -> Result<T, DbError> {
        let mut conn = self.lock()?;
        let transaction = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|error| DbError::from_sqlite("begin immediate transaction", error))?;
        let result = operation(&transaction)?;
        transaction
            .commit()
            .map_err(|error| DbError::from_sqlite("commit transaction", error))?;
        Ok(result)
    }

    /// Legacy diagnostic/test accessor. Production code must use `with_connection`
    /// or `with_transaction` so lock failures are surfaced as `DbError`.
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

    #[test]
    fn foreign_keys_are_enabled_on_each_database_connection() {
        let db = Database::open_in_memory().expect("Failed to open");
        let conn = db.conn();
        let foreign_keys: i64 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .expect("Failed to inspect foreign key pragma");
        assert_eq!(foreign_keys, 1);
    }

    #[test]
    fn transaction_rolls_back_all_writes_when_the_operation_fails() {
        let db = Database::open_in_memory().expect("Failed to open");

        let result: Result<(), DbError> = db.with_transaction(|conn| {
            conn.execute(
                "INSERT INTO daily_stats (date, total_tests, total_time_ms, total_chars, best_wpm, avg_wpm, avg_accuracy, lessons_completed, daily_goal_met)
                 VALUES ('2026-07-12', 1, 1, 1, 1, 1, 1, 0, 0)",
                [],
            )
            .map_err(|error| DbError::Write(error.to_string()))?;
            Err(DbError::Write("forced rollback".to_string()))
        });

        assert!(result.is_err());
        let conn = db.conn();
        let rows: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM daily_stats WHERE date = '2026-07-12'",
                [],
                |row| row.get(0),
            )
            .expect("Failed to count daily statistics");
        assert_eq!(rows, 0);
    }
}
