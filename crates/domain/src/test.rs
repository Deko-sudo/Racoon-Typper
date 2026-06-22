//! Типы тестов: записи, результаты, конфигурации.

use serde::{Deserialize, Serialize};

use crate::ids::TestId;

/// Тип режима теста.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ModeType {
    Time,
    Words,
    Quote,
    Custom,
}

/// Конфигурация режима (JSON-совместимая).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub word_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    pub language: String,
    #[serde(default)]
    pub punctuation: bool,
    #[serde(default)]
    pub numbers: bool,
}

/// Краткая запись теста для истории.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSummary {
    pub id: TestId,
    pub created_at: String,
    pub mode_type: String,
    pub mode_config: serde_json::Value,
    pub language: String,
    pub wpm: f64,
    pub raw_wpm: f64,
    pub accuracy: f64,
    pub raw_accuracy: f64,
    pub consistency: Option<f64>,
    pub duration_ms: u64,
    pub is_pb: bool,
}

/// Детальная запись теста.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestDetail {
    pub id: TestId,
    pub created_at: String,
    pub mode_type: String,
    pub mode_config: serde_json::Value,
    pub language: String,
    pub text_length: usize,
    pub duration_ms: u64,
    pub wpm: f64,
    pub raw_wpm: f64,
    pub accuracy: f64,
    pub raw_accuracy: f64,
    pub consistency: Option<f64>,
    pub correct_chars: usize,
    pub incorrect_chars: usize,
    pub backspaces: usize,
    pub char_stats: serde_json::Value,
    pub heatmap_data: serde_json::Value,
    pub graph_data: Option<serde_json::Value>,
    pub is_pb: bool,
    pub tags: String,
}

/// Запись теста для сохранения в БД.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRecord {
    pub created_at: String,
    pub mode_type: String,
    pub mode_config: serde_json::Value,
    pub language: String,
    pub text_length: usize,
    pub duration_ms: u64,
    pub wpm: f64,
    pub raw_wpm: f64,
    pub accuracy: f64,
    pub raw_accuracy: f64,
    pub consistency: Option<f64>,
    pub correct_chars: usize,
    pub incorrect_chars: usize,
    pub backspaces: usize,
    pub char_stats: serde_json::Value,
    pub heatmap_data: serde_json::Value,
    pub graph_data: Option<serde_json::Value>,
    pub is_pb: bool,
    pub tags: String,
}

/// Личный рекорд.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalBest {
    pub mode_type: String,
    pub mode_config: serde_json::Value,
    pub best_wpm: f64,
    pub best_wpm_test_id: Option<TestId>,
    pub best_accuracy: f64,
    pub best_accuracy_test_id: Option<TestId>,
    pub best_consistency: Option<f64>,
    pub best_consistency_test_id: Option<TestId>,
    pub updated_at: String,
}
