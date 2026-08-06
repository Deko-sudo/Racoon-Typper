//! Central validation for bounded Tauri command inputs.

use crate::error::AppError;

pub const MAX_PAGE_LIMIT: usize = 1_000;
const MAX_PAGE_OFFSET: usize = 1_000_000;
const MAX_SEARCH_QUERY_CHARS: usize = 200;
const MAX_LANGUAGE_CODE_CHARS: usize = 16;
const MAX_KEY_CHARS: usize = 16;
const MAX_KEY_CODE_CHARS: usize = 64;
const MAX_TEST_DURATION_SECS: u64 = 3_600;
const MAX_WORD_COUNT: usize = 1_000;
const MAX_PROGRESS_DAYS: u32 = 3_650;
const MAX_THEME_NAME_CHARS: usize = 64;
const MAX_EXPORT_FORMAT_CHARS: usize = 16;
const MAX_SOUND_EVENT_CHARS: usize = 32;
const MAX_SETTING_KEY_CHARS: usize = 64;
const MAX_MODE_CHARS: usize = 16;
const MAX_CUSTOM_TEST_CHARS: usize = racoon_data::repository::MAX_CUSTOM_TEXT_LENGTH;

pub fn validate_language(language: String) -> Result<String, AppError> {
    if language.is_empty()
        || language.chars().count() > MAX_LANGUAGE_CODE_CHARS
        || !language
            .chars()
            .all(|character| character.is_ascii_lowercase() || character == '-')
    {
        return Err(AppError::InvalidConfig(format!(
            "language must be a lowercase code of at most {MAX_LANGUAGE_CODE_CHARS} characters"
        )));
    }
    Ok(language)
}

pub fn validate_resource_identifier(value: &str, kind: &str) -> Result<(), AppError> {
    if value.is_empty()
        || value.chars().count() > 128
        || !value.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '_' || character == '-'
        })
    {
        return Err(AppError::InvalidConfig(format!(
            "{kind} identifier contains unsupported characters"
        )));
    }
    Ok(())
}

pub fn validate_duration(duration_secs: u64) -> Result<(), AppError> {
    if !(1..=MAX_TEST_DURATION_SECS).contains(&duration_secs) {
        return Err(AppError::InvalidConfig(format!(
            "duration must be between 1 and {MAX_TEST_DURATION_SECS} seconds"
        )));
    }
    Ok(())
}

pub fn validate_word_count(word_count: usize) -> Result<(), AppError> {
    if !(1..=MAX_WORD_COUNT).contains(&word_count) {
        return Err(AppError::InvalidConfig(format!(
            "word_count must be between 1 and {MAX_WORD_COUNT}"
        )));
    }
    Ok(())
}

pub fn validate_test_text(text: String) -> Result<String, AppError> {
    let length = text.chars().count();
    if text.trim().is_empty() {
        return Err(AppError::CustomTextEmpty);
    }
    if length > MAX_CUSTOM_TEST_CHARS {
        return Err(AppError::InvalidConfig(format!(
            "test text must contain at most {MAX_CUSTOM_TEST_CHARS} characters"
        )));
    }
    Ok(text)
}

pub fn validate_key_event(key: &str, code: &str) -> Result<(), AppError> {
    if key.is_empty() && code.is_empty() {
        // The frontend sends this bounded, explicit tick to ask the backend to
        // evaluate an elapsed time-mode session.
        return Ok(());
    }
    if key.is_empty()
        || code.is_empty()
        || key.chars().count() > MAX_KEY_CHARS
        || code.chars().count() > MAX_KEY_CODE_CHARS
        || key.chars().any(char::is_control)
        || code.chars().any(char::is_control)
    {
        return Err(AppError::InvalidKey);
    }
    Ok(())
}

pub fn validate_page_limit(limit: usize) -> Result<usize, AppError> {
    if !(1..=MAX_PAGE_LIMIT).contains(&limit) {
        return Err(AppError::InvalidConfig(format!(
            "limit must be between 1 and {MAX_PAGE_LIMIT}"
        )));
    }
    Ok(limit)
}

pub fn validate_page_offset(offset: usize) -> Result<usize, AppError> {
    if offset > MAX_PAGE_OFFSET {
        return Err(AppError::InvalidConfig(format!(
            "offset must be between 0 and {MAX_PAGE_OFFSET}"
        )));
    }
    Ok(offset)
}

pub fn validate_progress_days(days: u32) -> Result<u32, AppError> {
    if days > MAX_PROGRESS_DAYS {
        return Err(AppError::InvalidConfig(format!(
            "days must be between 0 and {MAX_PROGRESS_DAYS}"
        )));
    }
    Ok(days)
}

pub fn validate_positive_id(id: i64, kind: &str) -> Result<(), AppError> {
    if id <= 0 {
        return Err(AppError::InvalidConfig(format!(
            "{kind} id must be positive"
        )));
    }
    Ok(())
}

pub fn validate_mode_filter(mode: &str) -> Result<&str, AppError> {
    if matches!(mode, "time" | "words" | "quote" | "custom" | "lesson") {
        Ok(mode)
    } else {
        Err(AppError::InvalidMode(mode.to_string()))
    }
}

pub fn validate_test_mode(mode: &str) -> Result<(), AppError> {
    validate_bounded_token(mode, MAX_MODE_CHARS, "mode")?;
    if matches!(mode, "time" | "words" | "quote" | "custom") {
        Ok(())
    } else {
        Err(AppError::InvalidMode(mode.to_string()))
    }
}

pub fn validate_search_query(query: &str) -> Result<(), AppError> {
    if query.chars().count() > MAX_SEARCH_QUERY_CHARS {
        return Err(AppError::InvalidConfig(format!(
            "search query must contain at most {MAX_SEARCH_QUERY_CHARS} characters"
        )));
    }
    Ok(())
}

pub fn validate_theme_name(name: &str) -> Result<(), AppError> {
    validate_bounded_token(name, MAX_THEME_NAME_CHARS, "theme name")
}

pub fn validate_export_format(format: &str) -> Result<(), AppError> {
    validate_bounded_token(format, MAX_EXPORT_FORMAT_CHARS, "export format")
}

pub fn validate_sound_event(event: &str) -> Result<(), AppError> {
    validate_bounded_token(event, MAX_SOUND_EVENT_CHARS, "sound event")
}

pub fn validate_setting_key(key: &str) -> Result<(), AppError> {
    validate_bounded_token(key, MAX_SETTING_KEY_CHARS, "setting key")?;
    if !key
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '_')
    {
        return Err(AppError::InvalidConfig(
            "setting key contains unsupported characters".to_string(),
        ));
    }
    Ok(())
}

fn validate_bounded_token(value: &str, max_chars: usize, kind: &str) -> Result<(), AppError> {
    if value.is_empty() || value.chars().count() > max_chars || value.chars().any(char::is_control)
    {
        return Err(AppError::InvalidConfig(format!(
            "{kind} must contain between 1 and {max_chars} non-control characters"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unsafe_or_unbounded_inputs() {
        assert!(validate_duration(0).is_err());
        assert!(validate_duration(MAX_TEST_DURATION_SECS + 1).is_err());
        assert!(validate_word_count(0).is_err());
        assert!(validate_language("EN".to_string()).is_err());
        assert!(validate_key_event("a", "").is_err());
        assert!(validate_key_event("a\u{0000}", "KeyA").is_err());
        assert!(validate_page_offset(MAX_PAGE_OFFSET + 1).is_err());
        assert!(validate_theme_name("\u{0000}").is_err());
        assert!(validate_test_mode(&"x".repeat(MAX_MODE_CHARS + 1)).is_err());
    }

    #[test]
    fn rejects_traversal_oversize_and_repeated_hostile_inputs() {
        for identifier in ["../lesson", "lesson/one", "lesson\\one", ".", ".."] {
            assert!(validate_resource_identifier(identifier, "lesson").is_err());
        }
        assert!(validate_resource_identifier(&"a".repeat(129), "lesson").is_err());
        assert!(validate_setting_key("../verbose_logging").is_err());
        assert!(validate_search_query(&"q".repeat(MAX_SEARCH_QUERY_CHARS + 1)).is_err());

        let oversized_text = "x".repeat(MAX_CUSTOM_TEST_CHARS + 1);
        for _ in 0..8 {
            assert!(validate_test_text(oversized_text.clone()).is_err());
            assert!(validate_key_event(&"k".repeat(MAX_KEY_CHARS + 1), "KeyK").is_err());
        }
    }
}
