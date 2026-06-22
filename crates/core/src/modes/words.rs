//! WordsMode — stub. Режим печати N слов.
//! Текст фиксированной длины. Завершение по последнему символу.

use crate::modes::{ModeResult, ModeType, TestMode};
use crate::typing::{TextBuffer, TypingResult};

pub struct WordsMode {
    text: String,
    language: String,
    word_count: usize,
}

impl WordsMode {
    pub fn new(text: String, language: String, word_count: usize) -> Self {
        Self {
            text,
            language,
            word_count,
        }
    }
}

impl TestMode for WordsMode {
    fn mode_type(&self) -> ModeType {
        ModeType::Words
    }

    fn mode_config(&self) -> serde_json::Value {
        serde_json::json!({
            "word_count": self.word_count,
            "language": self.language
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
