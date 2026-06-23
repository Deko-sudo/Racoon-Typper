//! Tauri IPC commands.
//! All commands return Result<T, AppError>.

use racoon_core::{
    AdaptiveTextGenerator, CoreEngine, CustomMode, FrequencyAdaptiveGenerator, KeyEvent,
    LessonMode, QuoteMode, TestMode, TimeMode, WeakKeysAnalyzer, WordsMode,
};
use racoon_data::repository::{
    AppSettings, CustomTextRepository, LessonRepository, PersonalBestsRepository,
    SqliteCustomTextRepository, SqliteLessonRepository, SqlitePersonalBestsRepository,
    SqliteTestRepository, TestRepository,
};
use racoon_domain::PersonalBest;
use racoon_domain::TestDetail;
use racoon_domain::TestSummary;
use racoon_domain::{AppInfo, EngineOutput, TestRecord};
use racoon_resources::{course_loader, quote_loader, word_pack_loader};
use std::sync::Mutex;
use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

// ── System ──

#[tauri::command]
pub fn ping() -> String {
    "pong".to_string()
}

#[tauri::command]
pub fn get_app_info() -> AppInfo {
    crate::app_info()
}

// ── Test ──

#[tauri::command]
pub fn start_test(
    state: State<'_, Mutex<CoreEngine>>,
    mode: String,
    text: Option<String>,
    duration: Option<u64>,
    word_count: Option<usize>,
    quote_id: Option<i64>,
    language: Option<String>,
) -> Result<TestSessionResponse, AppError> {
    let mut engine = state.lock()?;
    let session_id = generate_session_id();
    let lang = language.unwrap_or_else(|| "en".to_string());

    let test_mode: Box<dyn TestMode> = match mode.as_str() {
        "time" => {
            let secs = duration.unwrap_or(30);
            let wc = TimeMode::recommended_word_count(secs);
            let test_text = text.unwrap_or_else(|| {
                word_pack_loader()
                    .generate_words(&lang, wc)
                    .unwrap_or_else(|| "The quick brown fox jumps over the lazy dog".to_string())
            });
            Box::new(TimeMode::new(test_text, lang, secs))
        }
        "words" => {
            let count = word_count.unwrap_or(25);
            let test_text = text.unwrap_or_else(|| {
                word_pack_loader()
                    .generate_words(&lang, count)
                    .unwrap_or_else(|| "The quick brown fox jumps over the lazy dog".to_string())
            });
            Box::new(WordsMode::new(test_text, lang, count))
        }
        "quote" => {
            let quote = if let Some(qid) = quote_id {
                quote_loader().get_quote_by_index(&lang, qid as usize)
            } else {
                quote_loader().get_random_quote(&lang)
            };
            let test_text = quote
                .map(|q| q.text.clone())
                .unwrap_or_else(|| "The quick brown fox jumps over the lazy dog".to_string());
            Box::new(QuoteMode::new(test_text, lang, quote_id))
        }
        "custom" => {
            let test_text = text.unwrap_or_else(|| "Custom text".to_string());
            Box::new(CustomMode::new(test_text, lang))
        }
        _ => return Err(AppError::InvalidMode(mode)),
    };

    let info = engine.start_test_mode(session_id.clone(), test_mode);

    Ok(TestSessionResponse {
        session_id,
        text: info.text,
        text_length: info.text_length,
        mode_type: info.mode_type,
        mode_config: info.mode_config,
        language: info.language,
    })
}

#[tauri::command]
pub fn process_key(
    engine_state: State<'_, Mutex<CoreEngine>>,
    app_state: State<'_, AppState>,
    key: String,
    code: String,
) -> Result<EngineOutput, AppError> {
    let (output, mode_info) = {
        let mut engine = engine_state.lock()?;
        // Timestamp генерируется в Rust, не передаётся из frontend
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let key_event = KeyEvent {
            key,
            code,
            timestamp,
        };
        let output = engine.process_key(&key_event);

        let mode_info = if output.test_complete.is_some() {
            let mt = engine
                .current_mode_type()
                .map(|m| m.to_string())
                .unwrap_or_else(|| "time".to_string());
            let mc = engine
                .current_mode_config()
                .unwrap_or(serde_json::json!({}));
            let lang = engine.current_language().unwrap_or("en").to_string();
            Some((mt, mc, lang))
        } else {
            None
        };

        (output, mode_info)
    };

    if let Some((mode_type, mode_config, language)) = mode_info {
        if let Some(ref final_stats) = output.test_complete {
            let db = app_state.db.lock()?;
            let conn = db.conn();
            let repo = SqliteTestRepository::new(&conn);

            let record = TestRecord {
                created_at: chrono::Utc::now().to_rfc3339(),
                mode_type: mode_type.clone(),
                mode_config: mode_config.clone(),
                language: language.clone(),
                text_length: 0,
                duration_ms: final_stats.duration_ms,
                wpm: final_stats.wpm,
                raw_wpm: final_stats.raw_wpm,
                accuracy: final_stats.accuracy,
                raw_accuracy: final_stats.raw_accuracy,
                consistency: final_stats.consistency,
                correct_chars: final_stats.correct_chars,
                incorrect_chars: final_stats.incorrect_chars,
                backspaces: final_stats.backspaces,
                char_stats: final_stats.char_stats.clone(),
                heatmap_data: final_stats.heatmap.clone(),
                graph_data: final_stats.graph_data.clone(),
                is_pb: false,
                tags: "".to_string(),
            };

            let test_id = repo.save_test(record)?;

            let pb_repo = SqlitePersonalBestsRepository::new(&conn);
            let mode_config_str = serde_json::to_string(&mode_config).unwrap_or_default();

            let _ = pb_repo.check_and_update(
                &mode_type,
                &mode_config_str,
                final_stats.wpm,
                final_stats.accuracy,
                test_id,
            )?;
        }
    }

    Ok(output)
}

#[tauri::command]
pub fn abort_session(state: State<'_, Mutex<CoreEngine>>) -> Result<(), AppError> {
    let mut engine = state.lock()?;
    engine.abort();
    Ok(())
}

// ── Stats ──

#[tauri::command]
pub fn get_stats_history(
    state: State<'_, AppState>,
    limit: Option<usize>,
    offset: Option<usize>,
    mode_filter: Option<String>,
) -> Result<StatsHistoryResponse, AppError> {
    let db = state.db.lock()?;
    let conn = db.conn();
    let repo = SqliteTestRepository::new(&conn);

    let lim = limit.unwrap_or(50);
    let off = offset.unwrap_or(0);

    let tests = repo.get_history(lim, off, mode_filter.as_deref())?;
    let total = repo.get_count(mode_filter.as_deref())?;

    Ok(StatsHistoryResponse { tests, total })
}

#[tauri::command]
pub fn get_test_detail(state: State<'_, AppState>, id: i64) -> Result<TestDetail, AppError> {
    let db = state.db.lock()?;
    let conn = db.conn();
    let repo = SqliteTestRepository::new(&conn);
    repo.get_by_id(id).map_err(AppError::from)
}

#[tauri::command]
pub fn get_personal_bests(
    state: State<'_, AppState>,
    mode_filter: Option<String>,
) -> Result<Vec<PersonalBest>, AppError> {
    let db = state.db.lock()?;
    let conn = db.conn();
    let repo = SqlitePersonalBestsRepository::new(&conn);
    repo.get_bests(mode_filter.as_deref())
        .map_err(AppError::from)
}

// ── Custom Texts ──

#[tauri::command]
pub fn get_custom_texts(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<racoon_data::CustomText>, AppError> {
    let db = state.db.lock()?;
    let conn = db.conn();
    let repo = SqliteCustomTextRepository::new(&conn);
    repo.get_all(limit.unwrap_or(50)).map_err(AppError::from)
}

#[tauri::command]
pub fn get_custom_text(
    state: State<'_, AppState>,
    id: i64,
) -> Result<racoon_data::CustomText, AppError> {
    let db = state.db.lock()?;
    let conn = db.conn();
    let repo = SqliteCustomTextRepository::new(&conn);
    repo.get_by_id(id).map_err(AppError::from)
}

#[tauri::command]
pub fn save_custom_text(
    state: State<'_, AppState>,
    name: String,
    text: String,
) -> Result<i64, AppError> {
    let db = state.db.lock()?;
    let conn = db.conn();
    let repo = SqliteCustomTextRepository::new(&conn);
    repo.save(&name, &text).map_err(AppError::from)
}

#[tauri::command]
pub fn update_custom_text(
    state: State<'_, AppState>,
    id: i64,
    name: String,
    text: String,
) -> Result<(), AppError> {
    let db = state.db.lock()?;
    let conn = db.conn();
    let repo = SqliteCustomTextRepository::new(&conn);
    repo.update(id, &name, &text).map_err(AppError::from)
}

#[tauri::command]
pub fn delete_custom_text(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    let db = state.db.lock()?;
    let conn = db.conn();
    let repo = SqliteCustomTextRepository::new(&conn);
    repo.delete(id).map_err(AppError::from)
}

#[tauri::command]
pub fn search_custom_texts(
    state: State<'_, AppState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<racoon_data::CustomText>, AppError> {
    let db = state.db.lock()?;
    let conn = db.conn();
    let repo = SqliteCustomTextRepository::new(&conn);
    repo.search(&query, limit.unwrap_or(20))
        .map_err(AppError::from)
}

#[tauri::command]
pub fn start_custom_text_test(
    engine_state: State<'_, Mutex<CoreEngine>>,
    app_state: State<'_, AppState>,
    custom_text_id: i64,
) -> Result<TestSessionResponse, AppError> {
    let custom_text = {
        let db = app_state.db.lock()?;
        let conn = db.conn();
        let repo = SqliteCustomTextRepository::new(&conn);
        let ct = repo.get_by_id(custom_text_id)?;
        repo.increment_use(custom_text_id)?;
        ct
    };

    let mut engine = engine_state.lock()?;
    let session_id = generate_session_id();
    let mode: Box<dyn TestMode> =
        Box::new(CustomMode::new(custom_text.text.clone(), "en".to_string()));
    let info = engine.start_test_mode(session_id.clone(), mode);

    Ok(TestSessionResponse {
        session_id,
        text: info.text,
        text_length: info.text_length,
        mode_type: info.mode_type,
        mode_config: info.mode_config,
        language: info.language,
    })
}

// ── Settings ──

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, AppError> {
    let store = state.settings_store();
    store.load().map_err(AppError::from)
}

#[tauri::command]
pub fn set_setting(
    state: State<'_, AppState>,
    key: String,
    value: serde_json::Value,
) -> Result<AppSettings, AppError> {
    let store = state.settings_store();
    let toml_value = json_to_toml_value(&value);
    store.set(&key, toml_value).map_err(AppError::from)
}

// ── Themes ──

#[tauri::command]
pub fn get_themes() -> Result<Vec<ThemeInfo>, AppError> {
    Ok(vec![
        ThemeInfo {
            name: "serika_dark".to_string(),
            display_name: "Serika Dark".to_string(),
            is_dark: true,
            preview_colors: ThemePreview {
                bg: "#323437".to_string(),
                main: "#e2b714".to_string(),
                text: "#999999".to_string(),
                error: "#ca4754".to_string(),
            },
        },
        ThemeInfo {
            name: "serika_light".to_string(),
            display_name: "Serika Light".to_string(),
            is_dark: false,
            preview_colors: ThemePreview {
                bg: "#f0f0f0".to_string(),
                main: "#e2b714".to_string(),
                text: "#333333".to_string(),
                error: "#ca4754".to_string(),
            },
        },
        ThemeInfo {
            name: "racoon_dark".to_string(),
            display_name: "Racoon Dark".to_string(),
            is_dark: true,
            preview_colors: ThemePreview {
                bg: "#1a1b26".to_string(),
                main: "#7aa2f7".to_string(),
                text: "#a9b1d6".to_string(),
                error: "#f7768e".to_string(),
            },
        },
    ])
}

#[tauri::command]
pub fn get_theme_css(name: String) -> Result<String, AppError> {
    let css = match name.as_str() {
        "serika_dark" => include_str!("../../../resources/themes/serika_dark/theme.css"),
        "serika_light" => include_str!("../../../resources/themes/serika_light/theme.css"),
        "racoon_dark" => include_str!("../../../resources/themes/racoon_dark/theme.css"),
        _ => return Err(AppError::ThemeNotFound(name)),
    };
    Ok(css.to_string())
}

// ── Lessons ──

#[derive(Debug, serde::Serialize)]
pub struct CourseResponse {
    pub language: String,
    pub modules: Vec<ModuleResponse>,
}

#[derive(Debug, serde::Serialize)]
pub struct ModuleResponse {
    pub id: String,
    pub name: String,
    pub difficulty: String,
    pub order: u32,
    pub lessons: Vec<LessonResponse>,
}

#[derive(Debug, serde::Serialize)]
pub struct LessonResponse {
    pub id: String,
    pub name: String,
    pub text_length: usize,
}

#[tauri::command]
pub fn get_course(language: String) -> Result<CourseResponse, AppError> {
    let course = course_loader()
        .load_course(&language)
        .ok_or_else(|| AppError::Internal(format!("Course not found: {}", language)))?;

    let modules = course
        .modules
        .iter()
        .map(|m| ModuleResponse {
            id: m.id.clone(),
            name: m.name.clone(),
            difficulty: m.difficulty.clone(),
            order: m.order,
            lessons: m
                .lessons
                .iter()
                .map(|l| LessonResponse {
                    id: l.id.clone(),
                    name: l.name.clone(),
                    text_length: l.text.len(),
                })
                .collect(),
        })
        .collect();

    Ok(CourseResponse { language, modules })
}

#[tauri::command]
pub fn get_lesson_progress(
    state: State<'_, AppState>,
    language: String,
) -> Result<serde_json::Value, AppError> {
    let db = state.db.lock()?;
    let conn = db.conn();
    let repo = SqliteLessonRepository::new(&conn);
    let progress = repo.get_course_progress(&language)?;
    serde_json::to_value(progress).map_err(AppError::from)
}

#[tauri::command]
pub fn start_lesson(
    engine_state: State<'_, Mutex<CoreEngine>>,
    app_state: State<'_, AppState>,
    lesson_id: String,
    language: String,
) -> Result<TestSessionResponse, AppError> {
    let lesson = course_loader()
        .load_lesson(&language, &lesson_id)
        .ok_or_else(|| AppError::Internal(format!("Lesson not found: {}", lesson_id)))?;

    let module_id = lesson_id.split('_').take(2).collect::<Vec<_>>().join("_");

    // Create progress record if not exists
    {
        let db = app_state.db.lock()?;
        let conn = db.conn();
        let repo = SqliteLessonRepository::new(&conn);
        let _ = repo.create_progress(&lesson_id, &module_id, &language, "beginner");
    }

    let mut engine = engine_state.lock()?;
    let session_id = generate_session_id();
    let mode: Box<dyn TestMode> = Box::new(LessonMode::new(
        lesson_id.clone(),
        module_id,
        language.clone(),
        lesson.text.clone(),
    ));
    let info = engine.start_test_mode(session_id.clone(), mode);

    Ok(TestSessionResponse {
        session_id,
        text: info.text,
        text_length: info.text_length,
        mode_type: "lesson".to_string(),
        mode_config: info.mode_config,
        language: info.language,
    })
}

#[tauri::command]
pub fn complete_lesson(
    app_state: State<'_, AppState>,
    lesson_id: String,
    wpm: f64,
    accuracy: f64,
) -> Result<(), AppError> {
    let db = app_state.db.lock()?;
    let conn = db.conn();
    let repo = SqliteLessonRepository::new(&conn);
    repo.complete_lesson(&lesson_id, wpm, accuracy)?;
    Ok(())
}

// ── Weak Keys ──

#[tauri::command]
pub fn analyze_weak_keys(
    engine_state: State<'_, Mutex<CoreEngine>>,
) -> Result<serde_json::Value, AppError> {
    let engine = engine_state.lock()?;
    let char_stats = engine.current_char_stats().unwrap_or_default();
    let analyzer = WeakKeysAnalyzer::new();
    let report = analyzer.analyze(&char_stats);
    serde_json::to_value(report).map_err(AppError::from)
}

#[tauri::command]
pub fn generate_weak_keys_training(
    engine_state: State<'_, Mutex<CoreEngine>>,
    language: String,
    word_count: Option<usize>,
) -> Result<String, AppError> {
    let engine = engine_state.lock()?;
    let char_stats = engine.current_char_stats().unwrap_or_default();

    let words = racoon_resources::word_pack_loader()
        .get_pack(&language)
        .map(|p| p.words.clone())
        .unwrap_or_default();

    let generator = FrequencyAdaptiveGenerator::new(words);
    let weak_chars = generator.analyze(&char_stats);
    let text = generator.generate(&weak_chars, word_count.unwrap_or(25));

    Ok(text)
}

// ── Helpers ──

#[derive(Debug, serde::Serialize)]
pub struct TestSessionResponse {
    pub session_id: String,
    pub text: String,
    pub text_length: usize,
    pub mode_type: String,
    pub mode_config: serde_json::Value,
    pub language: String,
}

#[derive(Debug, serde::Serialize)]
pub struct StatsHistoryResponse {
    pub tests: Vec<TestSummary>,
    pub total: i64,
}

#[derive(Debug, serde::Serialize)]
pub struct ThemeInfo {
    pub name: String,
    pub display_name: String,
    pub is_dark: bool,
    pub preview_colors: ThemePreview,
}

#[derive(Debug, serde::Serialize)]
pub struct ThemePreview {
    pub bg: String,
    pub main: String,
    pub text: String,
    pub error: String,
}

fn generate_session_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:016x}", ts)
}

fn json_to_toml_value(value: &serde_json::Value) -> toml::Value {
    match value {
        serde_json::Value::String(s) => toml::Value::String(s.clone()),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                toml::Value::Integer(i)
            } else if let Some(f) = n.as_f64() {
                toml::Value::Float(f)
            } else {
                toml::Value::String(n.to_string())
            }
        }
        serde_json::Value::Bool(b) => toml::Value::Boolean(*b),
        _ => toml::Value::String(value.to_string()),
    }
}
