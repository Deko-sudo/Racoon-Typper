//! Core Engine — синхронная архитектура.
//! Sprint 2: Input, Typing, CoreEngine, EngineOutput.
//! Sprint 3: StatisticsEngine, WPM, Accuracy, Heatmap.

pub mod engine;
pub mod input;
pub mod stats;
pub mod typing;

pub use engine::{CoreEngine, TestSession, TestSessionInfo};
pub use input::{KeyAction, KeyClassifier, KeyEvent};
pub use stats::{AccuracyCalculator, HeatmapBuilder, LiveTracker, StatisticsEngine, WpmCalculator};
pub use typing::{TextBuffer, TypingResult};
