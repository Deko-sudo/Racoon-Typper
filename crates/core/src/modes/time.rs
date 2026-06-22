//! TimeMode — режим печати на время (15/30/60/120s).
//! Текст генерируется извне и передаётся в конструктор.
//! Для длительных тестов (120s) текст может быть большим.
//! Таймер: тест завершается когда elapsed >= duration.

use crate::modes::{ModeResult, ModeType, TestMode};
use crate::typing::{TextBuffer, TypingResult};

/// Режим Time — печать на время.
/// text: сгенерированный текст (достаточно длинный для duration).
pub struct TimeMode {
    text: String,
    language: String,
    duration_secs: u64,
    start_ms: Option<u64>,
}

impl TimeMode {
    /// Создаёт TimeMode с заданным текстом и длительностью.
    /// Для 15-30s: 50 слов достаточно.
    /// Для 60s: 100 слов.
    /// Для 120s: 200 слов.
    pub fn new(text: String, language: String, duration_secs: u64) -> Self {
        Self {
            text,
            language,
            duration_secs,
            start_ms: None,
        }
    }

    /// Рекомендуемое количество слов для длительности.
    pub fn recommended_word_count(duration_secs: u64) -> usize {
        match duration_secs {
            15 => 50,
            30 => 50,
            60 => 100,
            120 => 200,
            _ => 50,
        }
    }

    /// Проверяет, истекло ли время.
    fn time_expired(&self, buf: &TextBuffer) -> bool {
        if self.start_ms.is_none() {
            return false;
        }
        let elapsed = buf.elapsed_ms();
        elapsed >= self.duration_secs * 1000
    }
}

impl TestMode for TimeMode {
    fn mode_type(&self) -> ModeType {
        ModeType::Time
    }

    fn mode_config(&self) -> serde_json::Value {
        serde_json::json!({
            "duration": self.duration_secs,
            "language": self.language
        })
    }

    fn on_key_press(&mut self, ch: char, timestamp: u64, buf: &mut TextBuffer) -> ModeResult {
        if self.start_ms.is_none() {
            if let Some(start) = buf.start_time {
                self.start_ms = Some(start.elapsed().as_millis() as u64);
            }
        }

        let result = buf.process_print(ch, timestamp);

        if self.time_expired(buf) {
            return ModeResult::Complete;
        }

        match result {
            TypingResult::TestEnded => ModeResult::Complete,
            _ => ModeResult::Continue,
        }
    }

    fn on_backspace(&mut self, buf: &mut TextBuffer) -> ModeResult {
        let _ = buf.process_backspace();

        if self.time_expired(buf) {
            return ModeResult::Complete;
        }

        ModeResult::Continue
    }

    fn is_complete(&self, buf: &TextBuffer) -> bool {
        self.time_expired(buf) || buf.is_complete
    }

    fn get_text(&self) -> &str {
        &self.text
    }

    fn language(&self) -> &str {
        &self.language
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_mode_basic() {
        let mode = TimeMode::new("hello".to_string(), "en".to_string(), 30);
        assert_eq!(mode.mode_type(), ModeType::Time);
        assert_eq!(mode.get_text(), "hello");
        assert_eq!(mode.language(), "en");
    }

    #[test]
    fn time_mode_config() {
        let mode = TimeMode::new("test".to_string(), "ru".to_string(), 60);
        let config = mode.mode_config();
        assert_eq!(config["duration"], 60);
        assert_eq!(config["language"], "ru");
    }

    #[test]
    fn time_mode_not_complete_before_start() {
        let mode = TimeMode::new("hello".to_string(), "en".to_string(), 30);
        let buf = TextBuffer::new("hello");
        assert!(!mode.is_complete(&buf));
    }

    #[test]
    fn time_mode_complete_on_text_end() {
        let mut mode = TimeMode::new("hi".to_string(), "en".to_string(), 30);
        let mut buf = TextBuffer::new("hi");

        mode.on_key_press('h', 0, &mut buf);
        mode.on_key_press('i', 10, &mut buf);

        assert!(mode.is_complete(&buf));
    }

    #[test]
    fn time_mode_backspace_returns_continue() {
        let mut mode = TimeMode::new("hello".to_string(), "en".to_string(), 30);
        let mut buf = TextBuffer::new("hello");

        mode.on_key_press('h', 0, &mut buf);
        let result = mode.on_backspace(&mut buf);
        assert_eq!(result, ModeResult::Continue);
    }

    #[test]
    fn recommended_word_count_15s() {
        assert_eq!(TimeMode::recommended_word_count(15), 50);
    }

    #[test]
    fn recommended_word_count_30s() {
        assert_eq!(TimeMode::recommended_word_count(30), 50);
    }

    #[test]
    fn recommended_word_count_60s() {
        assert_eq!(TimeMode::recommended_word_count(60), 100);
    }

    #[test]
    fn recommended_word_count_120s() {
        assert_eq!(TimeMode::recommended_word_count(120), 200);
    }

    #[test]
    fn recommended_word_count_default() {
        assert_eq!(TimeMode::recommended_word_count(45), 50);
    }

    #[test]
    fn long_text_120s_not_complete_early() {
        let long_text = "a ".repeat(200);
        let mode = TimeMode::new(long_text.clone(), "en".to_string(), 120);
        let buf = TextBuffer::new(&long_text);
        assert!(!mode.is_complete(&buf));
    }
}
