//! EngineOutput и связанные типы — контракт между Core Engine и app crate.

use serde::{Deserialize, Serialize};

/// Результат обработки нажатия клавиши.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum KeyResult {
    Correct,
    Incorrect,
    UndoneCorrect,
    UndoneIncorrect,
    Noop,
    TestEnded,
}

/// Видимая позиция курсора (row, col).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisiblePos {
    pub row: usize,
    pub col: usize,
}

/// Дельта скролла текста.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrollDelta {
    pub offset: usize,
    pub direction: ScrollDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScrollDirection {
    Down,
    Up,
}

/// Дельта прогресса урока.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LessonDelta {
    pub exercise_complete: bool,
    pub exercise_wpm: f64,
    pub exercise_accuracy: f64,
    pub exercise_errors: u32,
    pub exercise_passed: bool,
    pub lesson_complete: bool,
    pub next_exercise: Option<usize>,
    pub adaptive_repeat: bool,
    pub repeat_reason: Option<String>,
}

/// Обновление визуальной клавиатуры.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardVizUpdate {
    pub highlight_keys: Vec<String>,
    pub next_key: Option<String>,
    pub finger: Option<String>,
    pub hand: Option<String>,
}

/// Живая статистика (обновляется на каждый keystroke).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveStats {
    pub wpm: f64,
    pub raw_wpm: f64,
    pub accuracy: f64,
    pub elapsed_ms: u64,
}

/// EngineOutput — синхронный контракт возврата из core.process_key().
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineOutput {
    pub key_result: KeyResult,
    pub caret_pos: usize,
    pub visible_pos: VisiblePos,
    pub live_stats: Option<LiveStats>,
    pub lesson_delta: Option<LessonDelta>,
    pub test_complete: Option<FinalStats>,
    pub text_scrolled: Option<ScrollDelta>,
    pub keyboard_viz: Option<KeyboardVizUpdate>,
}

/// Финальная статистика теста.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalStats {
    pub wpm: f64,
    pub raw_wpm: f64,
    pub accuracy: f64,
    pub raw_accuracy: f64,
    pub consistency: Option<f64>,
    pub correct_chars: usize,
    pub incorrect_chars: usize,
    pub backspaces: usize,
    pub char_stats: serde_json::Value,
    pub heatmap: serde_json::Value,
    pub graph_data: Option<serde_json::Value>,
    pub duration_ms: u64,
}
