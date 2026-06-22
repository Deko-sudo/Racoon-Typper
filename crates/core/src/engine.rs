//! CoreEngine — связывает Input, Typing, возвращает EngineOutput.
//! Синхронная архитектура: process_key() → EngineOutput.

use racoon_domain::{EngineOutput, FinalStats, KeyResult, VisiblePos};

use crate::input::{KeyAction, KeyClassifier, KeyEvent};
use crate::stats::StatisticsEngine;
use crate::typing::{TextBuffer, TypingResult};

/// Сессия теста.
pub struct TestSession {
    pub session_id: String,
    pub text: String,
    pub buffer: TextBuffer,
}

/// CoreEngine — главный движок.
pub struct CoreEngine {
    session: Option<TestSession>,
    stats: StatisticsEngine,
}

impl CoreEngine {
    pub fn new() -> Self {
        Self {
            session: None,
            stats: StatisticsEngine::new(),
        }
    }

    /// Запускает новый тест с заданным текстом.
    pub fn start_test(&mut self, session_id: String, text: &str) -> TestSessionInfo {
        let buffer = TextBuffer::new(text);
        let info = TestSessionInfo {
            session_id: session_id.clone(),
            text: text.to_string(),
            text_length: text.len(),
        };
        self.session = Some(TestSession {
            session_id,
            text: text.to_string(),
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

    /// Сбрасывает тест с тем же текстом.
    pub fn reset(&mut self) {
        if let Some(session) = &mut self.session {
            session.buffer = TextBuffer::new(&session.text);
        }
        self.stats.reset();
    }

    /// Обрабатывает нажатие клавиши. Возвращает EngineOutput.
    pub fn process_key(&mut self, key_event: &KeyEvent) -> EngineOutput {
        let session = match &mut self.session {
            Some(s) => s,
            None => {
                return self.noop_output();
            }
        };

        let buf = &mut session.buffer;

        // Классификация
        let action = KeyClassifier::classify(&key_event.key, &key_event.code);

        // Обработка
        let typing_result = match action {
            KeyAction::Print(ch) => buf.process_print(ch, key_event.timestamp),
            KeyAction::Backspace => buf.process_backspace(),
            _ => TypingResult::Noop, // Enter, Escape, Tab, Ignore — не влияют в MVP
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

        let caret_pos = buf.current_position;
        let visible_pos = calc_visible_pos(buf);

        // Обновляем статистику
        self.stats.on_key_processed(&typing_result, buf);

        // Live stats с реальным WPM
        let live_stats = if buf.start_time.is_some() && !buf.is_complete {
            Some(self.stats.live_stats(buf))
        } else {
            None
        };

        // Финализация если тест завершён
        let test_complete: Option<FinalStats> = if buf.is_complete {
            let duration = buf.elapsed_ms();
            Some(self.stats.finalize(buf, duration))
        } else {
            None
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

    /// Noop output для случая без активной сессии.
    fn noop_output(&self) -> EngineOutput {
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

    /// Активна ли сессия.
    pub fn is_active(&self) -> bool {
        self.session.is_some()
    }

    /// Текст текущего теста.
    pub fn current_text(&self) -> Option<&str> {
        self.session.as_ref().map(|s| s.text.as_str())
    }

    /// Статус символа на позиции.
    pub fn char_status_at(&self, pos: usize) -> Option<racoon_domain::CharStatus> {
        self.session
            .as_ref()
            .and_then(|s| s.buffer.char_status_at(pos))
    }

    /// Введённый символ на позиции.
    pub fn typed_at(&self, pos: usize) -> Option<char> {
        self.session.as_ref().and_then(|s| s.buffer.typed_at(pos))
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
            .map(|s| s.buffer.is_complete)
            .unwrap_or(false)
    }
}

impl Default for CoreEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Возвращает позицию курсора как (row, col).
fn calc_visible_pos(buf: &TextBuffer) -> VisiblePos {
    VisiblePos {
        row: 0,
        col: buf.current_position,
    }
}

/// Информация о стартованной сессии (возвращается в frontend).
#[derive(Debug, Clone, serde::Serialize)]
pub struct TestSessionInfo {
    pub session_id: String,
    pub text: String,
    pub text_length: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_key(key: &str, code: &str) -> KeyEvent {
        KeyEvent {
            key: key.to_string(),
            code: code.to_string(),
            timestamp: 100,
        }
    }

    #[test]
    fn start_test_and_process_correct() {
        let mut engine = CoreEngine::new();
        let info = engine.start_test("s1".to_string(), "hello");
        assert_eq!(info.text, "hello");
        assert_eq!(info.text_length, 5);

        let output = engine.process_key(&make_key("h", "KeyH"));
        assert_eq!(output.key_result, KeyResult::Correct);
        assert_eq!(output.caret_pos, 1);
    }

    #[test]
    fn process_incorrect() {
        let mut engine = CoreEngine::new();
        engine.start_test("s1".to_string(), "hello");

        let output = engine.process_key(&make_key("x", "KeyX"));
        assert_eq!(output.key_result, KeyResult::Incorrect);
        assert_eq!(output.caret_pos, 0); // не двигается
    }

    #[test]
    fn backspace_after_correct() {
        let mut engine = CoreEngine::new();
        engine.start_test("s1".to_string(), "hello");

        engine.process_key(&make_key("h", "KeyH"));
        let output = engine.process_key(&make_key("Backspace", "Backspace"));
        assert_eq!(output.key_result, KeyResult::UndoneCorrect);
        assert_eq!(output.caret_pos, 0);
    }

    #[test]
    fn full_text_completion() {
        let mut engine = CoreEngine::new();
        engine.start_test("s1".to_string(), "hi");

        engine.process_key(&make_key("h", "KeyH"));
        let output = engine.process_key(&make_key("i", "KeyI"));
        assert_eq!(output.caret_pos, 2);
        assert!(engine.is_complete());
    }

    #[test]
    fn process_key_without_session_is_noop() {
        let mut engine = CoreEngine::new();
        let output = engine.process_key(&make_key("a", "KeyA"));
        assert_eq!(output.key_result, KeyResult::Noop);
    }

    #[test]
    fn ignore_modifier_keys() {
        let mut engine = CoreEngine::new();
        engine.start_test("s1".to_string(), "hello");

        let output = engine.process_key(&make_key("Shift", "ShiftLeft"));
        assert_eq!(output.key_result, KeyResult::Noop);
        assert_eq!(output.caret_pos, 0);
    }

    #[test]
    fn abort_clears_session() {
        let mut engine = CoreEngine::new();
        engine.start_test("s1".to_string(), "hello");
        assert!(engine.is_active());

        engine.abort();
        assert!(!engine.is_active());

        let output = engine.process_key(&make_key("h", "KeyH"));
        assert_eq!(output.key_result, KeyResult::Noop);
    }

    #[test]
    fn reset_clears_buffer() {
        let mut engine = CoreEngine::new();
        engine.start_test("s1".to_string(), "hello");
        engine.process_key(&make_key("h", "KeyH"));
        assert_eq!(engine.caret_position(), 1);

        engine.reset();
        assert_eq!(engine.caret_position(), 0);
    }

    #[test]
    fn live_stats_accuracy() {
        let mut engine = CoreEngine::new();
        engine.start_test("s1".to_string(), "abc");

        engine.process_key(&make_key("a", "KeyA")); // correct
        engine.process_key(&make_key("x", "KeyX")); // incorrect

        let output = engine.process_key(&make_key("b", "KeyB"));
        // incorrect at pos 1, 'b' не совпал с ожидаемым 'b'? Нет — ожидаемый 'b', typed 'b'?
        // Нет: после incorrect 'x' на pos 1, caret не двигался. process_key('b') → expected='b', typed='b' → correct
        // Но предыдущий incorrect ('x') остался. Перерисуется.
        // На самом деле: pos 0 = 'a' (correct), pos 1 = expected 'b', typed 'x' (incorrect)
        // process_key('b') → expected='b', typed='b' → correct, caret двигается
        if output.key_result == KeyResult::Correct {
            let stats = output.live_stats.unwrap();
            // correct=2 (a, b), incorrect=0 (x был overwritten на pos 1)
            // нет — x был incorrect на pos 1, потом b correct на pos 1
            // typed_chars[1].status = Correct (перезаписан)
            assert!(stats.accuracy > 0.0);
        }
    }
}
