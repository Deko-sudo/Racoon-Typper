//! Tauri IPC commands.
//! Sprint 1: ping, get_app_info
//! Sprint 2: start_test, process_key, abort_session
//! Sprint 3: статистика в EngineOutput
//! Sprint 4: get_stats_history, get_test_detail, get_personal_bests, save_test_result
//! Sprint 5: custom_texts CRUD, settings, themes

use racoon_core::{CoreEngine, CustomMode, KeyEvent, QuoteMode, TestMode, TimeMode, WordsMode};
use racoon_data::repository::{
    AppSettings, CustomTextRepository, PersonalBestsRepository, SqliteCustomTextRepository,
    SqlitePersonalBestsRepository, SqliteTestRepository, TestRepository,
};
use racoon_domain::PersonalBest;
use racoon_domain::TestDetail;
use racoon_domain::TestSummary;
use racoon_domain::{AppInfo, EngineOutput, TestRecord};
use racoon_resources::{quote_loader, word_pack_loader};
use std::sync::Mutex;
use tauri::State;

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
) -> Result<TestSessionResponse, String> {
    let mut engine = state.lock().map_err(|e| e.to_string())?;
    let session_id = generate_session_id();
    let lang = language.unwrap_or_else(|| "en".to_string());

    let test_mode: Box<dyn TestMode> = match mode.as_str() {
        "time" => {
            let secs = duration.unwrap_or(30);
            let word_count = TimeMode::recommended_word_count(secs);
            let test_text = text.unwrap_or_else(|| {
                word_pack_loader()
                    .generate_words(&lang, word_count)
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
        _ => return Err(format!("Unknown mode: {}", mode)),
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
    timestamp: u64,
) -> Result<EngineOutput, String> {
    let mut engine = engine_state.lock().map_err(|e| e.to_string())?;
    let key_event = KeyEvent {
        key,
        code,
        timestamp,
    };
    let output = engine.process_key(&key_event);

    if let Some(ref final_stats) = output.test_complete {
        // Получаем mode info из CoreEngine
        let (mode_type, mode_config, language) = {
            let engine = engine_state.lock().map_err(|e| e.to_string())?;
            let mt = engine
                .current_mode_type()
                .map(|m| m.to_string())
                .unwrap_or_else(|| "time".to_string());
            let mc = engine
                .current_mode_config()
                .unwrap_or(serde_json::json!({}));
            let lang = engine.current_language().unwrap_or("en").to_string();
            (mt, mc, lang)
        };

        let db = app_state.db.lock().map_err(|e| e.to_string())?;
        let conn = db.conn();
        let repo = SqliteTestRepository::new(&conn);

        let record = TestRecord {
            created_at: chrono::Utc::now().to_rfc3339(),
            mode_type,
            mode_config,
            language,
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

        let test_id = repo.save_test(record).map_err(|e| e.to_string())?;

        let pb_repo = SqlitePersonalBestsRepository::new(&conn);
        let mode_type_str = engine_state
            .lock()
            .map_err(|e| e.to_string())?
            .current_mode_type()
            .map(|m| m.to_string())
            .unwrap_or_else(|| "time".to_string());
        let mode_config_str = serde_json::to_string(
            &engine_state
                .lock()
                .map_err(|e| e.to_string())?
                .current_mode_config()
                .unwrap_or(serde_json::json!({})),
        )
        .unwrap_or_default();

        let _pb_updates = pb_repo
            .check_and_update(
                &mode_type_str,
                &mode_config_str,
                final_stats.wpm,
                final_stats.accuracy,
                test_id,
            )
            .map_err(|e| e.to_string())?;
    }

    Ok(output)
}

#[tauri::command]
pub fn abort_session(state: State<'_, Mutex<CoreEngine>>) -> Result<(), String> {
    let mut engine = state.lock().map_err(|e| e.to_string())?;
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
) -> Result<StatsHistoryResponse, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();
    let repo = SqliteTestRepository::new(&conn);

    let lim = limit.unwrap_or(50);
    let off = offset.unwrap_or(0);

    let tests = repo
        .get_history(lim, off, mode_filter.as_deref())
        .map_err(|e| e.to_string())?;
    let total = repo
        .get_count(mode_filter.as_deref())
        .map_err(|e| e.to_string())?;

    Ok(StatsHistoryResponse { tests, total })
}

#[tauri::command]
pub fn get_test_detail(state: State<'_, AppState>, id: i64) -> Result<TestDetail, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();
    let repo = SqliteTestRepository::new(&conn);
    repo.get_by_id(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_personal_bests(
    state: State<'_, AppState>,
    mode_filter: Option<String>,
) -> Result<Vec<PersonalBest>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();
    let repo = SqlitePersonalBestsRepository::new(&conn);
    repo.get_bests(mode_filter.as_deref())
        .map_err(|e| e.to_string())
}

// ── Custom Texts ──

#[tauri::command]
pub fn get_custom_texts(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<racoon_data::CustomText>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();
    let repo = SqliteCustomTextRepository::new(&conn);
    repo.get_all(limit.unwrap_or(50)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_custom_text(
    state: State<'_, AppState>,
    id: i64,
) -> Result<racoon_data::CustomText, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();
    let repo = SqliteCustomTextRepository::new(&conn);
    repo.get_by_id(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_custom_text(
    state: State<'_, AppState>,
    name: String,
    text: String,
) -> Result<i64, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();
    let repo = SqliteCustomTextRepository::new(&conn);
    repo.save(&name, &text).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_custom_text(
    state: State<'_, AppState>,
    id: i64,
    name: String,
    text: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();
    let repo = SqliteCustomTextRepository::new(&conn);
    repo.update(id, &name, &text).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_custom_text(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();
    let repo = SqliteCustomTextRepository::new(&conn);
    repo.delete(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search_custom_texts(
    state: State<'_, AppState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<racoon_data::CustomText>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.conn();
    let repo = SqliteCustomTextRepository::new(&conn);
    repo.search(&query, limit.unwrap_or(20))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn start_custom_text_test(
    engine_state: State<'_, Mutex<CoreEngine>>,
    app_state: State<'_, AppState>,
    custom_text_id: i64,
) -> Result<TestSessionResponse, String> {
    let custom_text = {
        let db = app_state.db.lock().map_err(|e| e.to_string())?;
        let conn = db.conn();
        let repo = SqliteCustomTextRepository::new(&conn);
        let ct = repo.get_by_id(custom_text_id).map_err(|e| e.to_string())?;
        repo.increment_use(custom_text_id)
            .map_err(|e| e.to_string())?;
        ct
    };

    let mut engine = engine_state.lock().map_err(|e| e.to_string())?;
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
pub fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let store = state.settings_store();
    store.load().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_setting(
    state: State<'_, AppState>,
    key: String,
    value: serde_json::Value,
) -> Result<AppSettings, String> {
    let store = state.settings_store();
    let toml_value = json_to_toml_value(&value);
    store.set(&key, toml_value).map_err(|e| e.to_string())
}

// ── Themes ──

#[tauri::command]
pub fn get_themes() -> Result<Vec<ThemeInfo>, String> {
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
pub fn get_theme_css(name: String) -> Result<String, String> {
    let css = match name.as_str() {
        "serika_dark" => include_str!("../../../resources/themes/serika_dark/theme.css"),
        "serika_light" => include_str!("../../../resources/themes/serika_light/theme.css"),
        "racoon_dark" => include_str!("../../../resources/themes/racoon_dark/theme.css"),
        _ => return Err(format!("Theme not found: {}", name)),
    };
    Ok(css.to_string())
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
