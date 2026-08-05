// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

//! Versioned, strict, bounded portable profile transfer.
//!
//! The format intentionally excludes operational recovery ledgers and raw SQLite
//! backups. Imports validate the complete document before opening a write
//! transaction; merge and replace then execute as one SQLite transaction.

use std::collections::HashSet;

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::repository::custom_texts::validate_text;
use crate::{Database, DbError};

pub const PROFILE_FORMAT: &str = "racoon-typper-profile";
pub const PROFILE_SCHEMA_VERSION: u32 = 1;
pub const MAX_PROFILE_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_PROFILE_ROWS_PER_COLLECTION: usize = 100_000;
const MAX_SHORT_TEXT: usize = 256;
const MAX_TIMESTAMP: usize = 64;
const MAX_JSON_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileExportV1 {
    pub format: String,
    pub schema_version: u32,
    pub exported_at: String,
    pub application_version: String,
    pub profile: ProfilePayloadV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfilePayloadV1 {
    pub tests: Vec<TestExportV1>,
    pub personal_bests: Vec<PersonalBestExportV1>,
    pub daily_stats: Vec<DailyStatExportV1>,
    pub streaks: Vec<StreakExportV1>,
    pub custom_texts: Vec<CustomTextExportV1>,
    pub lesson_progress: Vec<LessonProgressExportV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TestExportV1 {
    pub session_id: String,
    pub created_at: String,
    pub mode_type: String,
    pub mode_config: Value,
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
    pub char_stats: Value,
    pub heatmap_data: Value,
    pub graph_data: Option<Value>,
    pub is_pb: bool,
    pub tags: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PersonalBestExportV1 {
    pub mode_type: String,
    pub mode_config_hash: String,
    pub mode_config: Value,
    pub best_wpm: f64,
    pub best_wpm_session_id: Option<String>,
    pub best_accuracy: f64,
    pub best_accuracy_session_id: Option<String>,
    pub best_consistency: Option<f64>,
    pub best_consistency_session_id: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DailyStatExportV1 {
    pub date: String,
    pub total_tests: i64,
    pub total_time_ms: i64,
    pub total_chars: i64,
    pub best_wpm: f64,
    pub avg_wpm: f64,
    pub avg_accuracy: f64,
    pub lessons_completed: i64,
    pub daily_goal_met: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StreakExportV1 {
    pub streak_type: String,
    pub current_streak: i64,
    pub longest_streak: i64,
    pub last_date: Option<String>,
    pub started_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CustomTextExportV1 {
    pub name: String,
    pub text: String,
    pub language: String,
    pub created_at: String,
    pub last_used_at: Option<String>,
    pub use_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LessonProgressExportV1 {
    pub lesson_id: String,
    pub module_id: String,
    pub language: String,
    pub difficulty: String,
    pub status: String,
    pub best_wpm: f64,
    pub best_accuracy: f64,
    pub attempts: i64,
    pub last_attempt_at: Option<String>,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileImportPolicy {
    Merge,
    Replace,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionImportPlan {
    pub incoming: usize,
    pub existing: usize,
    pub to_insert: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportPlan {
    pub policy: ProfileImportPolicy,
    pub tests: CollectionImportPlan,
    pub personal_bests: CollectionImportPlan,
    pub daily_stats: CollectionImportPlan,
    pub streaks: CollectionImportPlan,
    pub custom_texts: CollectionImportPlan,
    pub lesson_progress: CollectionImportPlan,
}

/// Parses and semantically validates a current profile document without writes.
pub fn parse_profile_export(document: &[u8]) -> Result<ProfileExportV1, DbError> {
    if document.len() > MAX_PROFILE_BYTES {
        return Err(validation(format!(
            "Profile document exceeds the {MAX_PROFILE_BYTES}-byte limit"
        )));
    }
    let profile: ProfileExportV1 = serde_json::from_slice(document)
        .map_err(|error| validation(format!("Invalid profile document: {error}")))?;
    if profile.format != PROFILE_FORMAT {
        return Err(validation("Unsupported profile format"));
    }
    if profile.schema_version != PROFILE_SCHEMA_VERSION {
        return Err(validation(format!(
            "Unsupported profile schema version: {}",
            profile.schema_version
        )));
    }
    validate_timestamp(&profile.exported_at, "exported_at")?;
    validate_string(
        &profile.application_version,
        "application_version",
        MAX_SHORT_TEXT,
        false,
    )?;
    validate_payload(&profile.profile)?;
    Ok(profile)
}

/// Exports the portable profile tables from one consistent connection snapshot.
pub fn export_profile(
    database: &Database,
    application_version: &str,
    exported_at: &str,
) -> Result<ProfileExportV1, DbError> {
    validate_string(
        application_version,
        "application_version",
        MAX_SHORT_TEXT,
        false,
    )?;
    validate_timestamp(exported_at, "exported_at")?;
    let profile = database.with_connection(export_payload)?;
    Ok(ProfileExportV1 {
        format: PROFILE_FORMAT.to_string(),
        schema_version: PROFILE_SCHEMA_VERSION,
        exported_at: exported_at.to_string(),
        application_version: application_version.to_string(),
        profile,
    })
}

/// Returns an import preview. It never mutates the database.
pub fn plan_profile_import(
    database: &Database,
    document: &[u8],
    policy: ProfileImportPolicy,
) -> Result<ImportPlan, DbError> {
    let profile = parse_profile_export(document)?;
    database.with_connection(|conn| build_import_plan(conn, &profile.profile, policy))
}

/// Applies a prevalidated import in one transaction. Any error rolls back every
/// portable-table write. Replace deletes only portable profile tables, never the
/// operational recovery ledgers.
pub fn apply_profile_import(
    database: &Database,
    document: &[u8],
    policy: ProfileImportPolicy,
) -> Result<ImportPlan, DbError> {
    let profile = parse_profile_export(document)?;
    database.with_transaction(|conn| {
        let plan = build_import_plan(conn, &profile.profile, policy)?;
        if policy == ProfileImportPolicy::Replace {
            clear_portable_profile_tables(conn)?;
        }
        import_payload(conn, &profile.profile, policy)?;
        Ok(plan)
    })
}

fn validation(message: impl Into<String>) -> DbError {
    DbError::Validation(message.into())
}

fn validate_payload(payload: &ProfilePayloadV1) -> Result<(), DbError> {
    for (name, count) in [
        ("tests", payload.tests.len()),
        ("personal_bests", payload.personal_bests.len()),
        ("daily_stats", payload.daily_stats.len()),
        ("streaks", payload.streaks.len()),
        ("custom_texts", payload.custom_texts.len()),
        ("lesson_progress", payload.lesson_progress.len()),
    ] {
        if count > MAX_PROFILE_ROWS_PER_COLLECTION {
            return Err(validation(format!(
                "Profile {name} collection exceeds the {MAX_PROFILE_ROWS_PER_COLLECTION}-row limit"
            )));
        }
    }

    let mut sessions = HashSet::new();
    for row in &payload.tests {
        validate_string(&row.session_id, "test.session_id", 128, false)?;
        if !sessions.insert(row.session_id.as_str()) {
            return Err(validation("Profile contains duplicate test session IDs"));
        }
        validate_timestamp(&row.created_at, "test.created_at")?;
        validate_string(&row.mode_type, "test.mode_type", MAX_SHORT_TEXT, false)?;
        validate_string(&row.language, "test.language", 64, false)?;
        validate_json(&row.mode_config, "test.mode_config")?;
        validate_json(&row.char_stats, "test.char_stats")?;
        validate_json(&row.heatmap_data, "test.heatmap_data")?;
        if let Some(graph_data) = &row.graph_data {
            validate_json(graph_data, "test.graph_data")?;
        }
        validate_string(&row.tags, "test.tags", 4_096, true)?;
        validate_non_negative([
            ("test.text_length", row.text_length),
            ("test.duration_ms", row.duration_ms),
            ("test.correct_chars", row.correct_chars),
            ("test.incorrect_chars", row.incorrect_chars),
            ("test.backspaces", row.backspaces),
        ])?;
        validate_finite([
            ("test.wpm", row.wpm),
            ("test.raw_wpm", row.raw_wpm),
            ("test.accuracy", row.accuracy),
            ("test.raw_accuracy", row.raw_accuracy),
        ])?;
        if let Some(value) = row.consistency {
            validate_finite([("test.consistency", value)])?;
        }
    }

    let mut bests = HashSet::new();
    for row in &payload.personal_bests {
        validate_string(
            &row.mode_type,
            "personal_best.mode_type",
            MAX_SHORT_TEXT,
            false,
        )?;
        validate_string(
            &row.mode_config_hash,
            "personal_best.mode_config_hash",
            MAX_SHORT_TEXT,
            false,
        )?;
        if !bests.insert((row.mode_type.as_str(), row.mode_config_hash.as_str())) {
            return Err(validation(
                "Profile contains duplicate personal best identities",
            ));
        }
        validate_json(&row.mode_config, "personal_best.mode_config")?;
        validate_timestamp(&row.updated_at, "personal_best.updated_at")?;
        validate_finite([
            ("personal_best.best_wpm", row.best_wpm),
            ("personal_best.best_accuracy", row.best_accuracy),
        ])?;
        if let Some(value) = row.best_consistency {
            validate_finite([("personal_best.best_consistency", value)])?;
        }
        for reference in [
            row.best_wpm_session_id.as_deref(),
            row.best_accuracy_session_id.as_deref(),
            row.best_consistency_session_id.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            if !sessions.contains(reference) {
                return Err(validation(
                    "Personal-best test reference is not present in the profile",
                ));
            }
        }
    }

    let mut dates = HashSet::new();
    for row in &payload.daily_stats {
        validate_date(&row.date, "daily_stat.date")?;
        if !dates.insert(row.date.as_str()) {
            return Err(validation("Profile contains duplicate daily-stat dates"));
        }
        validate_non_negative([
            ("daily_stat.total_tests", row.total_tests),
            ("daily_stat.total_time_ms", row.total_time_ms),
            ("daily_stat.total_chars", row.total_chars),
            ("daily_stat.lessons_completed", row.lessons_completed),
        ])?;
        validate_finite([
            ("daily_stat.best_wpm", row.best_wpm),
            ("daily_stat.avg_wpm", row.avg_wpm),
            ("daily_stat.avg_accuracy", row.avg_accuracy),
        ])?;
    }

    let mut streak_types = HashSet::new();
    for row in &payload.streaks {
        validate_string(&row.streak_type, "streak.type", MAX_SHORT_TEXT, false)?;
        if !streak_types.insert(row.streak_type.as_str()) {
            return Err(validation("Profile contains duplicate streak types"));
        }
        validate_non_negative([
            ("streak.current_streak", row.current_streak),
            ("streak.longest_streak", row.longest_streak),
        ])?;
        validate_optional_date(row.last_date.as_deref(), "streak.last_date")?;
        validate_optional_date(row.started_date.as_deref(), "streak.started_date")?;
    }

    let mut texts = HashSet::new();
    for row in &payload.custom_texts {
        validate_string(&row.language, "custom_text.language", 32, false)?;
        validate_text(&row.name, &row.text).map_err(DbError::Validation)?;
        validate_timestamp(&row.created_at, "custom_text.created_at")?;
        validate_optional_timestamp(row.last_used_at.as_deref(), "custom_text.last_used_at")?;
        validate_non_negative([("custom_text.use_count", row.use_count)])?;
        if !texts.insert((row.name.as_str(), row.text.as_str(), row.language.as_str())) {
            return Err(validation(
                "Profile contains duplicate custom-text identities",
            ));
        }
    }

    let mut lessons = HashSet::new();
    for row in &payload.lesson_progress {
        validate_string(&row.lesson_id, "lesson.lesson_id", MAX_SHORT_TEXT, false)?;
        if !lessons.insert(row.lesson_id.as_str()) {
            return Err(validation("Profile contains duplicate lesson IDs"));
        }
        validate_string(&row.module_id, "lesson.module_id", MAX_SHORT_TEXT, false)?;
        validate_string(&row.language, "lesson.language", 64, false)?;
        validate_string(&row.difficulty, "lesson.difficulty", MAX_SHORT_TEXT, false)?;
        if !matches!(
            row.status.as_str(),
            "not_started" | "in_progress" | "completed"
        ) {
            return Err(validation("Unsupported lesson status"));
        }
        validate_non_negative([("lesson.attempts", row.attempts)])?;
        validate_finite([
            ("lesson.best_wpm", row.best_wpm),
            ("lesson.best_accuracy", row.best_accuracy),
        ])?;
        validate_optional_timestamp(row.last_attempt_at.as_deref(), "lesson.last_attempt_at")?;
        validate_optional_timestamp(row.completed_at.as_deref(), "lesson.completed_at")?;
    }
    Ok(())
}

fn validate_string(value: &str, name: &str, max: usize, allow_empty: bool) -> Result<(), DbError> {
    if (!allow_empty && value.trim().is_empty()) || value.chars().count() > max {
        return Err(validation(format!(
            "{name} must be non-empty and at most {max} characters"
        )));
    }
    Ok(())
}

fn validate_timestamp(value: &str, name: &str) -> Result<(), DbError> {
    validate_string(value, name, MAX_TIMESTAMP, false)?;
    chrono::DateTime::parse_from_rfc3339(value)
        .map_err(|_| validation(format!("{name} must be an RFC 3339 timestamp")))?;
    Ok(())
}

fn validate_optional_timestamp(value: Option<&str>, name: &str) -> Result<(), DbError> {
    if let Some(value) = value {
        validate_timestamp(value, name)?;
    }
    Ok(())
}

fn validate_date(value: &str, name: &str) -> Result<(), DbError> {
    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| validation(format!("{name} must be an ISO calendar date")))?;
    Ok(())
}

fn validate_optional_date(value: Option<&str>, name: &str) -> Result<(), DbError> {
    if let Some(value) = value {
        validate_date(value, name)?;
    }
    Ok(())
}

fn validate_non_negative<const N: usize>(values: [(&str, i64); N]) -> Result<(), DbError> {
    if values.iter().any(|(_, value)| *value < 0) {
        return Err(validation("Profile contains a negative counter"));
    }
    Ok(())
}

fn validate_finite<const N: usize>(values: [(&str, f64); N]) -> Result<(), DbError> {
    if let Some((name, _)) = values.iter().find(|(_, value)| !value.is_finite()) {
        return Err(validation(format!("{name} must be finite")));
    }
    Ok(())
}

fn validate_json(value: &Value, name: &str) -> Result<(), DbError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| validation(format!("{name} cannot be encoded: {error}")))?;
    if bytes.len() > MAX_JSON_BYTES {
        return Err(validation(format!(
            "{name} exceeds the {MAX_JSON_BYTES}-byte limit"
        )));
    }
    Ok(())
}

fn export_payload(conn: &Connection) -> Result<ProfilePayloadV1, DbError> {
    Ok(ProfilePayloadV1 {
        tests: query_all(conn, "SELECT session_id, created_at, mode_type, mode_config, language, text_length, duration_ms, wpm, raw_wpm, accuracy, raw_accuracy, consistency, correct_chars, incorrect_chars, backspaces, char_stats, heatmap_data, graph_data, is_pb, tags FROM tests ORDER BY created_at, session_id", |row| Ok(TestExportV1 { session_id: row.get(0)?, created_at: row.get(1)?, mode_type: row.get(2)?, mode_config: json_column(row, 3)?, language: row.get(4)?, text_length: row.get(5)?, duration_ms: row.get(6)?, wpm: row.get(7)?, raw_wpm: row.get(8)?, accuracy: row.get(9)?, raw_accuracy: row.get(10)?, consistency: row.get(11)?, correct_chars: row.get(12)?, incorrect_chars: row.get(13)?, backspaces: row.get(14)?, char_stats: json_column(row, 15)?, heatmap_data: json_column(row, 16)?, graph_data: optional_json_column(row, 17)?, is_pb: row.get(18)?, tags: row.get(19)? }))?,
        personal_bests: query_all(conn, "SELECT pb.mode_type, pb.mode_config_hash, pb.mode_config, pb.best_wpm, w.session_id, pb.best_accuracy, a.session_id, pb.best_consistency, c.session_id, pb.updated_at FROM personal_bests pb LEFT JOIN tests w ON w.id = pb.best_wpm_test_id LEFT JOIN tests a ON a.id = pb.best_accuracy_test_id LEFT JOIN tests c ON c.id = pb.best_consistency_test_id ORDER BY pb.mode_type, pb.mode_config_hash", |row| Ok(PersonalBestExportV1 { mode_type: row.get(0)?, mode_config_hash: row.get(1)?, mode_config: json_column(row, 2)?, best_wpm: row.get(3)?, best_wpm_session_id: row.get(4)?, best_accuracy: row.get(5)?, best_accuracy_session_id: row.get(6)?, best_consistency: row.get(7)?, best_consistency_session_id: row.get(8)?, updated_at: row.get(9)? }))?,
        daily_stats: query_all(conn, "SELECT date, total_tests, total_time_ms, total_chars, best_wpm, avg_wpm, avg_accuracy, lessons_completed, daily_goal_met FROM daily_stats ORDER BY date", |row| Ok(DailyStatExportV1 { date: row.get(0)?, total_tests: row.get(1)?, total_time_ms: row.get(2)?, total_chars: row.get(3)?, best_wpm: row.get(4)?, avg_wpm: row.get(5)?, avg_accuracy: row.get(6)?, lessons_completed: row.get(7)?, daily_goal_met: row.get(8)? }))?,
        streaks: query_all(conn, "SELECT type, current_streak, longest_streak, last_date, started_date FROM streaks ORDER BY type", |row| Ok(StreakExportV1 { streak_type: row.get(0)?, current_streak: row.get(1)?, longest_streak: row.get(2)?, last_date: row.get(3)?, started_date: row.get(4)? }))?,
        custom_texts: query_all(conn, "SELECT name, text, language, created_at, last_used_at, use_count FROM custom_texts ORDER BY id", |row| Ok(CustomTextExportV1 { name: row.get(0)?, text: row.get(1)?, language: row.get(2)?, created_at: row.get(3)?, last_used_at: row.get(4)?, use_count: row.get(5)? }))?,
        lesson_progress: query_all(conn, "SELECT lesson_id, module_id, language, difficulty, status, best_wpm, best_accuracy, attempts, last_attempt_at, completed_at FROM lesson_progress ORDER BY lesson_id", |row| Ok(LessonProgressExportV1 { lesson_id: row.get(0)?, module_id: row.get(1)?, language: row.get(2)?, difficulty: row.get(3)?, status: row.get(4)?, best_wpm: row.get(5)?, best_accuracy: row.get(6)?, attempts: row.get(7)?, last_attempt_at: row.get(8)?, completed_at: row.get(9)? }))?,
    })
}

fn query_all<T>(
    conn: &Connection,
    sql: &str,
    map: impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
) -> Result<Vec<T>, DbError> {
    let mut statement = conn
        .prepare(sql)
        .map_err(|error| DbError::Query(error.to_string()))?;
    let rows = statement
        .query_map([], map)
        .map_err(|error| DbError::Query(error.to_string()))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| DbError::Query(error.to_string()))
}

fn json_column(row: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<Value> {
    let json: String = row.get(index)?;
    serde_json::from_str(&json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            index,
            rusqlite::types::Type::Text,
            Box::new(error),
        )
    })
}

fn optional_json_column(row: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<Option<Value>> {
    row.get::<_, Option<String>>(index)?
        .map(|json| {
            serde_json::from_str(&json).map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    index,
                    rusqlite::types::Type::Text,
                    Box::new(error),
                )
            })
        })
        .transpose()
}

fn build_import_plan(
    conn: &Connection,
    payload: &ProfilePayloadV1,
    policy: ProfileImportPolicy,
) -> Result<ImportPlan, DbError> {
    let mut plan = ImportPlan {
        policy,
        tests: plan_collection(payload.tests.len(), count_existing(conn, "SELECT COUNT(*) FROM tests WHERE session_id = ?1", payload.tests.iter().map(|row| row.session_id.as_str()))?),
        personal_bests: plan_collection(payload.personal_bests.len(), count_existing_pairs(conn, "SELECT COUNT(*) FROM personal_bests WHERE mode_type = ?1 AND mode_config_hash = ?2", payload.personal_bests.iter().map(|row| (row.mode_type.as_str(), row.mode_config_hash.as_str())))?),
        daily_stats: plan_collection(payload.daily_stats.len(), count_existing(conn, "SELECT COUNT(*) FROM daily_stats WHERE date = ?1", payload.daily_stats.iter().map(|row| row.date.as_str()))?),
        streaks: plan_collection(payload.streaks.len(), count_existing(conn, "SELECT COUNT(*) FROM streaks WHERE type = ?1", payload.streaks.iter().map(|row| row.streak_type.as_str()))?),
        custom_texts: plan_collection(payload.custom_texts.len(), count_existing_triples(conn, payload.custom_texts.iter().map(|row| (row.name.as_str(), row.text.as_str(), row.language.as_str())))?),
        lesson_progress: plan_collection(payload.lesson_progress.len(), count_existing(conn, "SELECT COUNT(*) FROM lesson_progress WHERE lesson_id = ?1", payload.lesson_progress.iter().map(|row| row.lesson_id.as_str()))?),
    };
    if policy == ProfileImportPolicy::Replace {
        plan.tests.to_insert = plan.tests.incoming;
        plan.personal_bests.to_insert = plan.personal_bests.incoming;
        plan.daily_stats.to_insert = plan.daily_stats.incoming;
        plan.streaks.to_insert = plan.streaks.incoming;
        plan.custom_texts.to_insert = plan.custom_texts.incoming;
        plan.lesson_progress.to_insert = plan.lesson_progress.incoming;
    }
    Ok(plan)
}

fn plan_collection(incoming: usize, existing: usize) -> CollectionImportPlan {
    CollectionImportPlan {
        incoming,
        existing,
        to_insert: incoming.saturating_sub(existing),
    }
}
fn count_existing<'a>(
    conn: &Connection,
    sql: &str,
    mut values: impl Iterator<Item = &'a str>,
) -> Result<usize, DbError> {
    values.try_fold(0usize, |count, value| {
        conn.query_row(sql, params![value], |row| row.get::<_, i64>(0))
            .map(|exists| count + exists as usize)
            .map_err(|error| DbError::Query(error.to_string()))
    })
}
fn count_existing_pairs<'a>(
    conn: &Connection,
    sql: &str,
    mut values: impl Iterator<Item = (&'a str, &'a str)>,
) -> Result<usize, DbError> {
    values.try_fold(0usize, |count, (first, second)| {
        conn.query_row(sql, params![first, second], |row| row.get::<_, i64>(0))
            .map(|exists| count + exists as usize)
            .map_err(|error| DbError::Query(error.to_string()))
    })
}
fn count_existing_triples<'a>(
    conn: &Connection,
    mut values: impl Iterator<Item = (&'a str, &'a str, &'a str)>,
) -> Result<usize, DbError> {
    values.try_fold(0usize, |count, (name, text, language)| {
        conn.query_row(
            "SELECT COUNT(*) FROM custom_texts WHERE name = ?1 AND text = ?2 AND language = ?3",
            params![name, text, language],
            |row| row.get::<_, i64>(0),
        )
        .map(|exists| count + exists as usize)
        .map_err(|error| DbError::Query(error.to_string()))
    })
}

fn clear_portable_profile_tables(conn: &Connection) -> Result<(), DbError> {
    for table in [
        "personal_bests",
        "tests",
        "daily_stats",
        "streaks",
        "custom_texts",
        "lesson_progress",
    ] {
        conn.execute(&format!("DELETE FROM {table}"), [])
            .map_err(|error| DbError::Write(error.to_string()))?;
    }
    Ok(())
}

fn import_payload(
    conn: &Connection,
    payload: &ProfilePayloadV1,
    policy: ProfileImportPolicy,
) -> Result<(), DbError> {
    for row in &payload.tests {
        if policy == ProfileImportPolicy::Merge
            && exists(
                conn,
                "SELECT 1 FROM tests WHERE session_id = ?1",
                &[&row.session_id],
            )?
        {
            continue;
        }
        conn.execute("INSERT INTO tests (session_id, created_at, mode_type, mode_config, language, text_length, duration_ms, wpm, raw_wpm, accuracy, raw_accuracy, consistency, correct_chars, incorrect_chars, backspaces, char_stats, heatmap_data, graph_data, is_pb, tags) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)", params![row.session_id, row.created_at, row.mode_type, json_text(&row.mode_config)?, row.language, row.text_length, row.duration_ms, row.wpm, row.raw_wpm, row.accuracy, row.raw_accuracy, row.consistency, row.correct_chars, row.incorrect_chars, row.backspaces, json_text(&row.char_stats)?, json_text(&row.heatmap_data)?, row.graph_data.as_ref().map(json_text).transpose()?, row.is_pb, row.tags]).map_err(|error| DbError::Write(error.to_string()))?;
    }
    for row in &payload.personal_bests {
        let references = (
            session_id_to_id(conn, row.best_wpm_session_id.as_deref())?,
            session_id_to_id(conn, row.best_accuracy_session_id.as_deref())?,
            session_id_to_id(conn, row.best_consistency_session_id.as_deref())?,
        );
        conn.execute("INSERT INTO personal_bests (mode_type, mode_config_hash, mode_config, best_wpm, best_wpm_test_id, best_accuracy, best_accuracy_test_id, best_consistency, best_consistency_test_id, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10) ON CONFLICT(mode_type, mode_config_hash) DO UPDATE SET mode_config = excluded.mode_config, best_wpm = excluded.best_wpm, best_wpm_test_id = excluded.best_wpm_test_id, best_accuracy = excluded.best_accuracy, best_accuracy_test_id = excluded.best_accuracy_test_id, best_consistency = excluded.best_consistency, best_consistency_test_id = excluded.best_consistency_test_id, updated_at = excluded.updated_at", params![row.mode_type, row.mode_config_hash, json_text(&row.mode_config)?, row.best_wpm, references.0, row.best_accuracy, references.1, row.best_consistency, references.2, row.updated_at]).map_err(|error| DbError::Write(error.to_string()))?;
    }
    for row in &payload.daily_stats {
        conn.execute("INSERT INTO daily_stats (date, total_tests, total_time_ms, total_chars, best_wpm, avg_wpm, avg_accuracy, lessons_completed, daily_goal_met) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9) ON CONFLICT(date) DO UPDATE SET total_tests = excluded.total_tests, total_time_ms = excluded.total_time_ms, total_chars = excluded.total_chars, best_wpm = excluded.best_wpm, avg_wpm = excluded.avg_wpm, avg_accuracy = excluded.avg_accuracy, lessons_completed = excluded.lessons_completed, daily_goal_met = excluded.daily_goal_met", params![row.date, row.total_tests, row.total_time_ms, row.total_chars, row.best_wpm, row.avg_wpm, row.avg_accuracy, row.lessons_completed, row.daily_goal_met]).map_err(|error| DbError::Write(error.to_string()))?;
    }
    for row in &payload.streaks {
        conn.execute("INSERT INTO streaks (type, current_streak, longest_streak, last_date, started_date) VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(type) DO UPDATE SET current_streak = excluded.current_streak, longest_streak = excluded.longest_streak, last_date = excluded.last_date, started_date = excluded.started_date", params![row.streak_type, row.current_streak, row.longest_streak, row.last_date, row.started_date]).map_err(|error| DbError::Write(error.to_string()))?;
    }
    for row in &payload.custom_texts {
        if policy == ProfileImportPolicy::Merge
            && exists(
                conn,
                "SELECT 1 FROM custom_texts WHERE name = ?1 AND text = ?2 AND language = ?3",
                &[&row.name, &row.text, &row.language],
            )?
        {
            continue;
        }
        conn.execute("INSERT INTO custom_texts (name, text, language, created_at, last_used_at, use_count) VALUES (?1, ?2, ?3, ?4, ?5, ?6)", params![row.name, row.text, row.language, row.created_at, row.last_used_at, row.use_count]).map_err(|error| DbError::Write(error.to_string()))?;
    }
    for row in &payload.lesson_progress {
        conn.execute("INSERT INTO lesson_progress (lesson_id, module_id, language, difficulty, status, best_wpm, best_accuracy, attempts, last_attempt_at, completed_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10) ON CONFLICT(lesson_id) DO UPDATE SET module_id = excluded.module_id, language = excluded.language, difficulty = excluded.difficulty, status = excluded.status, best_wpm = excluded.best_wpm, best_accuracy = excluded.best_accuracy, attempts = excluded.attempts, last_attempt_at = excluded.last_attempt_at, completed_at = excluded.completed_at", params![row.lesson_id, row.module_id, row.language, row.difficulty, row.status, row.best_wpm, row.best_accuracy, row.attempts, row.last_attempt_at, row.completed_at]).map_err(|error| DbError::Write(error.to_string()))?;
    }
    Ok(())
}

fn exists(conn: &Connection, sql: &str, values: &[&str]) -> Result<bool, DbError> {
    conn.query_row(sql, rusqlite::params_from_iter(values.iter()), |_| Ok(()))
        .map(|_| true)
        .or_else(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => Ok(false),
            error => Err(DbError::Query(error.to_string())),
        })
}
fn session_id_to_id(conn: &Connection, session_id: Option<&str>) -> Result<Option<i64>, DbError> {
    session_id
        .map(|session_id| {
            conn.query_row(
                "SELECT id FROM tests WHERE session_id = ?1",
                params![session_id],
                |row| row.get(0),
            )
            .map(Some)
            .map_err(|error| {
                DbError::Write(format!(
                    "resolve imported personal-best test reference: {error}"
                ))
            })
        })
        .transpose()
        .map(|id| id.flatten())
}
fn json_text(value: &Value) -> Result<String, DbError> {
    serde_json::to_string(value)
        .map_err(|error| DbError::Validation(format!("encode profile JSON: {error}")))
}
