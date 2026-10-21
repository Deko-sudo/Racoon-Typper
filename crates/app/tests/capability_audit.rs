//! Regression audit for the application-command ACL.
//!
//! Tauri 2 only enforces application commands through an `AppManifest` generated
//! at build time. This test keeps the handler, frontend invoke wrappers, manifest,
//! and main-window capability in one explicit contract.

use std::collections::BTreeSet;

const BUILD_RS: &str = include_str!("../build.rs");
const CAPABILITY: &str = include_str!("../capabilities/main.json");
const IPC: &str = include_str!("../../../frontend/src/lib/api/ipc.ts");
const MAIN_RS: &str = include_str!("../src/main.rs");

fn lines_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_index = source
        .find(start)
        .unwrap_or_else(|| panic!("missing start marker {start:?}"));
    let remaining = &source[start_index + start.len()..];
    let end_index = remaining
        .find(end)
        .unwrap_or_else(|| panic!("missing end marker {end:?}"));
    &remaining[..end_index]
}

fn application_manifest_commands() -> BTreeSet<String> {
    lines_between(BUILD_RS, "const APP_COMMANDS: &[&str] = &[", "];")
        .lines()
        .filter_map(|line| {
            let command = line.trim().strip_prefix('"')?.strip_suffix("\",")?;
            Some(command.to_string())
        })
        .collect()
}

fn registered_commands() -> BTreeSet<String> {
    lines_between(MAIN_RS, "tauri::generate_handler![", "]);")
        .lines()
        .filter_map(|entry| {
            let entry = entry.trim();
            let entry = entry.strip_prefix("commands::")?;
            if entry.is_empty() {
                return None;
            }
            entry
                .rsplit("::")
                .next()
                .map(|command| command.trim_end_matches(',').to_string())
        })
        .collect()
}

fn frontend_invocations() -> BTreeSet<String> {
    let mut commands = BTreeSet::new();
    let mut remaining = IPC;

    while let Some(invoke_index) = remaining.find("invoke") {
        remaining = &remaining[invoke_index + "invoke".len()..];
        let Some(opening_quote_index) = remaining.find("('") else {
            break;
        };
        remaining = &remaining[opening_quote_index + 2..];
        let end_quote_index = remaining
            .find('\'')
            .expect("invoke command is missing its closing quote");
        commands.insert(remaining[..end_quote_index].to_string());
        remaining = &remaining[end_quote_index + 1..];
    }

    commands
}

fn capability_permissions() -> BTreeSet<String> {
    let capability: serde_json::Value =
        serde_json::from_str(CAPABILITY).expect("valid capability JSON");
    capability["permissions"]
        .as_array()
        .expect("capability permissions array")
        .iter()
        .map(|permission| {
            permission
                .as_str()
                .expect("unscoped application command permission")
                .to_string()
        })
        .collect()
}

/// External Tauri plugin permissions explicitly granted to the main window.
/// These are scoped capabilities from third-party plugins used by specific
/// features (custom-text import). Update this set when adding/removing a plugin.
const ALLOWED_PLUGIN_PERMISSIONS: &[&str] = &[
    "dialog:allow-open",
    "fs:allow-read-text-file",
    "clipboard-manager:allow-read-text",
];

#[test]
fn main_window_capability_matches_the_registered_frontend_command_surface() {
    let registered = registered_commands();
    let frontend = frontend_invocations();
    let manifest = application_manifest_commands();
    let permissions = capability_permissions();
    let expected_app_permissions: BTreeSet<String> = manifest
        .iter()
        .map(|command| format!("allow-{}", command.replace('_', "-")))
        .collect();

    assert_eq!(
        registered, frontend,
        "registered commands must have a frontend invoke wrapper"
    );
    assert_eq!(
        manifest, registered,
        "AppManifest must cover every registered command exactly once"
    );

    // Partition permissions into app-command grants and plugin grants.
    let actual_app_permissions: BTreeSet<String> = permissions
        .iter()
        .filter(|p| !p.contains(':'))
        .cloned()
        .collect();
    assert_eq!(
        actual_app_permissions, expected_app_permissions,
        "main capability must grant exactly the generated application-command permissions"
    );

    let allowed_plugin_set: BTreeSet<String> = ALLOWED_PLUGIN_PERMISSIONS
        .iter()
        .map(|s| s.to_string())
        .collect();
    let actual_plugin_permissions: BTreeSet<String> = permissions
        .iter()
        .filter(|p| p.contains(':'))
        .cloned()
        .collect();
    assert_eq!(
        actual_plugin_permissions, allowed_plugin_set,
        "main capability plugin permissions must match the explicit allow-list"
    );

    assert!(
        !permissions
            .iter()
            .any(|permission| permission.starts_with("core:")),
        "the frontend does not use Tauri core window, webview, event, or app APIs"
    );
}
