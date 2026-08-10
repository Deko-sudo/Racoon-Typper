//! Tauri adapters for the typing-session lifecycle.

use racoon_core::CoreEngine;
use racoon_domain::{EngineOutput, SessionId};
use std::sync::Mutex;
use tauri::State;

use crate::commands::contracts::TestSessionResponse;
use crate::error::AppError;
use crate::session_service::{self, StartTestRequest};
use crate::state::AppState;
use crate::validation::validate_key_event;

#[tauri::command]
#[allow(clippy::too_many_arguments)] // Tauri IPC parameters preserve the existing command shape.
pub(crate) fn start_test(
    state: State<'_, Mutex<CoreEngine>>,
    app_state: State<'_, AppState>,
    mode: String,
    text: Option<String>,
    duration: Option<u64>,
    word_count: Option<usize>,
    quote_id: Option<i64>,
    language: Option<String>,
) -> Result<TestSessionResponse, AppError> {
    app_state.require_startup_recovery_ready()?;
    let request = StartTestRequest {
        mode,
        text,
        duration,
        word_count,
        quote_id,
        language,
    };
    let mut engine = state.lock()?;
    let info = session_service::start_test(&mut engine, request)?;
    Ok(TestSessionResponse::from_session_info(info))
}

#[tauri::command]
pub(crate) fn process_key(
    engine_state: State<'_, Mutex<CoreEngine>>,
    app_state: State<'_, AppState>,
    session_id: SessionId,
    key: String,
    code: String,
) -> Result<EngineOutput, AppError> {
    app_state.require_startup_recovery_ready()?;
    validate_key_event(&key, &code)?;
    session_service::process_key(&engine_state, &app_state, session_id, key, code)
}

#[tauri::command]
pub(crate) fn abort_session(
    state: State<'_, Mutex<CoreEngine>>,
    app_state: State<'_, AppState>,
    session_id: SessionId,
) -> Result<(), AppError> {
    app_state.require_startup_recovery_ready()?;
    session_service::abort_session(&state, session_id)
}

/// Abandons whatever session is currently running, without needing its id.
///
/// This is used by the frontend on startup: a hot-reload (dev) or window
/// reload restarts the renderer but the in-memory `CoreEngine` keeps the
/// previous session, so a fresh `start_test` would be rejected with
/// TEST_ALREADY_ACTIVE. `engine.abort()` only discards a Running session and
/// never touches one that is awaiting/finalizing persistence, so calling this
/// before starting is safe.
#[tauri::command]
pub(crate) fn abandon_active_session(
    state: State<'_, Mutex<CoreEngine>>,
    app_state: State<'_, AppState>,
) -> Result<bool, AppError> {
    app_state.require_startup_recovery_ready()?;
    let mut engine = state.lock()?;
    Ok(engine.abort())
}

#[tauri::command]
pub(crate) fn start_custom_text_test(
    engine_state: State<'_, Mutex<CoreEngine>>,
    app_state: State<'_, AppState>,
    custom_text_id: i64,
) -> Result<TestSessionResponse, AppError> {
    app_state.require_startup_recovery_ready()?;
    let mut engine = engine_state.lock()?;
    let info = session_service::start_custom_text_test(&mut engine, &app_state, custom_text_id)?;
    Ok(TestSessionResponse::from_session_info(info))
}

#[tauri::command]
pub(crate) fn start_lesson(
    engine_state: State<'_, Mutex<CoreEngine>>,
    app_state: State<'_, AppState>,
    lesson_id: String,
    language: String,
) -> Result<TestSessionResponse, AppError> {
    app_state.require_startup_recovery_ready()?;
    let mut engine = engine_state.lock()?;
    let info = session_service::start_lesson(&mut engine, &app_state, lesson_id, language)?;
    Ok(TestSessionResponse::lesson_from_session_info(info))
}
