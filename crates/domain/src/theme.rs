//! Темы приложения.

use serde::{Deserialize, Serialize};

/// Список тем.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeList {
    pub themes: Vec<ThemeInfo>,
}

/// Информация о теме (manifest).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeInfo {
    pub name: String,
    pub display_name: String,
    pub author: String,
    pub version: String,
    pub description: String,
    pub is_dark: bool,
    pub preview_colors: ThemePreviewColors,
}

/// Цвета для превью темы.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemePreviewColors {
    pub bg: String,
    pub main: String,
    pub caret: String,
    pub sub: String,
    pub text: String,
    pub error: String,
}
