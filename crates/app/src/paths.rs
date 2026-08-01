// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

//! Platform-managed application paths with a one-way Linux legacy-path migration.

#[cfg(target_os = "linux")]
use std::env;
use std::path::{Path, PathBuf};

use tauri::{App, Manager, Wry};

/// Paths owned by the running application instance.
pub struct AppPaths {
    pub data_dir: PathBuf,
    pub config_dir: PathBuf,
    pub db_path: PathBuf,
    pub settings_path: PathBuf,
}

/// Resolves Tauri-managed paths and preserves data from the verified Linux
/// baseline before opening SQLite. Existing destination files always win; the
/// migration copies rather than moves legacy files so rollback is non-destructive.
pub fn resolve(app: &App<Wry>) -> Result<AppPaths, Box<dyn std::error::Error>> {
    let data_dir = app.path().app_data_dir()?;
    let config_dir = app.path().app_config_dir()?;
    std::fs::create_dir_all(&data_dir)?;
    std::fs::create_dir_all(&config_dir)?;

    let db_path = data_dir.join("data.db");
    let settings_path = config_dir.join("settings.toml");

    #[cfg(target_os = "linux")]
    migrate_linux_baseline_paths(&db_path, &settings_path)?;

    Ok(AppPaths {
        data_dir,
        config_dir,
        db_path,
        settings_path,
    })
}

#[cfg(target_os = "linux")]
fn migrate_linux_baseline_paths(
    destination_database: &Path,
    destination_settings: &Path,
) -> std::io::Result<()> {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let legacy_data_dir = env::var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(&home).join(".local/share/racoon-typper"));
    let legacy_config_dir = env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(&home).join(".config/racoon-typper"));

    let copied_database = copy_if_missing(&legacy_data_dir.join("data.db"), destination_database)?;
    // A prior SQLite run may have uncheckpointed WAL state. Copy its companions
    // only when the main database was copied, so a newer destination is never
    // combined with stale legacy WAL files.
    if copied_database {
        copy_if_missing(
            &legacy_data_dir.join("data.db-wal"),
            &destination_database.with_file_name("data.db-wal"),
        )?;
        copy_if_missing(
            &legacy_data_dir.join("data.db-shm"),
            &destination_database.with_file_name("data.db-shm"),
        )?;
    }
    let _ = copy_if_missing(
        &legacy_config_dir.join("settings.toml"),
        destination_settings,
    )?;
    Ok(())
}

/// Copies a file only when its destination is absent. The temporary sibling plus
/// rename keeps an interrupted migration from exposing a partially copied file.
#[cfg(any(target_os = "linux", test))]
fn copy_if_missing(source: &Path, destination: &Path) -> std::io::Result<bool> {
    if destination.exists() || !source.exists() {
        return Ok(false);
    }

    let file_name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("racoon-typper-migration");
    let temporary =
        destination.with_file_name(format!(".{file_name}.migration-{}.tmp", std::process::id()));
    std::fs::copy(source, &temporary)?;
    if let Err(error) = std::fs::rename(&temporary, destination) {
        let _ = std::fs::remove_file(&temporary);
        return Err(error);
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::copy_if_missing;

    #[test]
    fn copy_if_missing_preserves_an_existing_destination() {
        let directory =
            std::env::temp_dir().join(format!("racoon-typper-paths-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let source = directory.join("source");
        let destination = directory.join("destination");
        std::fs::write(&source, "legacy").unwrap();
        std::fs::write(&destination, "current").unwrap();

        assert!(!copy_if_missing(&source, &destination).unwrap());
        assert_eq!(std::fs::read_to_string(&destination).unwrap(), "current");

        std::fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn copy_if_missing_copies_a_legacy_file_once() {
        let directory =
            std::env::temp_dir().join(format!("racoon-typper-paths-copy-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let source = directory.join("source");
        let destination = directory.join("destination");
        std::fs::write(&source, "legacy").unwrap();

        assert!(copy_if_missing(&source, &destination).unwrap());
        assert_eq!(std::fs::read_to_string(&destination).unwrap(), "legacy");
        assert!(!copy_if_missing(&source, &destination).unwrap());

        std::fs::remove_dir_all(&directory).unwrap();
    }
}
