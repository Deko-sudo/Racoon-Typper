//! File-backed integration coverage for application-owned startup recovery.

use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use chrono::{DateTime, Utc};
use racoon_application::{
    CompletionIntent, CompletionPolicySnapshot, DurableSessionState, FinalizationClaimOutcome,
    FinalizationLedger, FinalizationLedgerClaimOutcome, FinalizationLedgerState,
    FinalizationOutcome, InterruptionReason, QuarantineReason, RecoveryReadiness,
    SessionCompletion, SessionFinalizer, SessionRecoveryLedger, SessionWallClock, StartedSession,
    StartupRecoveryCandidateAction, StartupRecoveryCoordinator, StartupRecoveryGate,
    StartupRecoveryRetryPolicy, StartupRecoveryRunOutcome, StartupRecoverySleeper,
};
use racoon_core::ReplayFrame;
use racoon_data::{
    Database, SqliteFinalizationLedger, SqliteSessionFinalizer, SqliteSessionRecoveryLedger,
};
use racoon_domain::{CharStatus, FinalStats, SessionId};
use rusqlite::{params, Connection};
use serde_json::json;

static NEXT_TEMP_DB: AtomicU64 = AtomicU64::new(0);
const SESSION_A: &str = "018f0c2e-7b8d-7abc-8def-0123456789aa";
const SESSION_B: &str = "018f0c2e-7b8d-7abc-8def-0123456789ab";
const SESSION_C: &str = "018f0c2e-7b8d-7abc-8def-0123456789ac";
const SESSION_D: &str = "018f0c2e-7b8d-7abc-8def-0123456789ad";
const SESSION_E: &str = "018f0c2e-7b8d-7abc-8def-0123456789ae";
const SESSION_F: &str = "018f0c2e-7b8d-7abc-8def-0123456789af";

fn temporary_database_path(name: &str) -> PathBuf {
    let sequence = NEXT_TEMP_DB.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "racoon-startup-recovery-{name}-{}-{sequence}.db",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(path.with_extension("db-wal"));
    let _ = std::fs::remove_file(path.with_extension("db-shm"));
    path
}

fn remove_database(path: &std::path::Path) {
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(path.with_extension("db-wal"));
    let _ = std::fs::remove_file(path.with_extension("db-shm"));
}

fn timestamp() -> DateTime<Utc> {
    DateTime::parse_from_rfc3339("2026-07-16T12:00:00Z")
        .expect("fixture timestamp")
        .with_timezone(&Utc)
}

fn session_id(value: &str) -> SessionId {
    SessionId::parse(value).expect("fixture UUIDv7")
}

fn started(id: SessionId) -> StartedSession {
    StartedSession::new(
        id,
        "custom",
        json!({"language": "en", "kind": "sanitized"}),
        "en",
        timestamp(),
    )
    .expect("fixture start")
}

fn intent(id: SessionId) -> CompletionIntent {
    CompletionIntent::from_completion(
        &SessionCompletion {
            session_id: id,
            completed_at: timestamp(),
            final_stats: FinalStats {
                wpm: 60.0,
                raw_wpm: 61.0,
                accuracy: 0.98,
                raw_accuracy: 0.99,
                consistency: Some(0.9),
                correct_chars: 5,
                incorrect_chars: 0,
                backspaces: 0,
                char_stats: json!({"a": {"correct": 1, "incorrect": 0}}),
                heatmap: json!({"a": {"count": 1}}),
                graph_data: Some(json!([60.0])),
                duration_ms: 1_000,
            },
            mode_type: "custom".to_string(),
            mode_config: json!({"language": "en"}),
            language: "en".to_string(),
            text_length: 5,
            replay_frames: vec![ReplayFrame {
                timestamp_ms: 10,
                key: "a".to_string(),
                caret_pos: 1,
                char_status: CharStatus::Correct,
                expected_char: 'a',
                typed_char: Some('a'),
            }],
            lesson_id: None,
        },
        CompletionPolicySnapshot::time(15.0),
    )
    .expect("fixture completion intent")
}

fn prepare_pending(database: &Database, id: SessionId) -> CompletionIntent {
    let recovery = SqliteSessionRecoveryLedger::new(database);
    let completion = intent(id.clone());
    assert!(recovery.record_started(&started(id.clone())).is_ok());
    assert!(recovery.record_completion_intent(&completion).is_ok());
    assert_eq!(
        recovery.claim_completion_for_finalization(&id, completion.fingerprint()),
        Ok(FinalizationClaimOutcome::Claimed)
    );
    assert_eq!(
        SqliteFinalizationLedger::new(database).claim_finalization(
            &id,
            completion.fingerprint(),
            timestamp(),
        ),
        Ok(FinalizationLedgerClaimOutcome::Claimed)
    );
    completion
}

struct FixedClock;

impl SessionWallClock for FixedClock {
    fn utc_now(&self) -> DateTime<Utc> {
        timestamp()
    }
}

struct NoopSleeper;

impl StartupRecoverySleeper for NoopSleeper {
    fn sleep(&self, _: Duration) {}
}

struct ReleasingSleeper {
    lock_connection: Mutex<Option<Connection>>,
}

impl StartupRecoverySleeper for ReleasingSleeper {
    fn sleep(&self, _: Duration) {
        // Dropping the separate write-reserving connection releases the real
        // SQLite lock before the bounded retry.
        let _ = self.lock_connection.lock().expect("fixture lock").take();
    }
}

fn run_coordinator<S: StartupRecoverySleeper>(
    database: &Database,
    sleeper: &S,
    gate: &StartupRecoveryGate,
) -> StartupRecoveryRunOutcome {
    // The adapters own no independent connection or mutex; all use the same
    // Database and application ports below.
    let recovery = SqliteSessionRecoveryLedger::new(database);
    let finalizations = SqliteFinalizationLedger::new(database);
    let finalizer = SqliteSessionFinalizer::new(database);
    let clock = FixedClock;
    StartupRecoveryCoordinator::new(
        &recovery,
        &finalizations,
        &finalizer,
        &clock,
        sleeper,
        StartupRecoveryRetryPolicy::new(
            NonZeroUsize::new(2).expect("nonzero retry count"),
            Duration::ZERO,
        ),
    )
    .run(gate)
    .expect("startup recovery gate")
}

#[test]
fn empty_startup_recovery_is_ready_and_metadata_scan_has_no_effect_rows() {
    let path = temporary_database_path("empty");
    let database = Database::open(&path).expect("open database");
    let sleeper = NoopSleeper;
    let gate = StartupRecoveryGate::new();

    let outcome = run_coordinator(&database, &sleeper, &gate);
    let StartupRecoveryRunOutcome::Completed(report) = outcome else {
        panic!("empty recovery must complete")
    };
    assert_eq!(report.scanned(), 0);
    assert_eq!(gate.state().expect("state"), RecoveryReadiness::Ready);
    let conn = database.conn();
    let effects: i64 = conn
        .query_row("SELECT COUNT(*) FROM tests", [], |row| row.get(0))
        .expect("count effects");
    assert_eq!(effects, 0);
    drop(conn);
    drop(database);
    remove_database(&path);
}

#[test]
fn startup_recovery_finalizes_a_pending_session_exactly_once_and_reopen_converges() {
    let path = temporary_database_path("finalize");
    let database = Database::open(&path).expect("open database");
    let id = session_id(SESSION_A);
    let completion = prepare_pending(&database, id.clone());
    let sleeper = NoopSleeper;
    let gate = StartupRecoveryGate::new();

    let outcome = run_coordinator(&database, &sleeper, &gate);
    let StartupRecoveryRunOutcome::Completed(report) = outcome else {
        panic!("pending recovery must complete")
    };
    assert_eq!(report.finalized(), 1);
    assert_eq!(gate.state().expect("state"), RecoveryReadiness::Ready);
    assert_eq!(
        SqliteFinalizationLedger::new(&database).load_finalization(&id),
        Ok(racoon_application::FinalizationLoadOutcome::Found(
            racoon_application::FinalizationRecord::new(
                id.clone(),
                completion.fingerprint().clone(),
                FinalizationLedgerState::Committed,
                timestamp(),
                Some(timestamp()),
                None,
            )
            .expect("valid record"),
        ))
    );
    let conn = database.conn();
    let tests: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM tests WHERE session_id = ?1",
            params![id.as_str()],
            |row| row.get(0),
        )
        .expect("count tests");
    let replay: i64 = conn
        .query_row("SELECT COUNT(*) FROM test_replays", [], |row| row.get(0))
        .expect("count replay");
    assert_eq!((tests, replay), (1, 1));
    drop(conn);
    drop(database);

    let reopened = Database::open(&path).expect("reopen database");
    let gate = StartupRecoveryGate::new();
    let outcome = run_coordinator(&reopened, &sleeper, &gate);
    let StartupRecoveryRunOutcome::Completed(report) = outcome else {
        panic!("terminal scan must complete")
    };
    assert_eq!(report.skipped_terminal(), 1);
    let conn = reopened.conn();
    let tests: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM tests WHERE session_id = ?1",
            params![id.as_str()],
            |row| row.get(0),
        )
        .expect("count tests");
    assert_eq!(tests, 1);
    drop(conn);
    drop(reopened);
    remove_database(&path);
}

#[test]
fn startup_recovery_interrupts_running_and_quarantines_missing_intent_without_aborting_scan() {
    let path = temporary_database_path("mixed");
    let database = Database::open(&path).expect("open database");
    let running = session_id(SESSION_A);
    let missing_intent = session_id(SESSION_B);
    let recovery = SqliteSessionRecoveryLedger::new(&database);
    recovery
        .record_started(&started(running.clone()))
        .expect("running start");
    recovery
        .record_started(&started(missing_intent.clone()))
        .expect("missing intent start");
    database
        .with_connection(|connection| {
            connection
                .execute(
                    "UPDATE session_ledger SET state = 'awaiting_persistence' WHERE session_id = ?1",
                    params![missing_intent.as_str()],
                )
                .map_err(|error| racoon_data::DbError::Write(error.to_string()))?;
            Ok(())
        })
        .expect("construct missing-intent corruption fixture");
    let sleeper = NoopSleeper;
    let gate = StartupRecoveryGate::new();

    let outcome = run_coordinator(&database, &sleeper, &gate);
    let StartupRecoveryRunOutcome::Completed(report) = outcome else {
        panic!("row-local corruption must not block unrelated recovery")
    };
    assert_eq!(report.interrupted(), 1);
    assert_eq!(report.quarantined(), 1);
    assert_eq!(gate.state().expect("state"), RecoveryReadiness::Ready);
    let candidates = recovery.list_recovery_candidates().expect("candidates");
    assert_eq!(
        candidates[0].state(),
        racoon_application::DurableSessionState::Interrupted
    );
    assert_eq!(
        candidates[1].state(),
        racoon_application::DurableSessionState::Quarantined
    );
    drop(database);
    remove_database(&path);
}

#[test]
fn startup_recovery_retries_a_real_sqlite_write_lock_then_reaches_ready() {
    let path = temporary_database_path("lock");
    let database = Database::open(&path).expect("open database");
    database
        .conn()
        .busy_timeout(Duration::from_millis(10))
        .expect("short test timeout");
    let lock = Connection::open(&path).expect("second connection");
    lock.execute_batch("BEGIN IMMEDIATE")
        .expect("reserve writer lock");
    let sleeper = ReleasingSleeper {
        lock_connection: Mutex::new(Some(lock)),
    };
    let gate = StartupRecoveryGate::new();

    let outcome = run_coordinator(&database, &sleeper, &gate);
    assert!(matches!(outcome, StartupRecoveryRunOutcome::Completed(_)));
    assert_eq!(gate.state().expect("state"), RecoveryReadiness::Ready);
    drop(database);
    remove_database(&path);
}

#[test]
fn startup_recovery_recovers_valid_awaiting_persistence() {
    let path = temporary_database_path("awaiting-persistence");
    let database = Database::open(&path).expect("open database");
    let id = session_id(SESSION_A);
    let completion = intent(id.clone());
    let recovery = SqliteSessionRecoveryLedger::new(&database);

    recovery
        .record_started(&started(id.clone()))
        .expect("record running session");
    recovery
        .record_completion_intent(&completion)
        .expect("record immutable intent");

    let gate = StartupRecoveryGate::new();
    let sleeper = NoopSleeper;
    let outcome = run_coordinator(&database, &sleeper, &gate);
    let StartupRecoveryRunOutcome::Completed(report) = outcome else {
        panic!("valid awaiting persistence must recover")
    };
    assert_eq!(report.scanned(), 1);
    assert_eq!(report.finalized(), 1);
    assert_eq!(gate.state().expect("readiness"), RecoveryReadiness::Ready);

    let conn = database.conn();
    let state: String = conn
        .query_row(
            "SELECT state FROM session_ledger WHERE session_id = ?1",
            params![id.as_str()],
            |row| row.get(0),
        )
        .expect("ledger state");
    let finalization_state: String = conn
        .query_row(
            "SELECT state FROM session_finalizations WHERE session_id = ?1",
            params![id.as_str()],
            |row| row.get(0),
        )
        .expect("finalization state");
    let tests: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM tests WHERE session_id = ?1",
            params![id.as_str()],
            |row| row.get(0),
        )
        .expect("test count");
    let replay: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM test_replays WHERE test_id = (SELECT id FROM tests WHERE session_id = ?1)",
            params![id.as_str()],
            |row| row.get(0),
        )
        .expect("replay count");
    let daily_tests: i64 = conn
        .query_row(
            "SELECT total_tests FROM daily_stats WHERE date = '2026-07-16'",
            [],
            |row| row.get(0),
        )
        .expect("daily test count");
    assert_eq!(state, "finalized");
    assert_eq!(finalization_state, "committed");
    assert_eq!((tests, replay, daily_tests), (1, 1, 1));
    drop(conn);
    drop(database);

    let reopened = Database::open(&path).expect("reopen database");
    let gate = StartupRecoveryGate::new();
    let outcome = run_coordinator(&reopened, &sleeper, &gate);
    let StartupRecoveryRunOutcome::Completed(report) = outcome else {
        panic!("terminal retry must complete")
    };
    assert_eq!(report.skipped_terminal(), 1);
    let conn = reopened.conn();
    let tests: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM tests WHERE session_id = ?1",
            params![id.as_str()],
            |row| row.get(0),
        )
        .expect("test count after retry");
    assert_eq!(tests, 1);
    drop(conn);
    drop(reopened);
    remove_database(&path);
}

#[test]
fn startup_recovery_processes_a_file_backed_mixed_candidate_set_in_deterministic_order() {
    let path = temporary_database_path("mixed-state-matrix");
    let database = Database::open(&path).expect("open database");
    let recovery = SqliteSessionRecoveryLedger::new(&database);

    let running = session_id(SESSION_A);
    let awaiting = session_id(SESSION_B);
    let pending = session_id(SESSION_C);
    let finalized = session_id(SESSION_D);
    let interrupted = session_id(SESSION_E);
    let quarantined = session_id(SESSION_F);

    recovery
        .record_started(&started(running.clone()))
        .expect("record running fixture");

    let awaiting_intent = intent(awaiting.clone());
    recovery
        .record_started(&started(awaiting.clone()))
        .expect("record awaiting fixture");
    recovery
        .record_completion_intent(&awaiting_intent)
        .expect("record immutable awaiting intent");

    let _pending_intent = prepare_pending(&database, pending.clone());

    let finalized_intent = prepare_pending(&database, finalized.clone());
    assert_eq!(
        SqliteSessionFinalizer::new(&database)
            .finalize_completion(&finalized, finalized_intent.fingerprint()),
        Ok(FinalizationOutcome::NewlyFinalized)
    );

    recovery
        .record_started(&started(interrupted.clone()))
        .expect("record interrupted fixture");
    recovery
        .mark_interrupted(&interrupted, InterruptionReason::ProcessRestart)
        .expect("interrupt terminal fixture");

    recovery
        .record_started(&started(quarantined.clone()))
        .expect("record quarantined fixture");
    recovery
        .quarantine(&quarantined, QuarantineReason::InvalidStateRecord)
        .expect("quarantine terminal fixture");

    let gate = StartupRecoveryGate::new();
    let sleeper = NoopSleeper;
    let outcome = run_coordinator(&database, &sleeper, &gate);
    let StartupRecoveryRunOutcome::Completed(report) = outcome else {
        panic!("row-local terminal and corrupt candidates must not block safe recovery")
    };

    assert_eq!(gate.state().expect("readiness"), RecoveryReadiness::Ready);
    assert_eq!(report.scanned(), 6);
    assert_eq!(report.finalized(), 2);
    assert_eq!(report.interrupted(), 2);
    assert_eq!(report.quarantined(), 1);
    assert_eq!(report.skipped_terminal(), 1);
    assert_eq!(report.already_finalized(), 0);
    assert_eq!(report.conflicts(), 0);
    assert_eq!(report.permanent_failures(), 0);
    assert_eq!(report.retryable_failures(), 0);
    assert!(!report.candidate_results_truncated());

    let observed = report
        .candidate_results()
        .iter()
        .map(|entry| {
            (
                entry.session_id().as_str().to_owned(),
                entry.original_state(),
                entry.action(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        observed,
        vec![
            (
                SESSION_A.to_owned(),
                DurableSessionState::Running,
                StartupRecoveryCandidateAction::Interrupted,
            ),
            (
                SESSION_B.to_owned(),
                DurableSessionState::AwaitingPersistence,
                StartupRecoveryCandidateAction::Finalized,
            ),
            (
                SESSION_C.to_owned(),
                DurableSessionState::FinalizationPending,
                StartupRecoveryCandidateAction::Finalized,
            ),
            (
                SESSION_D.to_owned(),
                DurableSessionState::Finalized,
                StartupRecoveryCandidateAction::SkippedTerminal(DurableSessionState::Finalized),
            ),
            (
                SESSION_E.to_owned(),
                DurableSessionState::Interrupted,
                StartupRecoveryCandidateAction::Interrupted,
            ),
            (
                SESSION_F.to_owned(),
                DurableSessionState::Quarantined,
                StartupRecoveryCandidateAction::Quarantined(QuarantineReason::InvalidStateRecord,),
            ),
        ]
    );

    let candidates = recovery
        .list_recovery_candidates()
        .expect("final candidates");
    assert_eq!(
        candidates
            .iter()
            .map(|candidate| candidate.state())
            .collect::<Vec<_>>(),
        vec![
            DurableSessionState::Interrupted,
            DurableSessionState::Finalized,
            DurableSessionState::Finalized,
            DurableSessionState::Finalized,
            DurableSessionState::Interrupted,
            DurableSessionState::Quarantined,
        ]
    );

    let connection = database.conn();
    let effects: (i64, i64, i64) = connection
        .query_row(
            "SELECT
                (SELECT COUNT(*) FROM tests),
                (SELECT COUNT(*) FROM test_replays),
                (SELECT total_tests FROM daily_stats WHERE date = '2026-07-16')",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("exactly-once mixed effects");
    assert_eq!(effects, (3, 3, 3));
    drop(connection);
    drop(database);
    remove_database(&path);
}

#[test]
fn two_independent_coordinators_converge_on_one_file() {
    use std::sync::Barrier;
    use std::thread;

    let path = temporary_database_path("two-coordinators");
    let setup = Database::open(&path).expect("open setup database");
    let id = session_id(SESSION_A);
    let completion = prepare_pending(&setup, id.clone());
    drop(setup);

    let barrier = std::sync::Arc::new(Barrier::new(2));
    let path_a = path.clone();
    let barrier_a = barrier.clone();
    let first = thread::spawn(move || {
        let database = Database::open(&path_a).expect("open database A");
        barrier_a.wait();
        let gate = StartupRecoveryGate::new();
        let sleeper = NoopSleeper;
        let outcome = run_coordinator(&database, &sleeper, &gate);
        (outcome, gate.state().expect("readiness A"))
    });
    let path_b = path.clone();
    let barrier_b = barrier;
    let second = thread::spawn(move || {
        let database = Database::open(&path_b).expect("open database B");
        barrier_b.wait();
        let gate = StartupRecoveryGate::new();
        let sleeper = NoopSleeper;
        let outcome = run_coordinator(&database, &sleeper, &gate);
        (outcome, gate.state().expect("readiness B"))
    });

    let (first_outcome, first_state) = first.join().expect("coordinator A");
    let (second_outcome, second_state) = second.join().expect("coordinator B");
    assert_eq!(first_state, RecoveryReadiness::Ready);
    assert_eq!(second_state, RecoveryReadiness::Ready);
    let reports = [first_outcome, second_outcome]
        .into_iter()
        .map(|outcome| match outcome {
            StartupRecoveryRunOutcome::Completed(report) => report,
            other => panic!("coordinator did not complete: {other:?}"),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        reports
            .iter()
            .filter(|report| report.finalized() == 1)
            .count(),
        1
    );
    let converged_terminal = reports
        .iter()
        .map(|report| report.already_finalized() + report.skipped_terminal())
        .sum::<usize>();
    assert_eq!(converged_terminal, 1, "reports: {reports:?}");

    let database = Database::open(&path).expect("reopen final database");
    let conn = database.conn();
    let tests: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM tests WHERE session_id = ?1",
            params![id.as_str()],
            |row| row.get(0),
        )
        .expect("test count");
    let replay: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM test_replays WHERE test_id = (SELECT id FROM tests WHERE session_id = ?1)",
            params![id.as_str()],
            |row| row.get(0),
        )
        .expect("replay count");
    let daily_tests: i64 = conn
        .query_row(
            "SELECT total_tests FROM daily_stats WHERE date = '2026-07-16'",
            [],
            |row| row.get(0),
        )
        .expect("daily test count");
    assert_eq!((tests, replay, daily_tests), (1, 1, 1));
    assert_eq!(completion.payload().session_id(), &id);
    drop(conn);
    drop(database);
    remove_database(&path);
}
