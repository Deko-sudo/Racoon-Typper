// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

//! Filesystem transaction for a CLOSED, exclusively leased profile. Pending
//! replacements roll back before SQLite opens; committed ones stay installed.
//! Sync ordering assumes a local filesystem honoring file/directory fsync and
//! same-filesystem rename. This does not promise physical power-loss durability.

use super::{restore_companion, restore_fail, sync_file, DEFAULT_KEEP};
use crate::DbError;
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const SUFFIXES: [&str; 4] = ["", "-wal", "-shm", "-journal"];
const TERMINAL: &[u8] = b"racoon-restore-v1\n";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    version: u8,
    sizes: [Option<u64>; 4],
}

fn regular_size(path: &Path) -> Result<Option<u64>, DbError> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.is_file() => Ok(Some(meta.len())),
        Ok(_) => Err(restore_fail("restore file must be regular")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(restore_fail(error)),
    }
}

fn read_small(path: &Path) -> Result<Option<Vec<u8>>, DbError> {
    let Some(size) = regular_size(path)? else {
        return Ok(None);
    };
    if size > 1024 {
        return Err(DbError::RestoreBlocked);
    }
    let mut bytes = Vec::new();
    File::open(path)
        .and_then(|f| f.take(1025).read_to_end(&mut bytes))
        .map_err(restore_fail)?;
    if bytes.len() > 1024 {
        return Err(DbError::RestoreBlocked);
    }
    Ok(Some(bytes))
}

fn private_file(path: &Path) -> Result<File, DbError> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path).map_err(restore_fail)
}

fn durable_copy(source: &Path, dest: &Path) -> Result<u64, DbError> {
    let mut input = File::open(source).map_err(restore_fail)?;
    let mut output = private_file(dest)?;
    let size = std::io::copy(&mut input, &mut output).map_err(restore_fail)?;
    output.sync_all().map_err(restore_fail)?;
    Ok(size)
}

fn remove_regular(path: &Path) -> Result<(), DbError> {
    if regular_size(path)?.is_some() {
        fs::remove_file(path).map_err(restore_fail)?;
    }
    Ok(())
}

fn sync_dir(path: &Path) -> Result<(), DbError> {
    #[cfg(unix)]
    {
        sync_file(path)?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

fn parent(path: &Path) -> &Path {
    match path.parent().filter(|p| !p.as_os_str().is_empty()) {
        Some(parent) => parent,
        None => Path::new("."),
    }
}

fn marker(stage: &Path, name: &str, bytes: &[u8]) -> Result<(), DbError> {
    let temp = stage.join(format!("{name}.tmp"));
    remove_regular(&temp)?;
    let mut file = private_file(&temp)?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(restore_fail)?;
    drop(file);
    fs::rename(temp, stage.join(name)).map_err(restore_fail)?;
    sync_dir(stage)
}

fn manifest(stage: &Path) -> Result<Option<Manifest>, DbError> {
    let Some(bytes) = read_small(&stage.join("pending"))? else {
        return Ok(None);
    };
    let value: Manifest = serde_json::from_slice(&bytes).map_err(restore_fail)?;
    if value.version != 1 {
        return Err(DbError::RestoreBlocked);
    }
    Ok(Some(value))
}

fn terminal(stage: &Path) -> Result<bool, DbError> {
    let committed = read_small(&stage.join("committed"))?;
    let aborted = read_small(&stage.join("aborted"))?;
    if committed.is_some() && aborted.is_some() {
        return Err(DbError::RestoreBlocked);
    }
    match committed.or(aborted) {
        Some(bytes) if bytes == TERMINAL => Ok(true),
        Some(_) => Err(DbError::RestoreBlocked),
        None => Ok(false),
    }
}

fn validate_snapshot(stage: &Path, manifest: &Manifest) -> Result<(), DbError> {
    for (suffix, size) in SUFFIXES.iter().zip(manifest.sizes) {
        if regular_size(&stage.join(format!("original.db{suffix}")))? != size {
            return Err(DbError::RestoreBlocked);
        }
    }
    Ok(())
}

fn rollback(
    stage: &Path,
    live: &Path,
    manifest: &Manifest,
    step: &mut impl FnMut(&'static str) -> Result<(), DbError>,
) -> Result<(), DbError> {
    // Check the ENTIRE snapshot and all destinations before the first write.
    validate_snapshot(stage, manifest)?;
    for suffix in SUFFIXES {
        regular_size(&restore_companion(live, suffix))?;
    }
    for (index, suffix) in SUFFIXES.iter().enumerate() {
        let dest = restore_companion(live, suffix);
        if manifest.sizes[index].is_some() {
            let temp = stage.join(format!("recovering.db{suffix}"));
            remove_regular(&temp)?;
            durable_copy(&stage.join(format!("original.db{suffix}")), &temp)?;
            fs::rename(temp, dest).map_err(restore_fail)?;
        } else {
            remove_regular(&dest)?;
        }
        step(
            [
                "rollback-main",
                "rollback-wal",
                "rollback-shm",
                "rollback-journal",
            ][index],
        )?;
    }
    sync_dir(parent(live))?;
    sync_dir(stage)?;
    marker(stage, "aborted", TERMINAL)?;
    step("aborted")
}

pub(super) fn publish(
    stage: &Path,
    live: &Path,
    step: &mut impl FnMut(&'static str) -> Result<(), DbError>,
) -> Result<(), DbError> {
    let mut value = Manifest {
        version: 1,
        sizes: [None; 4],
    };
    for (index, suffix) in SUFFIXES.iter().enumerate() {
        let original = restore_companion(live, suffix);
        if let Some(size) = regular_size(&original)? {
            let copied = durable_copy(&original, &stage.join(format!("original.db{suffix}")))?;
            if copied != size {
                return Err(restore_fail("live file changed during restore"));
            }
            value.sizes[index] = Some(size);
        }
        step(
            [
                "snapshot-main",
                "snapshot-wal",
                "snapshot-shm",
                "snapshot-journal",
            ][index],
        )?;
    }
    // A durable intent is published only AFTER every snapshot file is complete.
    // Before the first destructive operation both directory entries are synced.
    marker(
        stage,
        "pending",
        &serde_json::to_vec(&value).map_err(restore_fail)?,
    )?;
    let result = (|| {
        sync_dir(parent(live))?;
        step("pending")?;
        for index in 1..4 {
            if value.sizes[index].is_some() {
                fs::rename(
                    restore_companion(live, SUFFIXES[index]),
                    stage.join(format!("detached.db{}", SUFFIXES[index])),
                )
                .map_err(restore_fail)?;
            }
            step(["", "detach-wal", "detach-shm", "detach-journal"][index])?;
        }
        fs::rename(stage.join("replacement.db"), live).map_err(restore_fail)?;
        step("swapped")?;
        sync_dir(parent(live))?;
        sync_dir(stage)?;
        step("published")?;
        marker(stage, "committed", TERMINAL)?;
        step("committed")
    })();
    if result.is_err() {
        // A visible commit marker after an IO error is an uncertain commit.
        // Keep the caller closed; startup will choose the durable state.
        if read_small(&stage.join("committed"))?.is_some()
            || rollback(stage, live, &value, step).is_err()
        {
            return Err(DbError::RestoreBlocked);
        }
    }
    result
}

fn stages(live: &Path) -> Result<Vec<PathBuf>, DbError> {
    let mut prefix = live
        .file_name()
        .ok_or(DbError::RestoreBlocked)?
        .to_os_string();
    prefix.push(".restore-rollback-");
    let entries = match fs::read_dir(parent(live)) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        Err(error) => return Err(restore_fail(error)),
    };
    let mut stages = Vec::new();
    for entry in entries {
        let entry = entry.map_err(restore_fail)?;
        if entry
            .file_name()
            .as_encoded_bytes()
            .starts_with(prefix.as_encoded_bytes())
        {
            if !entry.file_type().map_err(restore_fail)?.is_dir() {
                return Err(DbError::RestoreBlocked);
            }
            stages.push(entry.path());
        }
    }
    Ok(stages)
}

/// MUST run with all connections/writers closed, before opening SQLite, taking
/// a pre-migration backup or starting another replacement. Never guess between
/// multiple pending intents, malformed manifests, or incomplete snapshots.
pub(crate) fn recover(live: &Path) -> Result<(), DbError> {
    recover_with_steps(live, &mut |_| Ok(()))
}

pub(super) fn recover_with_steps(
    live: &Path,
    step: &mut impl FnMut(&'static str) -> Result<(), DbError>,
) -> Result<(), DbError> {
    let result = (|| {
        let mut pending = Vec::new();
        for stage in stages(live)? {
            if let Some(value) = manifest(&stage)? {
                if !terminal(&stage)? {
                    validate_snapshot(&stage, &value)?;
                    pending.push((stage, value));
                }
            }
        }
        if pending.len() > 1 {
            return Err(DbError::RestoreBlocked);
        }
        if let Some((stage, value)) = pending.pop() {
            rollback(&stage, live, &value, step)?;
        }
        Ok(())
    })();
    result.map_err(|_| DbError::RestoreBlocked)?;
    // Retention is best effort; failure never discards a pending recovery copy
    // or changes whether the live database may be opened.
    let _ = prune(live);
    Ok(())
}

pub(super) fn prune(live: &Path) -> Result<(), DbError> {
    let mut completed = Vec::new();
    for stage in stages(live)? {
        if let Some(value) = manifest(&stage)? {
            if !terminal(&stage)? {
                return Err(DbError::RestoreBlocked);
            }
            if validate_snapshot(&stage, &value).is_ok() {
                let stamp = fs::metadata(stage.join("committed"))
                    .or_else(|_| fs::metadata(stage.join("aborted")))
                    .and_then(|meta| meta.modified())
                    .map_err(restore_fail)?;
                completed.push((stamp, stage));
            }
        }
    }
    completed.sort();
    let excess = completed.len().saturating_sub(DEFAULT_KEEP);
    for (_, stage) in completed.into_iter().take(excess) {
        // First durably leave the recovery namespace. A crash during deletion
        // must not turn a partially pruned completed snapshot into a pending one.
        let name = stage.file_name().ok_or(DbError::RestoreBlocked)?;
        let mut trash = std::ffi::OsString::from(".restore-trash-");
        trash.push(name);
        let trash = stage.with_file_name(trash);
        fs::rename(stage, &trash).map_err(restore_fail)?;
        sync_dir(parent(live))?;
        fs::remove_dir_all(trash).map_err(restore_fail)?;
    }
    Ok(())
}
