//! TestMode trait — контракт для режимов теста.
//! CoreEngine работает через этот trait, не зная конкретный режим.

pub mod custom;
pub mod quote;
pub mod time;
pub mod words;

use crate::typing::TextBuffer;

/// Тип режима теста.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModeType {
    Time,
    Words,
    Quote,
    Custom,
}

impl std::fmt::Display for ModeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModeType::Time => write!(f, "time"),
            ModeType::Words => write!(f, "words"),
            ModeType::Quote => write!(f, "quote"),
            ModeType::Custom => write!(f, "custom"),
        }
    }
}

/// Результат обработки нажатия в контексте режима.
#[derive(Debug, Clone, PartialEq)]
pub enum ModeResult {
    /// Продолжить тест.
    Continue,
    /// Тест завершён.
    Complete,
    /// Ошибка режима.
    Failed(String),
}

/// Trait для режима теста.
/// Каждый режим (Time, Words, Quote, Custom) реализует этот trait.
/// CoreEngine работает через dyn TestMode, не зная конкретный режим.
pub trait TestMode: Send + Sync {
    /// Тип режима.
    fn mode_type(&self) -> ModeType;

    /// Конфигурация режима (JSON-сериализуемая).
    fn mode_config(&self) -> serde_json::Value;

    /// Обработать нажатие печатного символа.
    fn on_key_press(&mut self, ch: char, timestamp: u64, buf: &mut TextBuffer) -> ModeResult;

    /// Обработать backspace.
    fn on_backspace(&mut self, buf: &mut TextBuffer) -> ModeResult;

    /// Завершён ли тест.
    fn is_complete(&self, buf: &TextBuffer) -> bool;

    /// Целевой текст для отображения.
    fn get_text(&self) -> &str;

    /// Язык текста.
    fn language(&self) -> &str;
}
