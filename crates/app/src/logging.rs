// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

const LOG_DIRECTORY: &str = "logs";
const LOG_FILE: &str = "app.jsonl";
const PRE_MIGRATION_BACKUP_FAILURE_EVENT: &str = concat!(
    "{\"event\":\"pre_migration_backup_failed\",",
    "\"error_class\":\"io\",",
    "\"path_kind\":\"database\",",
    "\"path_file\":\"data.db\"}\n"
);

#[derive(Clone, Copy, Debug)]
pub(crate) enum ErrorClass {
    Io,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct LogRetention {
    max_files: usize,
    max_bytes_per_file: u64,
}

impl LogRetention {
    #[cfg(test)]
    pub(crate) fn new(max_files: usize, max_bytes_per_file: u64) -> Option<Self> {
        (max_files > 0 && max_bytes_per_file > 0).then_some(Self {
            max_files,
            max_bytes_per_file,
        })
    }
}

impl Default for LogRetention {
    fn default() -> Self {
        Self {
            max_files: 3,
            max_bytes_per_file: 256 * 1024,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum LocalLogger {
    Disabled,
    Enabled {
        directory: PathBuf,
        retention: LogRetention,
    },
}

impl LocalLogger {
    pub(crate) fn disabled() -> Self {
        Self::Disabled
    }

    pub(crate) fn enabled(data_directory: &Path, retention: LogRetention) -> Self {
        Self::Enabled {
            directory: data_directory.join(LOG_DIRECTORY),
            retention,
        }
    }

    pub(crate) fn record_pre_migration_backup_failure(
        &self,
        error_class: ErrorClass,
        _database_path: &Path,
    ) {
        if let Self::Enabled {
            directory,
            retention,
        } = self
        {
            let event = match error_class {
                ErrorClass::Io => PRE_MIGRATION_BACKUP_FAILURE_EVENT,
            };
            write_event(directory, *retention, event);
        }
    }
}

fn write_event(directory: &Path, retention: LogRetention, event: &str) {
    let Ok(event_size) = u64::try_from(event.len()) else {
        return;
    };
    if event_size > retention.max_bytes_per_file || fs::create_dir_all(directory).is_err() {
        return;
    }

    let log_file = directory.join(LOG_FILE);
    if requires_rotation(&log_file, retention.max_bytes_per_file, event_size)
        && rotate(&log_file, retention).is_err()
    {
        return;
    }

    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_file) else {
        return;
    };
    let _ = file.write_all(event.as_bytes());
}

fn requires_rotation(log_file: &Path, max_bytes_per_file: u64, event_size: u64) -> bool {
    match fs::metadata(log_file) {
        Ok(metadata) => metadata.len().saturating_add(event_size) > max_bytes_per_file,
        Err(_) => false,
    }
}

fn rotate(log_file: &Path, retention: LogRetention) -> std::io::Result<()> {
    if !log_file.exists() {
        return Ok(());
    }

    if retention.max_files == 1 {
        return fs::remove_file(log_file);
    }

    let oldest = numbered_log_file(log_file, retention.max_files - 1);
    if oldest.exists() {
        fs::remove_file(oldest)?;
    }
    for index in (1..retention.max_files - 1).rev() {
        let source = numbered_log_file(log_file, index);
        if source.exists() {
            fs::rename(source, numbered_log_file(log_file, index + 1))?;
        }
    }
    fs::rename(log_file, numbered_log_file(log_file, 1))
}

fn numbered_log_file(log_file: &Path, index: usize) -> PathBuf {
    log_file.with_file_name(format!("{LOG_FILE}.{index}"))
}

#[cfg(test)]
mod tests {
    use super::{ErrorClass, LocalLogger, LogRetention};
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    fn temporary_directory(name: &str) -> PathBuf {
        let sequence = TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let directory = std::env::temp_dir().join(format!(
            "racoon-typper-logging-{name}-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(&directory).expect("create temporary directory");
        directory
    }

    #[test]
    fn records_only_allowlisted_metadata_for_sensitive_failure_context() {
        let directory = temporary_directory("redaction");
        let logger = LocalLogger::enabled(
            &directory,
            LogRetention::new(2, 1_024).expect("valid retention"),
        );
        let typed_text = "typed-secret-content";
        let custom_content = "custom-content-that-must-not-appear";
        let raw_profile_document = r#"{"custom_text":"profile-document-secret"}"#;
        let user_secret = "api-token-should-never-be-logged";
        let sensitive_path = directory.join(format!(
            "{typed_text}-{custom_content}-{raw_profile_document}-{user_secret}.db"
        ));

        logger.record_pre_migration_backup_failure(ErrorClass::Io, &sensitive_path);

        let output = fs::read_to_string(directory.join("logs/app.jsonl")).expect("read log");
        assert_eq!(
            output,
            "{\"event\":\"pre_migration_backup_failed\",\"error_class\":\"io\",\"path_kind\":\"database\",\"path_file\":\"data.db\"}\n"
        );
        for sensitive_value in [
            typed_text,
            custom_content,
            raw_profile_document,
            user_secret,
        ] {
            assert!(
                !output.contains(sensitive_value),
                "sensitive value appeared in log output"
            );
        }

        fs::remove_dir_all(directory).expect("remove temporary directory");
    }

    #[test]
    fn disabled_logger_creates_no_log_files() {
        let directory = temporary_directory("disabled");
        let logger = LocalLogger::disabled();

        logger.record_pre_migration_backup_failure(ErrorClass::Io, &directory.join("data.db"));

        assert!(!directory.join("logs").exists());
        fs::remove_dir_all(directory).expect("remove temporary directory");
    }

    #[test]
    fn rejects_zero_retention_limits() {
        assert!(LogRetention::new(0, 1).is_none());
        assert!(LogRetention::new(1, 0).is_none());
    }

    #[test]
    fn rotates_and_retains_a_bounded_number_of_log_files() {
        let directory = temporary_directory("retention");
        let logger = LocalLogger::enabled(
            &directory,
            LogRetention::new(2, 150).expect("valid retention"),
        );

        for _ in 0..10 {
            logger.record_pre_migration_backup_failure(ErrorClass::Io, &directory.join("data.db"));
        }

        let mut names = fs::read_dir(directory.join("logs"))
            .expect("read log directory")
            .map(|entry| entry.expect("directory entry"))
            .map(|entry| entry.file_name().into_string().expect("UTF-8 filename"))
            .collect::<Vec<_>>();
        names.sort();
        assert_eq!(names, ["app.jsonl", "app.jsonl.1"]);
        for name in names {
            let metadata = fs::metadata(directory.join("logs").join(name)).expect("log metadata");
            assert!(metadata.len() <= 150, "log file exceeded retention bound");
        }

        fs::remove_dir_all(directory).expect("remove temporary directory");
    }
}
