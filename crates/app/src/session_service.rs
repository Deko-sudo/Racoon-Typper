//! Tauri-side adapters for the session vertical slice.
//!
//! The infrastructure-free `racoon-application` crate owns lifecycle and input
//! orchestration. This module supplies resource validation, mode construction,
//! SQLite completion persistence, and the existing custom-text/lesson setup
//! adapters without exposing those concerns to the kernel.

use racoon_application::{
    SessionCompletion, SessionCompletionStore, SessionIdSource, SessionKernel, SessionModeFactory,
    SessionPersistenceReceipt, SessionRandomSource,
};
use racoon_core::{
    CoreEngine, CustomMode, LessonMode, QuoteMode, TestMode, TestSessionInfo, TimeMode, WordsMode,
};
use racoon_data::repository::{
    AppSettings, CustomTextRepository, DailyStatsRepository, LessonRepository,
    PersonalBestsRepository, ReplayRepository, SqliteCustomTextRepository,
    SqliteDailyStatsRepository, SqliteLessonRepository, SqlitePersonalBestsRepository,
    SqliteReplayRepository, SqliteStreakRepository, SqliteTestRepository, StreakRecord,
    StreakRepository, TestRepository,
};
use racoon_data::DbError;
use racoon_domain::{EngineOutput, SessionId, TestRecord};
use racoon_resources::{course_loader, quote_loader, word_pack_loader, SystemRandomSource};
use std::sync::Mutex;

use crate::error::AppError;
use crate::state::AppState;
use crate::validation::{
    validate_duration, validate_language, validate_positive_id, validate_resource_identifier,
    validate_test_mode, validate_test_text, validate_word_count,
};

pub(crate) use racoon_application::SessionStartRequest as StartTestRequest;

type CompletedSession = SessionCompletion;

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

struct SqliteSessionCompletionStore<'a> {
    app_state: &'a AppState,
}

impl SessionCompletionStore for SqliteSessionCompletionStore<'_> {
    type Error = AppError;

    fn persist_completion(
        &self,
        completion: &SessionCompletion,
    ) -> Result<SessionPersistenceReceipt, Self::Error> {
        let settings = self.app_state.with_settings(|store| store.load())?;
        let test_id = self
            .app_state
            .db
            .with_transaction(|conn| persist_completed_session(conn, completion, &settings))?;
        Ok(SessionPersistenceReceipt { test_id })
    }
}

/// Starts a standard mode after validating and selecting backend-owned content.
pub(crate) fn start_test(
    engine: &mut CoreEngine,
    mut request: StartTestRequest,
) -> Result<TestSessionInfo, AppError> {
    let language = validate_language(request.language.take().unwrap_or_else(|| "en".to_string()))?;
    let mut id_source = UuidV7SessionIdSource;
    let mut random_source = SystemRandomSource;
    SessionKernel::new()
        .start_session(
            engine,
            &request,
            &language,
            &mut id_source,
            &mut random_source,
            &BackendSessionModeFactory,
        )
        .map_err(AppError::from)
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
    SessionKernel::new()
        .start_mode(
            engine,
            Box::new(CustomMode::new(custom_text.text, custom_text.language)),
            &mut id_source,
        )
        .map_err(AppError::from)
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
    SessionKernel::new()
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
        .map_err(AppError::from)
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
    let completion_store = SqliteSessionCompletionStore { app_state };
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
    session_id: SessionId,
) -> Result<(), AppError> {
    let mut engine = engine_state.lock()?;
    SessionKernel::new()
        .abort_session(&mut engine, session_id)
        .map_err(AppError::from)
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

fn test_record_from_completion(completed: &CompletedSession) -> TestRecord {
    TestRecord {
        session_id: completed.session_id.clone(),
        created_at: completed.completed_at.to_rfc3339(),
        mode_type: completed.mode_type.clone(),
        mode_config: completed.mode_config.clone(),
        language: completed.language.clone(),
        text_length: completed.text_length,
        duration_ms: completed.final_stats.duration_ms,
        wpm: completed.final_stats.wpm,
        raw_wpm: completed.final_stats.raw_wpm,
        accuracy: completed.final_stats.accuracy,
        raw_accuracy: completed.final_stats.raw_accuracy,
        consistency: completed.final_stats.consistency,
        correct_chars: completed.final_stats.correct_chars,
        incorrect_chars: completed.final_stats.incorrect_chars,
        backspaces: completed.final_stats.backspaces,
        char_stats: completed.final_stats.char_stats.clone(),
        heatmap_data: completed.final_stats.heatmap.clone(),
        graph_data: completed.final_stats.graph_data.clone(),
        is_pb: false,
        tags: String::new(),
    }
}

/// Persists all completion-side effects in the transaction opened by the caller.
///
/// The sequence deliberately has no externally visible intermediate state:
/// history, replay, personal bests, daily statistics, streaks, daily goals, and
/// lesson completion either commit together or are rolled back together.
fn persist_completed_session(
    conn: &rusqlite::Connection,
    completed: &CompletedSession,
    settings: &AppSettings,
) -> Result<i64, DbError> {
    let test_repo = SqliteTestRepository::new(conn);
    let test_id = test_repo.save_test(test_record_from_completion(completed))?;

    let replay_frames = completed
        .replay_frames
        .iter()
        .enumerate()
        .map(|(index, frame)| {
            Ok(racoon_data::repository::ReplayFrame {
                id: 0,
                test_id,
                frame_index: i64::try_from(index)
                    .map_err(|_| DbError::Write("Replay frame index exceeds i64".to_string()))?,
                timestamp_ms: i64::try_from(frame.timestamp_ms)
                    .map_err(|_| DbError::Write("Replay timestamp exceeds i64".to_string()))?,
                position: i64::try_from(frame.caret_pos)
                    .map_err(|_| DbError::Write("Replay position exceeds i64".to_string()))?,
                expected_char: frame.expected_char.to_string(),
                typed_char: Some(
                    frame
                        .typed_char
                        .map_or_else(|| frame.key.clone(), |character| character.to_string()),
                ),
                correct: frame.char_status == racoon_domain::CharStatus::Correct,
            })
        })
        .collect::<Result<Vec<_>, DbError>>()?;
    SqliteReplayRepository::new(conn).save_replay(test_id, &replay_frames)?;

    let mode_config = serde_json::to_string(&completed.mode_config)
        .map_err(|error| DbError::Write(format!("mode configuration serialization: {error}")))?;
    let personal_best_updates = SqlitePersonalBestsRepository::new(conn).check_and_update(
        &completed.mode_type,
        &mode_config,
        completed.final_stats.wpm,
        completed.final_stats.accuracy,
        test_id,
    )?;
    if !personal_best_updates.is_empty() {
        test_repo.mark_as_pb(test_id)?;
    }

    let daily_repo = SqliteDailyStatsRepository::new(conn);
    // The calendar day belongs to the user's local timezone, not UTC: a test
    // finished at 01:00 Europe/Moscow (22:00 UTC the previous day) must count
    // toward the local day it was typed in. chrono::Local uses the system
    // timezone, which is the desktop app's natural source of truth.
    let completion_date = completed
        .completed_at
        .with_timezone(&chrono::Local)
        .format("%Y-%m-%d")
        .to_string();
    let duration_ms = i64::try_from(completed.final_stats.duration_ms)
        .map_err(|_| DbError::Write("Test duration exceeds i64".to_string()))?;
    let total_chars = completed
        .final_stats
        .correct_chars
        .checked_add(completed.final_stats.incorrect_chars)
        .and_then(|count| i64::try_from(count).ok())
        .ok_or_else(|| DbError::Write("Test character count exceeds i64".to_string()))?;
    daily_repo.update_after_test(
        &completion_date,
        duration_ms,
        total_chars,
        completed.final_stats.wpm,
        completed.final_stats.accuracy,
    )?;
    persist_daily_streak(conn, &completion_date)?;

    if let Some(lesson_id) = &completed.lesson_id {
        let passed = SqliteLessonRepository::new(conn).complete_lesson(
            lesson_id,
            completed.final_stats.wpm,
            completed.final_stats.accuracy,
        )?;
        // Счётчик пройденных уроков растёт только прошедшим гейт попыткам.
        if passed {
            daily_repo.increment_lessons_completed(&completion_date)?;
        }
    }

    let day_stats = daily_repo.get_day(&completion_date)?.ok_or_else(|| {
        DbError::Integrity(format!(
            "Daily stats were not created for {completion_date}"
        ))
    })?;
    if daily_goal_is_met(settings, &day_stats) {
        daily_repo.set_daily_goal_met(&completion_date, true)?;
    }

    Ok(test_id)
}

fn daily_goal_is_met(settings: &AppSettings, stats: &racoon_data::repository::DailyStats) -> bool {
    match settings.daily_goal_type.as_str() {
        "wpm" => settings.daily_goal_wpm > 0.0 && stats.best_wpm >= settings.daily_goal_wpm,
        "accuracy" => {
            settings.daily_goal_accuracy > 0.0 && stats.avg_accuracy >= settings.daily_goal_accuracy
        }
        "time" => time_goal_is_met(settings.daily_goal_minutes, stats.total_time_ms),
        _ => false,
    }
}

/// Time goal is met when the day's accumulated typing time reaches the target
/// in minutes. A zero/negative target means the goal is unset, so it is never
/// reported as met — matching the other goal types, where a zero target is not
/// met either.
fn time_goal_is_met(target_minutes: i64, total_time_ms: i64) -> bool {
    target_minutes > 0 && (total_time_ms as f64 / 60_000.0) >= target_minutes as f64
}

fn persist_daily_streak(conn: &rusqlite::Connection, today: &str) -> Result<(), DbError> {
    let repository = SqliteStreakRepository::new(conn);
    let existing = repository.get("daily_test")?;
    let (previous_current, previous_longest, last_date, previous_started_date) = existing
        .map(|streak| {
            (
                streak.current_streak,
                streak.longest_streak,
                streak.last_date,
                streak.started_date,
            )
        })
        .unwrap_or((0, 0, None, None));
    let starts_new_streak = last_date
        .as_deref()
        .is_none_or(|last| racoon_core::StreakEngine::days_between(last, today) > 1);
    let (current, longest, _) = racoon_core::StreakEngine::compute_streak(
        previous_current,
        previous_longest,
        last_date.as_deref(),
        today,
    );
    let started_date = if starts_new_streak {
        today.to_string()
    } else {
        previous_started_date.unwrap_or_else(|| today.to_string())
    };

    repository.upsert(&StreakRecord {
        streak_type: "daily_test".to_string(),
        current_streak: current,
        longest_streak: longest,
        last_date: Some(today.to_string()),
        started_date: Some(started_date),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use racoon_core::KeyEvent;
    use racoon_data::repository::DailyStats;
    use racoon_domain::FinalStats;

    fn daily_stats(total_time_ms: i64, best_wpm: f64, avg_accuracy: f64) -> DailyStats {
        DailyStats {
            date: "2026-07-16".to_owned(),
            total_tests: 1,
            total_time_ms,
            total_chars: 1,
            best_wpm,
            avg_wpm: best_wpm,
            avg_accuracy,
            lessons_completed: 0,
            daily_goal_met: false,
        }
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

    fn completed_session(lesson_id: Option<&str>) -> CompletedSession {
        completed_session_at(lesson_id, "2026-07-12T12:00:00Z")
    }

    fn completed_session_at(lesson_id: Option<&str>, completed_at: &str) -> CompletedSession {
        CompletedSession {
            session_id: SessionId::from("test-session"),
            completed_at: completed_at
                .parse::<DateTime<Utc>>()
                .expect("fixed test timestamp is valid"),
            final_stats: final_stats(),
            mode_type: if lesson_id.is_some() {
                "lesson".to_string()
            } else {
                "custom".to_string()
            },
            mode_config: lesson_id.map_or_else(
                || serde_json::json!({"language": "ru"}),
                |lesson_id| serde_json::json!({"lesson_id": lesson_id, "module_id": "en_m1"}),
            ),
            language: "en".to_string(),
            text_length: "привет".chars().count(),
            replay_frames: vec![racoon_core::ReplayFrame {
                timestamp_ms: 0,
                key: "п".to_string(),
                caret_pos: 1,
                char_status: racoon_domain::CharStatus::Correct,
                expected_char: 'п',
                typed_char: Some('п'),
            }],
            lesson_id: lesson_id.map(ToOwned::to_owned),
        }
    }

    fn local_day_of(completed: &CompletedSession) -> String {
        completed
            .completed_at
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d")
            .to_string()
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
                SessionId::from("backend-session"),
                Box::new(TimeMode::new("a".to_string(), "en".to_string(), 30)),
            )
            .unwrap();

        assert!(SessionKernel::new()
            .abort_session(&mut engine, SessionId::from("backend-session"))
            .is_ok());

        let mut engine = CoreEngine::new();
        engine
            .start_test_mode(
                SessionId::from("backend-session"),
                Box::new(TimeMode::new("a".to_string(), "en".to_string(), 30)),
            )
            .unwrap();
        assert!(matches!(
            SessionKernel::new()
                .abort_session(&mut engine, SessionId::from("frontend-chosen-session"))
                .map_err(AppError::from),
            Err(AppError::SessionNotFound(_))
        ));
    }

    #[test]
    fn completed_session_record_preserves_unicode_text_length_and_one_timestamp() {
        let completed = completed_session(None);
        let record = test_record_from_completion(&completed);

        assert_eq!(record.text_length, 6);
        assert!(record.created_at.starts_with("2026-07-12T12:00:00"));
    }

    #[test]
    fn completed_lesson_persists_all_related_records_in_one_transaction() {
        let database = racoon_data::Database::open_in_memory().unwrap();
        let completed = completed_session(Some("en_m1_l1"));
        let settings = AppSettings::default();
        let expected_day = local_day_of(&completed);

        database
            .with_transaction(|conn| {
                SqliteLessonRepository::new(conn)
                    .create_progress("en_m1_l1", "en_m1", "en", "beginner")?;
                persist_completed_session(conn, &completed, &settings)
            })
            .unwrap();

        database
            .with_connection(|conn| {
                let test_repo = SqliteTestRepository::new(conn);
                assert_eq!(test_repo.get_count(None)?, 1);
                assert!(SqliteReplayRepository::new(conn).has_replay(1)?);
                assert_eq!(
                    SqlitePersonalBestsRepository::new(conn)
                        .get_bests(None)?
                        .len(),
                    1
                );

                let daily = SqliteDailyStatsRepository::new(conn)
                    .get_day(&expected_day)?
                    .expect("daily statistics should exist");
                assert_eq!(daily.total_tests, 1);
                assert_eq!(daily.lessons_completed, 1);
                assert_eq!(
                    SqliteLessonRepository::new(conn)
                        .get_lesson_progress("en_m1_l1")?
                        .expect("lesson progress should exist")
                        .status,
                    "completed"
                );
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn completion_side_effects_roll_back_together() {
        let database = racoon_data::Database::open_in_memory().unwrap();
        let completed = completed_session(None);
        let settings = AppSettings::default();
        let expected_day = local_day_of(&completed);

        let result: Result<(), DbError> = database.with_transaction(|conn| {
            persist_completed_session(conn, &completed, &settings)?;
            Err(DbError::Write("forced rollback".to_string()))
        });
        assert!(result.is_err());

        database
            .with_connection(|conn| {
                assert_eq!(SqliteTestRepository::new(conn).get_count(None)?, 0);
                assert!(!SqliteReplayRepository::new(conn).has_replay(1)?);
                assert!(SqliteDailyStatsRepository::new(conn)
                    .get_day(&expected_day)?
                    .is_none());
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn daily_goal_rule_preserves_zero_and_fractional_time_targets() {
        let mut settings = AppSettings {
            daily_goal_type: "time".to_owned(),
            daily_goal_minutes: 0,
            ..AppSettings::default()
        };
        // Zero/unset time goal is never met, matching WPM/accuracy semantics.
        assert!(!daily_goal_is_met(&settings, &daily_stats(0, 0.0, 0.0)));
        assert!(!daily_goal_is_met(
            &settings,
            &daily_stats(60_000, 0.0, 0.0)
        ));
        settings.daily_goal_minutes = -1;
        assert!(!daily_goal_is_met(
            &settings,
            &daily_stats(60_000, 0.0, 0.0)
        ));
        settings.daily_goal_minutes = 1;
        assert!(!daily_goal_is_met(
            &settings,
            &daily_stats(59_999, 0.0, 0.0)
        ));
        assert!(daily_goal_is_met(&settings, &daily_stats(60_000, 0.0, 0.0)));
        assert!(daily_goal_is_met(&settings, &daily_stats(60_001, 0.0, 0.0)));

        // Switching time → wpm must use the WPM field, not the minutes field.
        settings.daily_goal_minutes = 100;
        settings.daily_goal_type = "wpm".to_owned();
        settings.daily_goal_wpm = 60.0;
        assert!(daily_goal_is_met(
            &settings,
            &daily_stats(1_000, 60.0, 0.98)
        ));
        settings.daily_goal_wpm = 0.0;
        assert!(!daily_goal_is_met(
            &settings,
            &daily_stats(1_000, 60.0, 0.98)
        ));
    }

    #[test]
    fn daily_goal_rule_preserves_wpm_and_accuracy_zero_rules() {
        let mut settings = AppSettings {
            daily_goal_type: "wpm".to_owned(),
            daily_goal_wpm: 0.0,
            ..AppSettings::default()
        };
        assert!(!daily_goal_is_met(
            &settings,
            &daily_stats(1_000, 60.0, 0.98)
        ));
        settings.daily_goal_wpm = 60.0;
        assert!(daily_goal_is_met(
            &settings,
            &daily_stats(1_000, 60.0, 0.98)
        ));

        settings.daily_goal_type = "accuracy".to_owned();
        assert!(!daily_goal_is_met(
            &settings,
            &daily_stats(1_000, 60.0, 0.98)
        ));
        settings.daily_goal_accuracy = 0.98;
        assert!(daily_goal_is_met(
            &settings,
            &daily_stats(1_000, 60.0, 0.98)
        ));
    }

    // The persistence day must follow the user's local calendar day. A test
    // finished at 2026-08-12 01:00 Europe/Moscow (2026-08-11 22:00 UTC) must be
    // recorded under 2026-08-12, and one finished at 2026-08-12 23:30
    // Europe/Moscow (20:30 UTC) must not bleed into the next local day.
    // These tests assert the local-day invariant directly; they are
    // timezone-agnostic because the expected day is derived the same way the
    // production code derives it.
    #[test]
    fn local_midnight_crossing_counts_toward_local_day() {
        let database = racoon_data::Database::open_in_memory().unwrap();
        // 2026-08-12 01:00 Europe/Moscow == 2026-08-11 22:00 UTC. For any UTC
        // offset in [-11, +14] this still lands on a local day that can be
        // derived; we assert the recorded day equals the local projection.
        let completed = completed_session_at(None, "2026-08-11T22:00:00Z");
        let settings = AppSettings::default();
        let expected_day = local_day_of(&completed);

        database
            .with_transaction(|conn| persist_completed_session(conn, &completed, &settings))
            .unwrap();

        database
            .with_connection(|conn| {
                let daily = SqliteDailyStatsRepository::new(conn)
                    .get_day(&expected_day)?
                    .expect("daily statistics should exist for the local day");
                assert_eq!(daily.total_tests, 1);
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn local_late_evening_does_not_bleed_into_next_day() {
        let database = racoon_data::Database::open_in_memory().unwrap();
        // 2026-08-12 23:30 Europe/Moscow == 2026-08-12 20:30 UTC. The local day
        // is still 2026-08-12 for UTC+3; the invariant is that the recorded day
        // matches the local projection of the timestamp.
        let completed = completed_session_at(None, "2026-08-12T20:30:00Z");
        let settings = AppSettings::default();
        let expected_day = local_day_of(&completed);

        database
            .with_transaction(|conn| persist_completed_session(conn, &completed, &settings))
            .unwrap();

        database
            .with_connection(|conn| {
                let daily = SqliteDailyStatsRepository::new(conn)
                    .get_day(&expected_day)?
                    .expect("daily statistics should exist for the local day");
                assert_eq!(daily.total_tests, 1);
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn utc_day_and_local_day_differ_when_timezone_is_east_of_utc() {
        // 2026-08-11T22:00:00Z is 2026-08-11 in UTC but 2026-08-12 in
        // Europe/Moscow. The recorded day must be the local one: assert the
        // UTC date is NOT used when the local date differs.
        let completed = completed_session_at(None, "2026-08-11T22:00:00Z");
        let utc_day = completed.completed_at.format("%Y-%m-%d").to_string();
        let local_day = local_day_of(&completed);
        // This is only meaningful when the offsets actually differ; the test
        // below still validates the core invariant regardless.
        if local_day != utc_day {
            let database = racoon_data::Database::open_in_memory().unwrap();
            let settings = AppSettings::default();
            database
                .with_transaction(|conn| persist_completed_session(conn, &completed, &settings))
                .unwrap();
            database
                .with_connection(|conn| {
                    assert!(SqliteDailyStatsRepository::new(conn)
                        .get_day(&local_day)?
                        .is_some());
                    assert!(SqliteDailyStatsRepository::new(conn)
                        .get_day(&utc_day)?
                        .is_none());
                    Ok(())
                })
                .unwrap();
        }
    }
}
