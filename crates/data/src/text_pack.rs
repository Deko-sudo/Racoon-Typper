// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

//! Versioned text-pack interchange for user-authored practice content.
//!
//! A text pack bundles named custom texts under one language so practice
//! material can be shared as a file. Imports are sandboxed the same way as
//! profile transfer: parse and semantically validate everything first, preview
//! without writes, then apply inside one transaction. Besides the versioned
//! JSON schema, imports accept plain text-block, TSV, and CSV sources with
//! explicit, deterministic mapping rules (see [`TextPackSourceFormat`]).

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::error::DbError;
use crate::repository::custom_texts::{
    validate_text, CustomTextRepository, SqliteCustomTextRepository,
};
use crate::Database;

pub const TEXT_PACK_FORMAT: &str = "racoon-typper-text-pack";
pub const TEXT_PACK_SCHEMA_VERSION: u32 = 1;

/// Hard input ceiling; a pack is a hand-authored share file, not a bulk dump.
pub const MAX_TEXT_PACK_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_TEXTS_PER_PACK: usize = 500;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextPackEntryV1 {
    pub name: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextPackPayloadV1 {
    pub language: String,
    pub texts: Vec<TextPackEntryV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextPackExportV1 {
    pub format: String,
    pub schema_version: u32,
    pub application_version: String,
    pub exported_at: String,
    #[serde(flatten)]
    pub payload: TextPackPayloadV1,
}

/// Explicit source mapping rules for pack files.
///
/// - `Json`: the versioned `racoon-typper-text-pack` document.
/// - `Blocks`: blank-line separated blocks; a multi-line block uses its first
///   line as the entry name; single-line blocks get deterministic names.
/// - `Tsv`: one entry per line; column 1 is the name, remaining columns are
///   joined with spaces into the text (Anki tab-separated export shape).
///   Single-column lines become text-only entries.
/// - `Csv`: RFC4180 rows mapped exactly like `Tsv`. Selected explicitly by the
///   caller (file extension); auto-sniffing never guesses CSV because quoted
///   commas inside prose are indistinguishable from delimiters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextPackSourceFormat {
    Json,
    Blocks,
    Tsv,
    Csv,
}

impl TextPackSourceFormat {
    pub fn parse(value: &str) -> Result<Self, DbError> {
        match value {
            "json" => Ok(Self::Json),
            "blocks" => Ok(Self::Blocks),
            "tsv" => Ok(Self::Tsv),
            "csv" => Ok(Self::Csv),
            other => Err(validation(format!(
                "Unknown text pack source format: {other}"
            ))),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Blocks => "blocks",
            Self::Tsv => "tsv",
            Self::Csv => "csv",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextPackImportPolicy {
    /// Skip entries whose (language, normalized name) already exists locally.
    Merge,
    /// Delete every custom text of the pack language before inserting. This is
    /// destructive but scoped: texts in other languages are never touched.
    Replace,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextPackImportPlan {
    pub policy: TextPackImportPolicy,
    pub source_format: TextPackSourceFormat,
    pub language: String,
    pub incoming: usize,
    pub duplicates_in_pack: usize,
    pub existing_in_language: usize,
    pub to_insert: usize,
    pub to_skip: usize,
    pub removed_by_replace: usize,
}

fn validation(message: impl Into<String>) -> DbError {
    DbError::Validation(message.into())
}

fn valid_language_code(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 16
        && value
            .chars()
            .all(|character| character.is_ascii_lowercase() || character == '-')
}

fn normalize_name(name: &str) -> String {
    name.trim().to_lowercase()
}

fn deduplicated_entries(texts: &[TextPackEntryV1]) -> Vec<&TextPackEntryV1> {
    let mut seen = std::collections::HashSet::new();
    texts
        .iter()
        .filter(|entry| seen.insert(normalize_name(&entry.name)))
        .collect()
}

/// Exports local custom texts as a versioned pack. The export targets exactly
/// one language: an explicit code filters the library, while `None` is
/// accepted only when the library is single-language (or empty), so a mixed
/// library can never be exported as an ambiguously scoped pack.
pub fn export_text_pack(
    database: &Database,
    language: Option<&str>,
    application_version: &str,
    exported_at: &str,
) -> Result<TextPackExportV1, DbError> {
    database.with_connection(|conn| {
        if let Some(code) = language {
            if !valid_language_code(code) {
                return Err(validation(
                    "Text pack language is not a valid language code",
                ));
            }
        }
        let repository = SqliteCustomTextRepository::new(conn);
        let all = repository.get_all(MAX_TEXTS_PER_PACK)?;
        let scoped: Vec<_> = match language {
            Some(code) => all
                .into_iter()
                .filter(|text| text.language == code)
                .collect(),
            None => {
                let mut languages: Vec<&str> =
                    all.iter().map(|text| text.language.as_str()).collect();
                languages.sort_unstable();
                languages.dedup();
                if languages.len() > 1 {
                    return Err(validation(
                        "Library mixes several languages; export requires one explicit language",
                    ));
                }
                all
            }
        };
        let detected_language = scoped
            .first()
            .map(|text| text.language.clone())
            .or_else(|| language.map(str::to_string))
            .unwrap_or_else(|| "en".to_string());
        Ok(TextPackExportV1 {
            format: TEXT_PACK_FORMAT.to_string(),
            schema_version: TEXT_PACK_SCHEMA_VERSION,
            application_version: application_version.to_string(),
            exported_at: exported_at.to_string(),
            payload: TextPackPayloadV1 {
                language: detected_language,
                texts: scoped
                    .into_iter()
                    .map(|text| TextPackEntryV1 {
                        name: text.name,
                        text: text.text,
                    })
                    .collect(),
            },
        })
    })
}

/// Returns an import preview. It never mutates the database.
pub fn plan_text_pack_import(
    database: &Database,
    document: &[u8],
    source_format: Option<TextPackSourceFormat>,
    policy: TextPackImportPolicy,
) -> Result<TextPackImportPlan, DbError> {
    let (payload, resolved_format) = resolve_import(document, source_format)?;
    database.with_connection(|conn| build_import_plan(conn, &payload, resolved_format, policy))
}

/// Applies a prevalidated import in one transaction; any error rolls back
/// every write. Replace deletes only the custom texts of the pack language.
pub fn apply_text_pack_import(
    database: &Database,
    document: &[u8],
    source_format: Option<TextPackSourceFormat>,
    policy: TextPackImportPolicy,
) -> Result<TextPackImportPlan, DbError> {
    let (payload, resolved_format) = resolve_import(document, source_format)?;
    database.with_transaction(|conn| {
        let repository = SqliteCustomTextRepository::new(conn);
        let existing = repository.get_all(MAX_TEXTS_PER_PACK)?;
        let existing_in_language = existing
            .iter()
            .filter(|text| text.language == payload.language)
            .count();

        let known: std::collections::HashSet<String> = if policy == TextPackImportPolicy::Merge {
            existing
                .iter()
                .filter(|text| text.language == payload.language)
                .map(|text| normalize_name(&text.name))
                .collect()
        } else {
            std::collections::HashSet::new()
        };

        if policy == TextPackImportPolicy::Replace && existing_in_language > 0 {
            for text in existing
                .iter()
                .filter(|text| text.language == payload.language)
            {
                repository.delete(text.id)?;
            }
        }

        let unique = deduplicated_entries(&payload.texts);
        let mut inserted = 0_usize;
        let mut skipped = 0_usize;
        for entry in &unique {
            if known.contains(&normalize_name(&entry.name)) {
                skipped += 1;
                continue;
            }
            repository.save_with_language(&entry.name, &entry.text, &payload.language)?;
            inserted += 1;
        }

        Ok(TextPackImportPlan {
            policy,
            source_format: resolved_format,
            language: payload.language.clone(),
            incoming: payload.texts.len(),
            duplicates_in_pack: payload.texts.len() - unique.len(),
            existing_in_language,
            to_insert: inserted,
            to_skip: skipped,
            removed_by_replace: if policy == TextPackImportPolicy::Replace {
                existing_in_language
            } else {
                0
            },
        })
    })
}

fn resolve_import(
    document: &[u8],
    source_format: Option<TextPackSourceFormat>,
) -> Result<(TextPackPayloadV1, TextPackSourceFormat), DbError> {
    if document.len() > MAX_TEXT_PACK_BYTES {
        return Err(validation(format!(
            "Text pack exceeds the {MAX_TEXT_PACK_BYTES}-byte limit"
        )));
    }
    let text =
        std::str::from_utf8(document).map_err(|_| validation("Text pack is not valid UTF-8"))?;
    let resolved_format = source_format.unwrap_or_else(|| sniff_source_format(text));
    let mut payload = parse_source(text, resolved_format)?;
    finalize_payload(&mut payload, resolved_format)?;
    Ok((payload, resolved_format))
}

fn sniff_source_format(text: &str) -> TextPackSourceFormat {
    let trimmed = text.trim_start();
    if trimmed.starts_with('{') {
        return TextPackSourceFormat::Json;
    }
    if text.lines().any(|line| line.contains('\t')) {
        return TextPackSourceFormat::Tsv;
    }
    TextPackSourceFormat::Blocks
}

fn parse_source(
    text: &str,
    source_format: TextPackSourceFormat,
) -> Result<TextPackPayloadV1, DbError> {
    let entries = match source_format {
        TextPackSourceFormat::Json => return parse_json_pack(text),
        TextPackSourceFormat::Blocks => named_entries(text.split("\n\n").filter_map(block_to_pair)),
        TextPackSourceFormat::Tsv => named_entries(delimited_pairs(
            text.lines().map(|line| line.trim_end_matches('\r')),
            '\t',
        )),
        TextPackSourceFormat::Csv => named_entries(rows_to_pairs(parse_csv_rows(text)?)),
    };
    Ok(TextPackPayloadV1 {
        language: String::new(),
        texts: entries,
    })
}

/// A multi-line blank-line-separated block uses its first line as the entry
/// name; a single-line block yields no name and gets a deterministic one later.
fn block_to_pair(block: &str) -> Option<(String, String)> {
    let block = block.trim();
    if block.is_empty() {
        return None;
    }
    let mut lines = block.lines().map(str::trim).filter(|line| !line.is_empty());
    let first = lines.next()?;
    let rest: Vec<&str> = lines.collect();
    Some(if rest.is_empty() {
        (String::new(), first.to_string())
    } else {
        (first.to_string(), rest.join("\n"))
    })
}

fn delimited_pairs<'a>(
    lines: impl Iterator<Item = &'a str>,
    delimiter: char,
) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    for line in lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let columns: Vec<&str> = line.split(delimiter).map(str::trim).collect();
        if columns.len() == 1 {
            pairs.push((String::new(), columns[0].to_string()));
        } else {
            pairs.push((columns[0].to_string(), columns[1..].join(" ")));
        }
    }
    pairs
}

/// Minimal RFC4180 row splitter: comma delimiter, double-quote escaping
/// (`""` inside quoted fields), quotes meaningful only at field start.
fn parse_csv_rows(text: &str) -> Result<Vec<Vec<String>>, DbError> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut field = String::new();
    let mut row: Vec<String> = Vec::new();
    let mut in_quotes = false;
    let mut field_started = false;
    let mut chars = text.chars().peekable();
    while let Some(character) = chars.next() {
        if in_quotes {
            match character {
                '"' => {
                    if chars.peek() == Some(&'"') {
                        chars.next();
                        field.push('"');
                    } else {
                        in_quotes = false;
                    }
                }
                _ => field.push(character),
            }
        } else {
            match character {
                '"' if !field_started => in_quotes = true,
                ',' => {
                    row.push(std::mem::take(&mut field));
                    field_started = false;
                }
                '\r' => {}
                '\n' => {
                    row.push(std::mem::take(&mut field));
                    field_started = false;
                    if row.iter().any(|column| !column.trim().is_empty()) {
                        rows.push(std::mem::take(&mut row));
                    } else {
                        row.clear();
                    }
                }
                _ => {
                    field_started = true;
                    field.push(character);
                }
            }
        }
    }
    if in_quotes {
        return Err(validation("Text pack CSV has an unterminated quoted field"));
    }
    if !field.is_empty() || !row.is_empty() {
        row.push(field);
        if row.iter().any(|column| !column.trim().is_empty()) {
            rows.push(row);
        }
    }
    Ok(rows)
}

fn rows_to_pairs(rows: Vec<Vec<String>>) -> Vec<(String, String)> {
    rows.into_iter()
        .map(|columns| {
            let trimmed: Vec<String> = columns
                .into_iter()
                .map(|column| column.trim().to_string())
                .collect();
            if trimmed.len() == 1 {
                (
                    String::new(),
                    trimmed.into_iter().next().unwrap_or_default(),
                )
            } else {
                let mut iter = trimmed.into_iter();
                let name = iter.next().unwrap_or_default();
                (name, iter.collect::<Vec<_>>().join(" "))
            }
        })
        .collect()
}

fn named_entries(pairs: impl IntoIterator<Item = (String, String)>) -> Vec<TextPackEntryV1> {
    pairs
        .into_iter()
        .enumerate()
        .map(|(index, (name, text))| {
            let name = if name.trim().is_empty() {
                format!("Text {}", index + 1)
            } else {
                name
            };
            TextPackEntryV1 { name, text }
        })
        .collect()
}

fn parse_json_pack(text: &str) -> Result<TextPackPayloadV1, DbError> {
    #[derive(Deserialize)]
    struct Envelope {
        format: String,
        schema_version: u32,
        #[serde(flatten)]
        payload: TextPackPayloadV1,
    }
    let envelope: Envelope = serde_json::from_str(text)
        .map_err(|error| validation(format!("Invalid text pack document: {error}")))?;
    if envelope.format != TEXT_PACK_FORMAT {
        return Err(validation(
            "Unsupported text pack format: expected racoon-typper-text-pack",
        ));
    }
    if envelope.schema_version != TEXT_PACK_SCHEMA_VERSION {
        return Err(validation(format!(
            "Unsupported text pack schema version: {}",
            envelope.schema_version
        )));
    }
    Ok(envelope.payload)
}

/// Applies semantic bounds shared by every source format. Plain-text sources
/// carry no language metadata and default to `"en"` (the editor default);
/// JSON packs must state their language explicitly.
fn finalize_payload(
    payload: &mut TextPackPayloadV1,
    source_format: TextPackSourceFormat,
) -> Result<(), DbError> {
    if payload.texts.len() > MAX_TEXTS_PER_PACK {
        return Err(validation(format!(
            "Text pack exceeds the {MAX_TEXTS_PER_PACK}-entry limit"
        )));
    }
    if source_format == TextPackSourceFormat::Json {
        if payload.language.is_empty() {
            return Err(validation("Text pack language is empty"));
        }
    } else if payload.language.is_empty() {
        payload.language = "en".to_string();
    }
    if !valid_language_code(&payload.language) {
        return Err(validation(
            "Text pack language is not a valid language code",
        ));
    }
    for entry in &payload.texts {
        validate_text(&entry.name, &entry.text).map_err(|message| {
            validation(format!("Invalid pack entry '{}': {message}", entry.name))
        })?;
    }
    Ok(())
}

fn build_import_plan(
    conn: &Connection,
    payload: &TextPackPayloadV1,
    source_format: TextPackSourceFormat,
    policy: TextPackImportPolicy,
) -> Result<TextPackImportPlan, DbError> {
    let repository = SqliteCustomTextRepository::new(conn);
    let existing = repository.get_all(MAX_TEXTS_PER_PACK)?;
    let existing_in_language = existing
        .iter()
        .filter(|text| text.language == payload.language)
        .count();

    let known: std::collections::HashSet<String> = if policy == TextPackImportPolicy::Merge {
        existing
            .iter()
            .filter(|text| text.language == payload.language)
            .map(|text| normalize_name(&text.name))
            .collect()
    } else {
        std::collections::HashSet::new()
    };
    let unique = deduplicated_entries(&payload.texts);
    let to_skip = unique
        .iter()
        .filter(|entry| known.contains(&normalize_name(&entry.name)))
        .count();

    Ok(TextPackImportPlan {
        policy,
        source_format,
        language: payload.language.clone(),
        incoming: payload.texts.len(),
        duplicates_in_pack: payload.texts.len() - unique.len(),
        existing_in_language,
        to_insert: unique.len() - to_skip,
        to_skip,
        removed_by_replace: if policy == TextPackImportPolicy::Replace {
            existing_in_language
        } else {
            0
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::custom_texts::{CustomText, MAX_CUSTOM_TEXT_LENGTH, MAX_NAME_LENGTH};

    fn db() -> Database {
        Database::open_in_memory().expect("open in-memory db")
    }

    fn seed(database: &Database, name: &str, text: &str, language: &str) -> i64 {
        database
            .with_transaction(|conn| {
                SqliteCustomTextRepository::new(conn).save_with_language(name, text, language)
            })
            .expect("seed custom text")
    }

    fn all_texts(database: &Database) -> Vec<CustomText> {
        database
            .with_connection(|conn| SqliteCustomTextRepository::new(conn).get_all(10_000))
            .unwrap()
    }

    fn json_pack(language: &str, entries: &[(&str, &str)]) -> String {
        let texts: Vec<String> = entries
            .iter()
            .map(|(name, text)| {
                format!(
                    r#"{{"name": {}, "text": {}}}"#,
                    serde_json::json!(name),
                    serde_json::json!(text)
                )
            })
            .collect();
        format!(
            r#"{{"format": "{TEXT_PACK_FORMAT}", "schema_version": {TEXT_PACK_SCHEMA_VERSION}, "language": "{}", "texts": [{}]}}"#,
            language,
            texts.join(",")
        )
    }

    #[test]
    fn export_then_merge_import_roundtrips_texts_with_language() {
        let source = db();
        seed(&source, "A", "alpha text", "en");
        seed(&source, "B", "beta\ntext", "en");

        let exported = export_text_pack(
            &source,
            Some("en"),
            env!("CARGO_PKG_VERSION"),
            "2026-08-25T00:00:00Z",
        )
        .unwrap();
        let document = serde_json::to_string(&exported).unwrap();

        let target = db();
        let plan = plan_text_pack_import(
            &target,
            document.as_bytes(),
            None,
            TextPackImportPolicy::Merge,
        )
        .unwrap();
        assert_eq!(plan.incoming, 2);
        assert_eq!(plan.to_insert, 2);
        assert_eq!(plan.source_format, TextPackSourceFormat::Json);
        assert_eq!(plan.language, "en");

        let applied = apply_text_pack_import(
            &target,
            document.as_bytes(),
            None,
            TextPackImportPolicy::Merge,
        )
        .unwrap();
        assert_eq!(applied.to_insert, 2);
        let texts = all_texts(&target);
        assert_eq!(texts.len(), 2);
        assert!(texts.iter().all(|text| text.language == "en"));
        assert!(texts.iter().any(|text| text.text == "beta\ntext"));
    }

    #[test]
    fn merge_skips_existing_names_only_within_the_pack_language() {
        let target = db();
        seed(&target, "shared", "old body", "en");
        seed(&target, "shared", "ru body", "ru");

        let document = json_pack("en", &[("Shared", "new body"), ("fresh", "fresh body")]);
        let applied = apply_text_pack_import(
            &target,
            document.as_bytes(),
            None,
            TextPackImportPolicy::Merge,
        )
        .unwrap();
        assert_eq!(applied.to_insert, 1);
        assert_eq!(applied.to_skip, 1);

        let en_texts: Vec<_> = all_texts(&target)
            .into_iter()
            .filter(|t| t.language == "en")
            .collect();
        assert_eq!(en_texts.len(), 2);
        assert!(en_texts
            .iter()
            .any(|t| t.name == "shared" && t.text == "old body"));
        assert!(en_texts.iter().any(|t| t.name == "fresh"));
        // The Russian duplicate is untouched by the English pack.
        assert!(all_texts(&target)
            .iter()
            .any(|t| t.language == "ru" && t.text == "ru body"));
    }

    #[test]
    fn replace_wipes_only_the_pack_language_inside_one_transaction() {
        let target = db();
        seed(&target, "gone1", "body", "en");
        seed(&target, "gone2", "body", "en");
        seed(&target, "kept", "русский текст", "ru");

        let document = json_pack("en", &[("new", "brand new")]);
        let applied = apply_text_pack_import(
            &target,
            document.as_bytes(),
            None,
            TextPackImportPolicy::Replace,
        )
        .unwrap();
        assert_eq!(applied.removed_by_replace, 2);
        assert_eq!(applied.to_insert, 1);

        let texts = all_texts(&target);
        assert_eq!(texts.len(), 2);
        assert!(texts.iter().any(|t| t.language == "ru" && t.name == "kept"));
        assert!(texts.iter().any(|t| t.name == "new" && t.language == "en"));
    }

    #[test]
    fn blocks_mapping_names_multiline_blocks_and_autonames_single_lines() {
        let document = "Intro line\n\nTitle\nfirst\nsecond\n\njust one line";
        let payload = resolve_import(document.as_bytes(), Some(TextPackSourceFormat::Blocks))
            .unwrap()
            .0;
        assert_eq!(payload.texts.len(), 3);
        // Plain-text sources default to the editor language.
        assert_eq!(payload.language, "en");
        assert_eq!(payload.texts[0].name, "Text 1");
        assert_eq!(payload.texts[0].text, "Intro line");
        assert_eq!(payload.texts[1].name, "Title");
        assert_eq!(payload.texts[1].text, "first\nsecond");
        assert_eq!(payload.texts[2].name, "Text 3");
    }

    #[test]
    fn tsv_mapping_joins_extra_columns_into_the_text() {
        let document = "Deck note\tfront side\tback side\nsingle column only";
        let payload = resolve_import(document.as_bytes(), None).unwrap();
        assert_eq!(payload.1, TextPackSourceFormat::Tsv);
        let texts = payload.0.texts;
        assert_eq!(texts[0].name, "Deck note");
        assert_eq!(texts[0].text, "front side back side");
        assert_eq!(texts[1].name, "Text 2");
        assert_eq!(texts[1].text, "single column only");
    }

    #[test]
    fn csv_mapping_supports_quoted_commas_and_escaped_quotes() {
        let document = "\"Quote, comma\",\"say \"\"hi\"\" now\"\nplain,plain text";
        let payload = resolve_import(document.as_bytes(), Some(TextPackSourceFormat::Csv))
            .unwrap()
            .0;
        assert_eq!(payload.texts.len(), 2);
        assert_eq!(payload.texts[0].name, "Quote, comma");
        assert_eq!(payload.texts[0].text, "say \"hi\" now");
        assert_eq!(payload.texts[1].name, "plain");
        assert_eq!(payload.texts[1].text, "plain text");
    }

    #[test]
    fn csv_unterminated_quote_is_rejected() {
        let broken = "name,\"never closed";
        assert!(resolve_import(broken.as_bytes(), Some(TextPackSourceFormat::Csv)).is_err());
    }

    #[test]
    fn json_sniffing_rejects_foreign_formats_and_versions() {
        let wrong_format = json_pack("en", &[("a", "b")]).replace(TEXT_PACK_FORMAT, "other-pack");
        assert!(resolve_import(wrong_format.as_bytes(), None).is_err());

        let wrong_version = json_pack("en", &[("a", "b")])
            .replace("\"schema_version\": 1", "\"schema_version\": 99");
        assert!(resolve_import(wrong_version.as_bytes(), None).is_err());

        let no_language =
            r#"{"format":"racoon-typper-text-pack","schema_version":1,"language":"","texts":[]}"#;
        assert!(resolve_import(no_language.as_bytes(), None).is_err());

        let bad_language = json_pack("RU", &[("a", "b")]);
        assert!(resolve_import(bad_language.as_bytes(), None).is_err());
    }

    #[test]
    fn invalid_utf8_and_entry_bounds_are_rejected() {
        assert!(resolve_import(&[0xFF, 0xFE, 0x00], None).is_err());

        let oversized_text = "x".repeat(MAX_CUSTOM_TEXT_LENGTH + 1);
        let document = json_pack("en", &[("big", &oversized_text)]);
        assert!(resolve_import(document.as_bytes(), None).is_err());

        let oversized_name = "n".repeat(MAX_NAME_LENGTH + 1);
        let document = json_pack("en", &[(&oversized_name, "ok")]);
        assert!(resolve_import(document.as_bytes(), None).is_err());
    }

    #[test]
    fn entry_count_limit_is_enforced() {
        let entries: Vec<(String, String)> = (0..=MAX_TEXTS_PER_PACK)
            .map(|index| (format!("entry-{index}"), "body".to_string()))
            .collect();
        let mut document = json_pack("en", &[]);
        let inserted = entries
            .iter()
            .map(|(name, text)| {
                format!(
                    r#"{{"name": {}, "text": {}}}"#,
                    serde_json::json!(name),
                    serde_json::json!(text)
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        document = document.replace("\"texts\": []", &format!("\"texts\": [{inserted}]"));
        assert!(resolve_import(document.as_bytes(), None).is_err());
    }

    #[test]
    fn duplicates_inside_one_pack_count_toward_the_plan_not_the_library() {
        let document = json_pack(
            "en",
            &[("same", "one"), ("Same", "two"), ("other", "three")],
        );
        let target = db();
        let applied = apply_text_pack_import(
            &target,
            document.as_bytes(),
            None,
            TextPackImportPolicy::Merge,
        )
        .unwrap();
        assert_eq!(applied.incoming, 3);
        assert_eq!(applied.duplicates_in_pack, 1);
        assert_eq!(applied.to_insert, 2);
        assert_eq!(all_texts(&target).len(), 2);
    }
}
