//! CoreEngine — связывает Input, Typing, TestMode, возвращает EngineOutput.
//! Синхронная архитектура: process_key() → EngineOutput.
//! CoreEngine не знает конкретный режим — работает через dyn TestMode.

use chrono::{DateTime, Utc};
use racoon_domain::{
    CharStatus, EngineOutput, FinalStats, KeyResult, SessionId, SessionState, VisiblePos,
};

use crate::input::{KeyAction, KeyClassifier, KeyEvent};
use crate::modes::{ModeResult, ModeType, TestMode};
use crate::stats::StatisticsEngine;
use crate::typing::{TextBuffer, TypingResult};

/// Сессия теста.
pub struct TestSession {
    pub session_id: SessionId,
    pub mode: Box<dyn TestMode>,
    pub buffer: TextBuffer,
}

/// CoreEngine — главный движок.
pub struct CoreEngine {
    session: Option<TestSession>,
    session_state: SessionState,
    stats: StatisticsEngine,
    replay_frames: Vec<ReplayFrame>,
    replay_start_timestamp_ms: Option<u64>,
    completed_stats: Option<FinalStats>,
    completed_at: Option<DateTime<Utc>>,
}

/// One authoritative input frame captured during a typing session.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ReplayFrame {
    pub timestamp_ms: u64,
    pub key: String,
    pub caret_pos: usize,
    pub char_status: CharStatus,
    pub expected_char: char,
    pub typed_char: Option<char>,
}

/// Информация о стартованной сессии (возвращается в frontend).
#[derive(Debug, Clone, serde::Serialize)]
pub struct TestSessionInfo {
    pub session_id: String,
    pub text: String,
    pub text_length: usize,
    pub mode_type: String,
    pub mode_config: serde_json::Value,
    pub language: String,
}

impl CoreEngine {
    pub fn new() -> Self {
        Self {
            session: None,
            session_state: SessionState::Idle,
            stats: StatisticsEngine::new(),
            replay_frames: Vec::new(),
            replay_start_timestamp_ms: None,
            completed_stats: None,
            completed_at: None,
        }
    }

    /// Starts a new test only from an empty or durably completed session.
    ///
    /// Returning the rejected state closes the core-level escape hatch that
    /// previously let a direct caller replace a running or retry-pending
    /// session. The app maps that state to its stable IPC error envelope.
    pub fn start_test_mode(
        &mut self,
        session_id: impl Into<SessionId>,
        mode: Box<dyn TestMode>,
    ) -> Result<TestSessionInfo, SessionState> {
        if !matches!(
            self.session_state,
            SessionState::Idle | SessionState::Persisted
        ) {
            return Err(self.session_state);
        }
        let session_id = session_id.into();
        let text = mode.get_text().to_string();
        let text_length = text.chars().count();
        let mode_type = mode.mode_type().to_string();
        let mode_config = mode.mode_config();
        let language = mode.language().to_string();

        let buffer = TextBuffer::new(&text);

        let info = TestSessionInfo {
            session_id: session_id.to_string(),
            text: text.clone(),
            text_length,
            mode_type,
            mode_config,
            language,
        };

        self.session = Some(TestSession {
            session_id,
            mode,
            buffer,
        });
        self.session_state = SessionState::Running;
        self.stats.reset();
        self.replay_frames.clear();
        self.replay_start_timestamp_ms = None;
        self.completed_stats = None;
        self.completed_at = None;
        Ok(info)
    }

    /// Aborts a session only while it is still safe to discard it.
    ///
    /// A completed result is never discarded while it is being persisted or is
    /// waiting to be retried. Callers must let that transaction settle so an
    /// already completed test cannot be silently lost or reported as failed.
    pub fn abort(&mut self) -> bool {
        if matches!(
            self.session_state,
            SessionState::AwaitingPersistence | SessionState::Persisting
        ) {
            return false;
        }
        self.session = None;
        self.session_state = SessionState::Idle;
        self.stats.reset();
        self.replay_frames.clear();
        self.replay_start_timestamp_ms = None;
        self.completed_stats = None;
        self.completed_at = None;
        true
    }

    /// Resets only a running session with the same mode and text.
    ///
    /// A completed result must remain immutable while persistence is pending or
    /// retryable, so reset is rejected outside `Running`.
    pub fn reset(&mut self) -> bool {
        if self.session_state != SessionState::Running {
            return false;
        }
        if let Some(session) = &mut self.session {
            let text = session.mode.get_text().to_string();
            session.buffer = TextBuffer::new(&text);
        }
        self.stats.reset();
        self.replay_frames.clear();
        self.replay_start_timestamp_ms = None;
        self.completed_stats = None;
        self.completed_at = None;
        true
    }

    /// Обрабатывает нажатие клавиши. Возвращает EngineOutput.
    pub fn process_key(&mut self, key_event: &KeyEvent) -> EngineOutput {
        match self.session_state {
            SessionState::Idle => return noop_output(SessionState::Idle),
            SessionState::AwaitingPersistence => {
                return completed_output(
                    SessionState::AwaitingPersistence,
                    self.completed_stats.clone(),
                );
            }
            SessionState::Persisting => return terminal_output(SessionState::Persisting),
            SessionState::Persisted => return terminal_output(SessionState::Persisted),
            SessionState::Running => {}
        }

        let session = match &mut self.session {
            Some(s) => s,
            None => {
                self.session_state = SessionState::Idle;
                return noop_output(SessionState::Idle);
            }
        };

        // Классификация
        let action = KeyClassifier::classify(&key_event.key, &key_event.code);
        let previous_caret_pos = session.buffer.current_position;

        // Делегируем обработку режиму
        let typing_result = match action {
            KeyAction::Print(ch) => {
                let mode_result =
                    session
                        .mode
                        .on_key_press(ch, key_event.timestamp, &mut session.buffer);
                match mode_result {
                    ModeResult::Complete => TypingResult::TestEnded,
                    ModeResult::Continue => {
                        match session
                            .buffer
                            .typed_chars
                            .get(previous_caret_pos)
                            .map(|typed| &typed.status)
                        {
                            Some(racoon_domain::CharStatus::Correct) => TypingResult::Correct,
                            Some(racoon_domain::CharStatus::Incorrect) => TypingResult::Incorrect,
                            _ => TypingResult::Noop,
                        }
                    }
                    ModeResult::Failed(_) => TypingResult::Noop,
                }
            }
            KeyAction::Backspace => {
                let backspace_result = match session
                    .buffer
                    .typed_chars
                    .get(previous_caret_pos)
                    .map(|typed| &typed.status)
                {
                    Some(CharStatus::Incorrect) => TypingResult::UndoneIncorrect,
                    _ if previous_caret_pos > 0 => match session
                        .buffer
                        .typed_chars
                        .get(previous_caret_pos - 1)
                        .map(|typed| &typed.status)
                    {
                        Some(CharStatus::Correct) => TypingResult::UndoneCorrect,
                        Some(CharStatus::Incorrect) => TypingResult::UndoneIncorrect,
                        _ => TypingResult::Noop,
                    },
                    _ => TypingResult::Noop,
                };
                let mode_result = session.mode.on_backspace(&mut session.buffer);
                match mode_result {
                    ModeResult::Complete => TypingResult::TestEnded,
                    _ => backspace_result,
                }
            }
            _ => TypingResult::Noop,
        };

        let caret_pos = session.buffer.current_position;
        let visible_pos = calc_visible_pos(caret_pos);
        let is_complete = session.mode.is_complete(&session.buffer);

        // Обновляем статистику
        self.stats
            .on_key_processed_at(&typing_result, &session.buffer, key_event.timestamp);

        if !matches!(typing_result, TypingResult::Noop | TypingResult::TestEnded) {
            let affected_position = match typing_result {
                TypingResult::Correct | TypingResult::Incorrect => previous_caret_pos,
                TypingResult::UndoneCorrect | TypingResult::UndoneIncorrect => caret_pos,
                TypingResult::Noop | TypingResult::TestEnded => unreachable!(),
            };
            if let Some(typed_char) = session.buffer.typed_chars.get(affected_position) {
                let start_timestamp = *self
                    .replay_start_timestamp_ms
                    .get_or_insert(key_event.timestamp);
                self.replay_frames.push(ReplayFrame {
                    timestamp_ms: key_event.timestamp.saturating_sub(start_timestamp),
                    key: key_event.key.clone(),
                    caret_pos,
                    char_status: typed_char.status.clone(),
                    expected_char: typed_char.expected,
                    typed_char: typed_char.typed,
                });
            }
        }

        // Live stats
        let live_stats = if session.buffer.start_time.is_some() && !is_complete {
            Some(self.stats.live_stats(&session.buffer))
        } else {
            None
        };

        // Финализация
        let test_complete: Option<FinalStats> = if is_complete {
            let duration = session.buffer.elapsed_ms();
            Some(self.stats.finalize(&session.buffer, duration))
        } else {
            None
        };

        if let Some(final_stats) = &test_complete {
            self.completed_stats = Some(final_stats.clone());
            self.session_state = SessionState::AwaitingPersistence;
        }

        // Маппинг TypingResult → KeyResult
        let key_result = match typing_result {
            TypingResult::Correct => KeyResult::Correct,
            TypingResult::Incorrect => KeyResult::Incorrect,
            TypingResult::UndoneCorrect => KeyResult::UndoneCorrect,
            TypingResult::UndoneIncorrect => KeyResult::UndoneIncorrect,
            TypingResult::Noop => KeyResult::Noop,
            TypingResult::TestEnded => KeyResult::TestEnded,
        };

        EngineOutput {
            session_state: self.session_state,
            key_result,
            caret_pos,
            visible_pos,
            live_stats,
            lesson_delta: None,
            test_complete,
            text_scrolled: None,
            keyboard_viz: None,
        }
    }

    /// Claims a completed session before a database transaction begins.
    /// Only one caller can make the transition, which prevents concurrent IPC
    /// requests from attempting the same completion persistence twice.
    pub fn begin_persistence(&mut self) -> bool {
        if self.session_state != SessionState::AwaitingPersistence {
            return false;
        }
        self.session_state = SessionState::Persisting;
        true
    }

    /// Marks the completion durable after its transaction has committed.
    pub fn mark_persisted(&mut self) -> bool {
        if self.session_state != SessionState::Persisting {
            return false;
        }
        self.session_state = SessionState::Persisted;
        true
    }

    /// Makes a failed persistence attempt retryable without accepting further
    /// typing input or recalculating the completed result.
    pub fn mark_persistence_failed(&mut self) -> bool {
        if self.session_state != SessionState::Persisting {
            return false;
        }
        self.session_state = SessionState::AwaitingPersistence;
        true
    }

    /// Returns the current authoritative session lifecycle state.
    pub fn session_state(&self) -> SessionState {
        self.session_state
    }

    /// Records the wall-clock metadata supplied by the application runtime at
    /// the authoritative completion transition. It is retained through retry
    /// so a failed persistence attempt cannot move a completed record into a
    /// different UTC day.
    pub fn set_completion_timestamp(&mut self, completed_at: DateTime<Utc>) -> bool {
        if self.session_state != SessionState::AwaitingPersistence
            || self.completed_stats.is_none()
            || self.completed_at.is_some()
        {
            return false;
        }
        self.completed_at = Some(completed_at);
        true
    }

    /// Wall-clock metadata captured once at the authoritative completion
    /// transition. It is retained through retry so a failed persistence attempt
    /// cannot move a completed record into a different UTC day.
    pub fn completion_timestamp(&self) -> Option<DateTime<Utc>> {
        self.completed_at
    }

    /// Returns the immutable identity of the current or retryable session.
    pub fn current_session_id(&self) -> Option<&SessionId> {
        self.session.as_ref().map(|session| &session.session_id)
    }

    /// Returns true while a completion transaction is in flight.
    pub fn is_finalizing(&self) -> bool {
        self.session_state == SessionState::Persisting
    }

    /// Активна ли сессия.
    pub fn is_active(&self) -> bool {
        self.session_state == SessionState::Running
    }

    /// Текст текущего теста.
    pub fn current_text(&self) -> Option<&str> {
        self.session.as_ref().map(|s| s.mode.get_text())
    }

    /// Тип режима текущего теста.
    pub fn current_mode_type(&self) -> Option<ModeType> {
        self.session.as_ref().map(|s| s.mode.mode_type())
    }

    /// Конфигурация режима текущего теста.
    pub fn current_mode_config(&self) -> Option<serde_json::Value> {
        self.session.as_ref().map(|s| s.mode.mode_config())
    }

    /// Язык текущего теста.
    pub fn current_language(&self) -> Option<&str> {
        self.session.as_ref().map(|s| s.mode.language())
    }

    /// Replay frames captured for the active or most recently completed session.
    pub fn replay_frames(&self) -> &[ReplayFrame] {
        &self.replay_frames
    }

    /// Возвращает char_stats из последней завершённой сессии.
    pub fn current_char_stats(
        &self,
    ) -> Option<std::collections::HashMap<String, racoon_domain::keyboard::CharStat>> {
        self.session
            .as_ref()
            .map(|s| crate::stats::HeatmapBuilder::build_char_stats(&s.buffer))
    }

    /// Статус символа на позиции.
    pub fn char_status_at(&self, pos: usize) -> Option<racoon_domain::CharStatus> {
        self.session
            .as_ref()
            .and_then(|s| s.buffer.char_status_at(pos))
    }

    /// Текущая позиция курсора.
    pub fn caret_position(&self) -> usize {
        self.session
            .as_ref()
            .map(|s| s.buffer.current_position)
            .unwrap_or(0)
    }

    /// Завершён ли тест.
    pub fn is_complete(&self) -> bool {
        self.session
            .as_ref()
            .map(|s| s.mode.is_complete(&s.buffer))
            .unwrap_or(false)
    }
}

impl Default for CoreEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Возвращает позицию курсора как (row, col).
fn calc_visible_pos(caret_pos: usize) -> VisiblePos {
    VisiblePos {
        row: 0,
        col: caret_pos,
    }
}

/// Noop output для случая без активной сессии.
fn noop_output(session_state: SessionState) -> EngineOutput {
    EngineOutput {
        session_state,
        key_result: KeyResult::Noop,
        caret_pos: 0,
        visible_pos: VisiblePos { row: 0, col: 0 },
        live_stats: None,
        lesson_delta: None,
        test_complete: None,
        text_scrolled: None,
        keyboard_viz: None,
    }
}

fn terminal_output(session_state: SessionState) -> EngineOutput {
    EngineOutput {
        session_state,
        key_result: KeyResult::TestEnded,
        caret_pos: 0,
        visible_pos: VisiblePos { row: 0, col: 0 },
        live_stats: None,
        lesson_delta: None,
        test_complete: None,
        text_scrolled: None,
        keyboard_viz: None,
    }
}

fn completed_output(
    session_state: SessionState,
    test_complete: Option<FinalStats>,
) -> EngineOutput {
    EngineOutput {
        session_state,
        key_result: KeyResult::TestEnded,
        caret_pos: 0,
        visible_pos: VisiblePos { row: 0, col: 0 },
        live_stats: None,
        lesson_delta: None,
        test_complete,
        text_scrolled: None,
        keyboard_viz: None,
    }
}

#[cfg(test)]
mod tests {
    #![allow(unused_must_use)]

    use super::*;
    use crate::modes::time::TimeMode;

    fn make_key(key: &str, code: &str) -> KeyEvent {
        KeyEvent {
            key: key.to_string(),
            code: code.to_string(),
            timestamp: 100,
        }
    }

    #[test]
    fn start_test_with_time_mode() {
        let mut engine = CoreEngine::new();
        let mode = Box::new(TimeMode::new("hello".to_string(), "en".to_string(), 30));
        let info = engine
            .start_test_mode("s1".to_string(), mode)
            .expect("idle engine should accept a session");

        assert_eq!(info.text, "hello");
        assert_eq!(info.mode_type, "time");
        assert_eq!(info.language, "en");
        assert_eq!(info.text_length, 5);
    }

    #[test]
    fn process_key_correct_with_mode() {
        let mut engine = CoreEngine::new();
        let mode = Box::new(TimeMode::new("hello".to_string(), "en".to_string(), 30));
        engine.start_test_mode("s1".to_string(), mode);

        let output = engine.process_key(&make_key("h", "KeyH"));
        assert_eq!(output.key_result, KeyResult::Correct);
        assert_eq!(output.caret_pos, 1);
    }

    #[test]
    fn process_key_incorrect_with_mode() {
        let mut engine = CoreEngine::new();
        let mode = Box::new(TimeMode::new("hello".to_string(), "en".to_string(), 30));
        engine.start_test_mode("s1".to_string(), mode);

        let output = engine.process_key(&make_key("x", "KeyX"));
        assert_eq!(output.key_result, KeyResult::Incorrect);
        assert_eq!(output.caret_pos, 0);
    }

    #[test]
    fn incorrect_key_after_progress_is_reported_as_incorrect() {
        let mut engine = CoreEngine::new();
        let mode = Box::new(TimeMode::new("hello".to_string(), "en".to_string(), 30));
        engine.start_test_mode("s1".to_string(), mode);

        engine.process_key(&make_key("h", "KeyH"));
        let output = engine.process_key(&make_key("x", "KeyX"));
        assert_eq!(output.key_result, KeyResult::Incorrect);
        assert_eq!(output.caret_pos, 1);
    }

    #[test]
    fn backspace_with_mode() {
        let mut engine = CoreEngine::new();
        let mode = Box::new(TimeMode::new("hello".to_string(), "en".to_string(), 30));
        engine.start_test_mode("s1".to_string(), mode);

        engine.process_key(&make_key("h", "KeyH"));
        let output = engine.process_key(&make_key("Backspace", "Backspace"));
        assert_eq!(output.key_result, KeyResult::UndoneCorrect);
        assert_eq!(output.caret_pos, 0);
    }

    #[test]
    fn backspace_at_start_is_noop_and_not_replayed() {
        let mut engine = CoreEngine::new();
        let mode = Box::new(TimeMode::new("hello".to_string(), "en".to_string(), 30));
        engine.start_test_mode("s1".to_string(), mode);

        let output = engine.process_key(&make_key("Backspace", "Backspace"));
        assert_eq!(output.key_result, KeyResult::Noop);
        assert!(engine.replay_frames().is_empty());
    }

    #[test]
    fn full_text_completion_with_mode() {
        let mut engine = CoreEngine::new();
        let mode = Box::new(TimeMode::new("hi".to_string(), "en".to_string(), 30));
        engine.start_test_mode("s1".to_string(), mode);

        engine.process_key(&make_key("h", "KeyH"));
        let output = engine.process_key(&make_key("i", "KeyI"));
        assert!(output.test_complete.is_some());
        let stats = output.test_complete.unwrap();
        assert_eq!(stats.correct_chars, 2);
        assert_eq!(engine.session_state(), SessionState::AwaitingPersistence);
    }

    #[test]
    fn completion_is_emitted_once_after_persistence_succeeds() {
        let mut engine = CoreEngine::new();
        let mode = Box::new(TimeMode::new("a".to_string(), "en".to_string(), 30));
        engine.start_test_mode("s1".to_string(), mode);

        let first = engine.process_key(&make_key("a", "KeyA"));
        assert!(first.test_complete.is_some());
        assert_eq!(first.session_state, SessionState::AwaitingPersistence);
        assert!(engine.begin_persistence());
        assert!(engine.mark_persisted());

        let repeated = engine.process_key(&make_key("a", "KeyA"));
        assert_eq!(repeated.session_state, SessionState::Persisted);
        assert_eq!(repeated.key_result, KeyResult::TestEnded);
        assert!(repeated.test_complete.is_none());
    }

    #[test]
    fn failed_persistence_reuses_the_same_completed_result_for_a_retry() {
        let mut engine = CoreEngine::new();
        let mode = Box::new(TimeMode::new("a".to_string(), "en".to_string(), 30));
        engine.start_test_mode("s1".to_string(), mode);

        let first = engine.process_key(&make_key("a", "KeyA"));
        let first_stats = first.test_complete.expect("completion expected");
        let expected_timestamp = DateTime::parse_from_rfc3339("2026-07-13T00:00:00Z")
            .expect("fixed timestamp should parse")
            .with_timezone(&Utc);
        assert!(engine.set_completion_timestamp(expected_timestamp));
        let completion_timestamp = engine
            .completion_timestamp()
            .expect("completion timestamp should be captured once");
        assert!(engine.begin_persistence());
        assert!(engine.mark_persistence_failed());

        let retry = engine.process_key(&make_key("ignored", "Unidentified"));
        let retry_stats = retry.test_complete.expect("retry completion expected");
        assert_eq!(retry.session_state, SessionState::AwaitingPersistence);
        assert_eq!(retry_stats.correct_chars, first_stats.correct_chars);
        assert_eq!(retry_stats.duration_ms, first_stats.duration_ms);
        assert_eq!(engine.completion_timestamp(), Some(completion_timestamp));
    }

    #[test]
    fn abort_is_rejected_while_persistence_is_in_flight() {
        let mut engine = CoreEngine::new();
        let mode = Box::new(TimeMode::new("a".to_string(), "en".to_string(), 30));
        engine.start_test_mode("s1".to_string(), mode);
        engine.process_key(&make_key("a", "KeyA"));
        assert!(engine.begin_persistence());

        assert!(!engine.abort());
        assert_eq!(engine.session_state(), SessionState::Persisting);
    }

    #[test]
    fn abort_is_rejected_while_a_failed_persistence_is_retryable() {
        let mut engine = CoreEngine::new();
        let mode = Box::new(TimeMode::new("a".to_string(), "en".to_string(), 30));
        engine.start_test_mode("s1".to_string(), mode);
        engine.process_key(&make_key("a", "KeyA"));
        assert!(engine.begin_persistence());
        assert!(engine.mark_persistence_failed());

        assert!(!engine.abort());
        assert_eq!(engine.session_state(), SessionState::AwaitingPersistence);
    }

    #[test]
    fn mode_type_accessible() {
        let mut engine = CoreEngine::new();
        let mode = Box::new(TimeMode::new("hello".to_string(), "en".to_string(), 30));
        engine.start_test_mode("s1".to_string(), mode);

        assert_eq!(engine.current_mode_type(), Some(ModeType::Time));
    }

    #[test]
    fn mode_config_accessible() {
        let mut engine = CoreEngine::new();
        let mode = Box::new(TimeMode::new("hello".to_string(), "en".to_string(), 60));
        engine.start_test_mode("s1".to_string(), mode);

        let config = engine.current_mode_config().unwrap();
        assert_eq!(config["duration"], 60);
        assert_eq!(config["language"], "en");
    }

    #[test]
    fn language_accessible() {
        let mut engine = CoreEngine::new();
        let mode = Box::new(TimeMode::new("привет".to_string(), "ru".to_string(), 30));
        engine.start_test_mode("s1".to_string(), mode);

        assert_eq!(engine.current_language(), Some("ru"));
    }

    #[test]
    fn process_key_without_session_is_noop() {
        let mut engine = CoreEngine::new();
        let output = engine.process_key(&make_key("a", "KeyA"));
        assert_eq!(output.key_result, KeyResult::Noop);
    }

    #[test]
    fn abort_clears_session() {
        let mut engine = CoreEngine::new();
        let mode = Box::new(TimeMode::new("hello".to_string(), "en".to_string(), 30));
        engine.start_test_mode("s1".to_string(), mode);
        assert!(engine.is_active());

        engine.abort();
        assert!(!engine.is_active());
    }

    #[test]
    fn reset_clears_buffer() {
        let mut engine = CoreEngine::new();
        let mode = Box::new(TimeMode::new("hello".to_string(), "en".to_string(), 30));
        engine.start_test_mode("s1".to_string(), mode);
        engine.process_key(&make_key("h", "KeyH"));
        assert_eq!(engine.caret_position(), 1);

        assert!(engine.reset());
        assert_eq!(engine.caret_position(), 0);
    }

    #[test]
    fn start_and_reset_cannot_discard_a_pending_completion() {
        let mut engine = CoreEngine::new();
        assert!(engine
            .start_test_mode(
                "s1".to_string(),
                Box::new(TimeMode::new("a".to_string(), "en".to_string(), 30)),
            )
            .is_ok());
        engine.process_key(&make_key("a", "KeyA"));
        assert_eq!(engine.session_state(), SessionState::AwaitingPersistence);

        assert!(matches!(
            engine.start_test_mode(
                "s2".to_string(),
                Box::new(TimeMode::new("b".to_string(), "en".to_string(), 30)),
            ),
            Err(SessionState::AwaitingPersistence)
        ));
        assert!(!engine.reset());
        assert_eq!(engine.session_state(), SessionState::AwaitingPersistence);

        assert!(engine.begin_persistence());
        assert!(matches!(
            engine.start_test_mode(
                "s2".to_string(),
                Box::new(TimeMode::new("b".to_string(), "en".to_string(), 30)),
            ),
            Err(SessionState::Persisting)
        ));
        assert!(!engine.reset());
    }

    #[test]
    fn persisted_session_can_be_replaced() {
        let mut engine = CoreEngine::new();
        assert!(engine
            .start_test_mode(
                "s1".to_string(),
                Box::new(TimeMode::new("a".to_string(), "en".to_string(), 30)),
            )
            .is_ok());
        engine.process_key(&make_key("a", "KeyA"));
        assert!(engine.begin_persistence());
        assert!(engine.mark_persisted());

        let replacement = engine
            .start_test_mode(
                "s2".to_string(),
                Box::new(TimeMode::new("b".to_string(), "en".to_string(), 30)),
            )
            .expect("persisted session may be replaced");
        assert_eq!(replacement.session_id, "s2");
        assert_eq!(engine.session_state(), SessionState::Running);
    }

    #[test]
    fn ignore_modifier_keys() {
        let mut engine = CoreEngine::new();
        let mode = Box::new(TimeMode::new("hello".to_string(), "en".to_string(), 30));
        engine.start_test_mode("s1".to_string(), mode);

        let output = engine.process_key(&make_key("Shift", "ShiftLeft"));
        assert_eq!(output.key_result, KeyResult::Noop);
    }

    #[test]
    fn test_session_info_has_mode_info() {
        let mut engine = CoreEngine::new();
        let mode = Box::new(TimeMode::new("test".to_string(), "en".to_string(), 15));
        let info = engine
            .start_test_mode("s1".to_string(), mode)
            .expect("idle engine should accept a session");

        assert_eq!(info.mode_type, "time");
        assert_eq!(info.mode_config["duration"], 15);
        assert_eq!(info.language, "en");
    }

    #[test]
    fn session_identity_is_immutable_through_completion_retry_and_replacement() {
        let mut engine = CoreEngine::new();
        let first = engine
            .start_test_mode(
                racoon_domain::SessionId::new(),
                Box::new(TimeMode::new("a".to_string(), "en".to_string(), 30)),
            )
            .unwrap();
        let first_id = engine.current_session_id().cloned().unwrap();
        assert_eq!(first.session_id, first_id.to_string());

        engine.process_key(&make_key("a", "KeyA"));
        assert_eq!(engine.current_session_id(), Some(&first_id));
        assert!(engine.begin_persistence());
        assert!(engine.mark_persistence_failed());
        assert_eq!(engine.current_session_id(), Some(&first_id));
        assert!(!engine.mark_persistence_failed());
        assert!(!engine.abort());

        assert!(engine.begin_persistence());
        assert!(engine.mark_persisted());
        let replacement = engine
            .start_test_mode(
                racoon_domain::SessionId::new(),
                Box::new(TimeMode::new("b".to_string(), "en".to_string(), 30)),
            )
            .unwrap();
        assert_ne!(replacement.session_id, first.session_id);
    }

    #[test]
    fn process_key_collects_authoritative_replay_frames() {
        let mut engine = CoreEngine::new();
        let mode = Box::new(TimeMode::new("hi".to_string(), "en".to_string(), 30));
        engine.start_test_mode("s1".to_string(), mode);

        engine.process_key(&KeyEvent {
            key: "h".to_string(),
            code: "KeyH".to_string(),
            timestamp: 1_000,
        });
        engine.process_key(&KeyEvent {
            key: "x".to_string(),
            code: "KeyX".to_string(),
            timestamp: 1_250,
        });

        assert_eq!(engine.replay_frames().len(), 2);
        assert_eq!(engine.replay_frames()[0].timestamp_ms, 0);
        assert_eq!(engine.replay_frames()[0].caret_pos, 1);
        assert_eq!(engine.replay_frames()[0].char_status, CharStatus::Correct);
        assert_eq!(engine.replay_frames()[1].timestamp_ms, 250);
        assert_eq!(engine.replay_frames()[1].key, "x");
        assert_eq!(engine.replay_frames()[1].char_status, CharStatus::Incorrect);
    }

    #[test]
    fn ignored_keys_are_not_replay_frames() {
        let mut engine = CoreEngine::new();
        let mode = Box::new(TimeMode::new("hi".to_string(), "en".to_string(), 30));
        engine.start_test_mode("s1".to_string(), mode);
        engine.process_key(&make_key("Shift", "ShiftLeft"));
        assert!(engine.replay_frames().is_empty());
    }
}
