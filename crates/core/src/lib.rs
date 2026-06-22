//! Core Engine — синхронная архитектура.
//! Sprint 2: Input, Typing, CoreEngine, EngineOutput.
//! Sprint 3: StatisticsEngine, WPM, Accuracy, Heatmap.
//! Sprint 5+: TestMode trait, TimeMode, WordsMode (stub), QuoteMode (stub), CustomMode.

pub mod engine;
pub mod input;
pub mod modes;
pub mod stats;
pub mod typing;

pub use engine::{CoreEngine, TestSession, TestSessionInfo};
pub use input::{KeyAction, KeyClassifier, KeyEvent};
pub use modes::{ModeResult, ModeType, TestMode};
pub use racoon_domain::KeyResult;
pub use stats::{AccuracyCalculator, HeatmapBuilder, LiveTracker, StatisticsEngine, WpmCalculator};
pub use typing::{TextBuffer, TypingResult};

// Re-export mode implementations
pub use modes::custom::CustomMode;
pub use modes::quote::QuoteMode;
pub use modes::time::TimeMode;
pub use modes::words::WordsMode;
