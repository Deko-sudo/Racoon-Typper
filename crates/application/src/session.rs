//! Session use-case data shared by transport adapters and application ports.

use std::fmt;

use chrono::{DateTime, Utc};
use racoon_core::ReplayFrame;
use racoon_domain::{FinalStats, SessionId, TestId};

/// Backend-facing request for starting a standard typing session.
///
/// This is intentionally transport-neutral: Tauri command arguments are
/// converted into this value before they reach application orchestration.
#[derive(Clone, PartialEq, Eq)]
pub struct SessionStartRequest {
    pub mode: String,
    pub text: Option<String>,
    pub duration: Option<u64>,
    pub word_count: Option<usize>,
    pub quote_id: Option<i64>,
    pub language: Option<String>,
}

impl fmt::Debug for SessionStartRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SessionStartRequest")
            .field("mode", &self.mode)
            .field("text_present", &self.text.is_some())
            .field(
                "text_length",
                &self.text.as_ref().map(|text| text.chars().count()),
            )
            .field("duration", &self.duration)
            .field("word_count", &self.word_count)
            .field("quote_id", &self.quote_id)
            .field("language", &self.language)
            .finish()
    }
}

/// Immutable completion snapshot handed to the persistence port.
///
/// The snapshot contains all values needed to persist the completion and is
/// kept separate from transport DTOs so a retry can reuse the same result.
#[derive(Clone)]
pub struct SessionCompletion {
    pub session_id: SessionId,
    pub completed_at: DateTime<Utc>,
    pub final_stats: FinalStats,
    pub mode_type: String,
    pub mode_config: serde_json::Value,
    pub language: String,
    pub text_length: usize,
    pub replay_frames: Vec<ReplayFrame>,
    pub lesson_id: Option<String>,
}

impl fmt::Debug for SessionCompletion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SessionCompletion")
            .field("session_id", &self.session_id)
            .field("completed_at", &self.completed_at)
            .field("mode_type", &self.mode_type)
            .field("language", &self.language)
            .field("text_length", &self.text_length)
            .field("replay_frame_count", &self.replay_frames.len())
            .field("lesson_id_present", &self.lesson_id.is_some())
            .finish()
    }
}

/// Result returned by a completion store after its transaction commits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionPersistenceReceipt {
    pub test_id: TestId,
}

#[cfg(test)]
mod tests {
    use super::*;
    use racoon_core::ReplayFrame;
    use racoon_domain::CharStatus;

    #[test]
    fn session_start_debug_redacts_custom_text() {
        let secret = "typed-secret-content";
        let request = SessionStartRequest {
            mode: "custom".to_string(),
            text: Some(secret.to_string()),
            duration: None,
            word_count: None,
            quote_id: None,
            language: Some("en".to_string()),
        };

        let diagnostic = format!("{request:?}");
        assert!(!diagnostic.contains(secret));
        assert!(diagnostic.contains("text_length"));
    }

    #[test]
    fn session_completion_debug_redacts_mode_configuration_and_replay_content() {
        let secret = "typed-secret-content";
        let completion = SessionCompletion {
            session_id: SessionId::from("018f0c2e-7b8d-7abc-8def-0123456789ab"),
            completed_at: DateTime::from_timestamp(1_752_969_600, 0).unwrap(),
            final_stats: FinalStats {
                wpm: 60.0,
                raw_wpm: 60.0,
                accuracy: 1.0,
                raw_accuracy: 1.0,
                consistency: None,
                correct_chars: 1,
                incorrect_chars: 0,
                backspaces: 0,
                char_stats: serde_json::json!({"content": secret}),
                heatmap: serde_json::json!({"content": secret}),
                graph_data: None,
                duration_ms: 1,
            },
            mode_type: "custom".to_string(),
            mode_config: serde_json::json!({"custom_text": secret}),
            language: "en".to_string(),
            text_length: secret.chars().count(),
            replay_frames: vec![ReplayFrame {
                timestamp_ms: 1,
                key: secret.to_string(),
                caret_pos: 1,
                char_status: CharStatus::Correct,
                expected_char: 'a',
                typed_char: Some('a'),
            }],
            lesson_id: None,
        };

        let diagnostic = format!("{completion:?}");
        assert!(!diagnostic.contains(secret));
        assert!(diagnostic.contains("replay_frame_count"));
        assert!(diagnostic.contains("text_length"));
    }
}
