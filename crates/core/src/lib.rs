//! Core Engine — синхронная архитектура.
//! Sprint 2: Input, Typing, CoreEngine, EngineOutput.
//! Sprint 3: StatisticsEngine, WPM, Accuracy, Heatmap.
//! Sprint 5+: TestMode trait, TimeMode, WordsMode (stub), QuoteMode (stub), CustomMode.
//! Sprint 9: Lesson Engine, LessonMode, LessonSession.

pub mod engine;
pub mod input;
pub mod lesson;
pub mod modes;
pub mod stats;
pub mod typing;

pub use engine::{CoreEngine, TestSession, TestSessionInfo};
pub use input::{KeyAction, KeyClassifier, KeyEvent};
pub use lesson::{LessonMode, LessonResult, LessonSession, LessonState};
pub use modes::{CustomMode, ModeResult, ModeType, QuoteMode, TestMode, TimeMode, WordsMode};
pub use racoon_domain::KeyResult;
pub use stats::{AccuracyCalculator, HeatmapBuilder, LiveTracker, StatisticsEngine, WpmCalculator};
pub use typing::{TextBuffer, TypingResult};
