//! Process-owned application state and lifecycle-safe access to local storage.

use chrono::{DateTime, Utc};
use racoon_application::{RecoveryReadiness, SessionClock, SessionWallClock, StartupRecoveryGate};
use racoon_core::sound::{SoundConfig, SoundEngine, SoundEvent, SoundOutput};
use racoon_data::repository::SettingsStore;
use racoon_data::{Database, DbError};
use std::path::PathBuf;
use std::sync::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockError};
use std::time::Instant;

use crate::error::AppError;

/// Состояние приложения, доступное всем Tauri commands.
pub struct AppState {
    pub db: Database,
    settings_path: PathBuf,
    db_path: PathBuf,
    settings_lock: Mutex<()>,
    process_started_at: Instant,
    sound_engine: Mutex<SoundEngine>,
    startup_recovery: StartupRecoveryGate,
    database_recovery: RwLock<Option<StartupRecoveryGate>>,
}

impl AppState {
    pub fn new(
        db: Database,
        settings_path: PathBuf,
        db_path: PathBuf,
        startup_recovery: StartupRecoveryGate,
    ) -> Self {
        Self {
            db,
            settings_path,
            db_path,
            settings_lock: Mutex::new(()),
            process_started_at: Instant::now(),
            sound_engine: Mutex::new(SoundEngine::new(SoundConfig::default())),
            startup_recovery,
            database_recovery: RwLock::new(None),
        }
    }

    /// Path of the live database file, used by whole-file restore.
    pub fn db_path(&self) -> &PathBuf {
        &self.db_path
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

    /// Holds a shared lifecycle lease for the whole mutation, before engine/DB
    /// locks. Restore excludes new mutations; they fail rather than queue stale
    /// IDs for a different database. An in-flight mutation also excludes restore.
    pub fn require_startup_recovery_ready(
        &self,
    ) -> Result<RwLockReadGuard<'_, Option<StartupRecoveryGate>>, AppError> {
        let recovery = self
            .database_recovery
            .try_read()
            .map_err(lifecycle_lock_error)?;
        self.ensure_recovery_ready(recovery.as_ref())?;
        Ok(recovery)
    }

    /// Lock order is lifecycle -> engine -> database, including whole-file restore.
    pub fn begin_database_restore(
        &self,
    ) -> Result<RwLockWriteGuard<'_, Option<StartupRecoveryGate>>, AppError> {
        let recovery = self
            .database_recovery
            .try_write()
            .map_err(lifecycle_lock_error)?;
        self.ensure_recovery_ready(recovery.as_ref())?;
        Ok(recovery)
    }

    fn ensure_recovery_ready(
        &self,
        restored: Option<&StartupRecoveryGate>,
    ) -> Result<(), AppError> {
        match restored.unwrap_or(&self.startup_recovery).state() {
            Ok(RecoveryReadiness::Ready) => Ok(()),
            Ok(RecoveryReadiness::NotStarted) => Err(AppError::RecoveryNotStarted),
            Ok(RecoveryReadiness::Recovering) => Err(AppError::RecoveryInProgress),
            Ok(RecoveryReadiness::Blocked) => Err(AppError::RecoveryBlocked),
            Err(_) => Err(AppError::StateUnavailable),
        }
    }

    /// The caller holds the exclusive lifecycle lease and supplies a fresh gate;
    /// bootstrap's completed gate must never stand in for a restored database.
    pub fn recover_restored_database(&self, gate: &StartupRecoveryGate) -> Result<(), AppError> {
        use racoon_application::{
            StartupRecoveryCoordinator, StartupRecoveryRetryPolicy, StartupRecoveryRunOutcome,
        };
        use racoon_data::{
            SqliteFinalizationLedger, SqliteSessionFinalizer, SqliteSessionRecoveryLedger,
        };
        let recovery = SqliteSessionRecoveryLedger::new(&self.db);
        let finalizations = SqliteFinalizationLedger::new(&self.db);
        let finalizer = SqliteSessionFinalizer::new(&self.db);
        let outcome = StartupRecoveryCoordinator::new(
            &recovery,
            &finalizations,
            &finalizer,
            self,
            &crate::ThreadStartupRecoverySleeper,
            StartupRecoveryRetryPolicy::default(),
        )
        .run(gate)
        .map_err(|_| AppError::StateUnavailable)?;
        match outcome {
            StartupRecoveryRunOutcome::Completed(_)
            | StartupRecoveryRunOutcome::AlreadyCompleted(_) => Ok(()),
            StartupRecoveryRunOutcome::Blocked(_) => Err(AppError::RecoveryBlocked),
            StartupRecoveryRunOutcome::InProgress => Err(AppError::RecoveryInProgress),
        }
    }

    #[cfg(test)]
    pub fn startup_recovery(&self) -> &StartupRecoveryGate {
        &self.startup_recovery
    }
}

fn lifecycle_lock_error<T>(error: TryLockError<T>) -> AppError {
    match error {
        TryLockError::WouldBlock => AppError::RecoveryInProgress,
        TryLockError::Poisoned(_) => AppError::StateUnavailable,
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
            PathBuf::from("unused-db.db"),
            StartupRecoveryGate::new(),
        );

        assert!(matches!(
            state.require_startup_recovery_ready(),
            Err(AppError::RecoveryNotStarted)
        ));
    }

    #[test]
    fn mutation_lease_excludes_restore_and_failed_fresh_recovery_blocks_mutations() {
        let state = AppState::new(
            Database::open_in_memory().unwrap(),
            PathBuf::from("private-fixture-settings.toml"),
            PathBuf::from("private-fixture-data.db"),
            StartupRecoveryGate::new(),
        );
        state
            .recover_restored_database(state.startup_recovery())
            .unwrap();
        let mutation = state.require_startup_recovery_ready().unwrap();
        assert!(matches!(
            state.database_recovery.try_write(),
            Err(std::sync::TryLockError::WouldBlock)
        ));
        assert!(matches!(
            state.begin_database_restore(),
            Err(AppError::RecoveryInProgress)
        ));
        drop(mutation);
        let mut restore = state.begin_database_restore().unwrap();
        assert!(matches!(
            state.require_startup_recovery_ready(),
            Err(AppError::RecoveryInProgress)
        ));
        *restore = Some(StartupRecoveryGate::new());
        state.db.with_connection(|conn| {
            conn.execute_batch(
                "INSERT INTO session_ledger(session_id,state,mode_type,mode_descriptor,language,created_at,updated_at)
                 VALUES ('018f0c2e-7b8d-7abc-8def-0123456789ab','running','custom','{}','en','2026-10-16T00:00:00Z','2026-10-16T00:00:00Z');
                 PRAGMA query_only=ON;",
            )?;
            Ok(())
        }).unwrap();
        let fresh = restore.as_ref().unwrap();
        let error = state.recover_restored_database(fresh).unwrap_err();
        assert!(matches!(fresh.state().unwrap(), RecoveryReadiness::Blocked));
        drop(restore);
        assert!(matches!(
            state.require_startup_recovery_ready(),
            Err(AppError::RecoveryBlocked)
        ));
        assert!(matches!(
            state.begin_database_restore(),
            Err(AppError::RecoveryBlocked)
        ));
        let payload = serde_json::to_string(&error).unwrap();
        assert!(payload.contains("RECOVERY_BLOCKED"));
        assert!(!payload.contains("private-fixture"));
        assert!(!format!("{error:?}").contains("private-fixture"));
    }
}
