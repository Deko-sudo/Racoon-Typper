//! Tauri adapters for local content, custom texts, lessons, and weak-key data.

use racoon_core::{
    AdaptiveTextGenerator, CoreEngine, FrequencyAdaptiveGenerator, WeakKeysAnalyzer, WeakKeysReport,
};
use racoon_data::repository::{
    CustomTextRepository, LessonProgressRecord, LessonRepository, SqliteCustomTextRepository,
    SqliteLessonRepository,
};
use racoon_resources::{course_loader, word_pack_loader};
use std::sync::Mutex;
use tauri::State;

use crate::commands::contracts::{CourseResponse, LessonResponse, ModuleResponse};
use crate::commands::with_db;
use crate::error::AppError;
use crate::state::AppState;
use crate::validation::{
    validate_language, validate_page_limit, validate_positive_id, validate_search_query,
    validate_word_count,
};

#[tauri::command]
pub(crate) fn get_custom_texts(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<racoon_data::CustomText>, AppError> {
    let limit = validate_page_limit(limit.unwrap_or(50))?;
    with_db(&state, |conn| {
        SqliteCustomTextRepository::new(conn).get_all(limit)
    })
}

#[tauri::command]
pub(crate) fn save_custom_text(
    state: State<'_, AppState>,
    name: String,
    text: String,
    language: String,
) -> Result<i64, AppError> {
    state.require_startup_recovery_ready()?;
    let language = validate_language(language)?;
    with_db(&state, |conn| {
        SqliteCustomTextRepository::new(conn).save_with_language(&name, &text, &language)
    })
}

#[tauri::command]
pub(crate) fn update_custom_text(
    state: State<'_, AppState>,
    id: i64,
    name: String,
    text: String,
    language: String,
) -> Result<(), AppError> {
    state.require_startup_recovery_ready()?;
    validate_positive_id(id, "custom text")?;
    let language = validate_language(language)?;
    with_db(&state, |conn| {
        SqliteCustomTextRepository::new(conn).update_with_language(id, &name, &text, &language)
    })
}

#[tauri::command]
pub(crate) fn delete_custom_text(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    state.require_startup_recovery_ready()?;
    validate_positive_id(id, "custom text")?;
    with_db(&state, |conn| {
        SqliteCustomTextRepository::new(conn).delete(id)
    })
}

#[tauri::command]
pub(crate) fn search_custom_texts(
    state: State<'_, AppState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<racoon_data::CustomText>, AppError> {
    validate_search_query(&query)?;
    let limit = validate_page_limit(limit.unwrap_or(20))?;
    with_db(&state, |conn| {
        SqliteCustomTextRepository::new(conn).search(&query, limit)
    })
}

#[tauri::command]
pub(crate) fn get_course(language: String) -> Result<CourseResponse, AppError> {
    let language = validate_language(language)?;
    let course = course_loader()
        .load_course(&language)
        .ok_or_else(|| AppError::ResourceNotFound(format!("course for language {language}")))?;
    let modules = course
        .modules
        .iter()
        .map(|module| ModuleResponse {
            id: module.id.clone(),
            name: module.name.clone(),
            difficulty: module.difficulty.clone(),
            order: module.order,
            lessons: module
                .lessons
                .iter()
                .map(|lesson| LessonResponse {
                    id: lesson.id.clone(),
                    name: lesson.name.clone(),
                    text_length: lesson.text.chars().count(),
                })
                .collect(),
        })
        .collect();

    Ok(CourseResponse { language, modules })
}

#[tauri::command]
pub(crate) fn get_lesson_progress(
    state: State<'_, AppState>,
    language: String,
) -> Result<Vec<LessonProgressRecord>, AppError> {
    let language = validate_language(language)?;
    with_db(&state, |conn| {
        SqliteLessonRepository::new(conn).get_progress(&language)
    })
}

#[tauri::command]
pub(crate) fn analyze_weak_keys(
    engine_state: State<'_, Mutex<CoreEngine>>,
) -> Result<WeakKeysReport, AppError> {
    let engine = engine_state.lock()?;
    let char_stats = engine.current_char_stats().unwrap_or_default();
    Ok(WeakKeysAnalyzer::new().analyze(&char_stats))
}

#[tauri::command]
pub(crate) fn generate_weak_keys_training(
    engine_state: State<'_, Mutex<CoreEngine>>,
    language: String,
    word_count: Option<usize>,
) -> Result<String, AppError> {
    let language = validate_language(language)?;
    let word_count = word_count.unwrap_or(25);
    validate_word_count(word_count)?;
    let engine = engine_state.lock()?;
    let char_stats = engine.current_char_stats().unwrap_or_default();
    let words = word_pack_loader()
        .get_pack(&language)
        .map(|pack| pack.words.clone())
        .ok_or_else(|| AppError::WordsEmpty(language.clone()))?;
    let generator = FrequencyAdaptiveGenerator::new(words);
    let weak_chars = generator.analyze(&char_stats);

    Ok(generator.generate(&weak_chars, word_count))
}
