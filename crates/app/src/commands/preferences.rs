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
    let toml_value = json_to_toml_value(&value)?;
    state
        .with_settings(|store| store.set(&key, toml_value))
        .map_err(AppError::from)
}

#[tauri::command]
pub(crate) fn get_themes() -> Result<Vec<ThemeInfo>, AppError> {
    Ok(vec![
        theme_info(
            "racoon_dark",
            "Racoon Dark",
            true,
            "#151a24",
            "#5eead4",
            "#e8f0f7",
            "#fb7185",
        ),
        theme_info(
            "racoon_light",
            "Racoon Light",
            false,
            "#f7fafc",
            "#0f766e",
            "#1f2937",
            "#dc2626",
        ),
        theme_info(
            "racoon_high_contrast",
            "Racoon High Contrast",
            true,
            "#000000",
            "#00ff9d",
            "#ffffff",
            "#ff4d6d",
        ),
    ])
}

#[tauri::command]
pub(crate) fn get_theme_css(name: String) -> Result<String, AppError> {
    validate_theme_name(&name)?;
    let css = match name.as_str() {
        "racoon_dark" => include_str!("../../../../resources/themes/racoon_dark/theme.css"),
        "racoon_light" => include_str!("../../../../resources/themes/racoon_light/theme.css"),
        "racoon_high_contrast" => {
            include_str!("../../../../resources/themes/racoon_high_contrast/theme.css")
        }
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
            PathBuf::from("data"),
            PathBuf::from("config"),
            gate,
        )
    }

    #[test]
    fn rejects_non_scalar_settings_values() {
        assert!(json_to_toml_value(&serde_json::json!(["not", "a", "scalar"])).is_err());
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
    fn original_theme_catalog_is_complete_and_unique() {
        let themes = get_themes().unwrap();
        assert_eq!(themes.len(), 3);
        let unique_names: std::collections::HashSet<_> =
            themes.iter().map(|theme| theme.name.as_str()).collect();
        assert_eq!(unique_names.len(), themes.len());
        assert!(unique_names.contains("racoon_dark"));
        assert!(unique_names.contains("racoon_light"));
        assert!(unique_names.contains("racoon_high_contrast"));
    }

    #[test]
    fn original_theme_css_contains_required_variables() {
        let css = get_theme_css("racoon_dark".to_string()).unwrap();
        for variable in [
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
    fn theme_css_is_available_for_original_themes() {
        let css = get_theme_css("racoon_high_contrast".to_string()).unwrap();
        assert!(css.contains("--main: #00ff9d;"));
    }
}
