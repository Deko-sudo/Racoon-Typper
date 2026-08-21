//! Tauri-side adapters for the session vertical slice.
//!
//! The infrastructure-free `racoon-application` crate owns lifecycle and input
//! orchestration. This module supplies resource validation, mode construction,
//! SQLite completion persistence, and the existing custom-text/lesson setup
//! adapters without exposing those concerns to the kernel.

use racoon_application::{
    CompletionIntent, CompletionPolicySnapshot, FinalizationClaimOutcome, FinalizationLedger,
    FinalizationLedgerClaimOutcome, FinalizationOutcome, LedgerMutationOutcome, SessionCompletion,
    SessionCompletionStore, SessionFinalizer, SessionIdSource, SessionKernel, SessionModeFactory,
    SessionPersistenceReceipt, SessionRandomSource, SessionRecoveryLedger, SessionWallClock,
    StartedSession,
};
use racoon_core::{
    CoreEngine, CustomMode, LessonMode, QuoteMode, TestMode, TestSessionInfo, TimeMode, WordsMode,
};
use racoon_data::repository::{
    AppSettings, CustomTextRepository, LessonRepository, SqliteCustomTextRepository,
    SqliteFinalizationLedger, SqliteLessonRepository, SqliteSessionFinalizer,
    SqliteSessionRecoveryLedger, SqliteTestRepository, TestRepository,
};
use racoon_domain::{EngineOutput, SessionId};
use racoon_resources::{course_loader, quote_loader, word_pack_loader, SystemRandomSource};
use std::sync::Mutex;

use crate::error::AppError;
use crate::state::AppState;
use crate::validation::{
    validate_duration, validate_language, validate_positive_id, validate_resource_identifier,
    validate_test_mode, validate_test_text, validate_word_count,
};

pub(crate) use racoon_application::SessionStartRequest as StartTestRequest;

struct UuidV7SessionIdSource;

impl SessionIdSource for UuidV7SessionIdSource {
    fn next_session_id(&mut self) -> SessionId {
        SessionId::new()
    }
}

struct BackendSessionModeFactory;

impl SessionModeFactory for BackendSessionModeFactory {
    type Error = AppError;

    fn build_mode(
        &self,
        request: &StartTestRequest,
        language: &str,
        random_source: &mut dyn SessionRandomSource,
    ) -> Result<Box<dyn TestMode>, Self::Error> {
        build_test_mode(request.clone(), language.to_string(), random_source)
    }
}

/// Durable completion store: routes live completion through the accepted
/// V006–V008 recovery protocol instead of writing effects directly.
///
/// The sequence is: record the immutable completion intent (V006 running →
/// awaiting_persistence, V007 payload), claim it for finalization (V006 →
/// finalization_pending, V008 pending), then finalize (effects + V008
/// committed + V006 finalized) in one transaction. Every step is idempotent
/// for retries: a repeated attempt converges on AlreadyExistsIdentical /
/// AlreadyPending / AlreadyFinalized, so the engine's retry-pending path
/// keeps working unchanged.
struct DurableSessionCompletionStore<'a> {
    app_state: &'a AppState,
}

impl SessionCompletionStore for DurableSessionCompletionStore<'_> {
    type Error = AppError;

    fn persist_completion(
        &self,
        completion: &SessionCompletion,
    ) -> Result<SessionPersistenceReceipt, Self::Error> {
        let settings = self.app_state.with_settings(|store| store.load())?;
        let policy = completion_policy_snapshot(&settings);
        let intent = CompletionIntent::from_completion(completion, policy)
            .map_err(|error| AppError::Internal(format!("completion intent: {error}")))?;

        let recovery = SqliteSessionRecoveryLedger::new(&self.app_state.db);
        let finalizations = SqliteFinalizationLedger::new(&self.app_state.db);
        let finalizer = SqliteSessionFinalizer::new(&self.app_state.db);

        match recovery.record_completion_intent(&intent) {
            Ok(LedgerMutationOutcome::Created | LedgerMutationOutcome::AlreadyExistsIdentical) => {}
            Ok(LedgerMutationOutcome::NotFound) => {
                return Err(AppError::Internal(
                    "completion intent has no durable session record".to_string(),
                ));
            }
            Ok(LedgerMutationOutcome::Conflicting(_)) => {
                return Err(AppError::Internal(
                    "completion intent conflicts with the stored intent".to_string(),
                ));
            }
            Ok(LedgerMutationOutcome::Quarantined(reason)) => {
                return Err(AppError::Internal(format!(
                    "completion intent quarantined: {reason:?}"
                )));
            }
            Err(_) => {
                return Err(AppError::DbWrite(
                    "completion intent recording failed".to_string(),
                ));
            }
        }

        match recovery
            .claim_completion_for_finalization(&completion.session_id, intent.fingerprint())
        {
            Ok(FinalizationClaimOutcome::Claimed | FinalizationClaimOutcome::AlreadyPending) => {}
            Ok(FinalizationClaimOutcome::AlreadyFinalized) => {
                return Ok(SessionPersistenceReceipt {
                    test_id: self.test_id_for(&completion.session_id)?,
                });
            }
            Ok(FinalizationClaimOutcome::NotFound) => {
                return Err(AppError::Internal(
                    "finalization claim has no durable session record".to_string(),
                ));
            }
            Ok(FinalizationClaimOutcome::Conflict(_)) => {
                return Err(AppError::Internal(
                    "finalization claim conflicts with the stored intent".to_string(),
                ));
            }
            Ok(FinalizationClaimOutcome::Quarantined(reason)) => {
                return Err(AppError::Internal(format!(
                    "finalization claim quarantined: {reason:?}"
                )));
            }
            Ok(FinalizationClaimOutcome::RejectedTerminal { state }) => {
                return Err(AppError::Internal(format!(
                    "finalization claim rejected from terminal state {state:?}"
                )));
            }
            Err(_) => {
                return Err(AppError::DbWrite("finalization claim failed".to_string()));
            }
        }

        match finalizations.claim_finalization(
            &completion.session_id,
            intent.fingerprint(),
            self.app_state.utc_now(),
        ) {
            Ok(
                FinalizationLedgerClaimOutcome::Claimed
                | FinalizationLedgerClaimOutcome::AlreadyPending
                | FinalizationLedgerClaimOutcome::AlreadyCommitted,
            ) => {}
            Ok(
                FinalizationLedgerClaimOutcome::NotFound
                | FinalizationLedgerClaimOutcome::MissingCompletionIntent,
            ) => {
                return Err(AppError::Internal(
                    "finalization ledger claim has no durable session record".to_string(),
                ));
            }
            Ok(FinalizationLedgerClaimOutcome::Conflict(_)) => {
                return Err(AppError::Internal(
                    "finalization ledger claim conflicts with the stored intent".to_string(),
                ));
            }
            Ok(FinalizationLedgerClaimOutcome::Quarantined(reason)) => {
                return Err(AppError::Internal(format!(
                    "finalization ledger claim quarantined: {reason:?}"
                )));
            }
            Ok(FinalizationLedgerClaimOutcome::Corrupt) => {
                return Err(AppError::Internal(
                    "finalization ledger metadata is corrupt".to_string(),
                ));
            }
            Err(_) => {
                return Err(AppError::DbWrite(
                    "finalization ledger claim failed".to_string(),
                ));
            }
        }

        match finalizer.finalize_completion(&completion.session_id, intent.fingerprint()) {
            Ok(FinalizationOutcome::NewlyFinalized | FinalizationOutcome::AlreadyFinalized) => {
                Ok(SessionPersistenceReceipt {
                    test_id: self.test_id_for(&completion.session_id)?,
                })
            }
            Ok(FinalizationOutcome::NotFound) => Err(AppError::Internal(
                "finalization has no durable session record".to_string(),
            )),
            Ok(FinalizationOutcome::Conflict(_)) => Err(AppError::Internal(
                "finalization conflicts with the stored intent".to_string(),
            )),
            Ok(FinalizationOutcome::Quarantined(reason)) => Err(AppError::Internal(format!(
                "finalization quarantined: {reason:?}"
            ))),
            Err(_) => Err(AppError::DbWrite("finalization failed".to_string())),
        }
    }
}

impl DurableSessionCompletionStore<'_> {
    fn test_id_for(&self, session_id: &SessionId) -> Result<i64, AppError> {
        self.app_state
            .db
            .with_connection(|conn| {
                SqliteTestRepository::new(conn).get_id_by_session_id(session_id)
            })
            .map_err(AppError::from)
    }
}

/// Captures the completion-affecting daily-goal setting as an immutable
/// policy snapshot, mirroring the legacy live-completion semantics.
fn completion_policy_snapshot(settings: &AppSettings) -> CompletionPolicySnapshot {
    match settings.daily_goal_type.as_str() {
        "wpm" => CompletionPolicySnapshot::wpm(settings.daily_goal_wpm),
        "accuracy" => CompletionPolicySnapshot::accuracy(settings.daily_goal_accuracy),
        _ => CompletionPolicySnapshot::time(settings.daily_goal_minutes as f64),
    }
}

/// Records the durable session start (V006 running) after the engine accepted
/// the session. A failure here fails the start: a session without a ledger row
/// would fail at completion with NotFound.
fn record_session_started(app_state: &AppState, info: &TestSessionInfo) -> Result<(), AppError> {
    let session_id = SessionId::parse(&info.session_id).map_err(|_| {
        AppError::Internal("backend issued an invalid session identity".to_string())
    })?;
    let started = StartedSession::new(
        session_id,
        info.mode_type.clone(),
        info.mode_config.clone(),
        info.language.clone(),
        app_state.utc_now(),
    )
    .map_err(|error| AppError::Internal(format!("session descriptor: {error}")))?;
    let recovery = SqliteSessionRecoveryLedger::new(&app_state.db);
    match recovery.record_started(&started) {
        Ok(LedgerMutationOutcome::Created | LedgerMutationOutcome::AlreadyExistsIdentical) => {
            Ok(())
        }
        Ok(LedgerMutationOutcome::Conflicting(_)) => Err(AppError::Internal(
            "session start conflicts with the durable ledger".to_string(),
        )),
        Ok(LedgerMutationOutcome::NotFound) => Err(AppError::Internal(
            "session start has no durable ledger row".to_string(),
        )),
        Ok(LedgerMutationOutcome::Quarantined(reason)) => Err(AppError::Internal(format!(
            "session start quarantined: {reason:?}"
        ))),
        Err(_) => Err(AppError::DbWrite(
            "session start recording failed".to_string(),
        )),
    }
}

/// Marks the durable session aborted after an explicit abort. Tolerates
/// terminal states: the startup coordinator may already have marked the
/// session interrupted, and interrupted → aborted is a forbidden transition.
fn record_session_aborted(app_state: &AppState, session_id: &SessionId) {
    let recovery = SqliteSessionRecoveryLedger::new(&app_state.db);
    match recovery.mark_aborted(session_id) {
        Ok(
            LedgerMutationOutcome::Created
            | LedgerMutationOutcome::AlreadyExistsIdentical
            | LedgerMutationOutcome::Quarantined(_),
        ) => {}
        Ok(LedgerMutationOutcome::Conflicting(_) | LedgerMutationOutcome::NotFound) => {}
        Err(_) => {}
    }
}

/// Marks the durable session aborted after an implicit abandon (startup
/// cleanup or view switch). Same tolerance as `record_session_aborted`.
pub(crate) fn record_session_abandoned(app_state: &AppState, session_id: &SessionId) {
    record_session_aborted(app_state, session_id);
}

/// Starts a standard mode after validating and selecting backend-owned content.
pub(crate) fn start_test(
    engine: &mut CoreEngine,
    app_state: &AppState,
    mut request: StartTestRequest,
) -> Result<TestSessionInfo, AppError> {
    let language = validate_language(request.language.take().unwrap_or_else(|| "en".to_string()))?;
    let mut id_source = UuidV7SessionIdSource;
    let mut random_source = SystemRandomSource;
    let info = SessionKernel::new()
        .start_session(
            engine,
            &request,
            &language,
            &mut id_source,
            &mut random_source,
            &BackendSessionModeFactory,
        )
        .map_err(AppError::from)?;
    record_session_started(app_state, &info)?;
    Ok(info)
}

/// Starts a stored custom text only after the engine lifecycle gate has been
/// accepted. The use-count update is part of that accepted start transaction.
pub(crate) fn start_custom_text_test(
    engine: &mut CoreEngine,
    app_state: &AppState,
    custom_text_id: i64,
) -> Result<TestSessionInfo, AppError> {
    validate_positive_id(custom_text_id, "custom text")?;
    ensure_session_can_start(engine)?;

    // Keep the lifecycle gate while the use counter is updated. A rejected
    // start must not mutate the custom-text record, and no concurrent start can
    // replace this session between the check and `start_test_mode`.
    let custom_text = app_state.db.with_transaction(|conn| {
        let repo = SqliteCustomTextRepository::new(conn);
        let custom_text = repo.get_by_id(custom_text_id)?;
        repo.increment_use(custom_text_id)?;
        Ok(custom_text)
    })?;

    let mut id_source = UuidV7SessionIdSource;
    let info = SessionKernel::new()
        .start_mode(
            engine,
            Box::new(CustomMode::new(custom_text.text, custom_text.language)),
            &mut id_source,
        )
        .map_err(AppError::from)?;
    record_session_started(app_state, &info)?;
    Ok(info)
}

/// Starts a lesson only after resource validation and the lifecycle gate. Its
/// initial progress row is committed atomically before the engine is started.
pub(crate) fn start_lesson(
    engine: &mut CoreEngine,
    app_state: &AppState,
    lesson_id: String,
    language: String,
) -> Result<TestSessionInfo, AppError> {
    let language = validate_language(language)?;
    validate_resource_identifier(&lesson_id, "lesson")?;
    let lesson = course_loader()
        .load_lesson(&language, &lesson_id)
        .ok_or_else(|| AppError::ResourceNotFound(format!("lesson {lesson_id}")))?;
    let module_id = lesson_id.split('_').take(2).collect::<Vec<_>>().join("_");

    ensure_session_can_start(engine)?;
    // The progress row belongs to an accepted start, not to an arbitrary IPC
    // request. Holding the lifecycle gate prevents a concurrent command from
    // replacing this session between the write and engine initialization.
    app_state.db.with_transaction(|conn| {
        SqliteLessonRepository::new(conn)
            .create_progress(&lesson_id, &module_id, &language, "beginner")?;
        Ok(())
    })?;

    let mut id_source = UuidV7SessionIdSource;
    let info = SessionKernel::new()
        .start_mode(
            engine,
            Box::new(LessonMode::new(
                lesson_id,
                module_id,
                language,
                lesson.text.clone(),
            )),
            &mut id_source,
        )
        .map_err(AppError::from)?;
    record_session_started(app_state, &info)?;
    Ok(info)
}

/// Processes one backend-authoritative input frame and commits a completed
/// session before returning success to the frontend.
pub(crate) fn process_key(
    engine_state: &Mutex<CoreEngine>,
    app_state: &AppState,
    session_id: SessionId,
    key: String,
    code: String,
) -> Result<EngineOutput, AppError> {
    let completion_store = DurableSessionCompletionStore { app_state };
    SessionKernel::new()
        .process_key(
            engine_state,
            app_state,
            app_state,
            &completion_store,
            session_id,
            key,
            code,
        )
        .map_err(AppError::from)
}

/// Aborts only the session identified by the backend-issued immutable ID.
/// The frontend supplies this value as a stale-request guard; it never chooses
/// the identity used when a session is created.
pub(crate) fn abort_session(
    engine_state: &Mutex<CoreEngine>,
    app_state: &AppState,
    session_id: SessionId,
) -> Result<(), AppError> {
    let mut engine = engine_state.lock()?;
    SessionKernel::new()
        .abort_session(&mut engine, session_id.clone())
        .map_err(AppError::from)?;
    record_session_aborted(app_state, &session_id);
    Ok(())
}

/// Allows a new session only after the prior one has either never started or is
/// durably persisted. Keeping retry-pending completions intact is essential:
/// replacing them would discard the only in-memory copy after a failed write.
pub(crate) fn ensure_session_can_start(engine: &CoreEngine) -> Result<(), AppError> {
    SessionKernel::ensure_session_can_start(engine).map_err(AppError::from)
}

fn build_test_mode(
    request: StartTestRequest,
    language: String,
    random_source: &mut dyn SessionRandomSource,
) -> Result<Box<dyn TestMode>, AppError> {
    validate_test_mode(&request.mode)?;
    match request.mode.as_str() {
        "time" => {
            let seconds = request.duration.unwrap_or(30);
            validate_duration(seconds)?;
            let test_text = match request.text {
                Some(text) => validate_test_text(text)?,
                None => word_pack_loader()
                    .generate_words_with_random(
                        &language,
                        TimeMode::recommended_word_count(seconds),
                        random_source,
                    )
                    .ok_or_else(|| AppError::WordsEmpty(language.clone()))?,
            };
            Ok(Box::new(TimeMode::new(test_text, language, seconds)))
        }
        "words" => {
            let word_count = request.word_count.unwrap_or(25);
            validate_word_count(word_count)?;
            let test_text = match request.text {
                Some(text) => validate_test_text(text)?,
                None => word_pack_loader()
                    .generate_words_with_random(&language, word_count, random_source)
                    .ok_or_else(|| AppError::WordsEmpty(language.clone()))?,
            };
            Ok(Box::new(WordsMode::new(test_text, language, word_count)))
        }
        "quote" => {
            let quote = if let Some(quote_id) = request.quote_id {
                let quote_index =
                    usize::try_from(quote_id).map_err(|_| AppError::QuoteNotFound(quote_id))?;
                quote_loader().get_quote_by_index(&language, quote_index)
            } else {
                quote_loader().get_random_quote_with_random(&language, random_source)
            };
            let test_text = quote
                .map(|quote| quote.text.clone())
                .ok_or_else(|| AppError::QuoteNotFound(request.quote_id.unwrap_or(-1)))?;
            Ok(Box::new(QuoteMode::new(
                test_text,
                language,
                request.quote_id,
            )))
        }
        "custom" => {
            let test_text = request.text.ok_or(AppError::CustomTextEmpty)?;
            Ok(Box::new(CustomMode::new(
                validate_test_text(test_text)?,
                language,
            )))
        }
        _ => Err(AppError::InvalidMode(request.mode)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use racoon_application::DurableSessionState;
    use racoon_core::KeyEvent;
    use racoon_data::repository::{
        DailyStatsRepository, PersonalBestsRepository, ReplayRepository,
        SqliteDailyStatsRepository, SqlitePersonalBestsRepository, SqliteReplayRepository,
        SqliteSessionRecoveryLedger,
    };
    use racoon_data::DbError;
    use racoon_domain::FinalStats;
    use rusqlite::OptionalExtension;

    fn test_app_state() -> AppState {
        let settings_path = std::env::temp_dir().join(format!(
            "racoon-durable-store-{}-{}.toml",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        let _ = std::fs::remove_file(&settings_path);
        AppState::new(
            racoon_data::Database::open_in_memory().expect("database"),
            settings_path,
            racoon_application::StartupRecoveryGate::new(),
        )
    }

    fn final_stats() -> FinalStats {
        FinalStats {
            wpm: 40.0,
            raw_wpm: 42.0,
            accuracy: 98.0,
            raw_accuracy: 96.0,
            consistency: Some(90.0),
            correct_chars: 6,
            incorrect_chars: 0,
            backspaces: 0,
            char_stats: serde_json::json!({}),
            heatmap: serde_json::json!({}),
            graph_data: Some(serde_json::json!([])),
            duration_ms: 10_000,
        }
    }

    fn completion(session_id: &str) -> SessionCompletion {
        SessionCompletion {
            session_id: SessionId::parse(session_id).expect("fixture UUIDv7"),
            completed_at: "2026-07-16T12:00:00Z"
                .parse::<DateTime<Utc>>()
                .expect("fixed timestamp"),
            final_stats: final_stats(),
            mode_type: "custom".to_string(),
            mode_config: serde_json::json!({"language": "en"}),
            language: "en".to_string(),
            text_length: 6,
            replay_frames: vec![racoon_core::ReplayFrame {
                timestamp_ms: 0,
                key: "a".to_string(),
                caret_pos: 1,
                char_status: racoon_domain::CharStatus::Correct,
                expected_char: 'a',
                typed_char: Some('a'),
            }],
            lesson_id: None,
        }
    }

    fn record_started_for(app_state: &AppState, session_id: &str) {
        let started = StartedSession::new(
            SessionId::parse(session_id).expect("fixture UUIDv7"),
            "custom",
            serde_json::json!({"language": "en"}),
            "en",
            "2026-07-16T10:00:00Z"
                .parse::<DateTime<Utc>>()
                .expect("fixed timestamp"),
        )
        .expect("fixture start");
        assert!(matches!(
            SqliteSessionRecoveryLedger::new(&app_state.db).record_started(&started),
            Ok(LedgerMutationOutcome::Created)
        ));
    }

    fn durable_state(app_state: &AppState, session_id: &str) -> Option<DurableSessionState> {
        app_state
            .db
            .with_connection(|conn| {
                let state = conn
                    .query_row(
                        "SELECT state FROM session_ledger WHERE session_id = ?1",
                        rusqlite::params![session_id],
                        |row| row.get::<_, String>(0),
                    )
                    .optional()
                    .map_err(|error| DbError::Query(error.to_string()))?;
                Ok(state.and_then(|state| match state.as_str() {
                    "running" => Some(DurableSessionState::Running),
                    "awaiting_persistence" => Some(DurableSessionState::AwaitingPersistence),
                    "finalization_pending" => Some(DurableSessionState::FinalizationPending),
                    "finalized" => Some(DurableSessionState::Finalized),
                    "aborted" => Some(DurableSessionState::Aborted),
                    "interrupted" => Some(DurableSessionState::Interrupted),
                    "quarantined" => Some(DurableSessionState::Quarantined),
                    _ => None,
                }))
            })
            .expect("durable state read")
    }

    #[test]
    fn durable_completion_finalizes_effects_and_terminal_markers() {
        let app_state = test_app_state();
        let session_id = "018f0c2e-7b8d-7abc-8def-0123456789aa";
        record_started_for(&app_state, session_id);

        let store = DurableSessionCompletionStore {
            app_state: &app_state,
        };
        let receipt = store
            .persist_completion(&completion(session_id))
            .expect("durable completion");
        assert!(receipt.test_id > 0);

        assert_eq!(
            durable_state(&app_state, session_id),
            Some(DurableSessionState::Finalized)
        );
        app_state
            .db
            .with_connection(|conn| {
                assert_eq!(SqliteTestRepository::new(conn).get_count(None)?, 1);
                assert!(SqliteReplayRepository::new(conn).has_replay(receipt.test_id)?);
                assert_eq!(
                    SqlitePersonalBestsRepository::new(conn)
                        .get_bests(None)?
                        .len(),
                    1
                );
                let day = SqliteDailyStatsRepository::new(conn)
                    .get_day("2026-07-16")?
                    .expect("daily stats");
                assert_eq!(day.total_tests, 1);
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn durable_completion_retry_is_idempotent() {
        let app_state = test_app_state();
        let session_id = "018f0c2e-7b8d-7abc-8def-0123456789ab";
        record_started_for(&app_state, session_id);

        let store = DurableSessionCompletionStore {
            app_state: &app_state,
        };
        let first = store
            .persist_completion(&completion(session_id))
            .expect("first completion");
        let second = store
            .persist_completion(&completion(session_id))
            .expect("retry completion");
        assert_eq!(first.test_id, second.test_id);

        app_state
            .db
            .with_connection(|conn| {
                assert_eq!(SqliteTestRepository::new(conn).get_count(None)?, 1);
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn durable_completion_without_ledger_row_fails() {
        let app_state = test_app_state();
        let store = DurableSessionCompletionStore {
            app_state: &app_state,
        };
        let error = store
            .persist_completion(&completion("018f0c2e-7b8d-7abc-8def-0123456789ac"))
            .unwrap_err();
        assert!(matches!(error, AppError::Internal(_)));
    }

    #[test]
    fn record_session_started_creates_running_row_and_abort_marks_aborted() {
        let app_state = test_app_state();
        let session_id = "018f0c2e-7b8d-7abc-8def-0123456789ad";
        let info = TestSessionInfo {
            session_id: session_id.to_string(),
            text: "hello".to_string(),
            text_length: 5,
            mode_type: "custom".to_string(),
            mode_config: serde_json::json!({"language": "en"}),
            language: "en".to_string(),
        };
        record_session_started(&app_state, &info).expect("start record");
        assert_eq!(
            durable_state(&app_state, session_id),
            Some(DurableSessionState::Running)
        );

        let session_id = SessionId::parse(session_id).expect("fixture UUIDv7");
        record_session_aborted(&app_state, &session_id);
        assert_eq!(
            durable_state(&app_state, session_id.as_str()),
            Some(DurableSessionState::Aborted)
        );
    }

    #[test]
    fn completion_policy_snapshot_mirrors_legacy_goal_rules() {
        let wpm_settings = AppSettings {
            daily_goal_type: "wpm".to_string(),
            daily_goal_wpm: 60.0,
            ..AppSettings::default()
        };
        assert!(matches!(
            completion_policy_snapshot(&wpm_settings).daily_goal(),
            racoon_application::DailyGoalPolicy::Wpm { target_wpm } if *target_wpm == 60.0
        ));

        let accuracy_settings = AppSettings {
            daily_goal_type: "accuracy".to_string(),
            daily_goal_accuracy: 0.98,
            ..AppSettings::default()
        };
        assert!(matches!(
            completion_policy_snapshot(&accuracy_settings).daily_goal(),
            racoon_application::DailyGoalPolicy::Accuracy { target_accuracy } if *target_accuracy == 0.98
        ));

        let time_settings = AppSettings {
            daily_goal_type: "time".to_string(),
            daily_goal_minutes: 25,
            ..AppSettings::default()
        };
        assert!(matches!(
            completion_policy_snapshot(&time_settings).daily_goal(),
            racoon_application::DailyGoalPolicy::Time { target_minutes } if *target_minutes == 25.0
        ));
    }

    #[test]
    fn session_start_gate_rejects_active_and_retry_pending_sessions() {
        let mut engine = CoreEngine::new();
        assert!(ensure_session_can_start(&engine).is_ok());

        assert!(engine
            .start_test_mode(
                "session-1".to_string(),
                Box::new(TimeMode::new("a".to_string(), "en".to_string(), 30)),
            )
            .is_ok());
        assert!(matches!(
            ensure_session_can_start(&engine),
            Err(AppError::TestAlreadyActive)
        ));

        engine.process_key(&KeyEvent {
            key: "a".to_string(),
            code: "KeyA".to_string(),
            timestamp: 1,
        });
        assert!(matches!(
            ensure_session_can_start(&engine),
            Err(AppError::SessionFinalizing)
        ));
    }

    #[test]
    fn ipc_session_correlation_rejects_a_replaced_or_forged_identity() {
        let mut engine = CoreEngine::new();
        engine
            .start_test_mode(
                "018f0c2e-7b8d-7abc-8def-0123456789ae",
                Box::new(TimeMode::new("a".to_string(), "en".to_string(), 30)),
            )
            .expect("start");
        let app_state = test_app_state();
        let result = process_key(
            &Mutex::new(engine),
            &app_state,
            SessionId::parse("018f0c2e-7b8d-7abc-8def-0123456789af").expect("fixture UUIDv7"),
            "a".to_string(),
            "KeyA".to_string(),
        );
        assert!(matches!(result, Err(AppError::SessionNotFound(_))));
    }
}
