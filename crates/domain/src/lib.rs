//! Domain crate — чистые типы данных, zero-logic.
//! Shared kernel для всех crates.

pub mod engine;
pub mod ids;
pub mod keyboard;
pub mod lesson;
pub mod settings;
pub mod stats;
pub mod test;
pub mod theme;

// Re-exports для удобства
pub use engine::*;
pub use ids::*;
pub use keyboard::*;
pub use lesson::*;
pub use settings::*;
pub use stats::*;
pub use test::*;
pub use theme::*;

/// Версия API контракта.
pub const API_VERSION: &str = "0.1.0";

/// Информация о приложении.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AppInfo {
    pub version: String,
    pub build_profile: String,
    pub data_dir: String,
    pub config_dir: String,
    pub db_path: String,
    pub settings_path: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_info_serializes() {
        let info = AppInfo {
            version: "0.1.0".to_string(),
            build_profile: "debug".to_string(),
            data_dir: "/tmp/data".to_string(),
            config_dir: "/tmp/config".to_string(),
            db_path: "/tmp/data/data.db".to_string(),
            settings_path: "/tmp/config/settings.toml".to_string(),
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("0.1.0"));
    }

    #[test]
    fn api_version_is_set() {
        assert_eq!(API_VERSION, "0.1.0");
    }
}
