//! Repository traits + SQLite implementations.

pub mod custom_texts;
pub mod daily_stats;
pub mod finalization_ledger;
pub mod lesson;
pub mod personal_bests;
pub mod replays;
pub mod reporting;
pub mod session_finalizer;
pub mod session_ledger;
pub mod settings;
pub mod streaks;
pub mod tests;

pub use custom_texts::{CustomTextRepository, SqliteCustomTextRepository, MAX_CUSTOM_TEXT_LENGTH};
pub use daily_stats::{DailyStats, DailyStatsRepository, SqliteDailyStatsRepository};
pub use finalization_ledger::SqliteFinalizationLedger;
pub use lesson::{LessonProgressRecord, LessonRepository, SqliteLessonRepository};
pub use personal_bests::{PersonalBestsRepository, SqlitePersonalBestsRepository};
pub use replays::{ReplayFrame, ReplayRepository, SqliteReplayRepository};
pub use reporting::{
    SqliteAnalyticsReportingPort, SqliteHistoryReportingPort, SqlitePersonalBestReportingPort,
    SqliteProgressReportingPort,
};
pub use session_finalizer::SqliteSessionFinalizer;
pub use session_ledger::SqliteSessionRecoveryLedger;
pub use settings::{AppSettings, SettingsStore};
pub use streaks::{SqliteStreakRepository, StreakRecord, StreakRepository};
pub use tests::{SqliteTestRepository, TestRepository};
