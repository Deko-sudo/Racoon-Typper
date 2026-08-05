const APP_COMMANDS: &[&str] = &[
    "start_test",
    "process_key",
    "abort_session",
    "get_stats_history",
    "get_personal_bests",
    "get_custom_texts",
    "save_custom_text",
    "update_custom_text",
    "delete_custom_text",
    "search_custom_texts",
    "start_custom_text_test",
    "get_settings",
    "set_setting",
    "get_themes",
    "get_theme_css",
    "get_course",
    "get_lesson_progress",
    "start_lesson",
    "analyze_weak_keys",
    "generate_weak_keys_training",
    "get_dashboard_stats",
    "get_progress_history",
    "get_achievements",
    "get_insights",
    "get_consistency",
    "export_data",
    "export_profile",
    "preview_profile_import",
    "import_profile",
    "get_replay",
    "get_sound_event",
];

fn main() {
    let attributes = tauri_build::Attributes::new()
        .app_manifest(tauri_build::AppManifest::new().commands(APP_COMMANDS));
    tauri_build::try_build(attributes).expect("failed to build Tauri application permissions");
}
