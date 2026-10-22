// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

//! Explicit administrative clear-all; ordinary ledger writes retain their guards.

use rusqlite::{Connection, OptionalExtension};

use crate::{Database, DbError};

const DELETE_GUARDS: [(&str, &str); 3] = [
    (
        "session_finalizations_not_deletable",
        "session_finalizations",
    ),
    (
        "session_completion_intents_not_deletable",
        "session_completion_intents",
    ),
    (
        "session_ledger_terminal_row_not_deletable",
        "session_ledger",
    ),
];
const STATISTICS_TABLES: [&str; 9] = [
    "session_finalizations",
    "session_completion_intents",
    "session_ledger",
    "personal_bests",
    "test_replays",
    "tests",
    "daily_stats",
    "streaks",
    "lesson_progress",
];

/// Clears all statistics atomically, preserving custom texts and the schema.
/// The caller must exclude live/pending sessions with its lifecycle/engine gate.
/// See ADR 0004: this administrative operation is never a per-row ledger API.
pub fn clear_statistics(database: &Database) -> Result<(), DbError> {
    database.with_transaction(|conn| reset_in_transaction(conn, Connection::execute_batch))
}

fn guard_sql(conn: &Connection, name: &str, table: &str) -> Result<String, DbError> {
    conn.query_row(
        "SELECT sql FROM sqlite_schema WHERE type = 'trigger' AND name = ?1 AND tbl_name = ?2",
        [name, table],
        |row| row.get::<_, Option<String>>(0),
    )
    .optional()?
    .flatten()
    .filter(|sql| !sql.trim().is_empty())
    .ok_or_else(|| {
        DbError::Integrity("required statistics reset guard is missing or invalid".into())
    })
}

fn reset_in_transaction(
    conn: &Connection,
    mut restore_guard: impl FnMut(&Connection, &str) -> rusqlite::Result<()>,
) -> Result<(), DbError> {
    if conn.query_row("PRAGMA foreign_keys", [], |row| row.get::<_, i64>(0))? != 1 {
        return Err(DbError::Integrity(
            "statistics reset requires foreign keys".into(),
        ));
    }
    // Read every definition before the first mutation. Missing guards fail closed.
    let guards = DELETE_GUARDS
        .iter()
        .map(|&(name, table)| guard_sql(conn, name, table).map(|sql| (name, table, sql)))
        .collect::<Result<Vec<_>, _>>()?;
    for (name, _, _) in &guards {
        conn.execute_batch(&format!("DROP TRIGGER \"{name}\""))?;
    }
    for table in STATISTICS_TABLES {
        conn.execute(&format!("DELETE FROM {table}"), [])?;
    }
    for (name, table, sql) in guards {
        restore_guard(conn, &sql)?;
        if guard_sql(conn, name, table)? != sql {
            return Err(DbError::Integrity(
                "statistics reset did not restore its guard".into(),
            ));
        }
    }
    // Database::with_transaction commits only after all original guards return.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SqliteFinalizationLedger, SqliteSessionFinalizer, SqliteSessionRecoveryLedger};
    use racoon_application::{
        CompletionIntent, CompletionPolicySnapshot, FinalizationLedger, FinalizationOutcome,
        SessionCompletion, SessionFinalizer, SessionRecoveryLedger, StartedSession,
    };
    use racoon_domain::{CharStatus, FinalStats, SessionId};

    fn seed(database: &Database, suffix: &str) {
        let id = SessionId::parse(format!("018f0c2e-7b8d-7abc-8def-0123456789{suffix}"))
            .expect("session id");
        let now = "2026-10-16T12:00:00Z".parse().expect("time");
        let started = StartedSession::new(
            id.clone(),
            "custom",
            serde_json::json!({"language":"en"}),
            "en",
            now,
        )
        .expect("start");
        let intent = CompletionIntent::from_completion(
            &SessionCompletion {
                session_id: id.clone(),
                completed_at: now,
                final_stats: FinalStats {
                    wpm: 40.0,
                    raw_wpm: 42.0,
                    accuracy: 100.0,
                    raw_accuracy: 100.0,
                    consistency: Some(90.0),
                    correct_chars: 1,
                    incorrect_chars: 0,
                    backspaces: 0,
                    duration_ms: 10_000,
                    char_stats: serde_json::json!({}),
                    heatmap: serde_json::json!({}),
                    graph_data: None,
                },
                mode_type: "custom".into(),
                mode_config: serde_json::json!({"language":"en"}),
                language: "en".into(),
                text_length: 1,
                lesson_id: None,
                replay_frames: vec![racoon_core::ReplayFrame {
                    timestamp_ms: 0,
                    key: "a".into(),
                    caret_pos: 1,
                    char_status: CharStatus::Correct,
                    expected_char: 'a',
                    typed_char: Some('a'),
                }],
            },
            CompletionPolicySnapshot::time(1.0),
        )
        .expect("intent");
        let recovery = SqliteSessionRecoveryLedger::new(database);
        recovery.record_started(&started).expect("started");
        recovery
            .record_completion_intent(&intent)
            .expect("record intent");
        recovery
            .claim_completion_for_finalization(&id, intent.fingerprint())
            .expect("claim intent");
        SqliteFinalizationLedger::new(database)
            .claim_finalization(&id, intent.fingerprint(), now)
            .expect("claim finalization");
        assert_eq!(
            SqliteSessionFinalizer::new(database)
                .finalize_completion(&id, intent.fingerprint())
                .expect("finalize"),
            FinalizationOutcome::NewlyFinalized
        );
    }
    fn fixture() -> Database {
        let database = Database::open_in_memory().expect("database");
        seed(&database, "ca");
        database.with_connection(|conn| {
            conn.execute_batch("INSERT INTO custom_texts (name, text, language, created_at) VALUES ('Keep', 'Текст 字', 'ru', '2026-10-16T12:00:00Z');
                INSERT INTO lesson_progress (lesson_id, module_id) VALUES ('keep-progress', 'module');")?;
            Ok(())
        }).expect("remaining fixtures");
        database
    }
    fn schema(database: &Database) -> Vec<(String, String, String)> {
        database.with_connection(|conn| {
            let mut statement = conn.prepare("SELECT type, name, sql FROM sqlite_schema WHERE sql IS NOT NULL ORDER BY type, name")?;
            let rows = statement.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
            Ok(rows.collect::<Result<Vec<_>, _>>()?)
        }).expect("schema")
    }
    fn counts(database: &Database) -> Vec<i64> {
        database
            .with_connection(|conn| {
                STATISTICS_TABLES
                    .iter()
                    .map(|table| {
                        conn.query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
                            row.get(0)
                        })
                        .map_err(DbError::from)
                    })
                    .collect()
            })
            .expect("counts")
    }

    #[test]
    fn reset_preserves_exact_schema_custom_texts_and_future_finalization() {
        let database = fixture();
        let before = schema(&database);
        assert!(counts(&database).iter().all(|count| *count > 0));
        clear_statistics(&database).expect("administrative reset");
        assert!(counts(&database).iter().all(|count| *count == 0));
        assert_eq!(before, schema(&database));
        database
            .with_connection(|conn| {
                let custom: String = conn.query_row(
                    "SELECT text FROM custom_texts WHERE name = 'Keep'",
                    [],
                    |row| row.get(0),
                )?;
                assert_eq!(custom, "Текст 字");
                assert_eq!(
                    conn.query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))?,
                    "ok"
                );
                assert_eq!(
                    conn.query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |row| {
                        row.get::<_, i64>(0)
                    })?,
                    0
                );
                Ok(())
            })
            .expect("preserved data and integrity");
        seed(&database, "cb");
        database
            .with_connection(|conn| {
                for sql in [
                    "UPDATE session_ledger SET state = 'running' WHERE state = 'finalized'",
                    "UPDATE session_completion_intents SET fingerprint = fingerprint",
                    "UPDATE session_finalizations SET fingerprint = fingerprint",
                ] {
                    assert!(conn.execute(sql, []).is_err(), "immutable guard: {sql}");
                }
                for table in [
                    "session_ledger",
                    "session_completion_intents",
                    "session_finalizations",
                ] {
                    assert!(
                        conn.execute(&format!("DELETE FROM {table}"), []).is_err(),
                        "normal deletion remains guarded: {table}"
                    );
                }
                Ok(())
            })
            .expect("guards remain active");
    }

    #[test]
    fn deletion_failure_rolls_back_data_and_all_guard_definitions() {
        let database = fixture();
        let before = (counts(&database), schema(&database));
        database.with_connection(|conn| {
            conn.execute_batch("CREATE TEMP TRIGGER stop_reset BEFORE DELETE ON tests BEGIN SELECT RAISE(ABORT, 'reset failure'); END;")?;
            Ok(())
        }).expect("temporary failure trigger");
        assert!(clear_statistics(&database).is_err());
        assert_eq!(before, (counts(&database), schema(&database)));
    }

    #[test]
    fn guard_recreation_sql_failure_rolls_back_data_and_schema() {
        let database = fixture();
        let before = (counts(&database), schema(&database));
        let result = database.with_transaction(|conn| {
            reset_in_transaction(conn, |conn, sql| {
                conn.execute_batch(if sql.contains(DELETE_GUARDS[1].0) {
                    "CREATE TRIGGER"
                } else {
                    sql
                })
            })
        });
        assert!(result.is_err());
        assert_eq!(before, (counts(&database), schema(&database)));
    }

    #[test]
    fn changed_guard_definition_rolls_back_data_and_schema() {
        let database = fixture();
        let before = (counts(&database), schema(&database));
        let result = database.with_transaction(|conn| {
            reset_in_transaction(conn, |conn, sql| {
                conn.execute_batch(&sql.replace("cannot be deleted", "changed definition"))
            })
        });
        assert!(matches!(result, Err(DbError::Integrity(_))));
        assert_eq!(before, (counts(&database), schema(&database)));
    }

    #[test]
    fn missing_guard_fails_before_any_deletion() {
        let database = fixture();
        database
            .with_connection(|conn| {
                conn.execute_batch(&format!("DROP TRIGGER {}", DELETE_GUARDS[2].0))?;
                Ok(())
            })
            .expect("missing guard fixture");
        let before = (counts(&database), schema(&database));
        assert!(matches!(
            clear_statistics(&database),
            Err(DbError::Integrity(_))
        ));
        assert_eq!(before, (counts(&database), schema(&database)));
    }

    #[test]
    fn disabled_foreign_keys_fail_closed() {
        let database = fixture();
        database
            .with_connection(|conn| {
                conn.pragma_update(None, "foreign_keys", "OFF")?;
                Ok(())
            })
            .expect("foreign key fixture");
        let before = (counts(&database), schema(&database));
        assert!(matches!(
            clear_statistics(&database),
            Err(DbError::Integrity(_))
        ));
        assert_eq!(before, (counts(&database), schema(&database)));
    }
}
