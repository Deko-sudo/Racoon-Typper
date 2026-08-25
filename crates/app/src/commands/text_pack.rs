// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

//! Tauri adapters for versioned text-pack interchange.

use racoon_data::text_pack::{
    apply_text_pack_import, export_text_pack as export_text_pack_document, plan_text_pack_import,
    TextPackImportPlan, TextPackSourceFormat,
};
use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

/// Builds a versioned text pack from one language slice of the library.
///
/// `None` exports the library only when it is single-language; mixed libraries
/// must name the language explicitly so a shared pack can never carry an
/// ambiguous scope.
#[tauri::command]
pub(crate) fn export_text_pack(
    state: State<'_, AppState>,
    language: Option<String>,
) -> Result<String, AppError> {
    export_text_pack_with_state(&state, language)
}

/// Validates an import and reports its effects without writing anything.
#[tauri::command]
pub(crate) fn preview_text_pack_import(
    state: State<'_, AppState>,
    document: String,
    source_format: Option<String>,
    policy: racoon_data::text_pack::TextPackImportPolicy,
) -> Result<TextPackImportPlan, AppError> {
    preview_text_pack_import_with_state(&state, &document, source_format, policy)
}

/// Applies a validated import inside one transaction. `replace` deletes only
/// the custom texts of the pack language.
#[tauri::command]
pub(crate) fn import_text_pack(
    state: State<'_, AppState>,
    document: String,
    source_format: Option<String>,
    policy: racoon_data::text_pack::TextPackImportPolicy,
) -> Result<TextPackImportPlan, AppError> {
    import_text_pack_with_state(&state, &document, source_format, policy)
}

fn export_text_pack_with_state(
    state: &AppState,
    language: Option<String>,
) -> Result<String, AppError> {
    let exported = export_text_pack_document(
        &state.db,
        language.as_deref(),
        env!("CARGO_PKG_VERSION"),
        &chrono::Utc::now().to_rfc3339(),
    )?;
    serde_json::to_string(&exported).map_err(AppError::from)
}

fn parse_source_format(
    source_format: Option<String>,
) -> Result<Option<TextPackSourceFormat>, AppError> {
    match source_format.as_deref() {
        // "auto" lets the data layer sniff JSON vs TSV vs text blocks.
        None | Some("auto") => Ok(None),
        Some(value) => Ok(Some(TextPackSourceFormat::parse(value)?)),
    }
}

fn preview_text_pack_import_with_state(
    state: &AppState,
    document: &str,
    source_format: Option<String>,
    policy: racoon_data::text_pack::TextPackImportPolicy,
) -> Result<TextPackImportPlan, AppError> {
    plan_text_pack_import(
        &state.db,
        document.as_bytes(),
        parse_source_format(source_format)?,
        policy,
    )
    .map_err(AppError::from)
}

fn import_text_pack_with_state(
    state: &AppState,
    document: &str,
    source_format: Option<String>,
    policy: racoon_data::text_pack::TextPackImportPolicy,
) -> Result<TextPackImportPlan, AppError> {
    state.require_startup_recovery_ready()?;
    apply_text_pack_import(
        &state.db,
        document.as_bytes(),
        parse_source_format(source_format)?,
        policy,
    )
    .map_err(AppError::from)
}
