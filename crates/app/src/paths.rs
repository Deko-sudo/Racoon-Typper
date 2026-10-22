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
    pub db_path: PathBuf,
    pub settings_path: PathBuf,
    pub profile_lock: ProfileLock,
}

/// Keeps this profile exclusive until application shutdown. The lock file is
/// retained: deleting it could let another process lock a different inode.
#[derive(Debug)]
pub struct ProfileLock {
    _file: std::fs::File,
}

impl ProfileLock {
    pub fn acquire(data_dir: &Path) -> std::io::Result<Self> {
        let mut options = std::fs::OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let file = options
            .open(data_dir.join(".profile.lock"))
            .map_err(|error| {
                std::io::Error::new(error.kind(), "Unable to open the local profile lock.")
            })?;
        file.try_lock().map_err(|error| match error {
            std::fs::TryLockError::WouldBlock => std::io::Error::new(
                std::io::ErrorKind::WouldBlock,
                "This profile is already open in Racoon Typper. Close its other window first.",
            ),
            std::fs::TryLockError::Error(error) => {
                std::io::Error::new(error.kind(), "Unable to lock the local profile.")
            }
        })?;
        Ok(Self { _file: file })
    }
}

/// Resolves Tauri-managed paths and preserves data from the verified Linux
/// baseline before opening SQLite. Existing destination files always win; the
/// migration copies rather than moves legacy files so rollback is non-destructive.
pub fn resolve(app: &App<Wry>) -> Result<AppPaths, Box<dyn std::error::Error>> {
    let data_dir = app.path().app_data_dir()?;
    let config_dir = app.path().app_config_dir()?;
    std::fs::create_dir_all(&data_dir)?;
    let profile_lock = ProfileLock::acquire(&data_dir)?;
    std::fs::create_dir_all(&config_dir)?;

    let db_path = data_dir.join("data.db");
    let settings_path = config_dir.join("settings.toml");

    #[cfg(target_os = "linux")]
    migrate_linux_baseline_paths(&db_path, &settings_path)?;

    Ok(AppPaths {
        data_dir,
        db_path,
        settings_path,
        profile_lock,
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
    struct ProfileDirectory(std::path::PathBuf);
    impl ProfileDirectory {
        fn new(name: &str) -> Self {
            use std::sync::atomic::{AtomicU64, Ordering};
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "racoon-profile-lock-{name}-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&path).expect("isolated profile directory");
            Self(path)
        }
    }
    impl Drop for ProfileDirectory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn audit_profile_lock_rejects_a_second_owner_and_releases_on_drop() {
        let directory = ProfileDirectory::new("lifecycle");
        let first = super::ProfileLock::acquire(&directory.0).expect("first profile owner");
        let error =
            super::ProfileLock::acquire(&directory.0).expect_err("second owner must be rejected");
        assert_eq!(error.kind(), std::io::ErrorKind::WouldBlock);
        assert!(!error
            .to_string()
            .contains(directory.0.to_str().expect("fixture path")));
        drop(first);
        assert!(
            directory.0.join(".profile.lock").is_file(),
            "retain the lock inode"
        );
        let _next =
            super::ProfileLock::acquire(&directory.0).expect("next profile owner after drop");
    }

    #[test]
    fn audit_profile_lock_prevents_a_second_start_from_interrupting_a_live_session() {
        use racoon_application::{
            SessionRecoveryLedger, StartedSession, StartupRecoveryCoordinator, StartupRecoveryGate,
            StartupRecoveryRetryPolicy,
        };
        use racoon_data::{
            Database, DbError, SqliteFinalizationLedger, SqliteSessionFinalizer,
            SqliteSessionRecoveryLedger,
        };
        let directory = ProfileDirectory::new("recovery");
        let _first_lock = super::ProfileLock::acquire(&directory.0).expect("first owner");
        let path = directory.0.join("data.db");
        let first = Database::open(&path).expect("first connection");
        let id =
            racoon_domain::SessionId::parse("018f0c2e-7b8d-7abc-8def-0123456789bb").expect("id");
        let started = StartedSession::new(
            id.clone(),
            "custom",
            serde_json::json!({"language":"en"}),
            "en",
            "2026-10-16T12:00:00Z".parse().expect("time"),
        )
        .expect("started");
        SqliteSessionRecoveryLedger::new(&first)
            .record_started(&started)
            .expect("running session");
        if let Ok(_second_lock) = super::ProfileLock::acquire(&directory.0) {
            let second = Database::open(&path).expect("second connection");
            let recovery = SqliteSessionRecoveryLedger::new(&second);
            let finalizations = SqliteFinalizationLedger::new(&second);
            let finalizer = SqliteSessionFinalizer::new(&second);
            StartupRecoveryCoordinator::new(
                &recovery,
                &finalizations,
                &finalizer,
                &crate::SystemRecoveryClock,
                &crate::ThreadStartupRecoverySleeper,
                StartupRecoveryRetryPolicy::default(),
            )
            .run(&StartupRecoveryGate::new())
            .expect("second startup recovery");
        }
        let observed: String = first
            .with_connection(|conn| {
                conn.query_row(
                    "SELECT state FROM session_ledger WHERE session_id = ?1",
                    [id.as_str()],
                    |row| row.get(0),
                )
                .map_err(|error| DbError::Query(error.to_string()))
            })
            .expect("first connection sees its live session");
        assert_eq!(observed, "running");
    }

    #[test]
    fn profile_lock_child_helper() {
        let Some(directory) = std::env::var_os("RACOON_TEST_PROFILE_LOCK_DIR") else {
            return;
        };
        let directory = std::path::PathBuf::from(directory);
        let _lock = super::ProfileLock::acquire(&directory).expect("child profile owner");
        std::fs::write(directory.join("child-ready"), "locked").expect("child readiness");
        loop {
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }

    #[test]
    fn audit_profile_lock_is_released_after_terminating_its_process() {
        struct ChildOwner(std::process::Child);
        impl Drop for ChildOwner {
            fn drop(&mut self) {
                let _ = self.0.kill();
                let _ = self.0.wait();
            }
        }
        let directory = ProfileDirectory::new("process-exit");
        let child = std::process::Command::new(std::env::current_exe().expect("test executable"))
            .args([
                "--exact",
                "paths::tests::profile_lock_child_helper",
                "--nocapture",
            ])
            .env("RACOON_TEST_PROFILE_LOCK_DIR", &directory.0)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("child process");
        let mut child = ChildOwner(child);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        while !directory.0.join("child-ready").exists() {
            assert!(
                std::time::Instant::now() < deadline,
                "child must acquire the lock"
            );
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let error = super::ProfileLock::acquire(&directory.0).expect_err("child excludes parent");
        assert_eq!(error.kind(), std::io::ErrorKind::WouldBlock);
        child.0.kill().expect("terminate child without Rust drop");
        child.0.wait().expect("wait for termination");
        let _next = super::ProfileLock::acquire(&directory.0)
            .expect("OS releases the terminated owner's lock");
    }
}
