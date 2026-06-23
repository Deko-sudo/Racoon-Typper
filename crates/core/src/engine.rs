//! CoreEngine — связывает Input, Typing, TestMode, возвращает EngineOutput.
//! Синхронная архитектура: process_key() → EngineOutput.
//! CoreEngine не знает конкретный режим — работает через dyn TestMode.

use racoon_domain::{EngineOutput, FinalStats, KeyResult, VisiblePos};

use crate::input::{KeyAction, KeyClassifier, KeyEvent};
use crate::modes::{ModeResult, ModeType, TestMode};
use crate::stats::StatisticsEngine;
use crate::typing::{TextBuffer, TypingResult};

/// Сессия теста.
pub struct TestSession {
    pub session_id: String,
    pub mode: Box<dyn TestMode>,
    pub buffer: TextBuffer,
}

/// CoreEngine — главный движок.
pub struct CoreEngine {
    session: Option<TestSession>,
    stats: StatisticsEngine,
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
            stats: StatisticsEngine::new(),
        }
    }

    /// Запускает новый тест с заданным режимом.
    pub fn start_test_mode(
        &mut self,
        session_id: String,
        mode: Box<dyn TestMode>,
    ) -> TestSessionInfo {
        let text = mode.get_text().to_string();
        let text_length = text.chars().count();
        let mode_type = mode.mode_type().to_string();
        let mode_config = mode.mode_config();
        let language = mode.language().to_string();

        let buffer = TextBuffer::new(&text);

        let info = TestSessionInfo {
            session_id: session_id.clone(),
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
        self.stats.reset();
        info
    }

    /// Прерывает тест.
    pub fn abort(&mut self) {
        self.session = None;
        self.stats.reset();
    }

    /// Сбрасывает тест с тем же режимом и текстом.
    pub fn reset(&mut self) {
        if let Some(session) = &mut self.session {
            let text = session.mode.get_text().to_string();
            session.buffer = TextBuffer::new(&text);
        }
        self.stats.reset();
    }

    /// Обрабатывает нажатие клавиши. Возвращает EngineOutput.
    pub fn process_key(&mut self, key_event: &KeyEvent) -> EngineOutput {
        let session = match &mut self.session {
            Some(s) => s,
            None => return noop_output(),
        };

        // Классификация
        let action = KeyClassifier::classify(&key_event.key, &key_event.code);

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
                        // Определяем TypingResult по статусу текущей позиции
                        if session.buffer.current_position > 0 {
                            let prev = session.buffer.typed_chars
                                [session.buffer.current_position - 1]
                                .status
                                .clone();
                            match prev {
                                racoon_domain::CharStatus::Correct => TypingResult::Correct,
                                racoon_domain::CharStatus::Incorrect => TypingResult::Incorrect,
                                _ => TypingResult::Noop,
                            }
                        } else {
                            // Позиция не двигалась — incorrect
                            let curr = session
                                .buffer
                                .typed_chars
                                .get(session.buffer.current_position);
                            match curr.map(|tc| tc.status.clone()) {
                                Some(racoon_domain::CharStatus::Incorrect) => {
                                    TypingResult::Incorrect
                                }
                                _ => TypingResult::Noop,
                            }
                        }
                    }
                    ModeResult::Failed(_) => TypingResult::Noop,
                }
            }
            KeyAction::Backspace => {
                let mode_result = session.mode.on_backspace(&mut session.buffer);
                match mode_result {
                    ModeResult::Complete => TypingResult::TestEnded,
                    _ => {
                        // Определяем результат backspace по смене позиции
                        // on_backspace уже вызвал buf.process_backspace() внутри режима
                        // Нужно определить результат — проверяем что caret изменился
                        // К сожалению результат потерян. Нужно вернуть TypingResult из режима.
                        // Пока определяем по статусу:
                        if session.buffer.current_position < session.buffer.typed_chars.len() {
                            let status = session.buffer.typed_chars
                                [session.buffer.current_position]
                                .status
                                .clone();
                            match status {
                                racoon_domain::CharStatus::Pending => TypingResult::UndoneIncorrect,
                                racoon_domain::CharStatus::Backspaced => {
                                    TypingResult::UndoneCorrect
                                }
                                _ => TypingResult::Noop,
                            }
                        } else {
                            TypingResult::Noop
                        }
                    }
                }
            }
            _ => TypingResult::Noop,
        };

        let caret_pos = session.buffer.current_position;
        let visible_pos = calc_visible_pos(caret_pos);
        let is_complete = session.mode.is_complete(&session.buffer);

        // Обновляем статистику
        self.stats.on_key_processed(&typing_result, &session.buffer);

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

    /// Активна ли сессия.
    pub fn is_active(&self) -> bool {
        self.session.is_some()
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
fn noop_output() -> EngineOutput {
    EngineOutput {
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

#[cfg(test)]
mod tests {
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
        let info = engine.start_test_mode("s1".to_string(), mode);

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
    fn full_text_completion_with_mode() {
        let mut engine = CoreEngine::new();
        let mode = Box::new(TimeMode::new("hi".to_string(), "en".to_string(), 30));
        engine.start_test_mode("s1".to_string(), mode);

        engine.process_key(&make_key("h", "KeyH"));
        let output = engine.process_key(&make_key("i", "KeyI"));
        assert!(output.test_complete.is_some());
        let stats = output.test_complete.unwrap();
        assert_eq!(stats.correct_chars, 2);
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

        engine.reset();
        assert_eq!(engine.caret_position(), 0);
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
        let info = engine.start_test_mode("s1".to_string(), mode);

        assert_eq!(info.mode_type, "time");
        assert_eq!(info.mode_config["duration"], 15);
        assert_eq!(info.language, "en");
    }
}
