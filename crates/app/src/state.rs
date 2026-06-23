//! AppState — состояние приложения: Database + SettingsStore.

use racoon_data::repository::SettingsStore;
use racoon_data::Database;
use std::path::PathBuf;
use std::sync::Mutex;

/// Состояние приложения, доступное всем Tauri commands.
pub struct AppState {
    pub db: Mutex<Database>,
    settings_path: PathBuf,
    pub data_dir: PathBuf,
}

impl AppState {
    pub fn new(db: Database, settings_path: PathBuf, data_dir: PathBuf) -> Self {
        Self {
            db: Mutex::new(db),
            settings_path,
            data_dir,
        }
    }

    pub fn settings_store(&self) -> SettingsStore {
        SettingsStore::new(self.settings_path.clone())
    }
}
