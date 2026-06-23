//! Tauri application entry point.
//! Sprint 5: Settings + Themes + Custom Texts

use racoon_core::CoreEngine;
use racoon_data::Database;
use racoon_domain::AppInfo;
use std::env;
use std::path::PathBuf;
use std::sync::Mutex;

mod commands;
mod error;
mod state;

use state::AppState;

fn main() {
    let (data_dir, config_dir) = get_app_dirs();
    std::fs::create_dir_all(&data_dir).ok();
    std::fs::create_dir_all(&config_dir).ok();

    let db_path = data_dir.join("data.db");
    let settings_path = config_dir.join("settings.toml");

    let db = Database::open(&db_path).expect("Failed to open database");

    tauri::Builder::default()
        .manage(Mutex::new(CoreEngine::new()))
        .manage(AppState::new(db, settings_path))
        .invoke_handler(tauri::generate_handler![
            // System
            commands::ping,
            commands::get_app_info,
            // Test
            commands::start_test,
            commands::process_key,
            commands::abort_session,
            // Stats
            commands::get_stats_history,
            commands::get_test_detail,
            commands::get_personal_bests,
            // Custom Texts
            commands::get_custom_texts,
            commands::get_custom_text,
            commands::save_custom_text,
            commands::update_custom_text,
            commands::delete_custom_text,
            commands::search_custom_texts,
            commands::start_custom_text_test,
            // Settings
            commands::get_settings,
            commands::set_setting,
            // Themes
            commands::get_themes,
            commands::get_theme_css,
            // Lessons
            commands::get_course,
            commands::get_lesson_progress,
            commands::start_lesson,
            commands::complete_lesson,
            // Weak Keys
            commands::analyze_weak_keys,
            commands::generate_weak_keys_training,
            // Dashboard
            commands::get_dashboard_stats,
            commands::get_streak_info,
            commands::get_progress_history,
            // Analytics
            commands::get_achievements,
            commands::get_insights,
            commands::get_consistency,
            commands::export_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn get_app_dirs() -> (PathBuf, PathBuf) {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let data_dir = env::var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(&home).join(".local/share/racoon-typper"));
    let config_dir = env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(&home).join(".config/racoon-typper"));
    (data_dir, config_dir)
}

pub fn app_info() -> AppInfo {
    let (data_dir, config_dir) = get_app_dirs();
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        build_profile: profile.to_string(),
        data_dir: data_dir.to_string_lossy().to_string(),
        config_dir: config_dir.to_string_lossy().to_string(),
        db_path: data_dir.join("data.db").to_string_lossy().to_string(),
        settings_path: config_dir
            .join("settings.toml")
            .to_string_lossy()
            .to_string(),
    }
}
