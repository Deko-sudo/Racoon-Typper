//! Small system/diagnostic Tauri adapters.

use racoon_domain::AppInfo;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub(crate) fn ping() -> String {
    "pong".to_string()
}

#[tauri::command]
pub(crate) fn get_app_info(state: State<'_, AppState>) -> AppInfo {
    crate::app_info(&state)
}
