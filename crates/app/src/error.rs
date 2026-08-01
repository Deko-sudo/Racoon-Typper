//! AppError — типизированная ошибка приложения.
//! Используется всеми IPC командами вместо String.

use std::fmt;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "code", content = "message")]
#[allow(dead_code)]
pub enum AppError {
    #[serde(rename = "SETTINGS_READ")]
    SettingsRead(String),
    #[serde(rename = "SETTINGS_WRITE")]
    SettingsWrite(String),
    #[serde(rename = "SETTINGS_PARSE")]
    SettingsParse(String),
    #[serde(rename = "SETTINGS_INVALID")]
    SettingsInvalid(String),
    #[serde(rename = "TEST_ALREADY_ACTIVE")]
    TestAlreadyActive,
    #[serde(rename = "SESSION_NOT_FOUND")]
    SessionNotFound(String),
    #[serde(rename = "TEST_NOT_ACTIVE")]
    TestNotActive,
    #[serde(rename = "SESSION_FINALIZING")]
    SessionFinalizing,
    #[serde(rename = "STATE_UNAVAILABLE")]
    StateUnavailable,
    #[serde(rename = "RECOVERY_NOT_STARTED")]
    RecoveryNotStarted,
    #[serde(rename = "RECOVERY_IN_PROGRESS")]
    RecoveryInProgress,
    #[serde(rename = "RECOVERY_BLOCKED")]
    RecoveryBlocked,
    #[serde(rename = "INVALID_MODE")]
    InvalidMode(String),
    #[serde(rename = "INVALID_CONFIG")]
    InvalidConfig(String),
    #[serde(rename = "INVALID_KEY")]
    InvalidKey,
    #[serde(rename = "WORDS_EMPTY")]
    WordsEmpty(String),
    #[serde(rename = "QUOTE_NOT_FOUND")]
    QuoteNotFound(i64),
    #[serde(rename = "CUSTOM_TEXT_EMPTY")]
    CustomTextEmpty,
    #[serde(rename = "CUSTOM_TEXT_NOT_FOUND")]
    CustomTextNotFound(i64),
    #[serde(rename = "DB_QUERY")]
    DbQuery(String),
    #[serde(rename = "DB_WRITE")]
    DbWrite(String),
    #[serde(rename = "DB_CONNECTION")]
    DbConnection(String),
    #[serde(rename = "THEME_NOT_FOUND")]
    ThemeNotFound(String),
    #[serde(rename = "RESOURCE_NOT_FOUND")]
    ResourceNotFound(String),
    #[serde(rename = "INTERNAL")]
    Internal(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::SettingsRead(msg) => write!(f, "Failed to read settings.toml: {}", msg),
            AppError::SettingsWrite(msg) => write!(f, "Failed to write settings.toml: {}", msg),
            AppError::SettingsParse(msg) => write!(f, "Failed to parse settings.toml: {}", msg),
            AppError::SettingsInvalid(msg) => write!(f, "Invalid setting: {}", msg),
            AppError::TestAlreadyActive => write!(f, "A test is already running"),
            AppError::SessionNotFound(id) => write!(f, "Session not found: {}", id),
            AppError::TestNotActive => write!(f, "No active test"),
            AppError::SessionFinalizing => write!(f, "The completed test is still being saved"),
            AppError::StateUnavailable => {
                write!(f, "Application state is unavailable; restart Racoon Typper")
            }
            AppError::RecoveryNotStarted => write!(f, "Startup recovery has not started"),
            AppError::RecoveryInProgress => write!(f, "Startup recovery is in progress"),
            AppError::RecoveryBlocked => write!(f, "Startup recovery is blocked"),
            AppError::InvalidMode(mode) => write!(f, "Unknown mode: {}", mode),
            AppError::InvalidConfig(msg) => write!(f, "Invalid config: {}", msg),
            AppError::InvalidKey => write!(f, "Invalid key event"),
            AppError::WordsEmpty(lang) => write!(f, "Word pack is empty for language: {}", lang),
            AppError::QuoteNotFound(id) => write!(f, "Quote not found: id={}", id),
            AppError::CustomTextEmpty => write!(f, "Custom text is empty"),
            AppError::CustomTextNotFound(id) => write!(f, "Custom text not found: id={}", id),
            AppError::DbQuery(msg) => write!(f, "DB query error: {}", msg),
            AppError::DbWrite(msg) => write!(f, "DB write error: {}", msg),
            AppError::DbConnection(msg) => write!(f, "DB connection error: {}", msg),
            AppError::ThemeNotFound(name) => write!(f, "Theme not found: {}", name),
            AppError::ResourceNotFound(name) => write!(f, "Resource not found: {}", name),
            AppError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Internal(format!("JSON: {}", e))
    }
}

impl From<toml::de::Error> for AppError {
    fn from(e: toml::de::Error) -> Self {
        AppError::SettingsParse(e.to_string())
    }
}

impl From<toml::ser::Error> for AppError {
    fn from(e: toml::ser::Error) -> Self {
        AppError::SettingsWrite(e.to_string())
    }
}

impl From<std::sync::PoisonError<std::sync::MutexGuard<'_, racoon_data::Database>>> for AppError {
    fn from(_: std::sync::PoisonError<std::sync::MutexGuard<'_, racoon_data::Database>>) -> Self {
        AppError::StateUnavailable
    }
}

impl From<std::sync::PoisonError<std::sync::MutexGuard<'_, racoon_core::CoreEngine>>> for AppError {
    fn from(_: std::sync::PoisonError<std::sync::MutexGuard<'_, racoon_core::CoreEngine>>) -> Self {
        AppError::StateUnavailable
    }
}

impl From<racoon_data::DbError> for AppError {
    fn from(e: racoon_data::DbError) -> Self {
        match e {
            racoon_data::DbError::Connection(msg) => AppError::DbConnection(msg),
            racoon_data::DbError::Query(msg) => AppError::DbQuery(msg),
            racoon_data::DbError::Write(msg) => AppError::DbWrite(msg),
            racoon_data::DbError::Transaction(msg) => AppError::DbWrite(msg),
            racoon_data::DbError::Sqlite { .. } => {
                AppError::DbWrite("SQLite operation failed".to_string())
            }
            racoon_data::DbError::Integrity(msg) => AppError::DbWrite(msg),
            racoon_data::DbError::Validation(msg) => AppError::InvalidConfig(msg),
            racoon_data::DbError::LockPoisoned => AppError::StateUnavailable,
            racoon_data::DbError::NotFound(msg) => AppError::ResourceNotFound(msg),
            racoon_data::DbError::Migration(msg) => AppError::DbConnection(msg),
        }
    }
}

impl From<racoon_application::SessionLifecycleError> for AppError {
    fn from(error: racoon_application::SessionLifecycleError) -> Self {
        match error {
            racoon_application::SessionLifecycleError::AlreadyActive => AppError::TestAlreadyActive,
            racoon_application::SessionLifecycleError::Finalizing => AppError::SessionFinalizing,
            racoon_application::SessionLifecycleError::InvalidTransition => AppError::Internal(
                "session start was rejected from an allowed lifecycle state".to_string(),
            ),
        }
    }
}

impl From<racoon_application::SessionStartError<AppError>> for AppError {
    fn from(error: racoon_application::SessionStartError<AppError>) -> Self {
        match error {
            racoon_application::SessionStartError::Mode(error) => error,
            racoon_application::SessionStartError::Lifecycle(error) => error.into(),
        }
    }
}

impl From<racoon_application::SessionProcessError<AppError>> for AppError {
    fn from(error: racoon_application::SessionProcessError<AppError>) -> Self {
        match error {
            racoon_application::SessionProcessError::SessionNotFound(id) => {
                AppError::SessionNotFound(id.to_string())
            }
            racoon_application::SessionProcessError::StateUnavailable => AppError::StateUnavailable,
            racoon_application::SessionProcessError::Finalizing => AppError::SessionFinalizing,
            racoon_application::SessionProcessError::Persistence(error) => error,
            racoon_application::SessionProcessError::InvalidCompletion(message) => {
                AppError::Internal(message.to_string())
            }
        }
    }
}

impl From<racoon_application::SessionAbortError> for AppError {
    fn from(error: racoon_application::SessionAbortError) -> Self {
        match error {
            racoon_application::SessionAbortError::SessionNotFound(id) => {
                AppError::SessionNotFound(id.to_string())
            }
            racoon_application::SessionAbortError::Finalizing => AppError::SessionFinalizing,
        }
    }
}
