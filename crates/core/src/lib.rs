//! Core Engine — синхронная архитектура.
//! Sprint 2: Input, Typing, CoreEngine, EngineOutput.
//! Sprint 3: StatisticsEngine, WPM, Accuracy, Heatmap.
//! Sprint 5+: TestMode trait, TimeMode, WordsMode (stub), QuoteMode (stub), CustomMode.
//! Sprint 9: Lesson Engine, LessonMode, LessonSession.
//! Sprint 10: AdaptiveTextGenerator, WeakKeysAnalyzer, Lesson Progression.

pub mod adaptive;
pub mod analytics;
pub mod burst;
pub mod consistency;
pub mod engine;
pub mod finger_map;
pub mod input;
pub mod lesson;
pub mod modes;
pub mod stats;
pub mod streaks;
pub mod typing;
pub mod viewport;
pub mod weak_keys;

pub use adaptive::{AdaptiveTextGenerator, FrequencyAdaptiveGenerator, WeakChar};
pub use engine::{CoreEngine, TestSession, TestSessionInfo};
pub use finger_map::{
    finger_for_char, finger_for_key_jcuken, finger_for_key_qwerty, is_home_row, Finger,
};
pub use input::{KeyAction, KeyClassifier, KeyEvent};
pub use lesson::{
    unlock_next_lesson, LessonMode, LessonResult, LessonSession, LessonState, NextKeyInfo,
    RepeatRecommendation,
};
pub use modes::{CustomMode, ModeResult, ModeType, QuoteMode, TestMode, TimeMode, WordsMode};
pub use racoon_domain::KeyResult;
pub use stats::{AccuracyCalculator, HeatmapBuilder, LiveTracker, StatisticsEngine, WpmCalculator};
pub use streaks::{StreakEngine, StreakInfo};
pub use typing::{TextBuffer, TypingResult};
pub use weak_keys::{WeakKey, WeakKeysAnalyzer, WeakKeysReport};
