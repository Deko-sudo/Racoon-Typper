// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

//! Tauri adapters for versioned portable profile transfer.

use chrono::Utc;
use racoon_core::CoreEngine;
use racoon_data::profile_transfer::{
    apply_profile_import, export_profile as export_profile_document, plan_profile_import,
    ImportPlan, ProfileImportPolicy,
};
use racoon_domain::SessionState;
use std::sync::Mutex;
use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

/// Builds a complete, versioned profile document from a consistent database snapshot.
#[tauri::command]
pub(crate) fn export_profile(state: State<'_, AppState>) -> Result<String, AppError> {
    export_profile_with_state(&state)
}

/// Validates an import and reports its effects without changing local profile data.
#[tauri::command]
pub(crate) fn preview_profile_import(
    state: State<'_, AppState>,
    document: String,
    policy: ProfileImportPolicy,
) -> Result<ImportPlan, AppError> {
    preview_profile_import_with_state(&state, &document, policy)
}

/// Applies a validated profile import. `replace` only replaces portable profile
/// tables; it never swaps the live SQLite file or recovery ledgers.
#[tauri::command]
pub(crate) fn import_profile(
    app_state: State<'_, AppState>,
    engine_state: State<'_, Mutex<CoreEngine>>,
    document: String,
    policy: ProfileImportPolicy,
) -> Result<ImportPlan, AppError> {
    import_profile_with_state(&app_state, &engine_state, &document, policy)
}

fn export_profile_with_state(state: &AppState) -> Result<String, AppError> {
    let profile = export_profile_document(
        &state.db,
        env!("CARGO_PKG_VERSION"),
        &Utc::now().to_rfc3339(),
    )?;
    serde_json::to_string(&profile).map_err(AppError::from)
}

fn preview_profile_import_with_state(
    state: &AppState,
    document: &str,
    policy: ProfileImportPolicy,
) -> Result<ImportPlan, AppError> {
    plan_profile_import(&state.db, document.as_bytes(), policy).map_err(AppError::from)
}

fn import_profile_with_state(
    state: &AppState,
    engine_state: &Mutex<CoreEngine>,
    document: &str,
    policy: ProfileImportPolicy,
) -> Result<ImportPlan, AppError> {
    let _readiness = state.require_startup_recovery_ready()?;
    let engine = engine_state.lock()?;
    ensure_profile_transfer_can_run(&engine)?;
    apply_profile_import(&state.db, document.as_bytes(), policy).map_err(AppError::from)
}

fn ensure_profile_transfer_can_run(engine: &CoreEngine) -> Result<(), AppError> {
    match engine.session_state() {
        SessionState::Idle | SessionState::Persisted => Ok(()),
        SessionState::Running => Err(AppError::TestAlreadyActive),
        SessionState::AwaitingPersistence | SessionState::Persisting => {
            Err(AppError::SessionFinalizing)
        }
    }
}

/// Restores the whole SQLite database file from a validated backup.
///
/// The live connection is closed, the backup replaces the live file (validated
/// at a sibling temporary path first), and the database reopens with migrations
/// re-applied. The in-memory engine is reset afterwards so no stale session
/// survives the swap. The frontend reloads the window after success.
#[tauri::command]
pub(crate) fn restore_database(
    app_state: State<'_, AppState>,
    engine_state: State<'_, Mutex<CoreEngine>>,
    backup_path: String,
) -> Result<(), AppError> {
    restore_database_with_state(&app_state, &engine_state, &backup_path)
}

fn restore_database_with_state(
    state: &AppState,
    engine_state: &Mutex<CoreEngine>,
    backup_path: &str,
) -> Result<(), AppError> {
    let mut lifecycle = state.begin_database_restore()?;
    let mut engine = engine_state.lock()?;
    ensure_profile_transfer_can_run(&engine)?;

    let backup = std::path::PathBuf::from(backup_path);
    if backup_path.is_empty() || backup_path.len() > 4096 {
        return Err(AppError::InvalidConfig(
            "backup path must be between 1 and 4096 characters".to_string(),
        ));
    }
    if !backup.is_file() {
        return Err(AppError::InvalidConfig(
            "backup path must reference a regular file".to_string(),
        ));
    }
    if backup == *state.db_path() {
        return Err(AppError::InvalidConfig(
            "backup path must differ from the live database path".to_string(),
        ));
    }

    *lifecycle = Some(racoon_application::StartupRecoveryGate::new());
    state.db.close()?;
    let restore_result = racoon_data::backup::restore_from_path(&backup, state.db_path());
    if matches!(restore_result, Err(racoon_data::DbError::RestoreBlocked)) {
        return Err(AppError::RestoreBlocked);
    }
    state.db.reopen(state.db_path())?;
    let gate = lifecycle.as_ref().ok_or(AppError::StateUnavailable)?;
    state.recover_restored_database(gate)?;
    restore_result?;

    // The restored file may contain a different session history; the in-memory
    // engine must not keep a stale session identity.
    let _ = engine.abort();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use racoon_application::{
        SessionWallClock, StartupRecoveryCoordinator, StartupRecoveryGate,
        StartupRecoveryRetryPolicy, StartupRecoverySleeper,
    };
    use racoon_core::{CoreEngine, TimeMode};
    use racoon_data::profile_transfer::{ProfileImportPolicy, PROFILE_FORMAT};
    use racoon_data::repository::{
        CustomTextRepository, SqliteCustomTextRepository, SqliteFinalizationLedger,
        SqliteSessionFinalizer, SqliteSessionRecoveryLedger,
    };
    use racoon_data::Database;
    use std::num::NonZeroUsize;
    use std::path::PathBuf;
    use std::sync::Mutex;
    use std::time::Duration;

    use crate::error::AppError;
    use crate::state::AppState;

    struct FixedClock;

    impl SessionWallClock for FixedClock {
        fn utc_now(&self) -> DateTime<Utc> {
            DateTime::parse_from_rfc3339("2026-08-06T12:00:00Z")
                .expect("timestamp")
                .with_timezone(&Utc)
        }
    }

    struct NoopSleeper;

    impl StartupRecoverySleeper for NoopSleeper {
        fn sleep(&self, _: Duration) {}
    }

    fn ready_state() -> AppState {
        let database = Database::open_in_memory().expect("database");
        let gate = StartupRecoveryGate::new();
        let recovery = SqliteSessionRecoveryLedger::new(&database);
        let finalizations = SqliteFinalizationLedger::new(&database);
        let finalizer = SqliteSessionFinalizer::new(&database);
        StartupRecoveryCoordinator::new(
            &recovery,
            &finalizations,
            &finalizer,
            &FixedClock,
            &NoopSleeper,
            StartupRecoveryRetryPolicy::new(NonZeroUsize::new(1).expect("nonzero"), Duration::ZERO),
        )
        .run(&gate)
        .expect("startup recovery");
        AppState::new(
            database,
            PathBuf::from("settings.toml"),
            PathBuf::from("unused-db.db"),
            gate,
        )
    }

    #[test]
    fn profile_commands_export_preview_and_apply_a_merge() {
        let source = ready_state();
        source
            .db
            .with_transaction(|tx| {
                SqliteCustomTextRepository::new(tx).save_with_language("Exported", "body", "en")?;
                Ok(())
            })
            .expect("seed source profile");
        let document = export_profile_with_state(&source).expect("export profile");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&document).expect("profile JSON")["format"],
            PROFILE_FORMAT
        );

        let target = ready_state();
        let preview =
            preview_profile_import_with_state(&target, &document, ProfileImportPolicy::Merge)
                .expect("preview profile import");
        assert_eq!(preview.custom_texts.to_insert, 1);
        assert_eq!(
            target
                .db
                .with_connection(|conn| SqliteCustomTextRepository::new(conn).get_all(10))
                .expect("custom texts before import")
                .len(),
            0,
            "preview must not mutate the profile"
        );

        let imported = import_profile_with_state(
            &target,
            &Mutex::new(CoreEngine::new()),
            &document,
            ProfileImportPolicy::Merge,
        )
        .expect("import profile");
        assert_eq!(imported.custom_texts.to_insert, 1);
        assert_eq!(
            target
                .db
                .with_connection(|conn| SqliteCustomTextRepository::new(conn).get_all(10))
                .expect("custom texts after import")
                .len(),
            1
        );
    }

    #[test]
    fn profile_import_rejects_an_active_typing_session() {
        let state = ready_state();
        let mut engine = CoreEngine::new();
        engine
            .start_test_mode(
                "active-session",
                Box::new(TimeMode::new("hello".to_string(), "en".to_string(), 30)),
            )
            .expect("start test");

        let document = export_profile_with_state(&state).expect("export profile");
        assert!(matches!(
            import_profile_with_state(
                &state,
                &Mutex::new(engine),
                &document,
                ProfileImportPolicy::Replace,
            ),
            Err(AppError::TestAlreadyActive)
        ));
    }
}

#[cfg(test)]
mod restore_tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use racoon_application::{
        SessionWallClock, StartupRecoveryCoordinator, StartupRecoveryGate,
        StartupRecoveryRetryPolicy, StartupRecoverySleeper,
    };
    use racoon_data::repository::{
        CustomTextRepository, SqliteCustomTextRepository, SqliteFinalizationLedger,
        SqliteSessionFinalizer, SqliteSessionRecoveryLedger,
    };
    use racoon_data::Database;
    use std::num::NonZeroUsize;
    use std::path::PathBuf;
    use std::sync::Mutex;
    use std::time::Duration;

    struct FixedClock;

    impl SessionWallClock for FixedClock {
        fn utc_now(&self) -> DateTime<Utc> {
            DateTime::parse_from_rfc3339("2026-08-06T12:00:00Z")
                .expect("timestamp")
                .with_timezone(&Utc)
        }
    }

    struct NoopSleeper;

    impl StartupRecoverySleeper for NoopSleeper {
        fn sleep(&self, _: Duration) {}
    }

    fn temp_path(label: &str) -> PathBuf {
        let dir = std::env::temp_dir();
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        dir.join(format!(
            "racoon-restore-{label}-{}-{ts}.db",
            std::process::id()
        ))
    }

    fn ready_file_state(db_path: &std::path::Path) -> AppState {
        let database = Database::open(db_path).expect("database");
        let gate = StartupRecoveryGate::new();
        let recovery = SqliteSessionRecoveryLedger::new(&database);
        let finalizations = SqliteFinalizationLedger::new(&database);
        let finalizer = SqliteSessionFinalizer::new(&database);
        StartupRecoveryCoordinator::new(
            &recovery,
            &finalizations,
            &finalizer,
            &FixedClock,
            &NoopSleeper,
            StartupRecoveryRetryPolicy::new(NonZeroUsize::new(1).expect("nonzero"), Duration::ZERO),
        )
        .run(&gate)
        .expect("startup recovery");
        AppState::new(
            database,
            PathBuf::from("settings.toml"),
            db_path.to_path_buf(),
            gate,
        )
    }

    fn custom_text_count(state: &AppState) -> i64 {
        state
            .db
            .with_connection(|conn| {
                conn.query_row("SELECT COUNT(*) FROM custom_texts", [], |row| row.get(0))
                    .map_err(|error| racoon_data::DbError::Query(error.to_string()))
            })
            .expect("count custom texts")
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[ignore = "requires the restore_unlink_failure LD_PRELOAD fixture"]
    fn restore_companion_failure_requires_manual_recovery() {
        let root = std::env::temp_dir().join(format!("r03-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let live = root.join("r03-live.db");
        let source = root.join("source.db");
        let state = ready_file_state(&live);
        racoon_data::backup::backup_to_path(&state.db, &source).unwrap();
        state.db.close().unwrap();
        rusqlite::Connection::open(&live).unwrap().execute("INSERT INTO custom_texts(name,text,created_at) VALUES('retained-history','body','2026-01-01T00:00:00Z')", []).unwrap();
        let original = std::fs::read(&live).unwrap();
        let wal = live.with_file_name("r03-live.db-wal");
        std::fs::write(&wal, b"retained WAL").unwrap();
        std::fs::write(live.with_file_name("r03-live.db-shm"), b"retained SHM").unwrap();
        std::env::set_var("RACOON_RESTORE_RENAME_FAIL", "rollback");
        std::env::set_var("RACOON_RESTORE_UNLINK_FAIL", "1");
        let engine = Mutex::new(CoreEngine::new());
        let result = restore_database_with_state(&state, &engine, source.to_str().unwrap());
        std::env::remove_var("RACOON_RESTORE_UNLINK_FAIL");
        std::env::remove_var("RACOON_RESTORE_RENAME_FAIL");
        let error = result.unwrap_err();
        assert!(state.db.with_connection(|_| Ok(())).is_err());
        let payload = serde_json::to_value(&error).unwrap();
        let retained = std::fs::read_dir(&root).unwrap().flatten().any(|entry| {
            std::fs::read(entry.path().join("original.db-wal"))
                .is_ok_and(|bytes| bytes == b"retained WAL")
        });
        let remaining = !wal.exists() && std::fs::read(&live).unwrap() == original;
        std::fs::remove_dir_all(&root).unwrap();
        assert!(retained && remaining);
        assert_eq!(payload["code"], "RESTORE_BLOCKED");
        assert!(error.to_string().contains("manual recovery"));
        assert!(!payload.to_string().contains(root.to_str().unwrap()));
    }

    #[test]
    fn restore_replaces_live_database_and_reopens() {
        let live_path = temp_path("live");
        let backup_path = temp_path("backup");
        let _ = std::fs::remove_file(&live_path);
        let _ = std::fs::remove_file(&backup_path);

        // Live database with one custom text.
        let state = ready_file_state(&live_path);
        state
            .db
            .with_transaction(|tx| {
                SqliteCustomTextRepository::new(tx).save_with_language("Live", "body", "en")?;
                Ok(())
            })
            .expect("seed live");
        assert_eq!(custom_text_count(&state), 1);

        // Backup database with two custom texts.
        let backup_db = Database::open(&backup_path).expect("backup database");
        backup_db
            .with_transaction(|tx| {
                SqliteCustomTextRepository::new(tx).save_with_language("One", "a", "en")?;
                SqliteCustomTextRepository::new(tx).save_with_language("Two", "b", "en")?;
                Ok(())
            })
            .expect("seed backup");
        drop(backup_db);

        let engine = Mutex::new(CoreEngine::new());
        restore_database_with_state(&state, &engine, backup_path.to_str().expect("utf8 path"))
            .expect("restore");

        assert_eq!(custom_text_count(&state), 2);

        let _ = std::fs::remove_file(&live_path);
        let _ = std::fs::remove_file(&backup_path);
    }

    #[test]
    fn restore_finalizes_backup_pending_sessions_before_returning_success() {
        use racoon_application::{
            CompletionIntent, CompletionPolicySnapshot, SessionCompletion, SessionRecoveryLedger,
            StartedSession,
        };
        use racoon_domain::{FinalStats, SessionId};
        use serde_json::json;

        let root = temp_path("pending").with_extension("dir");
        std::fs::create_dir(&root).unwrap();
        let live = root.join("live.db");
        let backup = root.join("backup.db");
        let state = ready_file_state(&live);
        let source = Database::open_in_memory().unwrap();
        let id = SessionId::parse("018f0c2e-7b8d-7abc-8def-0123456789aa").unwrap();
        let timestamp = FixedClock.utc_now();
        let recovery = SqliteSessionRecoveryLedger::new(&source);
        recovery
            .record_started(
                &StartedSession::new(
                    id.clone(),
                    "custom",
                    json!({"language": "en"}),
                    "en",
                    timestamp,
                )
                .unwrap(),
            )
            .unwrap();
        let intent = CompletionIntent::from_completion(
            &SessionCompletion {
                session_id: id.clone(),
                completed_at: timestamp,
                final_stats: FinalStats {
                    wpm: 60.0,
                    raw_wpm: 60.0,
                    accuracy: 1.0,
                    raw_accuracy: 1.0,
                    consistency: None,
                    correct_chars: 5,
                    incorrect_chars: 0,
                    backspaces: 0,
                    char_stats: json!({}),
                    heatmap: json!({}),
                    graph_data: None,
                    duration_ms: 1000,
                },
                mode_type: "custom".into(),
                mode_config: json!({"language": "en"}),
                language: "en".into(),
                text_length: 5,
                replay_frames: vec![],
                lesson_id: None,
            },
            CompletionPolicySnapshot::time(15.0),
        )
        .unwrap();
        recovery.record_completion_intent(&intent).unwrap();
        recovery
            .claim_completion_for_finalization(&id, intent.fingerprint())
            .unwrap();
        let running = SessionId::parse("018f0c2e-7b8d-7abc-8def-0123456789ab").unwrap();
        recovery
            .record_started(
                &StartedSession::new(
                    running.clone(),
                    "custom",
                    json!({"language": "en"}),
                    "en",
                    timestamp,
                )
                .unwrap(),
            )
            .unwrap();
        racoon_data::backup::backup_to_path(&source, &backup).unwrap();
        for _ in 0..2 {
            restore_database_with_state(
                &state,
                &Mutex::new(CoreEngine::new()),
                backup.to_str().unwrap(),
            )
            .unwrap();
            let restored: (String, i64) = state
                .db
                .with_connection(|conn| {
                    Ok((
                        conn.query_row(
                            "SELECT state FROM session_ledger WHERE session_id=?1",
                            [id.as_str()],
                            |row| row.get(0),
                        )?,
                        conn.query_row(
                            "SELECT COUNT(*) FROM tests WHERE session_id=?1",
                            [id.as_str()],
                            |row| row.get(0),
                        )?,
                    ))
                })
                .unwrap();
            assert_eq!(
                restored,
                ("finalized".into(), 1),
                "restore returned success before processing pending history"
            );
            let interrupted: String = state
                .db
                .with_connection(|conn| {
                    Ok(conn.query_row(
                        "SELECT state FROM session_ledger WHERE session_id=?1",
                        [running.as_str()],
                        |row| row.get(0),
                    )?)
                })
                .unwrap();
            assert_eq!(interrupted, "interrupted");
            assert!(state.require_startup_recovery_ready().is_ok());
        }
        drop(state);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejected_backup_recovers_the_original_database_and_keeps_mutations_available() {
        let root = temp_path("rejected-recovery").with_extension("dir");
        std::fs::create_dir(&root).unwrap();
        let live = root.join("live.db");
        let backup = root.join("invalid.db");
        let state = ready_file_state(&live);
        state
            .db
            .with_transaction(|conn| SqliteCustomTextRepository::new(conn).save("Retained", "body"))
            .unwrap();
        std::fs::write(&backup, b"not a SQLite database").unwrap();
        let result = restore_database_with_state(
            &state,
            &Mutex::new(CoreEngine::new()),
            backup.to_str().unwrap(),
        );
        assert!(result.is_err());
        assert!(state.require_startup_recovery_ready().is_ok());
        assert_eq!(custom_text_count(&state), 1);
        drop(state);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn restore_serializes_session_changes_through_replacement() {
        use std::sync::{Arc, TryLockError};
        use std::time::Instant;

        let root = temp_path("serialization").with_extension("dir");
        std::fs::create_dir(&root).expect("fixture directory");
        let live = root.join("live.db");
        let backup = root.join("backup.db");
        let state = Arc::new(ready_file_state(&live));
        racoon_data::backup::backup_to_path(&state.db, &backup).expect("valid backup");
        let source = rusqlite::Connection::open(&backup).expect("backup connection");
        source
            .execute_batch("PRAGMA journal_mode=DELETE; BEGIN EXCLUSIVE;")
            .expect("hold backup reader at a known filesystem boundary");
        let engine = Arc::new(Mutex::new(CoreEngine::new()));
        let state_for_restore = Arc::clone(&state);
        let engine_for_restore = Arc::clone(&engine);
        let handle = std::thread::spawn(move || {
            restore_database_with_state(
                &state_for_restore,
                &engine_for_restore,
                backup.to_str().expect("fixture path"),
            )
        });

        let deadline = Instant::now() + Duration::from_secs(2);
        let closed = loop {
            if state.db.with_connection(|_| Ok(())).is_err() {
                break true;
            }
            if Instant::now() >= deadline {
                break false;
            }
            std::thread::yield_now();
        };
        let session_changes_blocked = matches!(engine.try_lock(), Err(TryLockError::WouldBlock));
        let (started_tx, started_rx) = std::sync::mpsc::channel();
        let (ready_tx, ready_rx) = std::sync::mpsc::channel();
        let state_for_mutation = Arc::clone(&state);
        let mutation = std::thread::spawn(move || {
            started_tx.send(()).unwrap();
            let readiness = state_for_mutation.require_startup_recovery_ready();
            ready_tx.send(readiness.is_ok()).unwrap();
        });
        started_rx.recv().unwrap();
        let mutation_readiness_blocked = !ready_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        source
            .execute_batch("ROLLBACK;")
            .expect("release backup reader");
        handle
            .join()
            .expect("restore thread")
            .expect("restore completes");
        assert!(state.db.with_connection(|_| Ok(())).is_ok());
        mutation.join().unwrap();
        let mutation_ready = state.require_startup_recovery_ready().is_ok();
        drop(source);
        drop(state);
        std::fs::remove_dir_all(root).expect("fixture cleanup");
        assert!(closed, "restore must reach the closed-connection boundary");
        assert!(
            mutation_readiness_blocked,
            "mutation readiness escaped during replacement"
        );
        assert!(
            mutation_ready,
            "mutations must resume only after fresh recovery completes"
        );
        assert!(
            session_changes_blocked,
            "a session or second restore must not enter during replacement"
        );
    }

    #[test]
    fn restore_rejects_missing_backup_without_touching_live_data() {
        let live_path = temp_path("live-missing");
        let _ = std::fs::remove_file(&live_path);
        let state = ready_file_state(&live_path);
        state
            .db
            .with_transaction(|tx| {
                SqliteCustomTextRepository::new(tx).save_with_language("Live", "body", "en")?;
                Ok(())
            })
            .expect("seed live");

        let engine = Mutex::new(CoreEngine::new());
        let error = restore_database_with_state(&state, &engine, "/nonexistent/racoon-backup.db")
            .unwrap_err();
        assert!(matches!(error, AppError::InvalidConfig(_)));
        assert_eq!(custom_text_count(&state), 1);

        let _ = std::fs::remove_file(&live_path);
    }

    #[test]
    fn restore_rejects_live_path_as_backup() {
        let live_path = temp_path("live-self");
        let _ = std::fs::remove_file(&live_path);
        let state = ready_file_state(&live_path);
        let engine = Mutex::new(CoreEngine::new());
        let error =
            restore_database_with_state(&state, &engine, live_path.to_str().expect("utf8 path"))
                .unwrap_err();
        assert!(matches!(error, AppError::InvalidConfig(_)));

        let _ = std::fs::remove_file(&live_path);
    }

    #[test]
    fn restore_rejects_empty_and_oversized_paths() {
        let live_path = temp_path("live-bounds");
        let _ = std::fs::remove_file(&live_path);
        let state = ready_file_state(&live_path);
        let engine = Mutex::new(CoreEngine::new());

        assert!(matches!(
            restore_database_with_state(&state, &engine, ""),
            Err(AppError::InvalidConfig(_))
        ));
        assert!(matches!(
            restore_database_with_state(&state, &engine, &"x".repeat(4097)),
            Err(AppError::InvalidConfig(_))
        ));

        let _ = std::fs::remove_file(&live_path);
    }
}
