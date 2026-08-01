//! File-backed, abrupt-process-termination recovery campaign.
//!
//! This target is compiled only with `crash-test-support`. The parent test
//! process never shares a `Database`, connection, repository, coordinator, or
//! finalizer with the child. The child reaches one bounded checkpoint, writes
//! only that checkpoint name to a synchronized marker, and aborts without
//! Rust unwinding. The parent then opens the same SQLite file afresh and runs
//! the accepted startup recovery coordinator.

#![cfg(feature = "crash-test-support")]

use std::env;
use std::fs;
use std::io::Write;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use chrono::{DateTime, SecondsFormat, Utc};
use racoon_application::{
    CompletionIntent, CompletionPolicySnapshot, FinalizationClaimOutcome, FinalizationLedger,
    FinalizationLedgerClaimOutcome, RecoveryReadiness, SessionCompletion, SessionFinalizer,
    SessionRecoveryLedger, SessionWallClock, StartedSession, StartupRecoveryCoordinator,
    StartupRecoveryGate, StartupRecoveryRetryPolicy, StartupRecoveryRunOutcome,
    StartupRecoverySleeper,
};
use racoon_core::ReplayFrame;
use racoon_data::repository::session_finalizer::FinalizerCrashCheckpoint;
use racoon_data::repository::{LessonRepository, SqliteLessonRepository};
use racoon_data::{
    Database, SqliteFinalizationLedger, SqliteSessionFinalizer, SqliteSessionRecoveryLedger,
};
use racoon_domain::{CharStatus, FinalStats, SessionId};
use rusqlite::{params, OptionalExtension};
use serde_json::json;

static NEXT_TEMP_DB: AtomicU64 = AtomicU64::new(0);

const SESSION_ID: &str = "018f0c2e-7b8d-7abc-8def-0123456789ac";
const CHILD_MODE: &str = "RACOON_PROCESS_CRASH_CHILD_MODE";
const CHILD_DATABASE: &str = "RACOON_PROCESS_CRASH_DATABASE";
const CHILD_SCENARIO: &str = "RACOON_PROCESS_CRASH_SCENARIO";
const CHILD_CHECKPOINT: &str = "RACOON_PROCESS_CRASH_CHECKPOINT";
const CHILD_MARKER: &str = "RACOON_PROCESS_CRASH_MARKER";
const CHILD_TIMEOUT: Duration = Duration::from_secs(10);
const DATE: &str = "2026-07-16";
const LESSON_ID: &str = "en_m1_l1";
const FIXTURE_DURATION_MS: u64 = 1_000;
const STANDARD_TIME_GOAL_MINUTES: f64 = 15.0;
const GOAL_MET_TIME_GOAL_MINUTES: f64 = 0.01;

#[derive(Clone, Copy, PartialEq, Eq)]
enum CrashScenario {
    RunningPersisted,
    CompletionIntentPersisted,
    FinalizationPendingPersisted,
    V008PendingClaimed,
    StandardFinalizer,
    GoalMetDailyGoalFinalizer,
    LessonFinalizer,
}

impl CrashScenario {
    const fn storage_name(self) -> &'static str {
        match self {
            Self::RunningPersisted => "running_persisted",
            Self::CompletionIntentPersisted => "completion_intent_persisted",
            Self::FinalizationPendingPersisted => "finalization_pending_persisted",
            Self::V008PendingClaimed => "v008_pending_claimed",
            Self::StandardFinalizer => "standard_finalizer",
            Self::GoalMetDailyGoalFinalizer => "goal_met_daily_goal_finalizer",
            Self::LessonFinalizer => "lesson_finalizer",
        }
    }

    fn from_storage_name(value: &str) -> Option<Self> {
        match value {
            "running_persisted" => Some(Self::RunningPersisted),
            "completion_intent_persisted" => Some(Self::CompletionIntentPersisted),
            "finalization_pending_persisted" => Some(Self::FinalizationPendingPersisted),
            "v008_pending_claimed" => Some(Self::V008PendingClaimed),
            "standard_finalizer" => Some(Self::StandardFinalizer),
            "goal_met_daily_goal_finalizer" => Some(Self::GoalMetDailyGoalFinalizer),
            "lesson_finalizer" => Some(Self::LessonFinalizer),
            _ => None,
        }
    }

    const fn needs_finalizer_checkpoint(self) -> bool {
        matches!(
            self,
            Self::StandardFinalizer | Self::GoalMetDailyGoalFinalizer | Self::LessonFinalizer
        )
    }
}

struct CrashFixture {
    database_path: PathBuf,
    marker_path: PathBuf,
}

impl CrashFixture {
    fn new(name: &str) -> Self {
        let sequence = NEXT_TEMP_DB.fetch_add(1, Ordering::Relaxed);
        let database_path = env::temp_dir().join(format!(
            "racoon-process-crash-{name}-{}-{sequence}.db",
            std::process::id()
        ));
        let marker_path = database_path.with_extension("checkpoint");
        remove_database_files(&database_path);
        let _ = fs::remove_file(&marker_path);

        // The parent creates and migrates the unique SQLite file, then drops
        // all state before the independent child process starts.
        let database = Database::open(&database_path).expect("create parent database");
        drop(database);

        Self {
            database_path,
            marker_path,
        }
    }
}

impl Drop for CrashFixture {
    fn drop(&mut self) {
        remove_database_files(&self.database_path);
        let _ = fs::remove_file(&self.marker_path);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DailyEvidence {
    total_tests: i64,
    total_time_ms: i64,
    total_chars: i64,
    lessons_completed: i64,
    daily_goal_met: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StreakEvidence {
    current_streak: i64,
    longest_streak: i64,
    last_date: Option<String>,
    started_date: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LessonEvidence {
    status: String,
    attempts: i64,
}

/// Bounded evidence only: no canonical payload, typed text, replay content,
/// heatmap, or graph data escapes test diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
struct DurableEffectSnapshot {
    session_state: Option<String>,
    interruption_reason: Option<String>,
    intent_fingerprint: Option<String>,
    finalization_count: i64,
    finalization_state: Option<String>,
    finalization_fingerprint: Option<String>,
    test_count: i64,
    test_mode_type: Option<String>,
    test_immutable_values_match: bool,
    replay_count: i64,
    replay_values_match: bool,
    personal_best_count: i64,
    daily: Option<DailyEvidence>,
    streak: Option<StreakEvidence>,
    lesson: Option<LessonEvidence>,
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

fn timestamp() -> DateTime<Utc> {
    DateTime::parse_from_rfc3339("2026-07-16T12:00:00Z")
        .expect("fixture timestamp")
        .with_timezone(&Utc)
}

fn formatted_timestamp() -> String {
    timestamp().to_rfc3339_opts(SecondsFormat::Micros, true)
}

fn session_id() -> SessionId {
    SessionId::parse(SESSION_ID).expect("fixture UUIDv7")
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

fn completion(id: SessionId, lesson_id: Option<&str>) -> CompletionIntent {
    completion_with_policy(
        id,
        lesson_id,
        CompletionPolicySnapshot::time(STANDARD_TIME_GOAL_MINUTES),
    )
}

fn goal_met_completion(id: SessionId) -> CompletionIntent {
    completion_with_policy(
        id,
        None,
        CompletionPolicySnapshot::time(GOAL_MET_TIME_GOAL_MINUTES),
    )
}

fn completion_with_policy(
    id: SessionId,
    lesson_id: Option<&str>,
    completion_policy: CompletionPolicySnapshot,
) -> CompletionIntent {
    let is_lesson = lesson_id.is_some();
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
                duration_ms: FIXTURE_DURATION_MS,
            },
            mode_type: if is_lesson {
                "lesson".to_string()
            } else {
                "custom".to_string()
            },
            mode_config: match lesson_id {
                Some(lesson_id) => json!({"language": "en", "lesson_id": lesson_id}),
                None => json!({"language": "en"}),
            },
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
            lesson_id: lesson_id.map(str::to_owned),
        },
        completion_policy,
    )
    .expect("fixture completion intent")
}

/// Mirrors the accepted time-goal predicate used by the finalizer. This
/// explicit fixture assertion prevents the crash scenario from silently
/// regressing into the old no-op daily-goal branch.
fn goal_met_fixture_satisfies_time_policy() -> bool {
    (FIXTURE_DURATION_MS as f64 / 60_000.0) >= GOAL_MET_TIME_GOAL_MINUTES
}

fn run_coordinator(database: &Database, gate: &StartupRecoveryGate) -> StartupRecoveryRunOutcome {
    let recovery = SqliteSessionRecoveryLedger::new(database);
    let finalizations = SqliteFinalizationLedger::new(database);
    let finalizer = SqliteSessionFinalizer::new(database);
    let clock = FixedClock;
    let sleeper = NoopSleeper;
    StartupRecoveryCoordinator::new(
        &recovery,
        &finalizations,
        &finalizer,
        &clock,
        &sleeper,
        StartupRecoveryRetryPolicy::new(
            NonZeroUsize::new(2).expect("nonzero retry count"),
            Duration::ZERO,
        ),
    )
    .run(gate)
    .expect("startup recovery gate")
}

fn remove_database_files(path: &Path) {
    let _ = fs::remove_file(path);
    let _ = fs::remove_file(path.with_extension("db-wal"));
    let _ = fs::remove_file(path.with_extension("db-shm"));
}

fn child_or_exit<T, E>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(_) => std::process::exit(70),
    }
}

fn child_expect(condition: bool) {
    if !condition {
        std::process::exit(71);
    }
}

fn child_write_marker_and_abort(marker_path: &Path, value: &str) -> ! {
    let mut marker = child_or_exit(fs::File::create(marker_path));
    child_or_exit(marker.write_all(value.as_bytes()));
    child_or_exit(marker.sync_all());
    std::process::abort();
}

fn prepare_lesson(database: &Database) {
    child_or_exit(database.with_connection(|connection| {
        SqliteLessonRepository::new(connection)
            .create_progress(LESSON_ID, "en_m1", "en", "beginner")
    }));
}

fn prepare_finalization_pending(
    database: &Database,
    completion: &CompletionIntent,
    claim_v008: bool,
) {
    let recovery = SqliteSessionRecoveryLedger::new(database);
    let id = completion.payload().session_id().clone();
    child_expect(matches!(
        child_or_exit(recovery.record_started(&started(id.clone()))),
        racoon_application::LedgerMutationOutcome::Created
    ));
    child_expect(matches!(
        child_or_exit(recovery.record_completion_intent(completion)),
        racoon_application::LedgerMutationOutcome::Created
    ));
    child_expect(matches!(
        child_or_exit(recovery.claim_completion_for_finalization(&id, completion.fingerprint())),
        FinalizationClaimOutcome::Claimed
    ));
    if claim_v008 {
        child_expect(matches!(
            child_or_exit(SqliteFinalizationLedger::new(database).claim_finalization(
                &id,
                completion.fingerprint(),
                timestamp(),
            )),
            FinalizationLedgerClaimOutcome::Claimed
        ));
    }
}

fn run_child_scenario(
    database_path: &Path,
    marker_path: &Path,
    scenario: CrashScenario,
    checkpoint: Option<FinalizerCrashCheckpoint>,
) {
    let database = child_or_exit(Database::open(database_path));
    let id = session_id();
    match scenario {
        CrashScenario::RunningPersisted => {
            let recovery = SqliteSessionRecoveryLedger::new(&database);
            child_expect(matches!(
                child_or_exit(recovery.record_started(&started(id))),
                racoon_application::LedgerMutationOutcome::Created
            ));
            child_write_marker_and_abort(marker_path, scenario.storage_name());
        }
        CrashScenario::CompletionIntentPersisted => {
            let completion = completion(id.clone(), None);
            let recovery = SqliteSessionRecoveryLedger::new(&database);
            child_expect(matches!(
                child_or_exit(recovery.record_started(&started(id))),
                racoon_application::LedgerMutationOutcome::Created
            ));
            child_expect(matches!(
                child_or_exit(recovery.record_completion_intent(&completion)),
                racoon_application::LedgerMutationOutcome::Created
            ));
            child_write_marker_and_abort(marker_path, scenario.storage_name());
        }
        CrashScenario::FinalizationPendingPersisted => {
            let completion = completion(id, None);
            prepare_finalization_pending(&database, &completion, false);
            child_write_marker_and_abort(marker_path, scenario.storage_name());
        }
        CrashScenario::V008PendingClaimed => {
            let completion = completion(id, None);
            prepare_finalization_pending(&database, &completion, true);
            child_write_marker_and_abort(marker_path, scenario.storage_name());
        }
        CrashScenario::StandardFinalizer
        | CrashScenario::GoalMetDailyGoalFinalizer
        | CrashScenario::LessonFinalizer => {
            let is_lesson = scenario == CrashScenario::LessonFinalizer;
            if is_lesson {
                prepare_lesson(&database);
            }
            let completion = if scenario == CrashScenario::GoalMetDailyGoalFinalizer {
                goal_met_completion(id)
            } else {
                completion(id, is_lesson.then_some(LESSON_ID))
            };
            prepare_finalization_pending(&database, &completion, true);
            let checkpoint = checkpoint.unwrap_or_else(|| std::process::exit(72));
            let finalizer = SqliteSessionFinalizer::with_process_crash_checkpoint(
                &database,
                checkpoint,
                marker_path,
            );
            let _ = finalizer
                .finalize_completion(completion.payload().session_id(), completion.fingerprint());
            std::process::exit(73);
        }
    }
}

#[test]
fn process_crash_child_entry() {
    if env::var(CHILD_MODE).ok().as_deref() != Some("1") {
        return;
    }
    let database_path = env::var_os(CHILD_DATABASE)
        .map(PathBuf::from)
        .unwrap_or_else(|| std::process::exit(64));
    let marker_path = env::var_os(CHILD_MARKER)
        .map(PathBuf::from)
        .unwrap_or_else(|| std::process::exit(64));
    let scenario = env::var(CHILD_SCENARIO)
        .ok()
        .and_then(|value| CrashScenario::from_storage_name(&value))
        .unwrap_or_else(|| std::process::exit(64));
    let checkpoint = match env::var(CHILD_CHECKPOINT).ok() {
        Some(value) => FinalizerCrashCheckpoint::from_storage_name(&value),
        None => None,
    };
    if scenario.needs_finalizer_checkpoint() != checkpoint.is_some() {
        std::process::exit(64);
    }
    run_child_scenario(&database_path, &marker_path, scenario, checkpoint);
}

fn wait_for_aborted_child(
    mut child: Child,
    scenario: CrashScenario,
    marker_path: &Path,
    expected_marker: &str,
) {
    let deadline = Instant::now() + CHILD_TIMEOUT;
    let status = loop {
        if let Some(status) = child.try_wait().expect("poll child") {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "process-crash child timed out before bounded checkpoint {} for scenario {}",
                expected_marker,
                scenario.storage_name()
            );
        }
        // This polls process completion only. The synchronized marker is the
        // deterministic proof of the selected boundary, not a timing guess.
        thread::sleep(Duration::from_millis(5));
    };
    assert!(
        !status.success(),
        "process-crash child unexpectedly returned normally for {}",
        scenario.storage_name()
    );
    let marker = fs::read_to_string(marker_path).expect("child checkpoint marker");
    assert_eq!(marker, expected_marker);
}

fn spawn_child(
    fixture: &CrashFixture,
    scenario: CrashScenario,
    checkpoint: Option<FinalizerCrashCheckpoint>,
) {
    let executable = env::current_exe().expect("current integration-test executable");
    let mut command = Command::new(executable);
    command
        .arg("--exact")
        .arg("process_crash_child_entry")
        .arg("--nocapture")
        .arg("--test-threads=1")
        .env(CHILD_MODE, "1")
        .env(CHILD_DATABASE, &fixture.database_path)
        .env(CHILD_SCENARIO, scenario.storage_name())
        .env(CHILD_MARKER, &fixture.marker_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if let Some(checkpoint) = checkpoint {
        command.env(CHILD_CHECKPOINT, checkpoint.storage_name());
    } else {
        command.env_remove(CHILD_CHECKPOINT);
    }
    let child = command.spawn().expect("spawn crash child");

    let expected = checkpoint
        .map(FinalizerCrashCheckpoint::storage_name)
        .unwrap_or_else(|| scenario.storage_name());
    wait_for_aborted_child(child, scenario, &fixture.marker_path, expected);
}

fn snapshot(path: &Path) -> DurableEffectSnapshot {
    let database = Database::open(path).expect("open snapshot database");
    let connection = database.conn();
    let id = session_id();
    let session_state = connection
        .query_row(
            "SELECT state FROM session_ledger WHERE session_id = ?1",
            params![id.as_str()],
            |row| row.get(0),
        )
        .optional()
        .expect("read session state");
    let interruption_reason = connection
        .query_row(
            "SELECT interruption_reason FROM session_ledger WHERE session_id = ?1",
            params![id.as_str()],
            |row| row.get(0),
        )
        .optional()
        .expect("read interruption reason")
        .flatten();
    let intent_fingerprint = connection
        .query_row(
            "SELECT fingerprint FROM session_completion_intents WHERE session_id = ?1",
            params![id.as_str()],
            |row| row.get(0),
        )
        .optional()
        .expect("read intent fingerprint");
    let finalization = connection
        .query_row(
            "SELECT state, fingerprint FROM session_finalizations WHERE session_id = ?1",
            params![id.as_str()],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()
        .expect("read finalization");
    let finalization_count = connection
        .query_row(
            "SELECT COUNT(*) FROM session_finalizations WHERE session_id = ?1",
            params![id.as_str()],
            |row| row.get(0),
        )
        .expect("count finalizations");
    let test_count = connection
        .query_row(
            "SELECT COUNT(*) FROM tests WHERE session_id = ?1",
            params![id.as_str()],
            |row| row.get(0),
        )
        .expect("count tests");
    let test_values = connection
        .query_row(
            "SELECT created_at, mode_type, mode_config, language, text_length, duration_ms,
                    wpm, raw_wpm, accuracy, raw_accuracy, consistency, correct_chars,
                    incorrect_chars, backspaces, char_stats, heatmap_data, graph_data, is_pb, tags
             FROM tests WHERE session_id = ?1",
            params![id.as_str()],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, f64>(6)?,
                    row.get::<_, f64>(7)?,
                    row.get::<_, f64>(8)?,
                    row.get::<_, f64>(9)?,
                    row.get::<_, Option<f64>>(10)?,
                    row.get::<_, i64>(11)?,
                    row.get::<_, i64>(12)?,
                    row.get::<_, i64>(13)?,
                    row.get::<_, String>(14)?,
                    row.get::<_, String>(15)?,
                    row.get::<_, Option<String>>(16)?,
                    row.get::<_, i64>(17)?,
                    row.get::<_, String>(18)?,
                ))
            },
        )
        .optional()
        .expect("read test evidence");
    let test_mode_type = test_values.as_ref().map(|values| values.1.clone());
    let test_immutable_values_match = test_values.is_some_and(
        |(
            created_at,
            mode_type,
            mode_config,
            language,
            text_length,
            duration_ms,
            wpm,
            raw_wpm,
            accuracy,
            raw_accuracy,
            consistency,
            correct_chars,
            incorrect_chars,
            backspaces,
            char_stats,
            heatmap,
            graph,
            is_pb,
            tags,
        )| {
            let mode_matches = match (
                mode_type.as_str(),
                serde_json::from_str::<serde_json::Value>(&mode_config),
            ) {
                ("custom", Ok(value)) => value == json!({"language": "en"}),
                ("lesson", Ok(value)) => value == json!({"language": "en", "lesson_id": LESSON_ID}),
                _ => false,
            };
            created_at == formatted_timestamp()
                && mode_matches
                && language == "en"
                && text_length == 5
                && duration_ms
                    == i64::try_from(FIXTURE_DURATION_MS).expect("fixture duration fits i64")
                && wpm.to_bits() == 60.0_f64.to_bits()
                && raw_wpm.to_bits() == 61.0_f64.to_bits()
                && accuracy.to_bits() == 0.98_f64.to_bits()
                && raw_accuracy.to_bits() == 0.99_f64.to_bits()
                && consistency.is_some_and(|value| value.to_bits() == 0.9_f64.to_bits())
                && correct_chars == 5
                && incorrect_chars == 0
                && backspaces == 0
                && serde_json::from_str::<serde_json::Value>(&char_stats)
                    .is_ok_and(|value| value == json!({"a": {"correct": 1, "incorrect": 0}}))
                && serde_json::from_str::<serde_json::Value>(&heatmap)
                    .is_ok_and(|value| value == json!({"a": {"count": 1}}))
                && graph.is_some_and(|value| {
                    serde_json::from_str::<serde_json::Value>(&value)
                        .is_ok_and(|parsed| parsed == json!([60]))
                })
                && is_pb == 1
                && tags.is_empty()
        },
    );
    let replay_count = connection
        .query_row(
            "SELECT COUNT(*) FROM test_replays
             WHERE test_id = (SELECT id FROM tests WHERE session_id = ?1)",
            params![id.as_str()],
            |row| row.get(0),
        )
        .expect("count replay");
    let replay_values_match = connection
        .query_row(
            "SELECT frame_index, timestamp_ms, position, expected_char, typed_char, correct
             FROM test_replays
             WHERE test_id = (SELECT id FROM tests WHERE session_id = ?1)
             ORDER BY frame_index",
            params![id.as_str()],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, bool>(5)?,
                ))
            },
        )
        .optional()
        .expect("read replay evidence")
        .is_some_and(|frame| {
            replay_count == 1 && frame == (0, 10, 1, "a".to_owned(), Some("a".to_owned()), true)
        });
    let personal_best_count = connection
        .query_row("SELECT COUNT(*) FROM personal_bests", [], |row| row.get(0))
        .expect("count personal bests");
    let daily = connection
        .query_row(
            "SELECT total_tests, total_time_ms, total_chars, lessons_completed, daily_goal_met
             FROM daily_stats WHERE date = ?1",
            params![DATE],
            |row| {
                Ok(DailyEvidence {
                    total_tests: row.get(0)?,
                    total_time_ms: row.get(1)?,
                    total_chars: row.get(2)?,
                    lessons_completed: row.get(3)?,
                    daily_goal_met: row.get::<_, i64>(4)? == 1,
                })
            },
        )
        .optional()
        .expect("read daily evidence");
    let streak = connection
        .query_row(
            "SELECT current_streak, longest_streak, last_date, started_date
             FROM streaks WHERE type = 'daily_test'",
            [],
            |row| {
                Ok(StreakEvidence {
                    current_streak: row.get(0)?,
                    longest_streak: row.get(1)?,
                    last_date: row.get(2)?,
                    started_date: row.get(3)?,
                })
            },
        )
        .optional()
        .expect("read streak evidence");
    let lesson = connection
        .query_row(
            "SELECT status, attempts FROM lesson_progress WHERE lesson_id = ?1",
            params![LESSON_ID],
            |row| {
                Ok(LessonEvidence {
                    status: row.get(0)?,
                    attempts: row.get(1)?,
                })
            },
        )
        .optional()
        .expect("read lesson evidence");

    DurableEffectSnapshot {
        session_state,
        interruption_reason,
        intent_fingerprint,
        finalization_count,
        finalization_state: finalization.as_ref().map(|(state, _)| state.clone()),
        finalization_fingerprint: finalization.map(|(_, fingerprint)| fingerprint),
        test_count,
        test_mode_type,
        test_immutable_values_match,
        replay_count,
        replay_values_match,
        personal_best_count,
        daily,
        streak,
        lesson,
    }
}

fn expected_fingerprint(lesson: bool) -> String {
    completion(session_id(), lesson.then_some(LESSON_ID))
        .fingerprint()
        .as_str()
        .to_owned()
}

fn expected_goal_met_fingerprint() -> String {
    goal_met_completion(session_id())
        .fingerprint()
        .as_str()
        .to_owned()
}

fn assert_no_completion_effects(snapshot: &DurableEffectSnapshot, lesson_baseline: bool) {
    assert_eq!(snapshot.test_count, 0);
    assert_eq!(snapshot.test_mode_type, None);
    assert!(!snapshot.test_immutable_values_match);
    assert_eq!(snapshot.replay_count, 0);
    assert!(!snapshot.replay_values_match);
    assert_eq!(snapshot.personal_best_count, 0);
    assert_eq!(snapshot.daily, None);
    assert_eq!(snapshot.streak, None);
    if lesson_baseline {
        assert_eq!(
            snapshot.lesson,
            Some(LessonEvidence {
                status: "not_started".to_owned(),
                attempts: 0,
            })
        );
    } else {
        assert_eq!(snapshot.lesson, None);
    }
}

fn assert_finalized_effects_once_with_daily_goal(
    snapshot: &DurableEffectSnapshot,
    lesson: bool,
    fingerprint: &str,
    daily_goal_met: bool,
) {
    assert_eq!(snapshot.session_state.as_deref(), Some("finalized"));
    assert_eq!(snapshot.intent_fingerprint.as_deref(), Some(fingerprint));
    assert_eq!(snapshot.finalization_count, 1);
    assert_eq!(snapshot.finalization_state.as_deref(), Some("committed"));
    assert_eq!(
        snapshot.finalization_fingerprint.as_deref(),
        Some(fingerprint)
    );
    assert_eq!(snapshot.test_count, 1);
    assert_eq!(
        snapshot.test_mode_type.as_deref(),
        Some(if lesson { "lesson" } else { "custom" })
    );
    assert!(snapshot.test_immutable_values_match);
    assert_eq!(snapshot.replay_count, 1);
    assert!(snapshot.replay_values_match);
    assert_eq!(snapshot.personal_best_count, 1);
    assert_eq!(
        snapshot.daily,
        Some(DailyEvidence {
            total_tests: 1,
            total_time_ms: i64::try_from(FIXTURE_DURATION_MS).expect("fixture duration fits i64"),
            total_chars: 5,
            lessons_completed: i64::from(lesson),
            daily_goal_met,
        })
    );
    assert_eq!(
        snapshot.streak,
        Some(StreakEvidence {
            current_streak: 1,
            longest_streak: 1,
            last_date: Some(DATE.to_owned()),
            started_date: Some(DATE.to_owned()),
        })
    );
    if lesson {
        assert_eq!(
            snapshot.lesson,
            Some(LessonEvidence {
                status: "completed".to_owned(),
                attempts: 1,
            })
        );
    } else {
        assert_eq!(snapshot.lesson, None);
    }
}

fn assert_finalized_effects_once(snapshot: &DurableEffectSnapshot, lesson: bool) {
    let fingerprint = expected_fingerprint(lesson);
    assert_finalized_effects_once_with_daily_goal(snapshot, lesson, &fingerprint, false);
}

fn assert_goal_met_finalized_effects_once(snapshot: &DurableEffectSnapshot) {
    let fingerprint = expected_goal_met_fingerprint();
    assert_finalized_effects_once_with_daily_goal(snapshot, false, &fingerprint, true);
}

fn recover_and_snapshot(path: &Path) -> (StartupRecoveryRunOutcome, DurableEffectSnapshot) {
    let database = Database::open(path).expect("open recovery database");
    let gate = StartupRecoveryGate::new();
    assert_eq!(
        gate.state().expect("initial readiness"),
        RecoveryReadiness::NotStarted
    );
    let outcome = run_coordinator(&database, &gate);
    assert_eq!(
        gate.state().expect("recovery readiness"),
        RecoveryReadiness::Ready
    );
    let snapshot = snapshot(path);
    drop(database);
    (outcome, snapshot)
}

fn assert_first_recovery_finalizes_then_reopen_is_idempotent(fixture: &CrashFixture, lesson: bool) {
    let (first, first_snapshot) = recover_and_snapshot(&fixture.database_path);
    let StartupRecoveryRunOutcome::Completed(first_report) = first else {
        panic!("valid incomplete crash state must complete startup recovery");
    };
    assert_eq!(first_report.finalized(), 1);
    assert_finalized_effects_once(&first_snapshot, lesson);

    let (second, second_snapshot) = recover_and_snapshot(&fixture.database_path);
    let StartupRecoveryRunOutcome::Completed(second_report) = second else {
        panic!("terminal crash-recovery retry must complete");
    };
    assert_eq!(second_report.finalized(), 0);
    assert_eq!(second_report.skipped_terminal(), 1);
    assert_eq!(second_snapshot, first_snapshot);
}

fn assert_goal_met_daily_goal_crash_recovers() {
    // The finalizer and production completion path use elapsed minutes rather
    // than truncating configured minute targets. 1,000 ms is 1/60 minute,
    // which exceeds this fixture's 0.01-minute goal, so the real
    // `set_daily_goal_met(..., true)` branch must execute before the crash
    // checkpoint.
    assert!(
        goal_met_fixture_satisfies_time_policy(),
        "goal-met crash fixture must exercise the daily_goal_met false-to-true branch"
    );

    let fixture = CrashFixture::new("daily-goal-real-update");
    spawn_child(
        &fixture,
        CrashScenario::GoalMetDailyGoalFinalizer,
        Some(FinalizerCrashCheckpoint::AfterDailyGoalUpdate),
    );

    // Snapshot A is taken after the child aborts and before startup recovery.
    // The V007 fingerprint proves the configured immutable policy is the
    // goal-met fixture; no daily row survives because both its insertion and
    // the real false-to-true daily-goal update were in the uncommitted
    // finalizer transaction.
    let fingerprint = expected_goal_met_fingerprint();
    let immediate = snapshot(&fixture.database_path);
    assert_eq!(
        immediate.session_state.as_deref(),
        Some("finalization_pending")
    );
    assert_eq!(
        immediate.intent_fingerprint.as_deref(),
        Some(fingerprint.as_str())
    );
    assert_eq!(immediate.finalization_count, 1);
    assert_eq!(immediate.finalization_state.as_deref(), Some("pending"));
    assert_eq!(
        immediate.finalization_fingerprint.as_deref(),
        Some(fingerprint.as_str())
    );
    assert_no_completion_effects(&immediate, false);
    assert_eq!(
        immediate.daily, None,
        "pre-commit goal mutation rolled back"
    );

    // Snapshot B is produced only by a fresh Database, gate, coordinator,
    // and accepted finalizer. It must apply the daily-goal write exactly once.
    let (first, first_snapshot) = recover_and_snapshot(&fixture.database_path);
    let StartupRecoveryRunOutcome::Completed(first_report) = first else {
        panic!("goal-met crash state must complete startup recovery");
    };
    assert_eq!(first_report.scanned(), 1);
    assert_eq!(first_report.finalized(), 1);
    assert_goal_met_finalized_effects_once(&first_snapshot);

    // Snapshot C uses another new Database/gate/coordinator. Exact equality
    // proves recovery did not reapply totals or the daily-goal transition.
    let (second, second_snapshot) = recover_and_snapshot(&fixture.database_path);
    let StartupRecoveryRunOutcome::Completed(second_report) = second else {
        panic!("goal-met terminal retry must complete");
    };
    assert_eq!(second_report.finalized(), 0);
    assert_eq!(second_report.skipped_terminal(), 1);
    assert_eq!(second_snapshot, first_snapshot);
}

#[test]
fn process_crash_after_running_persistence_is_interrupted_without_effects() {
    let fixture = CrashFixture::new("running");
    spawn_child(&fixture, CrashScenario::RunningPersisted, None);

    let immediate = snapshot(&fixture.database_path);
    assert_eq!(immediate.session_state.as_deref(), Some("running"));
    assert_eq!(immediate.intent_fingerprint, None);
    assert_eq!(immediate.finalization_count, 0);
    assert_no_completion_effects(&immediate, false);

    let (outcome, recovered) = recover_and_snapshot(&fixture.database_path);
    let StartupRecoveryRunOutcome::Completed(report) = outcome else {
        panic!("running crash recovery must complete")
    };
    assert_eq!(report.interrupted(), 1);
    assert_eq!(recovered.session_state.as_deref(), Some("interrupted"));
    assert_eq!(
        recovered.interruption_reason.as_deref(),
        Some("process_restart")
    );
    assert_eq!(recovered.intent_fingerprint, None);
    assert_eq!(recovered.finalization_count, 0);
    assert_no_completion_effects(&recovered, false);
}

#[test]
fn process_crash_after_v007_persistence_recovers_exactly_once() {
    let fixture = CrashFixture::new("intent");
    spawn_child(&fixture, CrashScenario::CompletionIntentPersisted, None);

    let immediate = snapshot(&fixture.database_path);
    assert_eq!(
        immediate.session_state.as_deref(),
        Some("awaiting_persistence")
    );
    assert_eq!(
        immediate.intent_fingerprint.as_deref(),
        Some(expected_fingerprint(false).as_str())
    );
    assert_eq!(immediate.finalization_count, 0);
    assert_no_completion_effects(&immediate, false);

    assert_first_recovery_finalizes_then_reopen_is_idempotent(&fixture, false);
}

#[test]
fn process_crash_after_finalization_pending_recovers_exactly_once() {
    let fixture = CrashFixture::new("finalization-pending");
    spawn_child(&fixture, CrashScenario::FinalizationPendingPersisted, None);

    let immediate = snapshot(&fixture.database_path);
    assert_eq!(
        immediate.session_state.as_deref(),
        Some("finalization_pending")
    );
    assert_eq!(immediate.finalization_count, 0);
    assert_no_completion_effects(&immediate, false);

    assert_first_recovery_finalizes_then_reopen_is_idempotent(&fixture, false);
}

#[test]
fn process_crash_after_v008_pending_claim_recovers_exactly_once() {
    let fixture = CrashFixture::new("v008-pending");
    spawn_child(&fixture, CrashScenario::V008PendingClaimed, None);

    let immediate = snapshot(&fixture.database_path);
    assert_eq!(
        immediate.session_state.as_deref(),
        Some("finalization_pending")
    );
    assert_eq!(immediate.finalization_count, 1);
    assert_eq!(immediate.finalization_state.as_deref(), Some("pending"));
    assert_eq!(
        immediate.finalization_fingerprint.as_deref(),
        Some(expected_fingerprint(false).as_str())
    );
    assert_no_completion_effects(&immediate, false);

    assert_first_recovery_finalizes_then_reopen_is_idempotent(&fixture, false);
}

fn assert_standard_precommit_crash_recovers(checkpoint: FinalizerCrashCheckpoint) {
    let fixture = CrashFixture::new(checkpoint.storage_name());
    spawn_child(&fixture, CrashScenario::StandardFinalizer, Some(checkpoint));

    // Snapshot A proves that the abrupt process death rolled back the one
    // uncommitted IMMEDIATE finalizer transaction rather than merely being
    // repaired by recovery later.
    let immediate = snapshot(&fixture.database_path);
    assert_eq!(
        immediate.session_state.as_deref(),
        Some("finalization_pending")
    );
    assert_eq!(immediate.finalization_count, 1);
    assert_eq!(immediate.finalization_state.as_deref(), Some("pending"));
    assert_no_completion_effects(&immediate, false);

    // Snapshot B proves the accepted coordinator/finalizer converges from
    // that durable pending state exactly once.
    assert_first_recovery_finalizes_then_reopen_is_idempotent(&fixture, false);
}

macro_rules! standard_precommit_crash_test {
    ($name:ident, $checkpoint:expr) => {
        #[test]
        fn $name() {
            assert_standard_precommit_crash_recovers($checkpoint);
        }
    };
}

standard_precommit_crash_test!(
    process_crash_before_test_insertion_rolls_back,
    FinalizerCrashCheckpoint::BeforeTestInsertion
);
standard_precommit_crash_test!(
    process_crash_after_test_insertion_rolls_back,
    FinalizerCrashCheckpoint::AfterTestInsertion
);
standard_precommit_crash_test!(
    process_crash_after_replay_insertion_rolls_back,
    FinalizerCrashCheckpoint::AfterReplayInsertion
);
standard_precommit_crash_test!(
    process_crash_after_personal_best_update_rolls_back,
    FinalizerCrashCheckpoint::AfterPersonalBestUpdate
);
standard_precommit_crash_test!(
    process_crash_after_daily_statistics_update_rolls_back,
    FinalizerCrashCheckpoint::AfterDailyStatisticsUpdate
);
standard_precommit_crash_test!(
    process_crash_after_streak_update_rolls_back,
    FinalizerCrashCheckpoint::AfterStreakUpdate
);
#[test]
fn process_crash_after_real_daily_goal_update_rolls_back_and_recovers_once() {
    assert_goal_met_daily_goal_crash_recovers();
}
standard_precommit_crash_test!(
    process_crash_after_v008_commit_update_rolls_back,
    FinalizerCrashCheckpoint::AfterV008CommittedUpdate
);
standard_precommit_crash_test!(
    process_crash_after_v006_finalized_update_rolls_back,
    FinalizerCrashCheckpoint::AfterV006FinalizedUpdate
);

#[test]
fn process_crash_after_lesson_update_rolls_back_and_recovers_once() {
    let fixture = CrashFixture::new("lesson-update");
    spawn_child(
        &fixture,
        CrashScenario::LessonFinalizer,
        Some(FinalizerCrashCheckpoint::AfterLessonUpdate),
    );

    let immediate = snapshot(&fixture.database_path);
    assert_eq!(
        immediate.session_state.as_deref(),
        Some("finalization_pending")
    );
    assert_eq!(immediate.finalization_state.as_deref(), Some("pending"));
    assert_no_completion_effects(&immediate, true);

    assert_first_recovery_finalizes_then_reopen_is_idempotent(&fixture, true);
}

#[test]
fn process_crash_after_finalizer_commit_converges_without_duplicate_effects() {
    let fixture = CrashFixture::new("post-commit");
    spawn_child(
        &fixture,
        CrashScenario::StandardFinalizer,
        Some(FinalizerCrashCheckpoint::AfterFinalizerTransactionCommit),
    );

    // SQLite had already reported COMMIT success to the child. This snapshot
    // therefore distinguishes the ambiguous caller outcome from every
    // pre-commit checkpoint above.
    let immediate = snapshot(&fixture.database_path);
    assert_finalized_effects_once(&immediate, false);

    let (outcome, recovered) = recover_and_snapshot(&fixture.database_path);
    let StartupRecoveryRunOutcome::Completed(report) = outcome else {
        panic!("post-commit restart must complete")
    };
    assert_eq!(report.finalized(), 0);
    assert_eq!(report.skipped_terminal(), 1);
    assert_eq!(recovered, immediate);
}

#[test]
#[ignore = "extended process-crash stress campaign; run with --ignored"]
fn extended_process_crash_campaign_repeats_critical_boundaries() {
    // 8 ordinary standard boundaries x 10, the dedicated real daily-goal
    // false-to-true boundary x 10, and post-commit ambiguity x 25 = 115
    // independent child-process crashes.
    const STANDARD_PRE_COMMIT: [FinalizerCrashCheckpoint; 8] = [
        FinalizerCrashCheckpoint::BeforeTestInsertion,
        FinalizerCrashCheckpoint::AfterTestInsertion,
        FinalizerCrashCheckpoint::AfterReplayInsertion,
        FinalizerCrashCheckpoint::AfterPersonalBestUpdate,
        FinalizerCrashCheckpoint::AfterDailyStatisticsUpdate,
        FinalizerCrashCheckpoint::AfterStreakUpdate,
        FinalizerCrashCheckpoint::AfterV008CommittedUpdate,
        FinalizerCrashCheckpoint::AfterV006FinalizedUpdate,
    ];
    for checkpoint in STANDARD_PRE_COMMIT {
        for _ in 0..10 {
            assert_standard_precommit_crash_recovers(checkpoint);
        }
    }
    for _ in 0..10 {
        assert_goal_met_daily_goal_crash_recovers();
    }
    for _ in 0..25 {
        let fixture = CrashFixture::new("extended-post-commit");
        spawn_child(
            &fixture,
            CrashScenario::StandardFinalizer,
            Some(FinalizerCrashCheckpoint::AfterFinalizerTransactionCommit),
        );
        let immediate = snapshot(&fixture.database_path);
        assert_finalized_effects_once(&immediate, false);
        let (_, recovered) = recover_and_snapshot(&fixture.database_path);
        assert_eq!(recovered, immediate);
    }
}
