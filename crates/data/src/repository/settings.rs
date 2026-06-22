//! SettingsStore — загрузка/сохранение настроек в settings.toml.

use std::path::{Path, PathBuf};

use crate::error::DbError;

/// Настройки приложения (подмножество для MVP Sprint 5).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppSettings {
    pub theme: String,
    pub font_size: u32,
    pub caret_style: String,
    pub show_live_wpm: bool,
    pub show_accuracy: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "serika_dark".to_string(),
            font_size: 24,
            caret_style: "underline".to_string(),
            show_live_wpm: true,
            show_accuracy: true,
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

        let settings: AppSettings = toml::from_str(&content)
            .map_err(|e| DbError::Connection(format!("Failed to parse settings.toml: {}", e)))?;

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

        std::fs::write(&self.path, content)
            .map_err(|e| DbError::Write(format!("Failed to write settings.toml: {}", e)))?;

        Ok(())
    }

    /// Обновляет одну настройку по ключу и сохраняет.
    pub fn set(&self, key: &str, value: toml::Value) -> Result<AppSettings, DbError> {
        let mut settings = self.load()?;

        match key {
            "theme" => {
                if let Some(v) = value.as_str() {
                    settings.theme = v.to_string();
                }
            }
            "font_size" => {
                if let Some(v) = value.as_integer() {
                    settings.font_size = v as u32;
                }
            }
            "caret_style" => {
                if let Some(v) = value.as_str() {
                    settings.caret_style = v.to_string();
                }
            }
            "show_live_wpm" => {
                if let Some(v) = value.as_bool() {
                    settings.show_live_wpm = v;
                }
            }
            "show_accuracy" => {
                if let Some(v) = value.as_bool() {
                    settings.show_accuracy = v;
                }
            }
            _ => {
                return Err(DbError::Write(format!("Unknown setting key: {}", key)));
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
        assert_eq!(settings.theme, "serika_dark");
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

        let mut settings = AppSettings::default();
        settings.theme = "racoon_dark".to_string();
        settings.font_size = 28;
        store.save(&settings).unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded.theme, "racoon_dark");
        assert_eq!(loaded.font_size, 28);

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn set_theme() {
        let path = temp_settings_path();
        let store = SettingsStore::new(path.clone());

        let settings = store
            .set("theme", toml::Value::String("serika_light".to_string()))
            .unwrap();
        assert_eq!(settings.theme, "serika_light");

        let loaded = store.load().unwrap();
        assert_eq!(loaded.theme, "serika_light");

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
    fn set_unknown_key_fails() {
        let path = temp_settings_path();
        let store = SettingsStore::new(path.clone());

        let result = store.set("unknown_key", toml::Value::String("value".to_string()));
        assert!(result.is_err());

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn default_values() {
        let settings = AppSettings::default();
        assert_eq!(settings.theme, "serika_dark");
        assert_eq!(settings.font_size, 24);
        assert_eq!(settings.caret_style, "underline");
        assert!(settings.show_live_wpm);
        assert!(settings.show_accuracy);
    }

    #[test]
    fn serialization_roundtrip() {
        let settings = AppSettings {
            theme: "racoon_dark".to_string(),
            font_size: 30,
            caret_style: "solid".to_string(),
            show_live_wpm: false,
            show_accuracy: true,
        };

        let toml_str = toml::to_string(&settings).unwrap();
        let deserialized: AppSettings = toml::from_str(&toml_str).unwrap();

        assert_eq!(deserialized.theme, "racoon_dark");
        assert_eq!(deserialized.font_size, 30);
        assert_eq!(deserialized.caret_style, "solid");
        assert!(!deserialized.show_live_wpm);
        assert!(deserialized.show_accuracy);
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
