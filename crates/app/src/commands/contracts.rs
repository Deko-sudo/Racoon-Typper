//! Explicit response DTOs for the Tauri command boundary.

use racoon_core::TestSessionInfo;
use racoon_domain::TestSummary;

#[derive(Debug, serde::Serialize)]
pub struct TestSessionResponse {
    pub session_id: String,
    pub text: String,
    pub text_length: usize,
    pub mode_type: String,
    pub mode_config: serde_json::Value,
    pub language: String,
}

impl TestSessionResponse {
    pub fn from_session_info(info: TestSessionInfo) -> Self {
        Self {
            session_id: info.session_id,
            text: info.text,
            text_length: info.text_length,
            mode_type: info.mode_type,
            mode_config: info.mode_config,
            language: info.language,
        }
    }

    pub fn lesson_from_session_info(info: TestSessionInfo) -> Self {
        Self {
            mode_type: "lesson".to_string(),
            ..Self::from_session_info(info)
        }
    }
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

#[derive(Debug, serde::Serialize)]
pub struct SoundOutputResponse {
    pub frequency: f64,
    pub duration_ms: u64,
    pub volume: f64,
    pub event: String,
}
