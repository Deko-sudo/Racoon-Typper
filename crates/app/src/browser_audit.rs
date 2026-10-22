// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

//! Debug-only newline JSON transport to the existing command handlers. Tauri's
//! mock runtime replaces the shell, not the engine, repositories or resources.

use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::sync::Mutex;

use racoon_application::{
    StartupRecoveryCoordinator, StartupRecoveryGate, StartupRecoveryRetryPolicy,
};
use racoon_core::CoreEngine;
use racoon_data::{
    Database, SqliteFinalizationLedger, SqliteSessionFinalizer, SqliteSessionRecoveryLedger,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tauri::Manager;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    cmd: String,
    #[serde(default = "empty_args")]
    args: Value,
}

fn empty_args() -> Value {
    json!({})
}

pub(super) fn run() -> Result<(), Box<dyn std::error::Error>> {
    // The launcher creates this NEW private directory; never resolve desktop
    // paths, import legacy data, or accept a user-supplied existing profile.
    let profile = PathBuf::from(std::env::var("RACOON_BROWSER_PROFILE")?);
    if !profile.is_absolute() || !profile.is_dir() || profile.join("data.db").exists() {
        return Err("a new absolute test profile directory is required".into());
    }
    let profile = profile.canonicalize()?;
    let lock = crate::paths::ProfileLock::acquire(&profile)?;
    let db_path = profile.join("data.db");
    let database = Database::open(&db_path)?;
    let gate = StartupRecoveryGate::new();
    StartupRecoveryCoordinator::new(
        &SqliteSessionRecoveryLedger::new(&database),
        &SqliteFinalizationLedger::new(&database),
        &SqliteSessionFinalizer::new(&database),
        &crate::SystemRecoveryClock,
        &crate::ThreadStartupRecoverySleeper,
        StartupRecoveryRetryPolicy::default(),
    )
    .run(&gate)
    .map_err(|_| "audit profile recovery failed")?;
    let app = crate::register_commands(tauri::test::mock_builder())
        .build(tauri::test::mock_context(tauri::test::noop_assets()))?;
    app.manage(lock);
    app.manage(Mutex::new(CoreEngine::new()));
    app.manage(crate::state::AppState::new(
        database,
        profile.join("settings.toml"),
        db_path,
        gate,
    ));
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default()).build()?;
    let origin: tauri::Url = "tauri://localhost".parse()?;
    let mut stdout = std::io::stdout().lock();
    writeln!(stdout, "{}", json!({"ready": true}))?;
    stdout.flush()?;
    for line in std::io::stdin().lock().lines() {
        let line = line?;
        let result = match serde_json::from_str::<Request>(&line) {
            Ok(request) if valid_request(&request, &profile) => {
                match tauri::test::get_ipc_response(
                    &webview,
                    tauri::webview::InvokeRequest {
                        cmd: request.cmd,
                        callback: tauri::ipc::CallbackFn(0),
                        error: tauri::ipc::CallbackFn(1),
                        url: origin.clone(),
                        body: tauri::ipc::InvokeBody::Json(request.args),
                        headers: Default::default(),
                        invoke_key: tauri::test::INVOKE_KEY.into(),
                    },
                ) {
                    Ok(body) => json!({"ok": true, "value": body.deserialize::<Value>()?}),
                    Err(error) => json!({"ok": false, "error": error}),
                }
            }
            _ => {
                json!({"ok": false, "error": {"code": "AUDIT_REQUEST", "message": "Invalid audit request."}})
            }
        };
        writeln!(stdout, "{result}")?;
        stdout.flush()?;
    }
    Ok(())
}

fn valid_request(request: &Request, profile: &std::path::Path) -> bool {
    if !request.args.is_object() || request.cmd.starts_with("plugin:") {
        return false;
    }
    // Whole-file restore may read only files explicitly uploaded into the
    // private audit directory. Never expose arbitrary host filesystem reads.
    if request.cmd == "restore_database" {
        return request.args["backupPath"]
            .as_str()
            .and_then(|path| std::fs::canonicalize(path).ok())
            .is_some_and(|path| path.parent() == Some(profile.join("uploads").as_path()));
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_plugin_access_and_unselected_restore_paths() {
        let profile = std::path::Path::new("/nonexistent-audit-profile");
        for (cmd, args) in [
            ("plugin:fs|read_text_file", json!({"path": "/etc/passwd"})),
            ("restore_database", json!({"backupPath": "/etc/passwd"})),
            ("get_settings", json!([])),
        ] {
            assert!(!valid_request(
                &Request {
                    cmd: cmd.into(),
                    args
                },
                profile
            ));
        }
        assert!(valid_request(
            &Request {
                cmd: "get_settings".into(),
                args: json!({})
            },
            profile
        ));
    }
}
