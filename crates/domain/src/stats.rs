//! Типы статистики: LiveStats и FinalStats вынесены в engine.rs.

use serde::{Deserialize, Serialize};

/// Статус символа в TextBuffer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CharStatus {
    #[default]
    Pending,
    Correct,
    Incorrect,
    Backspaced,
}

/// Введённый символ с метаданными.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypedChar {
    pub expected: char,
    pub typed: Option<char>,
    pub status: CharStatus,
    pub first_typed: Option<char>,
    pub first_correct: bool,
    pub timestamp_ms: Option<u64>,
}

impl TypedChar {
    pub fn new(expected: char) -> Self {
        Self {
            expected,
            typed: None,
            status: CharStatus::Pending,
            first_typed: None,
            first_correct: false,
            timestamp_ms: None,
        }
    }
}
