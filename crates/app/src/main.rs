//! Tauri application entry point.
//! Sprint 5: Settings + Themes + Custom Texts

use chrono::{DateTime, Utc};
use racoon_application::{
    SessionWallClock, StartupRecoveryCoordinator, StartupRecoveryGate, StartupRecoveryRetryPolicy,
    StartupRecoverySleeper,
};
use racoon_core::CoreEngine;
use racoon_data::{
    repository::SettingsStore, Database, SqliteFinalizationLedger, SqliteSessionFinalizer,
    SqliteSessionRecoveryLedger,
};
use std::sync::Mutex;
use tauri::Manager;

mod commands;
mod error;
mod export;
mod logging;
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
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let paths = paths::resolve(app)?;
            let data_dir = paths.data_dir.clone();
            let logger = SettingsStore::new(paths.settings_path.clone())
                .load()
                .ok()
                .filter(|settings| settings.verbose_logging)
                .map(|_| logging::LocalLogger::enabled(&data_dir, logging::LogRetention::default()))
                .unwrap_or_else(logging::LocalLogger::disabled);
            // Take a rotating pre-migration backup before any schema work. A
            // failure here is warn-and-continue: migrations V005–V008 are
            // additive and low-risk, and a transient permissions/IO error must
            // not brick the application. The backup is defense-in-depth; the
            // operational recovery path remains "restore the most recent
            // snapshot or ship a forward fix".
            let database = Database::open_with_pre_migration(&paths.db_path, |live_path| {
                if racoon_data::backup::create_pre_migration_backup(
                    live_path,
                    &data_dir,
                    "data",
                    chrono::Utc::now(),
                    racoon_data::backup::DEFAULT_KEEP,
                )
                .is_err()
                {
                    logger.record_pre_migration_backup_failure(logging::ErrorClass::Io, live_path);
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
                startup_recovery,
            ));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Test
            commands::session::start_test,
            commands::session::process_key,
            commands::session::abort_session,
            commands::session::abandon_active_session,
            // Stats
            commands::reporting::get_stats_history,
            commands::reporting::get_personal_bests,
            // Custom Texts
            commands::content::get_custom_texts,
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
            // Custom text import
            commands::content::import_text_from_url,
            // Dashboard
            commands::reporting::get_dashboard_stats,
            commands::reporting::get_progress_history,
            // Analytics
            commands::reporting::get_achievements,
            commands::reporting::get_insights,
            commands::reporting::get_consistency,
            commands::reporting::export_data,
            commands::reporting::export_report,
            commands::reporting::export_heatmap_png,
            // Versioned portable profile transfer
            commands::profile_transfer::export_profile,
            commands::profile_transfer::preview_profile_import,
            commands::profile_transfer::import_profile,
            // Replay
            commands::reporting::get_replay,
            // Aggregated heatmap (training-of-the-day)
            commands::reporting::get_aggregated_heatmap,
            commands::reporting::clear_statistics,
            // Sound
            commands::preferences::get_sound_event,
        ]);

    if application.run(tauri::generate_context!()).is_err() {
        eprintln!("Racoon Typper failed to start");
        std::process::exit(1);
    }
}
