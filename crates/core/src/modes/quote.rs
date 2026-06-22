//! QuoteMode — stub. Режим печати цитат.

use crate::modes::{ModeResult, ModeType, TestMode};
use crate::typing::{TextBuffer, TypingResult};

pub struct QuoteMode {
    text: String,
    language: String,
    quote_id: Option<i64>,
}

impl QuoteMode {
    pub fn new(text: String, language: String, quote_id: Option<i64>) -> Self {
        Self {
            text,
            language,
            quote_id,
        }
    }
}

impl TestMode for QuoteMode {
    fn mode_type(&self) -> ModeType {
        ModeType::Quote
    }

    fn mode_config(&self) -> serde_json::Value {
        serde_json::json!({
            "quote_id": self.quote_id,
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
