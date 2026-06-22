//! Настройки приложения (хранятся в settings.toml).

use serde::{Deserialize, Serialize};

use crate::lesson::Difficulty;

/// Полные настройки приложения.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    pub theme: ThemeConfig,
    pub font: FontConfig,
    pub caret: CaretConfig,
    pub behavior: BehaviorConfig,
    pub default_mode: DefaultModeConfig,
    pub language: LanguageConfig,
    pub layout: LayoutConfig,
    pub daily_goal: DailyGoalConfig,
    pub difficulty: DifficultyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub name: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: "serika_dark".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    pub family: String,
    pub size: u32,
    pub line_height: f64,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            family: "JetBrains Mono".to_string(),
            size: 24,
            line_height: 1.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaretConfig {
    pub style: String,
    pub smooth: bool,
    pub blink: bool,
}

impl Default for CaretConfig {
    fn default() -> Self {
        Self {
            style: "underline".to_string(),
            smooth: true,
            blink: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BehaviorConfig {
    pub quick_tab: bool,
    pub quick_restart: bool,
    pub auto_accept_typos: bool,
    pub show_all_lines: bool,
    pub smooth_scroll: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultModeConfig {
    #[serde(rename = "type")]
    pub mode_type: String,
    pub duration: u32,
    pub word_count: u32,
}

impl Default for DefaultModeConfig {
    fn default() -> Self {
        Self {
            mode_type: "time".to_string(),
            duration: 30,
            word_count: 25,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    pub primary: String,
    pub secondary: String,
}

impl Default for LanguageConfig {
    fn default() -> Self {
        Self {
            primary: "en".to_string(),
            secondary: "ru".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    pub name: String,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            name: "qwerty".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyGoalConfig {
    #[serde(rename = "type")]
    pub goal_type: String,
    pub value_minutes: u32,
}

impl Default for DailyGoalConfig {
    fn default() -> Self {
        Self {
            goal_type: "time".to_string(),
            value_minutes: 15,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DifficultyConfig {
    pub level: Difficulty,
}
