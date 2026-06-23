//! Repository traits + SQLite implementations.

pub mod custom_texts;
pub mod daily_stats;
pub mod lesson;
pub mod personal_bests;
pub mod replays;
pub mod settings;
pub mod tests;

pub use custom_texts::{CustomTextRepository, SqliteCustomTextRepository};
pub use daily_stats::{DailyStats, DailyStatsRepository, SqliteDailyStatsRepository};
pub use lesson::{LessonRepository, SqliteLessonRepository};
pub use personal_bests::{PersonalBestsRepository, SqlitePersonalBestsRepository};
pub use replays::{ReplayFrame, ReplayRepository, SqliteReplayRepository};
pub use settings::{AppSettings, SettingsStore};
pub use tests::{SqliteTestRepository, TestRepository};
