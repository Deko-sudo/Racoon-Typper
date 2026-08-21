//! Tauri adapters for local content, custom texts, lessons, and weak-key data.

use racoon_core::{
    AdaptiveTextGenerator, CoreEngine, FrequencyAdaptiveGenerator, WeakKeysAnalyzer, WeakKeysReport,
};
use racoon_data::repository::{
    CustomTextRepository, LessonProgressRecord, LessonRepository, SqliteCustomTextRepository,
    SqliteLessonRepository, SqliteTestRepository, TestRepository,
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
    validate_test_text, validate_word_count,
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
    state: State<'_, AppState>,
) -> Result<WeakKeysReport, AppError> {
    let engine = engine_state.lock()?;
    let char_stats = engine.current_char_stats().unwrap_or_default();
    // Fallback: если текущая сессия пуста (например, после перезапуска),
    // используем агрегированный heatmap из истории тестов.
    let char_stats = if char_stats.is_empty() {
        aggregated_char_stats(&state, 50)?
    } else {
        char_stats
    };
    Ok(WeakKeysAnalyzer::new().analyze(&char_stats))
}

#[tauri::command]
pub(crate) fn generate_weak_keys_training(
    engine_state: State<'_, Mutex<CoreEngine>>,
    state: State<'_, AppState>,
    language: String,
    word_count: Option<usize>,
) -> Result<String, AppError> {
    let language = validate_language(language)?;
    let word_count = word_count.unwrap_or(25);
    validate_word_count(word_count)?;
    let engine = engine_state.lock()?;
    let char_stats = engine.current_char_stats().unwrap_or_default();
    let char_stats = if char_stats.is_empty() {
        aggregated_char_stats(&state, 50)?
    } else {
        char_stats
    };
    let words = word_pack_loader()
        .get_pack(&language)
        .map(|pack| pack.words.clone())
        .ok_or_else(|| AppError::WordsEmpty(language.clone()))?;
    let generator = FrequencyAdaptiveGenerator::new(words);
    let weak_chars = generator.analyze(&char_stats);

    Ok(generator.generate(&weak_chars, word_count))
}

/// Загружает агрегированный heatmap из последних N тестов и конвертирует его
/// в CharStatsMap для weak-keys анализа. Используется как fallback, когда
/// текущая in-memory сессия пуста.
fn aggregated_char_stats(
    state: &State<'_, AppState>,
    recent_count: usize,
) -> Result<racoon_domain::keyboard::CharStatsMap, AppError> {
    with_db(state, |conn| {
        let repo = SqliteTestRepository::new(conn);
        let rows = repo.get_recent_heatmaps(recent_count, None)?;
        let heatmap = racoon_core::merge_heatmaps(&rows);
        Ok(racoon_core::heatmap_to_char_stats(&heatmap))
    })
}

/// Максимальный размер ответа при импорте текста по URL (1 MiB).
const MAX_URL_FETCH_BYTES: usize = 1_048_576;

/// Таймаут HTTP-запроса при импорте по URL (10 секунд).
const URL_FETCH_TIMEOUT_SECS: u64 = 10;

/// Импортирует текст по URL: fetch → strip HTML-тегов → валидация.
///
/// Принимает только `https://` URLs. Возвращает очищенный текст (без HTML-разметки),
/// готовый для сохранения как custom text. Текст проходит через `validate_test_text`,
/// поэтому наследует лимит в 10 000 символов.
#[tauri::command]
pub(crate) fn import_text_from_url(url: String) -> Result<String, AppError> {
    // Валидация URL: только HTTPS, разумная длина.
    if url.len() > 2048 {
        return Err(AppError::InvalidConfig("URL too long".to_string()));
    }
    if !url.starts_with("https://") {
        return Err(AppError::InvalidConfig(
            "Only HTTPS URLs are supported".to_string(),
        ));
    }

    let response = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(URL_FETCH_TIMEOUT_SECS))
        .build()
        .map_err(|e| AppError::InvalidConfig(format!("HTTP client error: {e}")))?
        .get(&url)
        .header("User-Agent", "RacoonTypper/1.1")
        .send()
        .map_err(|e| AppError::InvalidConfig(format!("Fetch failed: {e}")))?;

    if !response.status().is_success() {
        return Err(AppError::InvalidConfig(format!(
            "Server returned HTTP {}",
            response.status()
        )));
    }

    // Ограничиваем размер ответа, чтобы предотвратить загрузку огромных файлов.
    let content_length = response.content_length().unwrap_or(0) as usize;
    if content_length > MAX_URL_FETCH_BYTES * 4 {
        return Err(AppError::InvalidConfig(format!(
            "Response too large (max {} bytes)",
            MAX_URL_FETCH_BYTES * 4
        )));
    }

    let body = response
        .text()
        .map_err(|e| AppError::InvalidConfig(format!("Read body failed: {e}")))?;

    let body = if body.len() > MAX_URL_FETCH_BYTES * 4 {
        body.chars()
            .take(MAX_URL_FETCH_BYTES * 4)
            .collect::<String>()
    } else {
        body
    };

    // Извлечение текста из HTML: убираем script/style, затем теги, декодируем сущности.
    let text = strip_html(&body);

    validate_test_text(text)
}

/// Простой HTML→text экстрактор: убирает script/style блоки, теги, сущности.
/// Нормализует пробелы и пустые строки. Не претендует на полную HTML-обработку,
/// но достаточен для извлечения основного контента статей/глав.
/// Returns the byte position of the first ASCII case-insensitive occurrence of
/// `needle` in `haystack`, or `None`.
///
/// HTML tag names are ASCII, so an ASCII-only case-insensitive comparison is
/// sufficient and never changes string length. Iterating with `char_indices`
/// guarantees the returned position is a UTF-8 char boundary of `haystack`.
fn find_ascii_case_insensitive(haystack: &str, needle: &str) -> Option<usize> {
    haystack.char_indices().find_map(|(index, _)| {
        haystack[index..]
            .get(..needle.len())
            .filter(|candidate| {
                candidate
                    .as_bytes()
                    .iter()
                    .zip(needle.as_bytes())
                    .all(|(a, b)| a.eq_ignore_ascii_case(b))
            })
            .map(|_| index)
    })
}

fn strip_html(html: &str) -> String {
    let mut result = html.to_string();

    // Убираем <script>...</script> и <style>...</style> целиком. Поиск ведётся
    // в оригинальной строке (ASCII case-insensitive): байтовые позиции из
    // lowercased-копии нельзя применять к оригиналу — Unicode-lowercase может
    // менять длину строки и приводить к панике или повреждению текста.
    for (open, close) in [("<script", "</script>"), ("<style", "</style>")] {
        while let Some(start) = find_ascii_case_insensitive(&result, open) {
            let Some(end) = find_ascii_case_insensitive(&result, close) else {
                break;
            };
            let end = end + close.len();
            if end <= start {
                // Закрывающий тег встретился раньше открывающего — не удаляем.
                break;
            }
            result.replace_range(start..end, "");
        }
    }

    // <br>, </p>, </div> → перенос строки.
    let result = regex_newline_replacements(&result);

    // Убираем все остальные HTML-теги.
    let mut text = String::with_capacity(result.len());
    let mut in_tag = false;
    for ch in result.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => text.push(ch),
            _ => {}
        }
    }

    // Декодируем базовые HTML-сущности.
    let text = text
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
        .replace("&mdash;", "—")
        .replace("&ndash;", "–")
        .replace("&hellip;", "…")
        .replace("&laquo;", "«")
        .replace("&raquo;", "»");

    // Нормализация пробелов: схлопываем множественные пробелы/переносы.
    let mut normalized = String::with_capacity(text.len());
    let mut prev_was_space = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !prev_was_space {
                normalized.push('\n');
                prev_was_space = true;
            }
        } else {
            // Схлопываем множественные пробелы внутри строки.
            let collapsed: String = trimmed.split_whitespace().collect::<Vec<_>>().join(" ");
            normalized.push_str(&collapsed);
            normalized.push('\n');
            prev_was_space = false;
        }
    }

    normalized.trim().to_string()
}

/// Заменяет закрывающие теги блочных элементов на переносы строк.
fn regex_newline_replacements(html: &str) -> String {
    let lower = html.to_lowercase();
    let mut result = String::with_capacity(html.len());
    let mut i = 0;
    let bytes = html.as_bytes();
    let lower_bytes = lower.as_bytes();

    while i < html.len() {
        // Ищем '<' и проверяем, начинается ли тег с блочного элемента.
        if lower_bytes[i] == b'<' {
            let rest = &lower[i..];
            if rest.starts_with("<br")
                || rest.starts_with("</p")
                || rest.starts_with("</div")
                || rest.starts_with("</h")
                || rest.starts_with("</li")
                || rest.starts_with("</tr")
                || rest.starts_with("<hr")
            {
                result.push('\n');
            }
        }
        // Копируем оригинальный символ (не lowercased).
        let ch = bytes[i] as char;
        if ch.is_ascii() {
            result.push(ch);
            i += 1;
        } else {
            // Многобайтовый UTF-8 — копируем целиком.
            let utf8_len = utf8_char_len(bytes[i]);
            if i + utf8_len <= html.len() {
                if let Ok(s) = std::str::from_utf8(&bytes[i..i + utf8_len]) {
                    result.push_str(s);
                }
            }
            i += utf8_len;
        }
    }
    result
}

/// Возвращает длину UTF-8 последовательности по первому байту.
fn utf8_char_len(first_byte: u8) -> usize {
    if first_byte < 0x80 {
        1
    } else if first_byte >> 5 == 0b110 {
        2
    } else if first_byte >> 4 == 0b1110 {
        3
    } else if first_byte >> 3 == 0b11110 {
        4
    } else {
        1
    }
}

#[cfg(test)]
mod strip_html_tests {
    use super::strip_html;

    #[test]
    fn ascii_script_removed() {
        assert_eq!(strip_html("a<script>alert(1)</script>b"), "ab");
    }

    #[test]
    fn ascii_style_removed() {
        assert_eq!(strip_html("a<style>p{color:red}</style>b"), "ab");
    }

    #[test]
    fn unicode_before_script_is_preserved() {
        // 'İ' lowercase expands to two chars; the old index-based removal
        // silently dropped the trailing text.
        assert_eq!(strip_html("İ<script>alert(1)</script>tail"), "İtail");
    }

    #[test]
    fn unicode_inside_script_body() {
        // Unicode inside the script body used to shift byte offsets and panic.
        assert_eq!(
            strip_html("<script>var x = \"İstanbul\";</script>after"),
            "after"
        );
    }

    #[test]
    fn capital_eszett_before_script() {
        // 'ẞ' (U+1E9E) lowercases to 'ß' (shorter); the old code panicked.
        assert_eq!(strip_html("ẞ<script>x</script>"), "ẞ");
    }

    #[test]
    fn cyrillic_before_script() {
        assert_eq!(
            strip_html("Привет мир<script>x</script>конец"),
            "Привет мирконец"
        );
    }

    #[test]
    fn script_only_unicode_body() {
        assert_eq!(strip_html("<script>İ</script>"), "");
    }

    #[test]
    fn unclosed_script_keeps_content() {
        // No closing tag: the script block is left in place and the rest of
        // the pipeline strips the tags themselves.
        assert_eq!(strip_html("a<script>alert(1)"), "aalert(1)");
    }

    #[test]
    fn multiple_script_and_style_blocks() {
        let html = "a<script>1</script>b<style>c</style>d<script>2</script>e";
        assert_eq!(strip_html(html), "abde");
    }

    #[test]
    fn script_open_before_close_removed() {
        // Opening tag before closing tag: block is removed.
        assert_eq!(strip_html("x<script>y</script>z"), "xz");
    }

    #[test]
    fn close_before_open_is_left_intact() {
        // A stray closing tag before any opening tag must not cause a panic
        // or an out-of-bounds removal.
        assert_eq!(strip_html("</script>after"), "after");
    }

    #[test]
    fn mixed_unicode_and_multiple_blocks() {
        let html = "Привет <script>var s=\"İ\";</script>мир<style>a{}</style>!";
        assert_eq!(strip_html(html), "Привет мир!");
    }
}
