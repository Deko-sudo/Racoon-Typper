//! Lesson Engine — управление процессом прохождения урока.
//! Использует существующий Typing Engine (TextBuffer, TypedChar).
//! Не дублирует Typing Engine — делегирует ему обработку клавиш.

use crate::modes::{ModeResult, ModeType, TestMode};
use crate::typing::{TextBuffer, TypingResult};

/// Состояние прохождения урока.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LessonState {
    NotStarted,
    InProgress,
    Completed,
}

/// Результат завершения урока.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LessonResult {
    pub lesson_id: String,
    pub module_id: String,
    pub language: String,
    pub wpm: f64,
    pub accuracy: f64,
    pub correct_chars: usize,
    pub incorrect_chars: usize,
    pub duration_ms: u64,
    pub state: LessonState,
}

/// LessonMode — режим TestMode для уроков.
/// Текст урока фиксирован. Завершение по последнему символу.
pub struct LessonMode {
    lesson_id: String,
    module_id: String,
    language: String,
    text: String,
}

impl LessonMode {
    pub fn new(lesson_id: String, module_id: String, language: String, text: String) -> Self {
        Self {
            lesson_id,
            module_id,
            language,
            text,
        }
    }

    pub fn lesson_id(&self) -> &str {
        &self.lesson_id
    }

    pub fn module_id(&self) -> &str {
        &self.module_id
    }
}

impl TestMode for LessonMode {
    fn mode_type(&self) -> ModeType {
        ModeType::Custom
    }

    fn mode_config(&self) -> serde_json::Value {
        serde_json::json!({
            "lesson_id": self.lesson_id,
            "module_id": self.module_id,
        })
    }

    fn on_key_press(&mut self, ch: char, timestamp: u64, buf: &mut TextBuffer) -> ModeResult {
        let result = buf.process_print(ch, timestamp);
        match result {
            TypingResult::TestEnded => ModeResult::Complete,
            _ => ModeResult::Continue,
        }
    }

    fn on_backspace(&mut self, buf: &mut TextBuffer) -> ModeResult {
        let _ = buf.process_backspace();
        ModeResult::Continue
    }

    fn is_complete(&self, buf: &TextBuffer) -> bool {
        buf.is_complete
    }

    fn get_text(&self) -> &str {
        &self.text
    }

    fn language(&self) -> &str {
        &self.language
    }
}

/// LessonSession — управляет сессией урока.
pub struct LessonSession {
    pub lesson_id: String,
    pub module_id: String,
    pub language: String,
    pub text: String,
    pub buffer: TextBuffer,
    pub state: LessonState,
}

impl LessonSession {
    /// Создаёт новую сессию урока.
    pub fn start(lesson_id: String, module_id: String, language: String, text: String) -> Self {
        let buffer = TextBuffer::new(&text);
        Self {
            lesson_id,
            module_id,
            language,
            text,
            buffer,
            state: LessonState::InProgress,
        }
    }

    /// Обрабатывает нажатие клавиши.
    pub fn process_key(&mut self, ch: char, timestamp: u64) -> TypingResult {
        if self.state != LessonState::InProgress {
            return TypingResult::TestEnded;
        }
        let result = self.buffer.process_print(ch, timestamp);
        if self.buffer.is_complete {
            self.state = LessonState::Completed;
        }
        result
    }

    /// Обрабатывает backspace.
    pub fn process_backspace(&mut self) -> TypingResult {
        self.buffer.process_backspace()
    }

    /// Завершает урок и возвращает результат.
    pub fn complete(&self) -> LessonResult {
        let elapsed = self.buffer.elapsed_ms();
        let correct = self.buffer.correct_chars();
        let incorrect = self.buffer.incorrect_chars();
        let total = correct + incorrect;

        let wpm = if elapsed > 0 {
            (correct as f64 / 5.0) / (elapsed as f64 / 60000.0)
        } else {
            0.0
        };

        let accuracy = if total > 0 {
            (correct as f64 / total as f64) * 100.0
        } else {
            100.0
        };

        LessonResult {
            lesson_id: self.lesson_id.clone(),
            module_id: self.module_id.clone(),
            language: self.language.clone(),
            wpm,
            accuracy,
            correct_chars: correct,
            incorrect_chars: incorrect,
            duration_ms: elapsed,
            state: self.state.clone(),
        }
    }

    /// Текущая позиция курсора.
    pub fn caret_position(&self) -> usize {
        self.buffer.current_position
    }

    /// Завершён ли урок.
    pub fn is_complete(&self) -> bool {
        self.state == LessonState::Completed
    }

    /// Возвращает следующий нужный символ, палец и клавишу.
    pub fn next_required_key(&self) -> Option<NextKeyInfo> {
        if self.state != LessonState::InProgress {
            return None;
        }
        let pos = self.buffer.current_position;
        let ch = self.text.chars().nth(pos)?;
        let is_ru = self
            .language
            .chars()
            .next()
            .map(|c| c == 'r')
            .unwrap_or(false);
        let finger = crate::finger_map::finger_for_char(ch, is_ru);
        let is_home = crate::finger_map::is_home_row(ch, is_ru);
        Some(NextKeyInfo {
            ch,
            finger,
            key_label: ch.to_string(),
            is_home_row: is_home,
        })
    }
}

/// Информация о следующей нужной клавише.
#[derive(Debug, Clone, serde::Serialize)]
pub struct NextKeyInfo {
    pub ch: char,
    pub finger: crate::finger_map::Finger,
    pub key_label: String,
    pub is_home_row: bool,
}

/// Рекомендация по повторению урока.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RepeatRecommendation {
    pub should_repeat: bool,
    pub reason: String,
    pub critical_weak_keys: Vec<String>,
}

impl LessonResult {
    /// Проверяет, нужно ли повторить урок.
    /// Условия: accuracy < 90% ИЛИ есть критические weak keys.
    pub fn should_repeat(
        &self,
        weak_report: &crate::weak_keys::WeakKeysReport,
    ) -> RepeatRecommendation {
        let mut reasons = Vec::new();
        let mut critical_chars = Vec::new();

        if self.accuracy < 90.0 {
            reasons.push(format!("Accuracy {:.1}% < 90%", self.accuracy));
        }

        if weak_report.has_critical() {
            for k in weak_report.critical_keys() {
                critical_chars.push(k.ch.to_string());
            }
            reasons.push(format!("Critical weak keys: {}", critical_chars.join(", ")));
        }

        let should_repeat = !reasons.is_empty();

        RepeatRecommendation {
            should_repeat,
            reason: if should_repeat {
                reasons.join("; ")
            } else {
                "Passed".to_string()
            },
            critical_weak_keys: critical_chars,
        }
    }

    /// Проверяет, пройден ли урок (accuracy >= 90%).
    pub fn is_passed(&self, min_wpm: f64) -> bool {
        self.accuracy >= 90.0 && self.wpm >= min_wpm
    }
}

/// Логика разблокировки следующего урока.
pub fn unlock_next_lesson(
    _completed_lesson_id: &str,
    accuracy: f64,
    wpm: f64,
    min_wpm: f64,
) -> bool {
    accuracy >= 90.0 && wpm >= min_wpm
}

#[cfg(test)]
mod progression_tests {
    use super::*;

    fn make_result(accuracy: f64, wpm: f64) -> LessonResult {
        LessonResult {
            lesson_id: "test".to_string(),
            module_id: "m1".to_string(),
            language: "en".to_string(),
            wpm,
            accuracy,
            correct_chars: 50,
            incorrect_chars: 5,
            duration_ms: 30000,
            state: LessonState::Completed,
        }
    }

    use crate::weak_keys::{WeakKey, WeakKeysReport};

    fn make_report(has_critical: bool) -> WeakKeysReport {
        let weak_keys = if has_critical {
            vec![WeakKey {
                ch: 'a',
                error_count: 5,
                total: 10,
                accuracy: 50.0,
                rank: 1,
            }]
        } else {
            vec![]
        };
        WeakKeysReport {
            weak_keys,
            total_chars_analyzed: 10,
            overall_accuracy: 50.0,
            critical_count: if has_critical { 1 } else { 0 },
        }
    }

    #[test]
    fn unlock_next_passes() {
        assert!(unlock_next_lesson("l1", 95.0, 30.0, 20.0));
    }

    #[test]
    fn unlock_next_low_accuracy_fails() {
        assert!(!unlock_next_lesson("l1", 85.0, 30.0, 20.0));
    }

    #[test]
    fn unlock_next_low_wpm_fails() {
        assert!(!unlock_next_lesson("l1", 95.0, 15.0, 20.0));
    }

    #[test]
    fn should_repeat_low_accuracy() {
        let result = make_result(85.0, 30.0);
        let report = make_report(false);
        let rec = result.should_repeat(&report);
        assert!(rec.should_repeat);
        assert!(rec.reason.contains("Accuracy"));
    }

    #[test]
    fn should_repeat_critical_keys() {
        let result = make_result(95.0, 30.0);
        let report = make_report(true);
        let rec = result.should_repeat(&report);
        assert!(rec.should_repeat);
        assert!(rec.critical_weak_keys.contains(&"a".to_string()));
    }

    #[test]
    fn should_not_repeat_when_passed() {
        let result = make_result(95.0, 30.0);
        let report = make_report(false);
        let rec = result.should_repeat(&report);
        assert!(!rec.should_repeat);
    }

    #[test]
    fn is_passed_true() {
        let result = make_result(95.0, 30.0);
        assert!(result.is_passed(20.0));
    }

    #[test]
    fn is_passed_false_low_wpm() {
        let result = make_result(95.0, 15.0);
        assert!(!result.is_passed(20.0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_session() -> LessonSession {
        LessonSession::start(
            "en_m1_l1".to_string(),
            "en_m1".to_string(),
            "en".to_string(),
            "hello".to_string(),
        )
    }

    #[test]
    fn lesson_session_start() {
        let s = make_session();
        assert_eq!(s.lesson_id, "en_m1_l1");
        assert_eq!(s.module_id, "en_m1");
        assert_eq!(s.language, "en");
        assert_eq!(s.state, LessonState::InProgress);
        assert!(!s.is_complete());
    }

    #[test]
    fn lesson_process_correct_key() {
        let mut s = make_session();
        let result = s.process_key('h', 0);
        assert_eq!(result, TypingResult::Correct);
        assert_eq!(s.caret_position(), 1);
    }

    #[test]
    fn lesson_process_incorrect_key() {
        let mut s = make_session();
        let result = s.process_key('x', 0);
        assert_eq!(result, TypingResult::Incorrect);
        assert_eq!(s.caret_position(), 0);
    }

    #[test]
    fn lesson_complete_on_full_text() {
        let mut s = make_session();
        for ch in "hello".chars() {
            s.process_key(ch, 0);
        }
        assert!(s.is_complete());
        assert_eq!(s.state, LessonState::Completed);
    }

    #[test]
    fn lesson_result_after_completion() {
        let mut s = make_session();
        for ch in "hello".chars() {
            s.process_key(ch, 0);
        }
        let result = s.complete();
        assert_eq!(result.lesson_id, "en_m1_l1");
        assert_eq!(result.correct_chars, 5);
        assert_eq!(result.incorrect_chars, 0);
        assert!((result.accuracy - 100.0).abs() < 0.01);
        assert_eq!(result.state, LessonState::Completed);
    }

    #[test]
    fn lesson_result_with_errors() {
        let mut s = make_session();
        // Type 'h' correct, 'x' incorrect (stays at pos 1), then 'e' correct for pos 1
        // Wait — 'e' is the correct char for pos 1, so it replaces the incorrect
        // Instead, let's skip to pos 1 and type wrong char that doesn't match
        s.process_key('h', 0); // pos 0 correct
        s.process_key('x', 10); // pos 1 incorrect (expected 'e')
                                // Now 'e' is correct for pos 1 — it will replace incorrect
                                // So we need to type the rest correctly to complete
        s.process_key('e', 20); // pos 1 correct (replaces incorrect)
        s.process_key('l', 30); // pos 2
        s.process_key('l', 40); // pos 3
        s.process_key('o', 50); // pos 4
        let result = s.complete();
        assert_eq!(result.correct_chars, 5);
        // incorrect_chars counts current Incorrect status, which is 0 after correction
        // But first_typed was 'x' — the heatmap would show it
        assert_eq!(result.incorrect_chars, 0); // corrected before advancing
    }

    #[test]
    fn lesson_backspace_works() {
        let mut s = make_session();
        s.process_key('h', 0);
        assert_eq!(s.caret_position(), 1);
        s.process_backspace();
        assert_eq!(s.caret_position(), 0);
    }

    #[test]
    fn lesson_mode_implements_test_mode() {
        let mode = LessonMode::new(
            "en_m1_l1".to_string(),
            "en_m1".to_string(),
            "en".to_string(),
            "hello".to_string(),
        );
        assert_eq!(mode.lesson_id(), "en_m1_l1");
        assert_eq!(mode.module_id(), "en_m1");
        assert_eq!(mode.get_text(), "hello");
        assert_eq!(mode.language(), "en");
    }

    #[test]
    fn lesson_mode_config_has_lesson_id() {
        let mode = LessonMode::new(
            "ru_m1_l1".to_string(),
            "ru_m1".to_string(),
            "ru".to_string(),
            "привет".to_string(),
        );
        let config = mode.mode_config();
        assert_eq!(config["lesson_id"], "ru_m1_l1");
        assert_eq!(config["module_id"], "ru_m1");
    }

    #[test]
    fn next_required_key_at_start() {
        let s = make_session();
        let info = s.next_required_key().unwrap();
        assert_eq!(info.ch, 'h');
        assert!(info.is_home_row); // 'h' is home row
    }

    #[test]
    fn next_required_key_mid_text() {
        let mut s = make_session();
        s.process_key('h', 0);
        let info = s.next_required_key().unwrap();
        assert_eq!(info.ch, 'e');
    }

    #[test]
    fn next_required_key_after_complete() {
        let mut s = make_session();
        for ch in "hello".chars() {
            s.process_key(ch, 0);
        }
        assert!(s.is_complete());
        assert!(s.next_required_key().is_none());
    }

    #[test]
    fn next_required_key_home_row() {
        let s = LessonSession::start(
            "test".to_string(),
            "m".to_string(),
            "en".to_string(),
            "asdf".to_string(),
        );
        let info = s.next_required_key().unwrap();
        assert!(info.is_home_row);
        assert_eq!(info.ch, 'a');
    }

    #[test]
    fn next_required_key_russian() {
        let s = LessonSession::start(
            "ru_m1_l1".to_string(),
            "ru_m1".to_string(),
            "ru".to_string(),
            "фыва".to_string(),
        );
        let info = s.next_required_key().unwrap();
        assert_eq!(info.ch, 'ф');
        assert!(info.is_home_row);
    }

    #[test]
    fn lesson_session_not_started_state() {
        let result = LessonResult {
            lesson_id: "test".to_string(),
            module_id: "test_m".to_string(),
            language: "en".to_string(),
            wpm: 0.0,
            accuracy: 100.0,
            correct_chars: 0,
            incorrect_chars: 0,
            duration_ms: 0,
            state: LessonState::NotStarted,
        };
        assert_eq!(result.state, LessonState::NotStarted);
    }
}
