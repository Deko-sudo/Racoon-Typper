//! AppError — типизированная ошибка приложения.
//! Используется всеми IPC командами вместо String.

use std::fmt;

#[derive(Clone)]
#[allow(dead_code)]
pub enum AppError {
    SettingsRead(String),
    SettingsWrite(String),
    SettingsParse(String),
    SettingsInvalid(String),
    TestAlreadyActive,
    SessionNotFound(String),
    TestNotActive,
    SessionFinalizing,
    StateUnavailable,
    RecoveryNotStarted,
    RecoveryInProgress,
    RecoveryBlocked,
    InvalidMode(String),
    InvalidConfig(String),
    InvalidKey,
    WordsEmpty(String),
    QuoteNotFound(i64),
    CustomTextEmpty,
    CustomTextNotFound(i64),
    DbQuery(String),
    DbWrite(String),
    DbConnection(String),
    ThemeNotFound(String),
    ResourceNotFound(String),
    Internal(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.public_message())
    }
}

impl fmt::Debug for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppError")
            .field("code", &self.code())
            .finish()
    }
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("code", self.code())?;
        state.serialize_field("message", self.public_message())?;
        state.end()
    }
}

impl AppError {
    fn code(&self) -> &'static str {
        match self {
            Self::SettingsRead(_) => "SETTINGS_READ",
            Self::SettingsWrite(_) => "SETTINGS_WRITE",
            Self::SettingsParse(_) => "SETTINGS_PARSE",
            Self::SettingsInvalid(_) => "SETTINGS_INVALID",
            Self::TestAlreadyActive => "TEST_ALREADY_ACTIVE",
            Self::SessionNotFound(_) => "SESSION_NOT_FOUND",
            Self::TestNotActive => "TEST_NOT_ACTIVE",
            Self::SessionFinalizing => "SESSION_FINALIZING",
            Self::StateUnavailable => "STATE_UNAVAILABLE",
            Self::RecoveryNotStarted => "RECOVERY_NOT_STARTED",
            Self::RecoveryInProgress => "RECOVERY_IN_PROGRESS",
            Self::RecoveryBlocked => "RECOVERY_BLOCKED",
            Self::InvalidMode(_) => "INVALID_MODE",
            Self::InvalidConfig(_) => "INVALID_CONFIG",
            Self::InvalidKey => "INVALID_KEY",
            Self::WordsEmpty(_) => "WORDS_EMPTY",
            Self::QuoteNotFound(_) => "QUOTE_NOT_FOUND",
            Self::CustomTextEmpty => "CUSTOM_TEXT_EMPTY",
            Self::CustomTextNotFound(_) => "CUSTOM_TEXT_NOT_FOUND",
            Self::DbQuery(_) => "DB_QUERY",
            Self::DbWrite(_) => "DB_WRITE",
            Self::DbConnection(_) => "DB_CONNECTION",
            Self::ThemeNotFound(_) => "THEME_NOT_FOUND",
            Self::ResourceNotFound(_) => "RESOURCE_NOT_FOUND",
            Self::Internal(_) => "INTERNAL",
        }
    }

    fn public_message(&self) -> &'static str {
        match self {
            Self::SettingsRead(_) => "Unable to read settings.",
            Self::SettingsWrite(_) => "Unable to save settings.",
            Self::SettingsParse(_) | Self::SettingsInvalid(_) => "Settings are invalid.",
            Self::TestAlreadyActive => "A test is already running.",
            Self::SessionNotFound(_) => "The requested session was not found.",
            Self::TestNotActive => "No active test.",
            Self::SessionFinalizing => "The completed test is still being saved.",
            Self::StateUnavailable => "Application state is unavailable; restart Racoon Typper.",
            Self::RecoveryNotStarted => "Startup recovery has not started.",
            Self::RecoveryInProgress => "Startup recovery is in progress.",
            Self::RecoveryBlocked => "Startup recovery is blocked.",
            Self::InvalidMode(_) => "The requested mode is invalid.",
            Self::InvalidConfig(_) => "The supplied configuration is invalid.",
            Self::InvalidKey => "The key event is invalid.",
            Self::WordsEmpty(_) => "No words are available for the requested language.",
            Self::QuoteNotFound(_) => "The requested quote was not found.",
            Self::CustomTextEmpty => "Custom text is empty.",
            Self::CustomTextNotFound(_) => "The requested custom text was not found.",
            Self::DbQuery(_) | Self::DbWrite(_) | Self::DbConnection(_) => {
                "The database operation failed."
            }
            Self::ThemeNotFound(_) => "The requested theme was not found.",
            Self::ResourceNotFound(_) => "The requested resource was not found.",
            Self::Internal(_) => "An internal error occurred.",
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
            // A failed backup/restore is surfaced as a DB connection issue so it
            // is visible at the IPC boundary and never silently swallowed. Pre-
            // migration backup failures are warn-and-continue at startup and do
            // not reach this mapping; this arm covers on-demand restore paths.
            racoon_data::DbError::Backup(msg) => AppError::DbConnection(msg),
            racoon_data::DbError::Restore(msg) => AppError::DbConnection(msg),
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

#[cfg(test)]
mod tests {
    use super::AppError;

    #[test]
    fn public_error_outputs_redact_hostile_dynamic_payloads() {
        let hostile_input = "../../typed-secret/profile.json?token=do-not-disclose";
        let errors = [
            AppError::SettingsRead(hostile_input.to_string()),
            AppError::SettingsWrite(hostile_input.to_string()),
            AppError::SettingsParse(hostile_input.to_string()),
            AppError::SettingsInvalid(hostile_input.to_string()),
            AppError::DbWrite(hostile_input.to_string()),
            AppError::DbQuery(hostile_input.to_string()),
            AppError::DbConnection(hostile_input.to_string()),
            AppError::InvalidConfig(hostile_input.to_string()),
            AppError::InvalidMode(hostile_input.to_string()),
            AppError::SessionNotFound(hostile_input.to_string()),
            AppError::WordsEmpty(hostile_input.to_string()),
            AppError::ThemeNotFound(hostile_input.to_string()),
            AppError::ResourceNotFound(hostile_input.to_string()),
            AppError::Internal(hostile_input.to_string()),
        ];

        for error in errors {
            let serialized = serde_json::to_string(&error).expect("serialize app error");
            let displayed = error.to_string();
            let debugged = format!("{error:?}");

            for output in [&serialized, &displayed, &debugged] {
                assert!(
                    !output.contains(hostile_input),
                    "public error output leaked hostile input: {output}"
                );
            }
        }
    }
}
