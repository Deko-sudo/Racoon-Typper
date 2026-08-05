//! Tauri application entry point.
//! Sprint 5: Settings + Themes + Custom Texts

use chrono::{DateTime, Utc};
use racoon_application::{
    SessionWallClock, StartupRecoveryCoordinator, StartupRecoveryGate, StartupRecoveryRetryPolicy,
    StartupRecoverySleeper,
};
use racoon_core::CoreEngine;
use racoon_data::{
    Database, SqliteFinalizationLedger, SqliteSessionFinalizer, SqliteSessionRecoveryLedger,
};
use racoon_domain::AppInfo;
use std::sync::Mutex;
use tauri::Manager;

mod commands;
mod error;
mod paths;
mod session_service;
mod state;
mod validation;

use state::AppState;

struct SystemRecoveryClock;

impl SessionWallClock for SystemRecoveryClock {
    fn utc_now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

struct ThreadStartupRecoverySleeper;

impl StartupRecoverySleeper for ThreadStartupRecoverySleeper {
    fn sleep(&self, delay: std::time::Duration) {
        if !delay.is_zero() {
            std::thread::sleep(delay);
        }
    }
}

fn main() {
    let application = tauri::Builder::default()
        .setup(|app| {
            let paths = paths::resolve(app)?;
            let data_dir = paths.data_dir.clone();
            let db_path = paths.db_path.clone();
            // Take a rotating pre-migration backup before any schema work. A
            // failure here is warn-and-continue: migrations V005–V008 are
            // additive and low-risk, and a transient permissions/IO error must
            // not brick the application. The backup is defense-in-depth; the
            // operational recovery path remains "restore the most recent
            // snapshot or ship a forward fix".
            let database = Database::open_with_pre_migration(&paths.db_path, |live_path| {
                if let Err(error) = racoon_data::backup::create_pre_migration_backup(
                    live_path,
                    &data_dir,
                    "data",
                    chrono::Utc::now(),
                    racoon_data::backup::DEFAULT_KEEP,
                ) {
                    // Structured logging arrives in Phase 5; until then this
                    // stderr warning is the visible surface. It records only the
                    // backup path and error class, never typed content.
                    eprintln!("warn: pre-migration backup failed for {db_path:?}: {error}");
                }
            })
            .map_err(|error| std::io::Error::other(error.to_string()))?;

            let startup_recovery = StartupRecoveryGate::new();
            let recovery_ledger = SqliteSessionRecoveryLedger::new(&database);
            let finalization_ledger = SqliteFinalizationLedger::new(&database);
            let session_finalizer = SqliteSessionFinalizer::new(&database);
            let recovery_clock = SystemRecoveryClock;
            let recovery_sleeper = ThreadStartupRecoverySleeper;
            let coordinator = StartupRecoveryCoordinator::new(
                &recovery_ledger,
                &finalization_ledger,
                &session_finalizer,
                &recovery_clock,
                &recovery_sleeper,
                StartupRecoveryRetryPolicy::default(),
            );
            coordinator
                .run(&startup_recovery)
                .map_err(|_| std::io::Error::other("startup recovery state unavailable"))?;

            app.manage(Mutex::new(CoreEngine::new()));
            app.manage(AppState::new(
                database,
                paths.settings_path,
                paths.data_dir,
                paths.config_dir,
                startup_recovery,
            ));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // System
            commands::system::ping,
            commands::system::get_app_info,
            // Test
            commands::session::start_test,
            commands::session::process_key,
            commands::session::abort_session,
            // Stats
            commands::reporting::get_stats_history,
            commands::reporting::get_personal_bests,
            // Custom Texts
            commands::content::get_custom_texts,
            commands::content::get_custom_text,
            commands::content::save_custom_text,
            commands::content::update_custom_text,
            commands::content::delete_custom_text,
            commands::content::search_custom_texts,
            commands::session::start_custom_text_test,
            // Settings
            commands::preferences::get_settings,
            commands::preferences::set_setting,
            // Themes
            commands::preferences::get_themes,
            commands::preferences::get_theme_css,
            // Lessons
            commands::content::get_course,
            commands::content::get_lesson_progress,
            commands::session::start_lesson,
            // Weak Keys
            commands::content::analyze_weak_keys,
            commands::content::generate_weak_keys_training,
            // Dashboard
            commands::reporting::get_dashboard_stats,
            commands::reporting::get_progress_history,
            // Analytics
            commands::reporting::get_achievements,
            commands::reporting::get_insights,
            commands::reporting::get_consistency,
            commands::reporting::export_data,
            // Versioned portable profile transfer
            commands::profile_transfer::export_profile,
            commands::profile_transfer::preview_profile_import,
            commands::profile_transfer::import_profile,
            // Replay
            commands::reporting::get_replay,
            // Sound
            commands::preferences::get_sound_event,
        ]);

    if let Err(error) = application.run(tauri::generate_context!()) {
        eprintln!("Racoon Typper failed to start: {error}");
        std::process::exit(1);
    }
}

pub fn app_info(state: &AppState) -> AppInfo {
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        build_profile: profile.to_string(),
        data_dir: state.data_dir().to_string_lossy().to_string(),
        config_dir: state.config_dir().to_string_lossy().to_string(),
        db_path: state
            .data_dir()
            .join("data.db")
            .to_string_lossy()
            .to_string(),
        settings_path: state.settings_path().to_string_lossy().to_string(),
    }
}
