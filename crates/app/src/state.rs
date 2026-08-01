//! Process-owned application state and lifecycle-safe access to local storage.

use chrono::{DateTime, Utc};
use racoon_application::{RecoveryReadiness, SessionClock, SessionWallClock, StartupRecoveryGate};
use racoon_core::sound::{SoundConfig, SoundEngine, SoundEvent, SoundOutput};
use racoon_data::repository::SettingsStore;
use racoon_data::{Database, DbError};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;

use crate::error::AppError;

/// Состояние приложения, доступное всем Tauri commands.
pub struct AppState {
    pub db: Database,
    settings_path: PathBuf,
    settings_lock: Mutex<()>,
    data_dir: PathBuf,
    config_dir: PathBuf,
    process_started_at: Instant,
    sound_engine: Mutex<SoundEngine>,
    startup_recovery: StartupRecoveryGate,
}

impl AppState {
    pub fn new(
        db: Database,
        settings_path: PathBuf,
        data_dir: PathBuf,
        config_dir: PathBuf,
        startup_recovery: StartupRecoveryGate,
    ) -> Self {
        Self {
            db,
            settings_path,
            settings_lock: Mutex::new(()),
            data_dir,
            config_dir,
            process_started_at: Instant::now(),
            sound_engine: Mutex::new(SoundEngine::new(SoundConfig::default())),
            startup_recovery,
        }
    }

    pub fn settings_store(&self) -> SettingsStore {
        SettingsStore::new(self.settings_path.clone())
    }

    /// Serializes read-modify-write settings operations within this process.
    /// The store itself remains filesystem-backed, while this lock prevents two
    /// IPC commands from loading the same old value and losing an update.
    pub fn with_settings<T>(
        &self,
        operation: impl FnOnce(&SettingsStore) -> Result<T, DbError>,
    ) -> Result<T, DbError> {
        let _guard = self
            .settings_lock
            .lock()
            .map_err(|_| DbError::LockPoisoned)?;
        operation(&self.settings_store())
    }

    /// A process-monotonic timestamp for replay and graph samples. It is not a
    /// wall-clock value and therefore cannot move backward after a system clock
    /// adjustment.
    pub fn monotonic_timestamp_ms(&self) -> u64 {
        self.process_started_at.elapsed().as_millis() as u64
    }

    /// Updates the persistent in-memory sound policy and applies its per-event
    /// cooldown using the same monotonic clock as the typing session.
    pub fn try_play_sound(
        &self,
        config: SoundConfig,
        event: SoundEvent,
    ) -> Result<Option<SoundOutput>, DbError> {
        let mut sound_engine = self
            .sound_engine
            .lock()
            .map_err(|_| DbError::LockPoisoned)?;
        sound_engine.set_config(config);
        Ok(sound_engine.try_play(event, self.monotonic_timestamp_ms()))
    }

    pub fn data_dir(&self) -> &std::path::Path {
        &self.data_dir
    }

    pub fn config_dir(&self) -> &std::path::Path {
        &self.config_dir
    }

    pub fn settings_path(&self) -> &std::path::Path {
        &self.settings_path
    }

    /// Rejects recovery-relevant mutation before it can touch the engine or
    /// durable stores. Read-only reporting remains available while startup is
    /// blocked so users can inspect existing data.
    pub fn require_startup_recovery_ready(&self) -> Result<(), AppError> {
        match self.startup_recovery.state() {
            Ok(RecoveryReadiness::Ready) => Ok(()),
            Ok(RecoveryReadiness::NotStarted) => Err(AppError::RecoveryNotStarted),
            Ok(RecoveryReadiness::Recovering) => Err(AppError::RecoveryInProgress),
            Ok(RecoveryReadiness::Blocked) => Err(AppError::RecoveryBlocked),
            Err(_) => Err(AppError::StateUnavailable),
        }
    }

    pub fn startup_recovery(&self) -> &StartupRecoveryGate {
        &self.startup_recovery
    }
}

impl SessionClock for AppState {
    fn monotonic_timestamp_ms(&self) -> u64 {
        AppState::monotonic_timestamp_ms(self)
    }
}

impl SessionWallClock for AppState {
    fn utc_now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readiness_guard_rejects_mutation_before_startup_recovery_completes() {
        let database = Database::open_in_memory().expect("test database");
        let state = AppState::new(
            database,
            PathBuf::from("settings.toml"),
            PathBuf::from("data"),
            PathBuf::from("config"),
            StartupRecoveryGate::new(),
        );

        assert!(matches!(
            state.require_startup_recovery_ready(),
            Err(AppError::RecoveryNotStarted)
        ));
    }
}
