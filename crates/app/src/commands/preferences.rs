//! Tauri adapters for settings, built-in themes, and sound policy.

use racoon_data::repository::AppSettings;
use tauri::State;

use crate::commands::contracts::{SoundOutputResponse, ThemeInfo, ThemePreview};
use crate::error::AppError;
use crate::state::AppState;
use crate::validation::{validate_setting_key, validate_sound_event, validate_theme_name};

#[tauri::command]
pub(crate) fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, AppError> {
    state
        .with_settings(|store| store.load())
        .map_err(AppError::from)
}

#[tauri::command]
pub(crate) fn set_setting(
    state: State<'_, AppState>,
    key: String,
    value: serde_json::Value,
) -> Result<AppSettings, AppError> {
    set_setting_with_state(&state, key, value)
}

fn set_setting_with_state(
    state: &AppState,
    key: String,
    value: serde_json::Value,
) -> Result<AppSettings, AppError> {
    state.require_startup_recovery_ready()?;
    validate_setting_key(&key)?;
    if key == "practice_language" {
        validate_practice_language(&value)?;
    }
    let toml_value = json_to_toml_value(&value)?;
    state
        .with_settings(|store| store.set(&key, toml_value))
        .map_err(AppError::from)
}

/// The data layer only checks the code format; membership against the bundled
/// course resources is enforced here so onboarding cannot persist a language
/// the application cannot serve.
fn validate_practice_language(value: &serde_json::Value) -> Result<(), AppError> {
    let language = value
        .as_str()
        .ok_or_else(|| AppError::InvalidConfig("practice_language must be a string".into()))?;
    let loader = racoon_resources::CourseLoader::new();
    if loader.load_course(language).is_none() {
        return Err(AppError::InvalidConfig(format!(
            "practice_language must be a supported course language, got '{language}'"
        )));
    }
    Ok(())
}

#[tauri::command]
pub(crate) fn get_themes() -> Result<Vec<ThemeInfo>, AppError> {
    Ok(vec![
        theme_info(
            "racoon_graphite",
            "Racoon Graphite",
            true,
            "#0d0f12",
            "#c5cbd4",
            "#e7e9ed",
            "#e39a9a",
        ),
        theme_info(
            "racoon_silver",
            "Racoon Silver",
            false,
            "#eeeff1",
            "#4e5865",
            "#202329",
            "#a33f46",
        ),
        theme_info(
            "racoon_warm",
            "Racoon Warm",
            true,
            "#171416",
            "#d48a63",
            "#eee7e2",
            "#e58c89",
        ),
        theme_info(
            "midnight_ink",
            "Midnight Ink",
            true,
            "#090d15",
            "#7495c9",
            "#e5eaf2",
            "#d47d83",
        ),
        theme_info(
            "arctic_slate",
            "Arctic Slate",
            true,
            "#11161a",
            "#8ab4c8",
            "#e4eaed",
            "#d38989",
        ),
        theme_info(
            "racoon_forest",
            "Racoon Forest",
            true,
            "#0d1411",
            "#75a486",
            "#e1e9e3",
            "#c98078",
        ),
        theme_info(
            "moss", "Moss", true, "#151711", "#9ca56a", "#e7e6d7", "#c48276",
        ),
        theme_info(
            "coffee", "Coffee", true, "#171310", "#b98b68", "#efe5d9", "#ce8177",
        ),
        theme_info(
            "paper", "Paper", false, "#edeae2", "#606c78", "#292b2d", "#a95454",
        ),
        theme_info(
            "sandstone",
            "Sandstone",
            false,
            "#e8dfd2",
            "#946e55",
            "#342e2a",
            "#a6534f",
        ),
        theme_info(
            "mist", "Mist", false, "#e5e9eb", "#647d8d", "#252b30", "#a85559",
        ),
        theme_info(
            "lavender_dusk",
            "Lavender Dusk",
            true,
            "#121017",
            "#9c86b5",
            "#eae4ef",
            "#cf7d88",
        ),
        theme_info(
            "plum", "Plum", true, "#171116", "#b17d9b", "#efe5ec", "#d27878",
        ),
        theme_info(
            "ocean", "Ocean", true, "#0a1417", "#68a2aa", "#ddebed", "#ce7c7a",
        ),
        theme_info(
            "deep_sea", "Deep Sea", true, "#071011", "#568c91", "#dce6e6", "#c87575",
        ),
        theme_info(
            "ember", "Ember", true, "#151110", "#b8654d", "#eee7e3", "#e06a67",
        ),
        theme_info(
            "burgundy", "Burgundy", true, "#160e11", "#a65e70", "#f0e5e7", "#d26c6c",
        ),
        theme_info(
            "amber_terminal",
            "Amber Terminal",
            true,
            "#10100c",
            "#c8a74d",
            "#eee5c5",
            "#d97a62",
        ),
        theme_info(
            "green_terminal",
            "Green Terminal",
            true,
            "#09100b",
            "#6daa78",
            "#dce8dd",
            "#d07c73",
        ),
        theme_info(
            "steel_blue",
            "Steel Blue",
            true,
            "#101419",
            "#718caa",
            "#e5e9ef",
            "#c87d82",
        ),
        theme_info(
            "carbon", "Carbon", true, "#0c0c0d", "#b4b4ba", "#e8e8e9", "#c98282",
        ),
        theme_info(
            "moonlight",
            "Moonlight",
            true,
            "#10131b",
            "#8296c0",
            "#e5e8f1",
            "#cc8189",
        ),
        theme_info(
            "dawn", "Dawn", false, "#eee8e2", "#ac7469", "#332e2d", "#a95353",
        ),
        theme_info(
            "sage", "Sage", false, "#e5e8e0", "#6e8875", "#29302b", "#a85b58",
        ),
        theme_info(
            "racoon_high_contrast",
            "Racoon High Contrast",
            true,
            "#000000",
            "#ffd84d",
            "#ffffff",
            "#ff7373",
        ),
        theme_info(
            "abyss", "Abyss", true, "#06080f", "#6d8bd6", "#d4d8e8", "#e08b8b",
        ),
        theme_info(
            "solar_flare",
            "Solar Flare",
            true,
            "#120e08",
            "#e8a83b",
            "#f0e6d6",
            "#e87878",
        ),
        theme_info(
            "volcanic", "Volcanic", true, "#0a0806", "#d4642a", "#e8ddd0", "#e85a3a",
        ),
        theme_info(
            "obsidian", "Obsidian", true, "#080a0c", "#3ac8d8", "#dde2e8", "#e88a8a",
        ),
        theme_info(
            "toxic", "Toxic", true, "#060a06", "#48d848", "#c8e8c0", "#e86848",
        ),
        theme_info(
            "chestnut", "Chestnut", true, "#100c08", "#c89858", "#ede0d0", "#dc8c6a",
        ),
        theme_info(
            "glacier", "Glacier", false, "#e8f0f5", "#5a9cb8", "#1a2a35", "#b85555",
        ),
        theme_info(
            "mint_frost",
            "Mint Frost",
            false,
            "#e6f2ee",
            "#4a9a7a",
            "#1a2e26",
            "#aa5050",
        ),
        theme_info(
            "porcelain",
            "Porcelain",
            false,
            "#f4f5f7",
            "#6888a8",
            "#2a2e35",
            "#b04848",
        ),
        // Community-inspired palettes (original Apache-2.0 implementations)
        theme_info(
            "catppuccin_mocha",
            "Catppuccin Mocha",
            true,
            "#1e1e2e",
            "#cba6f7",
            "#cdd6f4",
            "#f38ba8",
        ),
        theme_info(
            "coral", "Coral", false, "#fff0ed", "#e8654a", "#3d2620", "#c44536",
        ),
        theme_info(
            "dark", "Dark", true, "#1a1a1a", "#cccccc", "#e0e0e0", "#ff6666",
        ),
        theme_info(
            "dracula", "Dracula", true, "#282a36", "#bd93f9", "#f8f8f2", "#ff5555",
        ),
        theme_info(
            "foamy", "Foamy", false, "#e8f5f0", "#3aaa88", "#1a2e26", "#aa4848",
        ),
        theme_info(
            "gruvbox_dark",
            "Gruvbox Dark",
            true,
            "#282828",
            "#fe8019",
            "#ebdbb2",
            "#fb4934",
        ),
        theme_info(
            "light", "Light", false, "#f0f0f0", "#3a7ca5", "#1a1a1a", "#cc4444",
        ),
        theme_info(
            "lilac", "Lilac", false, "#f0ecf4", "#8a6db0", "#2a2230", "#aa4868",
        ),
        theme_info(
            "matrix", "Matrix", true, "#000000", "#00ff00", "#33ff33", "#ff3333",
        ),
        theme_info(
            "nautilus", "Nautilus", true, "#0c1620", "#5fb4ca", "#d8e8f0", "#e08080",
        ),
        theme_info(
            "nord", "Nord", true, "#2e3440", "#88c0d0", "#e5e9f0", "#bf616a",
        ),
        theme_info(
            "rose_pine",
            "Rosé Pine",
            true,
            "#191724",
            "#c4a7e7",
            "#e0def4",
            "#eb6f92",
        ),
        theme_info(
            "serika", "Serika", false, "#e1e1e3", "#e2b714", "#323437", "#ca3e3e",
        ),
        theme_info(
            "serika_dark",
            "Serika Dark",
            true,
            "#323437",
            "#e2b714",
            "#e1e1e3",
            "#ca4754",
        ),
        theme_info(
            "serika_light",
            "Serika Light",
            false,
            "#eceef0",
            "#e2b714",
            "#2c2e31",
            "#ca3e3e",
        ),
        theme_info(
            "terra", "Terra", true, "#1c1814", "#c68855", "#e8dcc8", "#c85544",
        ),
    ])
}

#[tauri::command]
pub(crate) fn get_theme_css(name: String) -> Result<String, AppError> {
    validate_theme_name(&name)?;
    let css = match name.as_str() {
        "racoon_graphite" => {
            include_str!("../../../../resources/themes/racoon_graphite/theme.css")
        }
        "racoon_silver" => {
            include_str!("../../../../resources/themes/racoon_silver/theme.css")
        }
        "racoon_warm" => include_str!("../../../../resources/themes/racoon_warm/theme.css"),
        "racoon_high_contrast" => {
            include_str!("../../../../resources/themes/racoon_high_contrast/theme.css")
        }
        "midnight_ink" => include_str!("../../../../resources/themes/midnight_ink/theme.css"),
        "arctic_slate" => include_str!("../../../../resources/themes/arctic_slate/theme.css"),
        "racoon_forest" => include_str!("../../../../resources/themes/racoon_forest/theme.css"),
        "moss" => include_str!("../../../../resources/themes/moss/theme.css"),
        "coffee" => include_str!("../../../../resources/themes/coffee/theme.css"),
        "paper" => include_str!("../../../../resources/themes/paper/theme.css"),
        "sandstone" => include_str!("../../../../resources/themes/sandstone/theme.css"),
        "mist" => include_str!("../../../../resources/themes/mist/theme.css"),
        "lavender_dusk" => include_str!("../../../../resources/themes/lavender_dusk/theme.css"),
        "plum" => include_str!("../../../../resources/themes/plum/theme.css"),
        "ocean" => include_str!("../../../../resources/themes/ocean/theme.css"),
        "deep_sea" => include_str!("../../../../resources/themes/deep_sea/theme.css"),
        "ember" => include_str!("../../../../resources/themes/ember/theme.css"),
        "burgundy" => include_str!("../../../../resources/themes/burgundy/theme.css"),
        "amber_terminal" => include_str!("../../../../resources/themes/amber_terminal/theme.css"),
        "green_terminal" => include_str!("../../../../resources/themes/green_terminal/theme.css"),
        "steel_blue" => include_str!("../../../../resources/themes/steel_blue/theme.css"),
        "carbon" => include_str!("../../../../resources/themes/carbon/theme.css"),
        "moonlight" => include_str!("../../../../resources/themes/moonlight/theme.css"),
        "dawn" => include_str!("../../../../resources/themes/dawn/theme.css"),
        "sage" => include_str!("../../../../resources/themes/sage/theme.css"),
        "abyss" => include_str!("../../../../resources/themes/abyss/theme.css"),
        "solar_flare" => include_str!("../../../../resources/themes/solar_flare/theme.css"),
        "volcanic" => include_str!("../../../../resources/themes/volcanic/theme.css"),
        "obsidian" => include_str!("../../../../resources/themes/obsidian/theme.css"),
        "toxic" => include_str!("../../../../resources/themes/toxic/theme.css"),
        "chestnut" => include_str!("../../../../resources/themes/chestnut/theme.css"),
        "glacier" => include_str!("../../../../resources/themes/glacier/theme.css"),
        "mint_frost" => include_str!("../../../../resources/themes/mint_frost/theme.css"),
        "porcelain" => include_str!("../../../../resources/themes/porcelain/theme.css"),
        "catppuccin_mocha" => {
            include_str!("../../../../resources/themes/catppuccin_mocha/theme.css")
        }
        "coral" => include_str!("../../../../resources/themes/coral/theme.css"),
        "dark" => include_str!("../../../../resources/themes/dark/theme.css"),
        "dracula" => include_str!("../../../../resources/themes/dracula/theme.css"),
        "foamy" => include_str!("../../../../resources/themes/foamy/theme.css"),
        "gruvbox_dark" => include_str!("../../../../resources/themes/gruvbox_dark/theme.css"),
        "light" => include_str!("../../../../resources/themes/light/theme.css"),
        "lilac" => include_str!("../../../../resources/themes/lilac/theme.css"),
        "matrix" => include_str!("../../../../resources/themes/matrix/theme.css"),
        "nautilus" => include_str!("../../../../resources/themes/nautilus/theme.css"),
        "nord" => include_str!("../../../../resources/themes/nord/theme.css"),
        "rose_pine" => include_str!("../../../../resources/themes/rose_pine/theme.css"),
        "serika" => include_str!("../../../../resources/themes/serika/theme.css"),
        "serika_dark" => include_str!("../../../../resources/themes/serika_dark/theme.css"),
        "serika_light" => include_str!("../../../../resources/themes/serika_light/theme.css"),
        "terra" => include_str!("../../../../resources/themes/terra/theme.css"),
        _ => return Err(AppError::ThemeNotFound(name)),
    };
    Ok(css.to_string())
}

#[tauri::command]
pub(crate) fn get_sound_event(
    state: State<'_, AppState>,
    event: String,
) -> Result<Option<SoundOutputResponse>, AppError> {
    validate_sound_event(&event)?;
    let settings = state.with_settings(|store| store.load())?;
    if !settings.sound_enabled {
        return Ok(None);
    }

    let sound_event = match event.as_str() {
        "key_press" => racoon_core::sound::SoundEvent::KeyPress,
        "error" => racoon_core::sound::SoundEvent::Error,
        "lesson_complete" => racoon_core::sound::SoundEvent::LessonComplete,
        "achievement_unlocked" => racoon_core::sound::SoundEvent::AchievementUnlocked,
        _ => return Ok(None),
    };
    let output = state.try_play_sound(
        racoon_core::sound::SoundConfig {
            enabled: settings.sound_enabled,
            volume: settings.sound_volume,
        },
        sound_event,
    )?;
    Ok(output.map(|output| SoundOutputResponse {
        frequency: output.frequency,
        duration_ms: output.duration_ms,
        volume: output.volume,
        event,
    }))
}

fn theme_info(
    name: &str,
    display_name: &str,
    is_dark: bool,
    bg: &str,
    main: &str,
    text: &str,
    error: &str,
) -> ThemeInfo {
    ThemeInfo {
        name: name.to_string(),
        display_name: display_name.to_string(),
        is_dark,
        preview_colors: ThemePreview {
            bg: bg.to_string(),
            main: main.to_string(),
            text: text.to_string(),
            error: error.to_string(),
        },
    }
}

fn json_to_toml_value(value: &serde_json::Value) -> Result<toml::Value, AppError> {
    match value {
        serde_json::Value::String(string) => Ok(toml::Value::String(string.clone())),
        serde_json::Value::Number(number) => {
            if let Some(integer) = number.as_i64() {
                Ok(toml::Value::Integer(integer))
            } else if let Some(float) = number.as_f64() {
                Ok(toml::Value::Float(float))
            } else {
                Err(AppError::InvalidConfig(
                    "Unsupported numeric setting value".to_string(),
                ))
            }
        }
        serde_json::Value::Bool(boolean) => Ok(toml::Value::Boolean(*boolean)),
        serde_json::Value::Null | serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
            Err(AppError::InvalidConfig(
                "Settings values must be scalar strings, numbers, or booleans".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use racoon_application::{
        SessionWallClock, StartupRecoveryCoordinator, StartupRecoveryGate,
        StartupRecoveryRetryPolicy, StartupRecoverySleeper,
    };
    use racoon_data::{
        Database, SqliteFinalizationLedger, SqliteSessionFinalizer, SqliteSessionRecoveryLedger,
    };
    use std::num::NonZeroUsize;
    use std::path::PathBuf;
    use std::time::Duration;

    struct FixedClock;
    impl SessionWallClock for FixedClock {
        fn utc_now(&self) -> DateTime<Utc> {
            DateTime::parse_from_rfc3339("2026-07-16T12:00:00Z")
                .expect("timestamp")
                .with_timezone(&Utc)
        }
    }
    struct NoopSleeper;
    impl StartupRecoverySleeper for NoopSleeper {
        fn sleep(&self, _: Duration) {}
    }

    fn app_state(gate: StartupRecoveryGate, settings_path: PathBuf) -> AppState {
        AppState::new(
            Database::open_in_memory().expect("database"),
            settings_path,
            PathBuf::from("unused-db.db"),
            gate,
        )
    }

    fn mark_recovery_ready(state: &AppState) {
        let database = &state.db;
        let recovery = SqliteSessionRecoveryLedger::new(database);
        let finalizations = SqliteFinalizationLedger::new(database);
        let finalizer = SqliteSessionFinalizer::new(database);
        let clock = FixedClock;
        let sleeper = NoopSleeper;
        StartupRecoveryCoordinator::new(
            &recovery,
            &finalizations,
            &finalizer,
            &clock,
            &sleeper,
            StartupRecoveryRetryPolicy::new(NonZeroUsize::new(1).expect("nonzero"), Duration::ZERO),
        )
        .run(state.startup_recovery())
        .expect("startup recovery");
    }

    #[test]
    fn rejects_non_scalar_settings_values() {
        assert!(json_to_toml_value(&serde_json::json!(["not", "a", "scalar"])).is_err());
    }

    #[test]
    fn practice_language_setting_accepts_bundled_courses_and_rejects_unknown_codes() {
        let settings_path = std::env::temp_dir().join(format!(
            "racoon-settings-practice-lang-{}-{}.toml",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        let _ = std::fs::remove_file(&settings_path);
        let state = app_state(StartupRecoveryGate::new(), settings_path.clone());
        mark_recovery_ready(&state);

        let updated = set_setting_with_state(
            &state,
            "practice_language".to_string(),
            serde_json::json!("de"),
        )
        .expect("supported course language");
        assert_eq!(updated.practice_language, "de");

        assert!(set_setting_with_state(
            &state,
            "practice_language".to_string(),
            serde_json::json!("kk"),
        )
        .is_err());
        assert!(set_setting_with_state(
            &state,
            "practice_language".to_string(),
            serde_json::json!(42),
        )
        .is_err());
        let _ = std::fs::remove_file(settings_path);
    }

    #[test]
    fn set_setting_command_is_gated_before_ready_and_succeeds_after_ready() {
        let settings_path = std::env::temp_dir().join(format!(
            "racoon-settings-gate-{}-{}.toml",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        let _ = std::fs::remove_file(&settings_path);
        let gate = StartupRecoveryGate::new();
        let state = app_state(gate, settings_path.clone());

        let before = set_setting_with_state(&state, "font_size".to_string(), serde_json::json!(30));
        assert!(matches!(before, Err(AppError::RecoveryNotStarted)));
        assert!(!settings_path.exists());

        let recovery_gate = state.startup_recovery();
        let database = &state.db;
        let recovery = SqliteSessionRecoveryLedger::new(database);
        let finalizations = SqliteFinalizationLedger::new(database);
        let finalizer = SqliteSessionFinalizer::new(database);
        let clock = FixedClock;
        let sleeper = NoopSleeper;
        StartupRecoveryCoordinator::new(
            &recovery,
            &finalizations,
            &finalizer,
            &clock,
            &sleeper,
            StartupRecoveryRetryPolicy::new(NonZeroUsize::new(1).expect("nonzero"), Duration::ZERO),
        )
        .run(recovery_gate)
        .expect("startup recovery");

        let after = set_setting_with_state(&state, "font_size".to_string(), serde_json::json!(30))
            .expect("ready command");
        assert_eq!(after.font_size, 30);
        assert!(settings_path.exists());
        let _ = std::fs::remove_file(settings_path);
    }

    #[test]
    fn built_in_theme_catalog_is_complete_and_unique() {
        let themes = get_themes().unwrap();
        assert_eq!(themes.len(), 50);
        let unique_names: std::collections::HashSet<_> =
            themes.iter().map(|theme| theme.name.as_str()).collect();
        assert_eq!(unique_names.len(), themes.len());
        assert!(unique_names.contains("racoon_graphite"));
        assert!(unique_names.contains("racoon_silver"));
        assert!(unique_names.contains("racoon_warm"));
        assert!(unique_names.contains("racoon_high_contrast"));
    }

    #[test]
    fn built_in_theme_css_contains_required_semantic_variables() {
        let css = get_theme_css("racoon_graphite".to_string()).unwrap();
        for variable in [
            "--color-app-background:",
            "--color-surface-raised:",
            "--color-text-primary:",
            "--color-focus-ring:",
            "--color-typing-current:",
            "--color-chart-primary:",
            "--bg:",
            "--bg-sub:",
            "--main:",
            "--caret:",
            "--sub:",
            "--text:",
            "--error:",
        ] {
            assert!(css.contains(variable), "missing CSS variable {variable}");
        }
    }

    #[test]
    fn high_contrast_theme_exposes_accessible_focus_and_caret_tokens() {
        let css = get_theme_css("racoon_high_contrast".to_string()).unwrap();
        assert!(css.contains("--color-focus-ring: #ffd84d"));
        assert!(css.contains("--color-caret: #ffd84d"));
        assert!(css.contains("--color-typing-current: #ffffff"));
    }

    #[test]
    fn unknown_theme_identifier_is_rejected_without_a_fallback_payload() {
        let error = get_theme_css("../../untrusted".to_string()).unwrap_err();
        assert!(matches!(error, AppError::ThemeNotFound(_)));
    }
}
