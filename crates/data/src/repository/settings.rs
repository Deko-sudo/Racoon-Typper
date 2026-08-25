//! SettingsStore — загрузка/сохранение настроек в settings.toml.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

use crate::error::DbError;

/// Настройки приложения (подмножество для MVP Sprint 5).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_font_size")]
    pub font_size: i64,
    #[serde(default = "default_caret_style")]
    pub caret_style: String,
    /// Позиция каретки относительно символа: "before" (перед следующей буквой,
    /// индустриальный стандарт — дефолт) или "after" (за последней напечатанной).
    #[serde(default = "default_caret_position")]
    pub caret_position: String,
    /// Анимация каретки: "blink" (мигание) или "pulse" (мягкая пульсация).
    #[serde(default = "default_caret_animation")]
    pub caret_animation: String,
    #[serde(default = "default_true")]
    pub show_live_wpm: bool,
    #[serde(default = "default_true")]
    pub show_accuracy: bool,
    #[serde(default = "default_true")]
    pub show_keyboard_trainer: bool,
    #[serde(default = "default_true")]
    pub show_hand_guide: bool,
    #[serde(default = "default_true")]
    pub show_layout_warnings: bool,
    #[serde(default = "default_true")]
    pub show_capslock_warnings: bool,
    #[serde(default)]
    pub sound_enabled: bool,
    #[serde(default)]
    pub verbose_logging: bool,
    #[serde(default = "default_sound_volume")]
    pub sound_volume: f64,
    #[serde(default)]
    pub zen_mode_enabled: bool,
    #[serde(default)]
    pub blind_mode_enabled: bool,
    #[serde(default = "default_ui_language")]
    pub ui_language: String,
    /// Practice language for tests and lessons; lowercase resource code
    /// (for example "en" or "zh-hk"). Membership against the bundled course
    /// resources is enforced by the application layer.
    #[serde(default = "default_practice_language")]
    pub practice_language: String,
    /// First-run onboarding has been completed or explicitly skipped.
    #[serde(default)]
    pub onboarding_completed: bool,
    /// Physical keyboard layout used for finger hints and visualization:
    /// "qwerty", "jcuken", or "dvorak". Cyrillic characters always resolve to
    /// the JCUKEN map regardless of this value.
    #[serde(default = "default_keyboard_layout")]
    pub keyboard_layout: String,
    #[serde(default)]
    pub vim_mode: bool,
    #[serde(default = "default_daily_goal_type")]
    pub daily_goal_type: String,
    #[serde(default)]
    pub daily_goal_wpm: f64,
    #[serde(default)]
    pub daily_goal_accuracy: f64,
    /// Daily typing-time goal in minutes, used when `daily_goal_type == "time"`.
    #[serde(default)]
    pub daily_goal_minutes: i64,
    /// Pomodoro work phase length in minutes.
    #[serde(default = "default_pomodoro_work_min")]
    pub pomodoro_work_min: i64,
    /// Pomodoro break phase length in minutes.
    #[serde(default = "default_pomodoro_break_min")]
    pub pomodoro_break_min: i64,
    /// Кастомная тема: JSON-объект { "--color-*": "#rrggbb" }. Пустая строка —
    /// тема не настроена. Ключи валидируются по белому списку при set().
    #[serde(default)]
    pub custom_theme_colors: String,
}

fn default_theme() -> String {
    "racoon_graphite".to_string()
}

fn normalize_theme(theme: &str) -> String {
    match theme {
        "racoon_graphite"
        | "racoon_silver"
        | "racoon_warm"
        | "racoon_high_contrast"
        | "midnight_ink"
        | "arctic_slate"
        | "racoon_forest"
        | "moss"
        | "coffee"
        | "paper"
        | "sandstone"
        | "mist"
        | "lavender_dusk"
        | "plum"
        | "ocean"
        | "deep_sea"
        | "ember"
        | "burgundy"
        | "amber_terminal"
        | "green_terminal"
        | "steel_blue"
        | "carbon"
        | "moonlight"
        | "dawn"
        | "sage"
        | "custom" => theme.to_string(),
        "racoon_dark" => "racoon_graphite".to_string(),
        "racoon_light" => "racoon_silver".to_string(),
        _ => default_theme(),
    }
}

fn default_font_size() -> i64 {
    24
}

fn default_caret_style() -> String {
    "underline".to_string()
}

fn default_caret_position() -> String {
    "before".to_string()
}

fn default_caret_animation() -> String {
    "blink".to_string()
}

fn valid_caret_position(value: &str) -> bool {
    matches!(value, "before" | "after")
}

fn valid_caret_animation(value: &str) -> bool {
    matches!(value, "blink" | "pulse")
}

/// Валидирует JSON кастомной темы: объект, ключи из белого списка,
/// значения — hex-цвета "#rrggbb" (или "rrggbb"). Возвращает нормализованный
/// компактный JSON (ключи отсортированы) или ошибку.
fn validate_custom_theme_json(value: &str) -> Result<String, DbError> {
    if value.trim().is_empty() {
        return Ok(String::new());
    }
    if value.chars().count() > MAX_CUSTOM_THEME_JSON_CHARS {
        return Err(validation_error(format!(
            "custom_theme_colors must be at most {MAX_CUSTOM_THEME_JSON_CHARS} characters"
        )));
    }
    let parsed: serde_json::Value = serde_json::from_str(value).map_err(|error| {
        validation_error(format!("custom_theme_colors must be valid JSON: {error}"))
    })?;
    let object = parsed
        .as_object()
        .ok_or_else(|| validation_error("custom_theme_colors must be a JSON object"))?;

    let mut normalized: std::collections::BTreeMap<String, String> =
        std::collections::BTreeMap::new();
    for (key, color) in object {
        if !CUSTOM_THEME_VARIABLES.contains(&key.as_str()) {
            return Err(validation_error(format!(
                "Unsupported custom theme variable: {key}"
            )));
        }
        let color = color
            .as_str()
            .ok_or_else(|| validation_error(format!("{key} must be a hex color string")))?;
        let hex = color.trim().trim_start_matches('#');
        if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(validation_error(format!(
                "{key} must be a six-digit hex color like #rrggbb"
            )));
        }
        normalized.insert(key.clone(), format!("#{}", hex.to_ascii_lowercase()));
    }
    serde_json::to_string(&normalized).map_err(|error| {
        validation_error(format!("custom_theme_colors serialization failed: {error}"))
    })
}

fn default_true() -> bool {
    true
}

fn default_sound_volume() -> f64 {
    0.5
}

fn default_ui_language() -> String {
    "en".to_string()
}

fn default_practice_language() -> String {
    "en".to_string()
}

fn default_keyboard_layout() -> String {
    "qwerty".to_string()
}

fn valid_keyboard_layout(value: &str) -> bool {
    matches!(value, "qwerty" | "jcuken" | "dvorak")
}

fn default_daily_goal_type() -> String {
    "time".to_string()
}

fn default_pomodoro_work_min() -> i64 {
    25
}

fn default_pomodoro_break_min() -> i64 {
    5
}

const MIN_FONT_SIZE: i64 = 12;
const MAX_FONT_SIZE: i64 = 72;
const MAX_DAILY_GOAL_WPM: f64 = 300.0;
const MAX_DAILY_GOAL_ACCURACY: f64 = 100.0;
const MAX_DAILY_GOAL_MINUTES: i64 = 1_440;
const MAX_POMODORO_MINUTES: i64 = 180;
const MAX_CUSTOM_THEME_JSON_CHARS: usize = 16_384;

/// Белый список CSS-переменных, которые может задавать кастомная тема.
/// Совпадает с семантическим контрактом тем (docs/THEMES.md).
const CUSTOM_THEME_VARIABLES: &[&str] = &[
    "--color-app-background",
    "--color-surface-primary",
    "--color-surface-raised",
    "--color-surface-hover",
    "--color-surface-active",
    "--color-text-primary",
    "--color-text-secondary",
    "--color-text-muted",
    "--color-text-disabled",
    "--color-border",
    "--color-border-strong",
    "--color-accent",
    "--color-accent-hover",
    "--color-accent-active",
    "--color-accent-text",
    "--color-focus-ring",
    "--color-selection",
    "--color-caret",
    "--color-typing-pending",
    "--color-typing-current",
    "--color-typing-correct",
    "--color-typing-incorrect",
    "--color-typing-corrected",
    "--color-key-background",
    "--color-key-border",
    "--color-key-active",
    "--color-key-pressed",
    "--color-success",
    "--color-warning",
    "--color-error",
    "--color-information",
    "--color-chart-primary",
    "--color-chart-secondary",
    "--color-chart-positive",
    "--color-chart-negative",
    "--color-chart-grid",
    "--color-chart-axis",
    "--color-chart-label",
    "--color-chart-tooltip-background",
    "--color-chart-tooltip-border",
    "--color-chart-selected",
    "--color-progress-track",
    "--color-progress-fill",
    "--color-overlay",
    "--color-modal-surface",
    "--color-tooltip-surface",
    "--color-scrollbar",
    "--color-scrollbar-hover",
];
static SETTINGS_TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn validation_error(message: impl Into<String>) -> DbError {
    DbError::Validation(message.into())
}

fn valid_caret_style(value: &str) -> bool {
    // thin/thick/bubble/off — актуальные стили. underline/solid/block —
    // legacy-значения из старых настроек: принимаются, фронтенд маппит их
    // на новые рендеры (underline→thin, solid→thick, block→bubble).
    matches!(
        value,
        "thin" | "thick" | "bubble" | "off" | "underline" | "block" | "solid"
    )
}

fn valid_daily_goal_type(value: &str) -> bool {
    matches!(value, "time" | "wpm" | "accuracy")
}

fn valid_language_code(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 16
        && value
            .chars()
            .all(|character| character.is_ascii_lowercase() || character == '-')
}

fn integer_value(value: &toml::Value, key: &str) -> Result<i64, DbError> {
    value
        .as_integer()
        .ok_or_else(|| validation_error(format!("{key} must be an integer")))
}

fn number_value(value: &toml::Value, key: &str) -> Result<f64, DbError> {
    value
        .as_float()
        .or_else(|| value.as_integer().map(|number| number as f64))
        .filter(|number| number.is_finite())
        .ok_or_else(|| validation_error(format!("{key} must be a finite number")))
}

fn string_value<'a>(value: &'a toml::Value, key: &str) -> Result<&'a str, DbError> {
    value
        .as_str()
        .ok_or_else(|| validation_error(format!("{key} must be a string")))
}

fn boolean_value(value: &toml::Value, key: &str) -> Result<bool, DbError> {
    value
        .as_bool()
        .ok_or_else(|| validation_error(format!("{key} must be a boolean")))
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            font_size: 24,
            caret_style: "underline".to_string(),
            caret_position: "before".to_string(),
            caret_animation: "blink".to_string(),
            show_live_wpm: true,
            show_accuracy: true,
            show_keyboard_trainer: true,
            show_hand_guide: true,
            show_layout_warnings: true,
            show_capslock_warnings: true,
            sound_enabled: false,
            verbose_logging: false,
            sound_volume: 0.5,
            zen_mode_enabled: false,
            blind_mode_enabled: false,
            ui_language: "en".to_string(),
            practice_language: "en".to_string(),
            onboarding_completed: false,
            keyboard_layout: "qwerty".to_string(),
            vim_mode: false,
            daily_goal_type: "time".to_string(),
            daily_goal_wpm: 0.0,
            daily_goal_accuracy: 0.0,
            daily_goal_minutes: 0,
            pomodoro_work_min: 25,
            pomodoro_break_min: 5,
            custom_theme_colors: String::new(),
        }
    }
}

/// SettingsStore — загрузка/сохранение settings.toml.
pub struct SettingsStore {
    path: PathBuf,
}

impl SettingsStore {
    /// Создаёт SettingsStore с путём к settings.toml.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Загружает настройки. Если файл не существует — создаёт с дефолтными.
    pub fn load(&self) -> Result<AppSettings, DbError> {
        if !self.path.exists() {
            let default = AppSettings::default();
            self.save(&default)?;
            return Ok(default);
        }

        let content = std::fs::read_to_string(&self.path)
            .map_err(|e| DbError::Connection(format!("Failed to read settings.toml: {}", e)))?;

        let mut settings: AppSettings = toml::from_str(&content)
            .map_err(|e| DbError::Connection(format!("Failed to parse settings.toml: {}", e)))?;

        let normalized_theme = normalize_theme(&settings.theme);
        if normalized_theme != settings.theme {
            settings.theme = normalized_theme;
            self.save(&settings)?;
        }

        Ok(settings)
    }

    /// Сохраняет настройки в settings.toml.
    pub fn save(&self, settings: &AppSettings) -> Result<(), DbError> {
        // Создаём родительскую директорию если не существует
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| DbError::Write(format!("Failed to create config dir: {}", e)))?;
        }

        let content = toml::to_string(settings)
            .map_err(|e| DbError::Write(format!("Failed to serialize settings: {}", e)))?;

        let file_name = self
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| DbError::Write("settings path has no valid file name".to_string()))?;
        let sequence = SETTINGS_TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let temporary_path = self.path.with_file_name(format!(
            ".{file_name}.{}.{}.tmp",
            std::process::id(),
            sequence
        ));

        let write_result = (|| -> Result<(), DbError> {
            let mut temporary = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&temporary_path)
                .map_err(|error| {
                    DbError::Write(format!("Failed to create temporary settings file: {error}"))
                })?;
            temporary.write_all(content.as_bytes()).map_err(|error| {
                DbError::Write(format!("Failed to write temporary settings file: {error}"))
            })?;
            temporary.sync_all().map_err(|error| {
                DbError::Write(format!("Failed to sync temporary settings file: {error}"))
            })?;
            std::fs::rename(&temporary_path, &self.path).map_err(|error| {
                DbError::Write(format!("Failed to replace settings.toml: {error}"))
            })?;
            Ok(())
        })();

        if write_result.is_err() {
            let _ = std::fs::remove_file(&temporary_path);
        }
        write_result?;

        Ok(())
    }

    /// Обновляет одну настройку по ключу и сохраняет.
    pub fn set(&self, key: &str, value: toml::Value) -> Result<AppSettings, DbError> {
        let mut settings = self.load()?;

        match key {
            "theme" => {
                let value = string_value(&value, key)?;
                let normalized = normalize_theme(value);
                if normalized != value {
                    return Err(validation_error(format!("Unsupported theme: {value}")));
                }
                settings.theme = normalized;
            }
            "font_size" => {
                let value = integer_value(&value, key)?;
                if !(MIN_FONT_SIZE..=MAX_FONT_SIZE).contains(&value) {
                    return Err(validation_error(format!(
                        "font_size must be between {MIN_FONT_SIZE} and {MAX_FONT_SIZE}"
                    )));
                }
                settings.font_size = value;
            }
            "caret_style" => {
                let value = string_value(&value, key)?;
                if !valid_caret_style(value) {
                    return Err(validation_error(format!(
                        "Unsupported caret style: {value}"
                    )));
                }
                settings.caret_style = value.to_string();
            }
            "caret_position" => {
                let value = string_value(&value, key)?;
                if !valid_caret_position(value) {
                    return Err(validation_error(format!(
                        "Unsupported caret position: {value}"
                    )));
                }
                settings.caret_position = value.to_string();
            }
            "caret_animation" => {
                let value = string_value(&value, key)?;
                if !valid_caret_animation(value) {
                    return Err(validation_error(format!(
                        "Unsupported caret animation: {value}"
                    )));
                }
                settings.caret_animation = value.to_string();
            }
            "show_live_wpm" => {
                settings.show_live_wpm = boolean_value(&value, key)?;
            }
            "show_accuracy" => {
                settings.show_accuracy = boolean_value(&value, key)?;
            }
            "show_keyboard_trainer" => {
                settings.show_keyboard_trainer = boolean_value(&value, key)?;
            }
            "show_hand_guide" => {
                settings.show_hand_guide = boolean_value(&value, key)?;
            }
            "show_layout_warnings" => {
                settings.show_layout_warnings = boolean_value(&value, key)?;
            }
            "show_capslock_warnings" => {
                settings.show_capslock_warnings = boolean_value(&value, key)?;
            }
            "sound_enabled" => {
                settings.sound_enabled = boolean_value(&value, key)?;
            }
            "verbose_logging" => {
                settings.verbose_logging = boolean_value(&value, key)?;
            }
            "sound_volume" => {
                let value = number_value(&value, key)?;
                if !(0.0..=1.0).contains(&value) {
                    return Err(validation_error("sound_volume must be between 0 and 1"));
                }
                settings.sound_volume = value;
            }
            "zen_mode_enabled" => {
                settings.zen_mode_enabled = boolean_value(&value, key)?;
            }
            "blind_mode_enabled" => {
                settings.blind_mode_enabled = boolean_value(&value, key)?;
            }
            "ui_language" => {
                let value = string_value(&value, key)?;
                if !valid_language_code(value) {
                    return Err(validation_error(
                        "ui_language must be a supported language code",
                    ));
                }
                settings.ui_language = value.to_string();
            }
            "practice_language" => {
                let value = string_value(&value, key)?;
                if !valid_language_code(value) {
                    return Err(validation_error(
                        "practice_language must be a supported language code",
                    ));
                }
                settings.practice_language = value.to_string();
            }
            "onboarding_completed" => {
                settings.onboarding_completed = boolean_value(&value, key)?;
            }
            "keyboard_layout" => {
                let value = string_value(&value, key)?;
                if !valid_keyboard_layout(value) {
                    return Err(validation_error(
                        "keyboard_layout must be one of: qwerty, jcuken, dvorak",
                    ));
                }
                settings.keyboard_layout = value.to_string();
            }
            "vim_mode" => {
                settings.vim_mode = boolean_value(&value, key)?;
            }
            "daily_goal_type" => {
                let value = string_value(&value, key)?;
                if !valid_daily_goal_type(value) {
                    return Err(validation_error(format!(
                        "Unsupported daily goal type: {value}"
                    )));
                }
                settings.daily_goal_type = value.to_string();
            }
            "daily_goal_wpm" => {
                let value = number_value(&value, key)?;
                if !(0.0..=MAX_DAILY_GOAL_WPM).contains(&value) {
                    return Err(validation_error(format!(
                        "daily_goal_wpm must be between 0 and {MAX_DAILY_GOAL_WPM}"
                    )));
                }
                settings.daily_goal_wpm = value;
            }
            "daily_goal_accuracy" => {
                let value = number_value(&value, key)?;
                if !(0.0..=MAX_DAILY_GOAL_ACCURACY).contains(&value) {
                    return Err(validation_error(format!(
                        "daily_goal_accuracy must be between 0 and {MAX_DAILY_GOAL_ACCURACY}"
                    )));
                }
                settings.daily_goal_accuracy = value;
            }
            "daily_goal_minutes" => {
                let value = integer_value(&value, key)?;
                if !(0..=MAX_DAILY_GOAL_MINUTES).contains(&value) {
                    return Err(validation_error(format!(
                        "daily_goal_minutes must be between 0 and {MAX_DAILY_GOAL_MINUTES}"
                    )));
                }
                settings.daily_goal_minutes = value;
            }
            "pomodoro_work_min" => {
                let value = integer_value(&value, key)?;
                if !(1..=MAX_POMODORO_MINUTES).contains(&value) {
                    return Err(validation_error(format!(
                        "pomodoro_work_min must be between 1 and {MAX_POMODORO_MINUTES}"
                    )));
                }
                settings.pomodoro_work_min = value;
            }
            "pomodoro_break_min" => {
                let value = integer_value(&value, key)?;
                if !(1..=MAX_POMODORO_MINUTES).contains(&value) {
                    return Err(validation_error(format!(
                        "pomodoro_break_min must be between 1 and {MAX_POMODORO_MINUTES}"
                    )));
                }
                settings.pomodoro_break_min = value;
            }
            "custom_theme_colors" => {
                let value = string_value(&value, key)?;
                settings.custom_theme_colors = validate_custom_theme_json(value)?;
            }
            _ => {
                return Err(validation_error(format!("Unknown setting key: {key}")));
            }
        }

        self.save(&settings)?;
        Ok(settings)
    }

    /// Возвращает путь к settings.toml.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_settings_path() -> PathBuf {
        let dir = std::env::temp_dir();
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = dir.join(format!(
            "racoon_test_settings_{}_{}.toml",
            std::process::id(),
            ts
        ));
        std::fs::remove_file(&path).ok();
        path
    }

    #[test]
    fn load_nonexistent_creates_default() {
        let path = temp_settings_path();
        let store = SettingsStore::new(path.clone());

        let settings = store.load().unwrap();
        assert_eq!(settings.theme, "racoon_graphite");
        assert_eq!(settings.font_size, 24);
        assert!(settings.show_live_wpm);

        // Файл создан
        assert!(path.exists());

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn save_and_reload() {
        let path = temp_settings_path();
        let store = SettingsStore::new(path.clone());

        let settings = AppSettings {
            theme: "racoon_graphite".to_string(),
            font_size: 28,
            ..AppSettings::default()
        };
        store.save(&settings).unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded.theme, "racoon_graphite");
        assert_eq!(loaded.font_size, 28);

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn set_theme() {
        let path = temp_settings_path();
        let store = SettingsStore::new(path.clone());

        let settings = store
            .set("theme", toml::Value::String("racoon_warm".to_string()))
            .unwrap();
        assert_eq!(settings.theme, "racoon_warm");

        let loaded = store.load().unwrap();
        assert_eq!(loaded.theme, "racoon_warm");

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn legacy_theme_aliases_load_to_their_current_ids() {
        for (legacy_theme, expected_theme) in [
            ("racoon_dark", "racoon_graphite"),
            ("racoon_light", "racoon_silver"),
        ] {
            let path = temp_settings_path();
            std::fs::write(&path, format!("theme = \"{legacy_theme}\"\n")).unwrap();

            let settings = SettingsStore::new(path.clone()).load().unwrap();
            assert_eq!(settings.theme, expected_theme);

            std::fs::remove_file(&path).ok();
        }
    }

    #[test]
    fn unknown_persisted_theme_loads_as_graphite() {
        let path = temp_settings_path();
        std::fs::write(&path, "theme = \"removed_theme\"\n").unwrap();

        let settings = SettingsStore::new(path.clone()).load().unwrap();
        assert_eq!(settings.theme, "racoon_graphite");

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn every_current_theme_id_survives_save_and_reload() {
        const BUILT_IN_THEME_IDS: &[&str] = &[
            "racoon_graphite",
            "racoon_silver",
            "racoon_warm",
            "racoon_high_contrast",
            "midnight_ink",
            "arctic_slate",
            "lavender_dusk",
            "plum",
            "ocean",
            "deep_sea",
            "steel_blue",
            "carbon",
            "moonlight",
            "racoon_forest",
            "moss",
            "sage",
            "coffee",
            "ember",
            "burgundy",
            "paper",
            "sandstone",
            "mist",
            "dawn",
            "amber_terminal",
            "green_terminal",
        ];

        assert_eq!(BUILT_IN_THEME_IDS.len(), 25);
        let path = temp_settings_path();
        let store = SettingsStore::new(path.clone());

        for theme_id in BUILT_IN_THEME_IDS {
            let saved = store
                .set("theme", toml::Value::String((*theme_id).to_string()))
                .unwrap();
            assert_eq!(saved.theme, *theme_id);
            assert_eq!(store.load().unwrap().theme, *theme_id);
        }

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn set_font_size() {
        let path = temp_settings_path();
        let store = SettingsStore::new(path.clone());

        let settings = store.set("font_size", toml::Value::Integer(32)).unwrap();
        assert_eq!(settings.font_size, 32);

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn set_caret_style() {
        let path = temp_settings_path();
        let store = SettingsStore::new(path.clone());

        let settings = store
            .set("caret_style", toml::Value::String("block".to_string()))
            .unwrap();
        assert_eq!(settings.caret_style, "block");

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn caret_animation_defaults_to_blink_and_validates() {
        let path = temp_settings_path();
        let store = SettingsStore::new(path.clone());

        assert_eq!(store.load().unwrap().caret_animation, "blink");
        let settings = store
            .set("caret_animation", toml::Value::String("pulse".to_string()))
            .unwrap();
        assert_eq!(settings.caret_animation, "pulse");
        assert_eq!(store.load().unwrap().caret_animation, "pulse");
        assert!(store
            .set("caret_animation", toml::Value::String("spin".to_string()))
            .is_err());

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn set_show_live_wpm() {
        let path = temp_settings_path();
        let store = SettingsStore::new(path.clone());

        let settings = store
            .set("show_live_wpm", toml::Value::Boolean(false))
            .unwrap();
        assert!(!settings.show_live_wpm);

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn verbose_logging_is_disabled_by_default_and_can_be_enabled() {
        let path = temp_settings_path();
        let store = SettingsStore::new(path.clone());

        assert!(!store.load().unwrap().verbose_logging);
        let settings = store
            .set("verbose_logging", toml::Value::Boolean(true))
            .unwrap();
        assert!(settings.verbose_logging);
        assert!(store.load().unwrap().verbose_logging);

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn practice_language_roundtrips_and_validates_format() {
        let path = temp_settings_path();
        let store = SettingsStore::new(path.clone());

        assert_eq!(store.load().unwrap().practice_language, "en");
        let settings = store
            .set("practice_language", toml::Value::String("ru".to_string()))
            .unwrap();
        assert_eq!(settings.practice_language, "ru");
        assert_eq!(store.load().unwrap().practice_language, "ru");

        for invalid in ["", "RU", "ru_RU", "russian language!"] {
            assert!(store
                .set(
                    "practice_language",
                    toml::Value::String(invalid.to_string())
                )
                .is_err());
        }
        assert!(store
            .set("practice_language", toml::Value::Boolean(true))
            .is_err());

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn onboarding_completed_persists_and_defaults_to_false() {
        let path = temp_settings_path();
        let store = SettingsStore::new(path.clone());

        assert!(!store.load().unwrap().onboarding_completed);
        let settings = store
            .set("onboarding_completed", toml::Value::Boolean(true))
            .unwrap();
        assert!(settings.onboarding_completed);
        assert!(store.load().unwrap().onboarding_completed);
        assert!(store
            .set(
                "onboarding_completed",
                toml::Value::String("yes".to_string())
            )
            .is_err());

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn keyboard_layout_roundtrips_and_rejects_unknown_values() {
        let path = temp_settings_path();
        let store = SettingsStore::new(path.clone());

        assert_eq!(store.load().unwrap().keyboard_layout, "qwerty");
        for layout in ["jcuken", "dvorak"] {
            let settings = store
                .set("keyboard_layout", toml::Value::String(layout.to_string()))
                .unwrap();
            assert_eq!(settings.keyboard_layout, layout);
        }
        assert_eq!(store.load().unwrap().keyboard_layout, "dvorak");
        for invalid in ["", "colemak", "QWERTY"] {
            assert!(store
                .set("keyboard_layout", toml::Value::String(invalid.to_string()))
                .is_err());
        }

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn set_unknown_key_fails() {
        let path = temp_settings_path();
        let store = SettingsStore::new(path.clone());

        let result = store.set("unknown_key", toml::Value::String("value".to_string()));
        assert!(result.is_err());

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn set_rejects_wrong_value_type_without_overwriting_settings() {
        let path = temp_settings_path();
        let store = SettingsStore::new(path.clone());
        let original = store.load().unwrap();

        let error = store
            .set("sound_volume", toml::Value::String("loud".to_string()))
            .unwrap_err();
        assert!(matches!(error, DbError::Validation(_)));
        assert_eq!(store.load().unwrap(), original);

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn set_accepts_integer_numeric_values_for_numeric_settings() {
        let path = temp_settings_path();
        let store = SettingsStore::new(path.clone());

        let settings = store
            .set("daily_goal_wpm", toml::Value::Integer(65))
            .unwrap();
        assert_eq!(settings.daily_goal_wpm, 65.0);

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn pomodoro_settings_default_and_update() {
        let path = temp_settings_path();
        let store = SettingsStore::new(path.clone());

        let defaults = store.load().unwrap();
        assert_eq!(defaults.pomodoro_work_min, 25);
        assert_eq!(defaults.pomodoro_break_min, 5);

        let settings = store
            .set("pomodoro_work_min", toml::Value::Integer(50))
            .unwrap();
        assert_eq!(settings.pomodoro_work_min, 50);
        let settings = store
            .set("pomodoro_break_min", toml::Value::Integer(10))
            .unwrap();
        assert_eq!(settings.pomodoro_break_min, 10);
        assert_eq!(store.load().unwrap().pomodoro_work_min, 50);

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn pomodoro_settings_reject_out_of_range() {
        let path = temp_settings_path();
        let store = SettingsStore::new(path.clone());

        assert!(store
            .set("pomodoro_work_min", toml::Value::Integer(0))
            .is_err());
        assert!(store
            .set("pomodoro_break_min", toml::Value::Integer(181))
            .is_err());

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn custom_theme_colors_accepts_whitelisted_hex_json() {
        let path = temp_settings_path();
        let store = SettingsStore::new(path.clone());

        let json = r##"{"--color-app-background":"#0d0f12","--color-accent":"#C5CBD4"}"##;
        let settings = store
            .set("custom_theme_colors", toml::Value::String(json.to_string()))
            .unwrap();
        // Нормализация: lowercase hex, отсортированные ключи.
        assert!(settings.custom_theme_colors.contains("\"#c5cbd4\""));
        assert_eq!(
            store.load().unwrap().custom_theme_colors,
            settings.custom_theme_colors
        );

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn custom_theme_colors_rejects_unknown_keys_and_bad_hex() {
        let path = temp_settings_path();
        let store = SettingsStore::new(path.clone());

        assert!(store
            .set(
                "custom_theme_colors",
                toml::Value::String(r##"{"--evil":"#000000"}"##.to_string()),
            )
            .is_err());
        assert!(store
            .set(
                "custom_theme_colors",
                toml::Value::String(r#"{"--color-accent":"red"}"#.to_string()),
            )
            .is_err());
        assert!(store
            .set(
                "custom_theme_colors",
                toml::Value::String("not json".to_string()),
            )
            .is_err());
        // Пустая строка — валидный сброс.
        let settings = store
            .set("custom_theme_colors", toml::Value::String(String::new()))
            .unwrap();
        assert_eq!(settings.custom_theme_colors, "");

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn custom_theme_id_survives_save_and_reload() {
        let path = temp_settings_path();
        let store = SettingsStore::new(path.clone());

        let settings = store
            .set("theme", toml::Value::String("custom".to_string()))
            .unwrap();
        assert_eq!(settings.theme, "custom");
        assert_eq!(store.load().unwrap().theme, "custom");

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn default_values() {
        let settings = AppSettings::default();
        assert_eq!(settings.theme, "racoon_graphite");
        assert_eq!(settings.font_size, 24);
        assert_eq!(settings.caret_style, "underline");
        assert!(settings.show_live_wpm);
        assert!(settings.show_accuracy);
    }

    #[test]
    fn serialization_roundtrip() {
        let settings = AppSettings {
            theme: "racoon_graphite".to_string(),
            font_size: 30,
            caret_style: "solid".to_string(),
            caret_position: "after".to_string(),
            caret_animation: "pulse".to_string(),
            show_live_wpm: false,
            show_accuracy: true,
            show_keyboard_trainer: true,
            show_hand_guide: true,
            show_layout_warnings: true,
            show_capslock_warnings: true,
            sound_enabled: false,
            verbose_logging: false,
            sound_volume: 0.5,
            zen_mode_enabled: false,
            blind_mode_enabled: false,
            ui_language: "ru".to_string(),
            practice_language: "de".to_string(),
            onboarding_completed: true,
            keyboard_layout: "dvorak".to_string(),
            vim_mode: true,
            daily_goal_type: "time".to_string(),
            daily_goal_wpm: 0.0,
            daily_goal_accuracy: 0.0,
            daily_goal_minutes: 0,
            pomodoro_work_min: 25,
            pomodoro_break_min: 5,
            custom_theme_colors: String::new(),
        };

        let toml_str = toml::to_string(&settings).unwrap();
        let deserialized: AppSettings = toml::from_str(&toml_str).unwrap();

        assert_eq!(deserialized.theme, "racoon_graphite");
        assert_eq!(deserialized.font_size, 30);
        assert_eq!(deserialized.caret_style, "solid");
        assert_eq!(deserialized.practice_language, "de");
        assert!(deserialized.onboarding_completed);
        assert_eq!(deserialized.keyboard_layout, "dvorak");
        assert!(!deserialized.show_live_wpm);
        assert!(deserialized.show_accuracy);
    }

    #[test]
    fn legacy_settings_loads_with_missing_defaults() {
        let legacy = r#"
theme = "legacy_theme"
font_size = 24
caret_style = "underline"
show_live_wpm = true
show_accuracy = true
"#;

        let settings: AppSettings = toml::from_str(legacy).unwrap();

        assert_eq!(normalize_theme(&settings.theme), "racoon_graphite");
        assert!(!settings.sound_enabled);
        assert!(!settings.verbose_logging);
        assert_eq!(settings.sound_volume, 0.5);
        assert!(!settings.zen_mode_enabled);
        assert!(settings.show_keyboard_trainer);
        assert!(settings.show_hand_guide);
        assert!(settings.show_layout_warnings);
        assert!(settings.show_capslock_warnings);
    }

    #[test]
    fn persistence_across_instances() {
        let path = temp_settings_path();

        // Первый instance: сохраняем
        let store1 = SettingsStore::new(path.clone());
        store1.set("font_size", toml::Value::Integer(48)).unwrap();

        // Второй instance: загружаем
        let store2 = SettingsStore::new(path.clone());
        let settings = store2.load().unwrap();
        assert_eq!(settings.font_size, 48);

        std::fs::remove_file(&path).ok();
    }
}
