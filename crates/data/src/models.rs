//! Row-структуры для DB запросов (внутренние).

use racoon_domain::{PersonalBest, TestDetail, TestSummary};

/// Внутренняя структура для маппинга строки tests → TestSummary.
#[derive(Debug, Clone)]
pub struct TestRow {
    pub id: i64,
    pub created_at: String,
    pub mode_type: String,
    pub mode_config: String,
    pub language: String,
    pub text_length: i64,
    pub duration_ms: i64,
    pub wpm: f64,
    pub raw_wpm: f64,
    pub accuracy: f64,
    pub raw_accuracy: f64,
    pub consistency: Option<f64>,
    pub correct_chars: i64,
    pub incorrect_chars: i64,
    pub backspaces: i64,
    pub char_stats: String,
    pub heatmap_data: String,
    pub graph_data: Option<String>,
    pub is_pb: bool,
    pub tags: String,
}

impl From<TestRow> for TestSummary {
    fn from(row: TestRow) -> Self {
        TestSummary {
            id: row.id,
            created_at: row.created_at,
            mode_type: row.mode_type,
            mode_config: serde_json::from_str(&row.mode_config).unwrap_or(serde_json::Value::Null),
            language: row.language,
            wpm: row.wpm,
            raw_wpm: row.raw_wpm,
            accuracy: row.accuracy,
            raw_accuracy: row.raw_accuracy,
            consistency: row.consistency,
            duration_ms: row.duration_ms as u64,
            is_pb: row.is_pb,
        }
    }
}

impl From<TestRow> for TestDetail {
    fn from(row: TestRow) -> Self {
        TestDetail {
            id: row.id,
            created_at: row.created_at,
            mode_type: row.mode_type,
            mode_config: serde_json::from_str(&row.mode_config).unwrap_or(serde_json::Value::Null),
            language: row.language,
            text_length: row.text_length as usize,
            duration_ms: row.duration_ms as u64,
            wpm: row.wpm,
            raw_wpm: row.raw_wpm,
            accuracy: row.accuracy,
            raw_accuracy: row.raw_accuracy,
            consistency: row.consistency,
            correct_chars: row.correct_chars as usize,
            incorrect_chars: row.incorrect_chars as usize,
            backspaces: row.backspaces as usize,
            char_stats: serde_json::from_str(&row.char_stats).unwrap_or(serde_json::Value::Null),
            heatmap_data: serde_json::from_str(&row.heatmap_data)
                .unwrap_or(serde_json::Value::Null),
            graph_data: row
                .graph_data
                .as_ref()
                .and_then(|s| serde_json::from_str(s).ok()),
            is_pb: row.is_pb,
            tags: row.tags,
        }
    }
}

/// Внутренняя структура для personal_bests.
#[derive(Debug, Clone)]
pub struct PersonalBestRow {
    pub id: i64,
    pub mode_type: String,
    pub mode_config_hash: String,
    pub mode_config: String,
    pub best_wpm: f64,
    pub best_wpm_test_id: Option<i64>,
    pub best_accuracy: f64,
    pub best_accuracy_test_id: Option<i64>,
    pub best_consistency: Option<f64>,
    pub best_consistency_test_id: Option<i64>,
    pub updated_at: String,
}

impl From<PersonalBestRow> for PersonalBest {
    fn from(row: PersonalBestRow) -> Self {
        PersonalBest {
            mode_type: row.mode_type,
            mode_config: serde_json::from_str(&row.mode_config).unwrap_or(serde_json::Value::Null),
            best_wpm: row.best_wpm,
            best_wpm_test_id: row.best_wpm_test_id,
            best_accuracy: row.best_accuracy,
            best_accuracy_test_id: row.best_accuracy_test_id,
            best_consistency: row.best_consistency,
            best_consistency_test_id: row.best_consistency_test_id,
            updated_at: row.updated_at,
        }
    }
}

/// Хэш конфигурации для personal_bests (простой hash).
pub fn config_hash(mode_type: &str, mode_config: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    mode_type.hash(&mut hasher);
    mode_config.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}
