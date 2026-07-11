//! Tauri IPC commands.
//! All commands return Result<T, AppError>.

use racoon_core::{
    AdaptiveTextGenerator, CoreEngine, CustomMode, FrequencyAdaptiveGenerator, KeyEvent,
    LessonMode, QuoteMode, TestMode, TimeMode, WeakKeysAnalyzer, WordsMode,
};
use racoon_data::repository::{
    AppSettings, CustomTextRepository, DailyStatsRepository, LessonRepository,
    PersonalBestsRepository, ReplayRepository, SqliteCustomTextRepository,
    SqliteDailyStatsRepository, SqliteLessonRepository, SqlitePersonalBestsRepository,
    SqliteReplayRepository, SqliteTestRepository, TestRepository,
};
use racoon_domain::PersonalBest;
use racoon_domain::TestSummary;
use racoon_domain::{AppInfo, EngineOutput, FinalStats, TestRecord};
use racoon_resources::{course_loader, quote_loader, word_pack_loader};
use rusqlite::{params, Connection, OptionalExtension};
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
    let (output, completed_session) = {
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

        let completed_session = output
            .test_complete
            .as_ref()
            .map(|final_stats| CompletedSession {
                final_stats: final_stats.clone(),
                mode_type: engine
                    .current_mode_type()
                    .map(|mode| mode.to_string())
                    .unwrap_or_else(|| "time".to_string()),
                mode_config: engine
                    .current_mode_config()
                    .unwrap_or_else(|| serde_json::json!({})),
                language: engine.current_language().unwrap_or("en").to_string(),
                text_length: engine.current_text().map_or(0, |text| text.chars().count()),
                replay_frames: engine.replay_frames().to_vec(),
            });

        (output, completed_session)
    };

    if let Some(completed) = completed_session {
        let db = app_state.db.lock()?;
        let conn = db.conn();
        let test_repo = SqliteTestRepository::new(&conn);
        let test_id = test_repo.save_test(test_record_from_completion(&completed))?;

        let replay_frames = completed
            .replay_frames
            .iter()
            .enumerate()
            .map(|(index, frame)| racoon_data::repository::ReplayFrame {
                id: 0,
                test_id,
                frame_index: index as i64,
                timestamp_ms: frame.timestamp_ms.min(i64::MAX as u64) as i64,
                position: frame.caret_pos.min(i64::MAX as usize) as i64,
                expected_char: frame.expected_char.to_string(),
                typed_char: Some(
                    frame
                        .typed_char
                        .map_or_else(|| frame.key.clone(), |character| character.to_string()),
                ),
                correct: frame.char_status == racoon_domain::CharStatus::Correct,
            })
            .collect::<Vec<_>>();
        SqliteReplayRepository::new(&conn).save_replay(test_id, &replay_frames)?;

        let mode_config_string = serde_json::to_string(&completed.mode_config)?;
        let pb_updates = SqlitePersonalBestsRepository::new(&conn).check_and_update(
            &completed.mode_type,
            &mode_config_string,
            completed.final_stats.wpm,
            completed.final_stats.accuracy,
            test_id,
        )?;
        if !pb_updates.is_empty() {
            test_repo.mark_as_pb(test_id)?;
        }

        let daily_repo = SqliteDailyStatsRepository::new(&conn);
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        daily_repo.update_after_test(
            &today,
            completed.final_stats.duration_ms as i64,
            (completed.final_stats.correct_chars + completed.final_stats.incorrect_chars) as i64,
            completed.final_stats.wpm,
            completed.final_stats.accuracy,
        )?;
        persist_daily_streak(&conn, &today)?;

        // Check daily goal
        if let Ok(settings) = app_state.settings_store().load() {
            if let Some(day_stats) = daily_repo.get_day(&today)? {
                let goal_met = match settings.daily_goal_type.as_str() {
                    "wpm" => {
                        settings.daily_goal_wpm > 0.0
                            && day_stats.best_wpm >= settings.daily_goal_wpm
                    }
                    "accuracy" => {
                        settings.daily_goal_accuracy > 0.0
                            && day_stats.avg_accuracy >= settings.daily_goal_accuracy
                    }
                    "time" => {
                        day_stats.total_time_ms >= (settings.daily_goal_wpm as i64 * 60_000).max(0)
                    }
                    _ => false,
                };
                if goal_met {
                    conn.execute(
                        "UPDATE daily_stats SET daily_goal_met = 1 WHERE date = ?1",
                        params![today],
                    )
                    .map_err(|error| AppError::DbWrite(error.to_string()))?;
                }
            }
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
    language: String,
) -> Result<i64, AppError> {
    let db = state.db.lock()?;
    let conn = db.conn();
    let repo = SqliteCustomTextRepository::new(&conn);
    repo.save_with_language(&name, &text, &language)
        .map_err(AppError::from)
}

#[tauri::command]
pub fn update_custom_text(
    state: State<'_, AppState>,
    id: i64,
    name: String,
    text: String,
    language: String,
) -> Result<(), AppError> {
    let db = state.db.lock()?;
    let conn = db.conn();
    let repo = SqliteCustomTextRepository::new(&conn);
    repo.update_with_language(id, &name, &text, &language)
        .map_err(AppError::from)
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
    let mode: Box<dyn TestMode> = Box::new(CustomMode::new(
        custom_text.text.clone(),
        custom_text.language.clone(),
    ));
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
        ThemeInfo {
            name: "catppuccin_mocha".to_string(),
            display_name: "Catppuccin Mocha".to_string(),
            is_dark: true,
            preview_colors: ThemePreview {
                bg: "#1e1e2e".to_string(),
                main: "#cba6f7".to_string(),
                text: "#cdd6f4".to_string(),
                error: "#f38ba8".to_string(),
            },
        },
        ThemeInfo {
            name: "dracula".to_string(),
            display_name: "Dracula".to_string(),
            is_dark: true,
            preview_colors: ThemePreview {
                bg: "#282a36".to_string(),
                main: "#bd93f9".to_string(),
                text: "#f8f8f2".to_string(),
                error: "#ff5555".to_string(),
            },
        },
        ThemeInfo {
            name: "nord".to_string(),
            display_name: "Nord".to_string(),
            is_dark: true,
            preview_colors: ThemePreview {
                bg: "#2e3440".to_string(),
                main: "#88c0d0".to_string(),
                text: "#d8dee9".to_string(),
                error: "#bf616a".to_string(),
            },
        },
        ThemeInfo {
            name: "dark".to_string(),
            display_name: "Dark".to_string(),
            is_dark: true,
            preview_colors: ThemePreview {
                bg: "#111111".to_string(),
                main: "#e2b714".to_string(),
                text: "#bbbbbb".to_string(),
                error: "#ca4754".to_string(),
            },
        },
        ThemeInfo {
            name: "light".to_string(),
            display_name: "Light".to_string(),
            is_dark: false,
            preview_colors: ThemePreview {
                bg: "#eeeeee".to_string(),
                main: "#e2b714".to_string(),
                text: "#222222".to_string(),
                error: "#ca4754".to_string(),
            },
        },
        ThemeInfo {
            name: "matrix".to_string(),
            display_name: "Matrix".to_string(),
            is_dark: true,
            preview_colors: ThemePreview {
                bg: "#000000".to_string(),
                main: "#00ff00".to_string(),
                text: "#00cc00".to_string(),
                error: "#ff0000".to_string(),
            },
        },
        ThemeInfo {
            name: "terra".to_string(),
            display_name: "Terra".to_string(),
            is_dark: false,
            preview_colors: ThemePreview {
                bg: "#d4b996".to_string(),
                main: "#8b5e3c".to_string(),
                text: "#4a3010".to_string(),
                error: "#cc4444".to_string(),
            },
        },
        ThemeInfo {
            name: "lilac".to_string(),
            display_name: "Lilac".to_string(),
            is_dark: false,
            preview_colors: ThemePreview {
                bg: "#e0e0e0".to_string(),
                main: "#b5a3d4".to_string(),
                text: "#444444".to_string(),
                error: "#cc4444".to_string(),
            },
        },
        ThemeInfo {
            name: "nautilus".to_string(),
            display_name: "Nautilus".to_string(),
            is_dark: true,
            preview_colors: ThemePreview {
                bg: "#0b132b".to_string(),
                main: "#5bc0be".to_string(),
                text: "#e0e1dd".to_string(),
                error: "#ff6b6b".to_string(),
            },
        },
        ThemeInfo {
            name: "coral".to_string(),
            display_name: "Coral".to_string(),
            is_dark: false,
            preview_colors: ThemePreview {
                bg: "#fff5f5".to_string(),
                main: "#ff6b6b".to_string(),
                text: "#555555".to_string(),
                error: "#e84545".to_string(),
            },
        },
        ThemeInfo {
            name: "foamy".to_string(),
            display_name: "Foamy".to_string(),
            is_dark: false,
            preview_colors: ThemePreview {
                bg: "#eef2f5".to_string(),
                main: "#4fc3a1".to_string(),
                text: "#37474f".to_string(),
                error: "#e57373".to_string(),
            },
        },
        ThemeInfo {
            name: "rose_pine".to_string(),
            display_name: "Rose Pine".to_string(),
            is_dark: true,
            preview_colors: ThemePreview {
                bg: "#191724".to_string(),
                main: "#ebbcba".to_string(),
                text: "#e0def4".to_string(),
                error: "#eb6f92".to_string(),
            },
        },
        ThemeInfo {
            name: "gruvbox_dark".to_string(),
            display_name: "Gruvbox Dark".to_string(),
            is_dark: true,
            preview_colors: ThemePreview {
                bg: "#282828".to_string(),
                main: "#fabd2f".to_string(),
                text: "#ebdbb2".to_string(),
                error: "#fb4934".to_string(),
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
        "catppuccin_mocha" => include_str!("../../../resources/themes/catppuccin_mocha/theme.css"),
        "dracula" => include_str!("../../../resources/themes/dracula/theme.css"),
        "nord" => include_str!("../../../resources/themes/nord/theme.css"),
        "dark" => include_str!("../../../resources/themes/dark/theme.css"),
        "light" => include_str!("../../../resources/themes/light/theme.css"),
        "matrix" => include_str!("../../../resources/themes/matrix/theme.css"),
        "terra" => include_str!("../../../resources/themes/terra/theme.css"),
        "lilac" => include_str!("../../../resources/themes/lilac/theme.css"),
        "nautilus" => include_str!("../../../resources/themes/nautilus/theme.css"),
        "coral" => include_str!("../../../resources/themes/coral/theme.css"),
        "foamy" => include_str!("../../../resources/themes/foamy/theme.css"),
        "rose_pine" => include_str!("../../../resources/themes/rose_pine/theme.css"),
        "gruvbox_dark" => include_str!("../../../resources/themes/gruvbox_dark/theme.css"),
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
    let progress = repo.get_progress(&language)?;
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
        repo.create_progress(&lesson_id, &module_id, &language, "beginner")?;
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
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    SqliteDailyStatsRepository::new(&conn).increment_lessons_completed(&today)?;
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

// ── Dashboard ──

#[derive(Debug, serde::Serialize)]
pub struct DashboardStatsResponse {
    pub current_streak: i64,
    pub longest_streak: i64,
    pub avg_wpm: f64,
    pub avg_accuracy: f64,
    pub tests_today: i64,
    pub tests_this_week: i64,
    pub total_tests: i64,
    pub daily_goal_met: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct ProgressPoint {
    pub date: String,
    pub wpm: f64,
    pub accuracy: f64,
    pub tests: i64,
}

#[tauri::command]
pub fn get_dashboard_stats(state: State<'_, AppState>) -> Result<DashboardStatsResponse, AppError> {
    let db = state.db.lock()?;
    let conn = db.conn();
    let test_repo = SqliteTestRepository::new(&conn);
    let daily_repo = SqliteDailyStatsRepository::new(&conn);

    let total = test_repo.get_count(None)?;

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let today_stats = daily_repo.get_day(&today)?;
    let tests_today = today_stats.as_ref().map(|s| s.total_tests).unwrap_or(0);

    let week_ago = (chrono::Utc::now() - chrono::Duration::days(7))
        .format("%Y-%m-%d")
        .to_string();
    let week_stats = daily_repo.get_range(&week_ago, &today)?;
    let tests_this_week: i64 = week_stats.iter().map(|s| s.total_tests).sum();

    let avg_wpm = if week_stats.is_empty() {
        0.0
    } else {
        let weighted: f64 = week_stats
            .iter()
            .map(|s| s.avg_wpm * s.total_tests as f64)
            .sum();
        let total_count: i64 = week_stats.iter().map(|s| s.total_tests).sum();
        if total_count > 0 {
            weighted / total_count as f64
        } else {
            0.0
        }
    };

    let avg_accuracy = if week_stats.is_empty() {
        0.0
    } else {
        let weighted: f64 = week_stats
            .iter()
            .map(|s| s.avg_accuracy * s.total_tests as f64)
            .sum();
        let total_count: i64 = week_stats.iter().map(|s| s.total_tests).sum();
        if total_count > 0 {
            weighted / total_count as f64
        } else {
            0.0
        }
    };

    // Streak: get all test dates
    let history = test_repo.get_history(1000, 0, None)?;
    let dates: Vec<String> = history
        .iter()
        .map(|t| t.created_at.split('T').next().unwrap_or("").to_string())
        .filter(|d| !d.is_empty())
        .collect();
    let (current_streak, longest_streak) = racoon_core::StreakEngine::streak_from_dates(&dates);

    let daily_goal_met = if today_stats.is_some() {
        conn.query_row(
            "SELECT daily_goal_met FROM daily_stats WHERE date = ?1",
            params![today],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0)
            == 1
    } else {
        false
    };

    Ok(DashboardStatsResponse {
        current_streak,
        longest_streak,
        avg_wpm,
        avg_accuracy,
        tests_today,
        tests_this_week,
        total_tests: total,
        daily_goal_met,
    })
}

#[tauri::command]
pub fn get_progress_history(
    state: State<'_, AppState>,
    days: Option<u32>,
) -> Result<Vec<ProgressPoint>, AppError> {
    let db = state.db.lock()?;
    let conn = db.conn();
    let daily_repo = SqliteDailyStatsRepository::new(&conn);

    let d = days.unwrap_or(30);
    let from = (chrono::Utc::now() - chrono::Duration::days(d as i64))
        .format("%Y-%m-%d")
        .to_string();
    let to = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let stats = daily_repo.get_range(&from, &to)?;

    let points: Vec<ProgressPoint> = stats
        .iter()
        .map(|s| ProgressPoint {
            date: s.date.clone(),
            wpm: s.avg_wpm,
            accuracy: s.avg_accuracy,
            tests: s.total_tests,
        })
        .collect();

    Ok(points)
}

// ── Analytics ──

#[tauri::command]
pub fn get_achievements(state: State<'_, AppState>) -> Result<Vec<serde_json::Value>, AppError> {
    let db = state.db.lock()?;
    let conn = db.conn();
    let test_repo = SqliteTestRepository::new(&conn);
    let lesson_repo = SqliteLessonRepository::new(&conn);

    let total_tests = test_repo.get_count(None).unwrap_or(0);
    let history = test_repo.get_history(500, 0, None).unwrap_or_default();
    let best_wpm = history.iter().map(|t| t.wpm).fold(0.0_f64, f64::max);
    let best_acc = history.iter().map(|t| t.accuracy).fold(0.0_f64, f64::max);

    let dates: Vec<String> = history
        .iter()
        .map(|t| t.created_at.split('T').next().unwrap_or("").to_string())
        .filter(|d| !d.is_empty())
        .collect();
    let (_, longest_streak) = racoon_core::StreakEngine::streak_from_dates(&dates);

    let lessons = lesson_repo.get_progress("en").unwrap_or_default();
    let lessons_completed = lessons.iter().filter(|l| l.status == "completed").count() as i64;
    let lessons_ru = lesson_repo.get_progress("ru").unwrap_or_default();
    let lessons_completed_ru = lessons_ru
        .iter()
        .filter(|l| l.status == "completed")
        .count() as i64;

    let achievements = racoon_core::analytics::check_achievements(
        total_tests,
        best_wpm,
        best_acc,
        0,
        longest_streak,
        lessons_completed + lessons_completed_ru,
    );

    serde_json::to_value(achievements)
        .map(|v| vec![v])
        .map_err(AppError::from)
}

#[tauri::command]
pub fn get_insights(state: State<'_, AppState>) -> Result<Vec<serde_json::Value>, AppError> {
    let db = state.db.lock()?;
    let conn = db.conn();
    let daily_repo = SqliteDailyStatsRepository::new(&conn);
    let test_repo = SqliteTestRepository::new(&conn);

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let week_ago = (chrono::Utc::now() - chrono::Duration::days(7))
        .format("%Y-%m-%d")
        .to_string();
    let week_stats = daily_repo.get_range(&week_ago, &today).unwrap_or_default();

    let avg_wpm: f64 = if week_stats.is_empty() {
        0.0
    } else {
        let total: i64 = week_stats.iter().map(|s| s.total_tests).sum();
        let weighted: f64 = week_stats
            .iter()
            .map(|s| s.avg_wpm * s.total_tests as f64)
            .sum();
        if total > 0 {
            weighted / total as f64
        } else {
            0.0
        }
    };

    let avg_acc: f64 = if week_stats.is_empty() {
        0.0
    } else {
        let total: i64 = week_stats.iter().map(|s| s.total_tests).sum();
        let weighted: f64 = week_stats
            .iter()
            .map(|s| s.avg_accuracy * s.total_tests as f64)
            .sum();
        if total > 0 {
            weighted / total as f64
        } else {
            0.0
        }
    };

    let history = test_repo.get_history(100, 0, None).unwrap_or_default();
    let wpm_samples: Vec<f64> = history.iter().map(|t| t.wpm).collect();
    let consistency = racoon_core::consistency::calc_consistency(&wpm_samples);

    let insights = racoon_core::analytics::generate_insights(
        avg_wpm,
        avg_acc,
        consistency.score,
        0, // weak_key_count — needs engine state
        0, // streak — simplified
    );

    serde_json::to_value(insights)
        .map(|v| vec![v])
        .map_err(AppError::from)
}

#[tauri::command]
pub fn get_consistency(state: State<'_, AppState>) -> Result<serde_json::Value, AppError> {
    let db = state.db.lock()?;
    let conn = db.conn();
    let test_repo = SqliteTestRepository::new(&conn);
    let history = test_repo.get_history(100, 0, None)?;
    let wpm_samples: Vec<f64> = history.iter().map(|t| t.wpm).collect();
    let report = racoon_core::consistency::calc_consistency(&wpm_samples);
    serde_json::to_value(report).map_err(AppError::from)
}

#[tauri::command]
pub fn export_data(state: State<'_, AppState>, format: String) -> Result<String, AppError> {
    let db = state.db.lock()?;
    let conn = db.conn();
    let test_repo = SqliteTestRepository::new(&conn);
    let history = test_repo.get_history(1000, 0, None)?;

    match format.as_str() {
        "json" => {
            let data = serde_json::json!({
                "tests": history.iter().map(|t| serde_json::json!({
                    "date": t.created_at,
                    "mode": t.mode_type,
                    "wpm": t.wpm,
                    "accuracy": t.accuracy,
                    "duration_ms": t.duration_ms,
                })).collect::<Vec<_>>(),
            });
            Ok(racoon_core::analytics::export_json(&data))
        }
        "csv" => {
            let mut rows = vec![vec![
                "Date".to_string(),
                "Mode".to_string(),
                "WPM".to_string(),
                "Accuracy".to_string(),
                "Duration_ms".to_string(),
            ]];
            for t in &history {
                rows.push(vec![
                    t.created_at.clone(),
                    t.mode_type.clone(),
                    format!("{:.1}", t.wpm),
                    format!("{:.1}", t.accuracy),
                    t.duration_ms.to_string(),
                ]);
            }
            Ok(racoon_core::analytics::export_csv(&rows))
        }
        _ => Err(AppError::Internal(format!("Unknown format: {}", format))),
    }
}

// ── Replay ──

#[tauri::command]
pub fn get_replay(
    state: State<'_, AppState>,
    test_id: i64,
) -> Result<Vec<racoon_data::repository::ReplayFrame>, AppError> {
    let db = state.db.lock()?;
    let conn = db.conn();
    let repo = SqliteReplayRepository::new(&conn);
    repo.load_replay(test_id).map_err(AppError::from)
}

#[tauri::command]
pub fn has_replay(state: State<'_, AppState>, test_id: i64) -> Result<bool, AppError> {
    let db = state.db.lock()?;
    let conn = db.conn();
    let repo = SqliteReplayRepository::new(&conn);
    repo.has_replay(test_id).map_err(AppError::from)
}

// ── Sound ──

#[derive(Debug, serde::Serialize)]
pub struct SoundOutputResponse {
    pub frequency: f64,
    pub duration_ms: u64,
    pub volume: f64,
    pub event: String,
}

#[tauri::command]
pub fn get_sound_event(
    _engine_state: State<'_, Mutex<CoreEngine>>,
    state: State<'_, AppState>,
    event: String,
) -> Result<Option<SoundOutputResponse>, AppError> {
    let settings = state.settings_store().load()?;

    if !settings.sound_enabled {
        return Ok(None);
    }

    let sound_event = match event.as_str() {
        "key_press" => racoon_core::sound::SoundEvent::KeyPress,
        "error" => racoon_core::sound::SoundEvent::Error,
        "lesson_complete" => racoon_core::sound::SoundEvent::LessonComplete,
        "achievement_unlocked" => racoon_core::sound::SoundEvent::AchievementUnlocked,
        _ => return Ok(None),
    };

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let mut sound_engine = racoon_core::sound::SoundEngine::new(racoon_core::sound::SoundConfig {
        enabled: settings.sound_enabled,
        volume: settings.sound_volume,
    });

    let output = sound_engine.try_play(sound_event, now_ms);
    Ok(output.map(|o| SoundOutputResponse {
        frequency: o.frequency,
        duration_ms: o.duration_ms,
        volume: o.volume,
        event: event.clone(),
    }))
}

// ── Helpers ──

struct CompletedSession {
    final_stats: FinalStats,
    mode_type: String,
    mode_config: serde_json::Value,
    language: String,
    text_length: usize,
    replay_frames: Vec<racoon_core::ReplayFrame>,
}

fn test_record_from_completion(completed: &CompletedSession) -> TestRecord {
    TestRecord {
        created_at: chrono::Utc::now().to_rfc3339(),
        mode_type: completed.mode_type.clone(),
        mode_config: completed.mode_config.clone(),
        language: completed.language.clone(),
        text_length: completed.text_length,
        duration_ms: completed.final_stats.duration_ms,
        wpm: completed.final_stats.wpm,
        raw_wpm: completed.final_stats.raw_wpm,
        accuracy: completed.final_stats.accuracy,
        raw_accuracy: completed.final_stats.raw_accuracy,
        consistency: completed.final_stats.consistency,
        correct_chars: completed.final_stats.correct_chars,
        incorrect_chars: completed.final_stats.incorrect_chars,
        backspaces: completed.final_stats.backspaces,
        char_stats: completed.final_stats.char_stats.clone(),
        heatmap_data: completed.final_stats.heatmap.clone(),
        graph_data: completed.final_stats.graph_data.clone(),
        is_pb: false,
        tags: String::new(),
    }
}

fn persist_daily_streak(conn: &Connection, today: &str) -> Result<(), AppError> {
    let existing = conn
        .query_row(
            "SELECT current_streak, longest_streak, last_date, started_date
             FROM streaks WHERE type = 'daily_test'",
            [],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                ))
            },
        )
        .optional()
        .map_err(|error| AppError::DbQuery(error.to_string()))?;

    let (previous_current, previous_longest, last_date, previous_started_date) =
        existing.unwrap_or((0, 0, None, None));
    let starts_new_streak = last_date
        .as_deref()
        .is_none_or(|last| racoon_core::StreakEngine::days_between(last, today) > 1);
    let (current, longest, _) = racoon_core::StreakEngine::compute_streak(
        previous_current,
        previous_longest,
        last_date.as_deref(),
        today,
    );
    let started_date = if starts_new_streak {
        today.to_string()
    } else {
        previous_started_date.unwrap_or_else(|| today.to_string())
    };

    conn.execute(
        "INSERT INTO streaks (
            type, current_streak, longest_streak, last_date, started_date
         ) VALUES ('daily_test', ?1, ?2, ?3, ?4)
         ON CONFLICT(type) DO UPDATE SET
            current_streak = excluded.current_streak,
            longest_streak = excluded.longest_streak,
            last_date = excluded.last_date,
            started_date = excluded.started_date",
        params![current, longest, today, started_date],
    )
    .map_err(|error| AppError::DbWrite(error.to_string()))?;
    Ok(())
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completed_session_record_preserves_unicode_text_length() {
        let completed = CompletedSession {
            final_stats: FinalStats {
                wpm: 40.0,
                raw_wpm: 42.0,
                accuracy: 98.0,
                raw_accuracy: 96.0,
                consistency: Some(90.0),
                correct_chars: 6,
                incorrect_chars: 0,
                backspaces: 0,
                char_stats: serde_json::json!({}),
                heatmap: serde_json::json!({}),
                graph_data: Some(serde_json::json!([])),
                duration_ms: 10_000,
            },
            mode_type: "custom".to_string(),
            mode_config: serde_json::json!({"language": "ru"}),
            language: "ru".to_string(),
            text_length: "привет".chars().count(),
            replay_frames: Vec::new(),
        };

        assert_eq!(test_record_from_completion(&completed).text_length, 6);
    }

    #[test]
    fn daily_streak_is_persisted() {
        let database = racoon_data::Database::open_in_memory().unwrap();
        let conn = database.conn();
        persist_daily_streak(&conn, "2026-07-10").unwrap();
        persist_daily_streak(&conn, "2026-07-11").unwrap();

        let streak = conn
            .query_row(
                "SELECT current_streak, longest_streak FROM streaks WHERE type = 'daily_test'",
                [],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
            )
            .unwrap();
        assert_eq!(streak, (2, 2));
    }
}
