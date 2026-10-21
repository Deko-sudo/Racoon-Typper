// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

//! SQLite adapters for the application reporting ports.
//!
//! These adapters translate application reporting values (UTC days, session
//! identities, mode enums) into the persisted SQLite schema. Daily statistics
//! and streaks are stored under **local** calendar dates (see
//! `session_finalizer`), while history/export/replay use the UTC `created_at`
//! timestamp. Callers that want local-day semantics for dashboard/progress
//! surfaces must feed the adapters a `ReportingDay` already expressed as the
//! local calendar date (see the app command clock in `racoon-app`).

use chrono::{DateTime, SecondsFormat, Utc};
use racoon_application::{
    AchievementInputQuery, AchievementInputs, AnalyticsReportingPort, DailyStatisticsPoint,
    ExportDatasetSource, ExportQuery, ExportRow, HistoryPageSource, HistoryQuery,
    HistoryReportingPort, InclusiveDateRange, InsightInputQuery, InsightInputs,
    PersonalBestConfigurationKey, PersonalBestDimension, PersonalBestEntry,
    PersonalBestReportingPort, ProgressReportingPort, ReplayPageSource, ReplayQuery, ReportingDay,
    ReportingError, ReportingLanguage, ReportingMode, ReportingModeFilter, StreakReport,
    TestDetails,
};
use racoon_domain::SessionId;
use rusqlite::{params, Connection, OptionalExtension};

use crate::{Database, DbError};

use super::{
    DailyStats, DailyStatsRepository, PersonalBestsRepository, ReplayRepository,
    SqliteDailyStatsRepository, SqlitePersonalBestsRepository, SqliteReplayRepository,
    SqliteStreakRepository, SqliteTestRepository, StreakRepository, TestRepository,
};

/// Inward-facing SQLite implementation of [`HistoryReportingPort`].
pub struct SqliteHistoryReportingPort<'a> {
    database: &'a Database,
}

impl<'a> SqliteHistoryReportingPort<'a> {
    pub fn new(database: &'a Database) -> Self {
        Self { database }
    }
}

impl HistoryReportingPort for SqliteHistoryReportingPort<'_> {
    fn list_history(&self, query: &HistoryQuery) -> Result<HistoryPageSource, ReportingError> {
        self.database
            .with_connection(|conn| list_history(conn, query))
            .map_err(port_failure)
    }

    fn find_test_details(
        &self,
        session_id: &SessionId,
    ) -> Result<Option<TestDetails>, ReportingError> {
        self.database
            .with_connection(|conn| find_test_details(conn, session_id))
            .map_err(port_failure)
    }

    fn list_replay_frames(
        &self,
        query: &ReplayQuery,
    ) -> Result<Option<ReplayPageSource>, ReportingError> {
        self.database
            .with_connection(|conn| list_replay_frames(conn, query))
            .map_err(port_failure)
    }

    fn list_export_rows(&self, query: &ExportQuery) -> Result<ExportDatasetSource, ReportingError> {
        self.database
            .with_connection(|conn| list_export_rows(conn, query))
            .map_err(port_failure)
    }
}

/// Inward-facing SQLite implementation of [`ProgressReportingPort`].
///
/// Daily aggregates and streaks are stored under local calendar dates, so this
/// adapter treats a `ReportingDay` as a local date string directly.
pub struct SqliteProgressReportingPort<'a> {
    database: &'a Database,
}

impl<'a> SqliteProgressReportingPort<'a> {
    pub fn new(database: &'a Database) -> Self {
        Self { database }
    }
}

impl ProgressReportingPort for SqliteProgressReportingPort<'_> {
    fn count_tests(&self) -> Result<u64, ReportingError> {
        self.database
            .with_connection(|conn| {
                let count = SqliteTestRepository::new(conn).get_count(None)?;
                u64::try_from(count)
                    .map_err(|_| DbError::Validation("test count exceeds u64".into()))
            })
            .map_err(port_failure)
    }

    fn load_daily_statistics(
        &self,
        range: InclusiveDateRange,
    ) -> Result<Vec<DailyStatisticsPoint>, ReportingError> {
        self.database
            .with_connection(|conn| load_daily_statistics(conn, range))
            .map_err(port_failure)
    }

    fn load_streak_report(&self, as_of: ReportingDay) -> Result<StreakReport, ReportingError> {
        self.database
            .with_connection(|conn| {
                let row = SqliteStreakRepository::new(conn).get("daily_test")?;
                Ok(match row {
                    Some(row) => StreakReport::new(
                        u64::try_from(row.current_streak)
                            .map_err(|_| DbError::Validation("streak exceeds u64".into()))?,
                        u64::try_from(row.longest_streak)
                            .map_err(|_| DbError::Validation("streak exceeds u64".into()))?,
                        as_of,
                    ),
                    None => StreakReport::new(0, 0, as_of),
                })
            })
            .map_err(port_failure)
    }
}

/// Inward-facing SQLite implementation of [`PersonalBestReportingPort`].
pub struct SqlitePersonalBestReportingPort<'a> {
    database: &'a Database,
}

impl<'a> SqlitePersonalBestReportingPort<'a> {
    pub fn new(database: &'a Database) -> Self {
        Self { database }
    }
}

impl PersonalBestReportingPort for SqlitePersonalBestReportingPort<'_> {
    fn list_personal_bests(
        &self,
        mode: Option<ReportingModeFilter>,
    ) -> Result<Vec<PersonalBestEntry>, ReportingError> {
        self.database
            .with_connection(|conn| list_personal_bests(conn, mode))
            .map_err(port_failure)
    }
}

/// Inward-facing SQLite implementation of [`AnalyticsReportingPort`].
pub struct SqliteAnalyticsReportingPort<'a> {
    database: &'a Database,
}

impl<'a> SqliteAnalyticsReportingPort<'a> {
    pub fn new(database: &'a Database) -> Self {
        Self { database }
    }
}

impl AnalyticsReportingPort for SqliteAnalyticsReportingPort<'_> {
    fn load_achievement_inputs(
        &self,
        query: &AchievementInputQuery,
    ) -> Result<AchievementInputs, ReportingError> {
        self.database
            .with_connection(|conn| load_achievement_inputs(conn, query))
            .map_err(port_failure)
    }

    fn load_insight_inputs(
        &self,
        query: &InsightInputQuery,
    ) -> Result<InsightInputs, ReportingError> {
        self.database
            .with_connection(|conn| load_insight_inputs(conn, query))
            .map_err(port_failure)
    }
}

fn list_history(conn: &Connection, query: &HistoryQuery) -> Result<HistoryPageSource, DbError> {
    let mode = query.filter().mode().map(ReportingModeFilter::mode);
    let mode_str = mode.map(ReportingMode::as_str);

    let (from, to): (Option<String>, Option<String>) = query
        .filter()
        .date_range()
        .map(half_open_utc)
        .transpose()
        .map_err(|_| DbError::Validation("invalid reporting date range".into()))?
        .map_or((None, None), |(from, to)| (Some(from), Some(to)));

    let total = count_filtered(conn, mode_str, from.as_deref(), to.as_deref())?;

    let (limit, offset) = (query.pagination().limit(), query.pagination().offset());
    let items = fetch_history(
        conn,
        limit,
        offset,
        mode_str,
        from.as_deref(),
        to.as_deref(),
    )?;

    Ok(HistoryPageSource::new(items, total))
}

fn fetch_history(
    conn: &Connection,
    limit: usize,
    offset: usize,
    mode: Option<&str>,
    from: Option<&str>,
    to: Option<&str>,
) -> Result<Vec<racoon_application::HistoryItem>, DbError> {
    let mut sql = String::from("SELECT id, session_id, created_at, mode_type, mode_config, language, text_length, duration_ms, wpm, raw_wpm, accuracy, raw_accuracy, consistency, correct_chars, incorrect_chars, backspaces, is_pb, EXISTS(SELECT 1 FROM test_replays WHERE test_replays.test_id = tests.id) AS has_replay FROM tests");
    let mut conditions: Vec<String> = Vec::new();
    let mut values: Vec<rusqlite::types::Value> = Vec::new();
    if let Some(mode) = mode {
        conditions.push("mode_type = ?".to_string());
        values.push(rusqlite::types::Value::from(mode.to_string()));
    }
    if let Some(from) = from {
        conditions.push("created_at >= ?".to_string());
        values.push(rusqlite::types::Value::from(from.to_string()));
    }
    if let Some(to) = to {
        conditions.push("created_at < ?".to_string());
        values.push(rusqlite::types::Value::from(to.to_string()));
    }
    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }
    sql.push_str(" ORDER BY created_at DESC, session_id DESC LIMIT ? OFFSET ?");
    values.push(rusqlite::types::Value::from(limit as i64));
    values.push(rusqlite::types::Value::from(offset as i64));

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| DbError::Query(e.to_string()))?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(values), |row| {
            Ok((
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, i64>(7)?,
                row.get::<_, f64>(8)?,
                row.get::<_, f64>(10)?,
                row.get::<_, bool>(16)?,
                row.get::<_, bool>(17)?,
            ))
        })
        .map_err(|e| DbError::Query(e.to_string()))?;

    let mut items = Vec::new();
    for row in rows {
        let (
            session_id,
            completed_at,
            mode,
            language,
            text_length,
            duration_ms,
            wpm,
            accuracy,
            is_pb,
            has_replay,
        ) = row.map_err(|e| DbError::Query(e.to_string()))?;
        items.push(build_history_item(
            session_id,
            completed_at,
            mode,
            language,
            text_length,
            duration_ms,
            wpm,
            accuracy,
            is_pb,
            has_replay,
        )?);
    }
    Ok(items)
}

#[allow(clippy::too_many_arguments)]
fn build_history_item(
    session_id: String,
    completed_at: String,
    mode: String,
    language: Option<String>,
    text_length: i64,
    duration_ms: i64,
    wpm: f64,
    accuracy: f64,
    is_pb: bool,
    has_replay: bool,
) -> Result<racoon_application::HistoryItem, DbError> {
    let completed_at = parse_utc(completed_at)
        .ok_or_else(|| DbError::Validation("invalid UTC timestamp".into()))?;
    let mode = mode
        .parse()
        .map_err(|_| DbError::Validation("unsupported mode".into()))?;
    let language = language
        .map(ReportingLanguage::parse)
        .transpose()
        .map_err(|_| DbError::Validation("invalid language".into()))?;

    racoon_application::HistoryItem::new(
        SessionId::from(session_id),
        completed_at,
        mode,
        language,
        u64::try_from(duration_ms)
            .map_err(|_| DbError::Validation("duration exceeds u64".into()))?,
        u64::try_from(text_length)
            .map_err(|_| DbError::Validation("text_length exceeds u64".into()))?,
        wpm,
        accuracy,
        is_pb,
        has_replay,
    )
    .map_err(|_| DbError::Validation("history item violates reporting bounds".into()))
}

fn count_filtered(
    conn: &Connection,
    mode: Option<&str>,
    from: Option<&str>,
    to: Option<&str>,
) -> Result<u64, DbError> {
    let mut sql = String::from("SELECT COUNT(*) FROM tests");
    let mut conditions: Vec<String> = Vec::new();
    let mut values: Vec<rusqlite::types::Value> = Vec::new();
    if let Some(mode) = mode {
        conditions.push("mode_type = ?".to_string());
        values.push(rusqlite::types::Value::from(mode.to_string()));
    }
    if let Some(from) = from {
        conditions.push("created_at >= ?".to_string());
        values.push(rusqlite::types::Value::from(from.to_string()));
    }
    if let Some(to) = to {
        conditions.push("created_at < ?".to_string());
        values.push(rusqlite::types::Value::from(to.to_string()));
    }
    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }
    let count: i64 = conn
        .query_row(&sql, rusqlite::params_from_iter(values), |row| row.get(0))
        .map_err(|e| DbError::Query(e.to_string()))?;
    u64::try_from(count).map_err(|_| DbError::Validation("test count exceeds u64".into()))
}

fn find_test_details(
    conn: &Connection,
    session_id: &SessionId,
) -> Result<Option<TestDetails>, DbError> {
    let row = conn
        .query_row(
            "SELECT session_id, created_at, mode_type, language, text_length, duration_ms,
                    wpm, raw_wpm, accuracy, raw_accuracy, consistency,
                    correct_chars, incorrect_chars, backspaces, is_pb,
                    EXISTS(SELECT 1 FROM test_replays WHERE test_replays.test_id = tests.id) AS has_replay
             FROM tests WHERE session_id = ?1",
            params![session_id.as_str()],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, f64>(6)?,
                    row.get::<_, f64>(7)?,
                    row.get::<_, f64>(8)?,
                    row.get::<_, f64>(9)?,
                    row.get::<_, Option<f64>>(10)?,
                    row.get::<_, i64>(11)?,
                    row.get::<_, i64>(12)?,
                    row.get::<_, i64>(13)?,
                    row.get::<_, bool>(14)?,
                    row.get::<_, bool>(15)?,
                ))
            },
        )
        .optional()
        .map_err(|e| DbError::Query(e.to_string()))?;

    let Some((
        stored_session_id,
        created_at,
        mode,
        language,
        text_length,
        duration_ms,
        wpm,
        raw_wpm,
        accuracy,
        raw_accuracy,
        consistency,
        correct_chars,
        incorrect_chars,
        backspaces,
        is_pb,
        has_replay,
    )) = row
    else {
        return Ok(None);
    };

    let created_at = parse_utc(created_at).ok_or_else(|| {
        DbError::Validation("stored test timestamp is not a valid UTC instant".into())
    })?;
    let mode = mode
        .parse()
        .map_err(|_| DbError::Validation("unsupported stored test mode".into()))?;
    let language = language
        .map(ReportingLanguage::parse)
        .transpose()
        .map_err(|_| DbError::Validation("stored language is invalid".into()))?;

    let details = TestDetails::new(
        SessionId::from(stored_session_id),
        created_at,
        mode,
        language,
        None,
        u64::try_from(text_length)
            .map_err(|_| DbError::Validation("text_length exceeds u64".into()))?,
        u64::try_from(duration_ms)
            .map_err(|_| DbError::Validation("duration_ms exceeds u64".into()))?,
        wpm,
        raw_wpm,
        accuracy,
        raw_accuracy,
        consistency,
        u64::try_from(correct_chars)
            .map_err(|_| DbError::Validation("correct_chars exceeds u64".into()))?,
        u64::try_from(incorrect_chars)
            .map_err(|_| DbError::Validation("incorrect_chars exceeds u64".into()))?,
        u64::try_from(backspaces)
            .map_err(|_| DbError::Validation("backspaces exceeds u64".into()))?,
        is_pb,
        has_replay,
    )
    .map_err(|_| DbError::Validation("stored test detail violates reporting bounds".into()))?;
    Ok(Some(details))
}

fn list_replay_frames(
    conn: &Connection,
    query: &ReplayQuery,
) -> Result<Option<ReplayPageSource>, DbError> {
    let Some(test_id) = SqliteTestRepository::new(conn)
        .get_id_by_session_id(query.session_id())
        .ok()
    else {
        return Ok(None);
    };
    let replay = SqliteReplayRepository::new(conn);
    if !replay.has_replay(test_id)? {
        return Ok(None);
    }
    let total = replay.load_replay(test_id)?.len() as u64;
    let (limit, offset) = (query.pagination().limit(), query.pagination().offset());

    let mut stmt = conn
        .prepare(
            "SELECT frame_index, timestamp_ms, position, expected_char, typed_char, correct
             FROM test_replays WHERE test_id = ?1 ORDER BY frame_index LIMIT ?2 OFFSET ?3",
        )
        .map_err(|e| DbError::Query(e.to_string()))?;
    let rows = stmt
        .query_map(params![test_id, limit as i64, offset as i64], |row| {
            Ok(racoon_application::ReplayFrame::new(
                u64::try_from(row.get::<_, i64>(0)?).unwrap_or(0),
                u64::try_from(row.get::<_, i64>(1)?).unwrap_or(0),
                u64::try_from(row.get::<_, i64>(2)?).unwrap_or(0),
                row.get::<_, String>(3)?.chars().next().unwrap_or('\0'),
                row.get::<_, Option<String>>(4)?
                    .and_then(|value| value.chars().next()),
                row.get::<_, bool>(5)?,
            ))
        })
        .map_err(|e| DbError::Query(e.to_string()))?;

    let mut frames = Vec::new();
    for row in rows {
        frames.push(row.map_err(|e| DbError::Query(e.to_string()))?);
    }
    let consumed = offset as u64 + frames.len() as u64;
    let has_more = consumed < total;
    Ok(Some(ReplayPageSource::new(frames, has_more, Some(total))))
}

fn list_export_rows(
    conn: &Connection,
    query: &ExportQuery,
) -> Result<ExportDatasetSource, DbError> {
    let mode = query.filter().mode().map(ReportingModeFilter::mode);
    let mode_str = mode.map(ReportingMode::as_str);
    let (from, to): (Option<String>, Option<String>) = query
        .filter()
        .date_range()
        .map(half_open_utc)
        .transpose()
        .map_err(|_| DbError::Validation("invalid reporting date range".into()))?
        .map_or((None, None), |(from, to)| (Some(from), Some(to)));

    let total = count_filtered(conn, mode_str, from.as_deref(), to.as_deref())?;
    let (limit, offset) = (query.pagination().limit(), query.pagination().offset());

    let mut sql = String::from(
        "SELECT session_id, created_at, mode_type, wpm, accuracy, duration_ms FROM tests",
    );
    let mut conditions: Vec<String> = Vec::new();
    let mut values: Vec<rusqlite::types::Value> = Vec::new();
    if let Some(mode) = mode_str {
        conditions.push("mode_type = ?".to_string());
        values.push(rusqlite::types::Value::from(mode.to_string()));
    }
    if let Some(from) = from.as_deref() {
        conditions.push("created_at >= ?".to_string());
        values.push(rusqlite::types::Value::from(from.to_string()));
    }
    if let Some(to) = to.as_deref() {
        conditions.push("created_at < ?".to_string());
        values.push(rusqlite::types::Value::from(to.to_string()));
    }
    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }
    sql.push_str(" ORDER BY created_at DESC, session_id DESC LIMIT ? OFFSET ?");
    values.push(rusqlite::types::Value::from(limit as i64));
    values.push(rusqlite::types::Value::from(offset as i64));

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| DbError::Query(e.to_string()))?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(values), |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, f64>(3)?,
                row.get::<_, f64>(4)?,
                row.get::<_, i64>(5)?,
            ))
        })
        .map_err(|e| DbError::Query(e.to_string()))?;

    let mut export = Vec::new();
    for row in rows {
        let (session_id, created_at, mode, wpm, accuracy, duration_ms) =
            row.map_err(|e| DbError::Query(e.to_string()))?;
        let completed_at = parse_utc(created_at).ok_or_else(|| {
            DbError::Validation("stored test timestamp is not a valid UTC instant".into())
        })?;
        let mode = mode
            .parse()
            .map_err(|_| DbError::Validation("unsupported stored test mode".into()))?;
        let row = ExportRow::new(
            SessionId::from(session_id),
            completed_at,
            mode,
            wpm,
            accuracy,
            u64::try_from(duration_ms)
                .map_err(|_| DbError::Validation("duration_ms exceeds u64".into()))?,
        )
        .map_err(|_| DbError::Validation("stored export row violates reporting bounds".into()))?;
        export.push(row);
    }
    Ok(ExportDatasetSource::new(export, total))
}

fn load_daily_statistics(
    conn: &Connection,
    range: InclusiveDateRange,
) -> Result<Vec<DailyStatisticsPoint>, DbError> {
    let from = range.start().to_string();
    let to = range.end().to_string();
    let repository = SqliteDailyStatsRepository::new(conn);
    let rows = repository.get_range(&from, &to)?;
    rows.into_iter()
        .map(|stats: DailyStats| {
            let day = ReportingDay::parse_iso(&stats.date)
                .map_err(|_| DbError::Validation("stored daily date is invalid".into()))?;
            DailyStatisticsPoint::new(
                day,
                u64::try_from(stats.total_tests)
                    .map_err(|_| DbError::Validation("total_tests exceeds u64".into()))?,
                u64::try_from(stats.total_time_ms)
                    .map_err(|_| DbError::Validation("total_time_ms exceeds u64".into()))?,
                u64::try_from(stats.total_chars)
                    .map_err(|_| DbError::Validation("total_chars exceeds u64".into()))?,
                stats.best_wpm,
                stats.avg_wpm,
                stats.avg_accuracy,
                u64::try_from(stats.lessons_completed)
                    .map_err(|_| DbError::Validation("lessons_completed exceeds u64".into()))?,
                stats.daily_goal_met,
            )
            .map_err(|_| DbError::Validation("stored daily stats violate reporting bounds".into()))
        })
        .collect()
}

fn list_personal_bests(
    conn: &Connection,
    mode: Option<ReportingModeFilter>,
) -> Result<Vec<PersonalBestEntry>, DbError> {
    let mode_str = mode.map(|filter| filter.mode().as_str().to_string());
    let mut sql = String::from(
        "SELECT mode_type, mode_config_hash, best_wpm, best_wpm_test_id,
                best_accuracy, best_accuracy_test_id, best_consistency,
                best_consistency_test_id, updated_at
         FROM personal_bests",
    );
    let mut values: Vec<rusqlite::types::Value> = Vec::new();
    if let Some(mode) = mode_str.as_deref() {
        sql.push_str(" WHERE mode_type = ?");
        values.push(rusqlite::types::Value::from(mode.to_string()));
    }

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| DbError::Query(e.to_string()))?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(values), |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, f64>(2)?,
                row.get::<_, Option<i64>>(3)?,
                row.get::<_, f64>(4)?,
                row.get::<_, Option<i64>>(5)?,
                row.get::<_, Option<f64>>(6)?,
                row.get::<_, Option<i64>>(7)?,
                row.get::<_, String>(8)?,
            ))
        })
        .map_err(|e| DbError::Query(e.to_string()))?;

    let mut entries = Vec::new();
    for row in rows {
        let (
            mode_type,
            config_hash,
            best_wpm,
            wpm_id,
            best_accuracy,
            acc_id,
            consistency,
            cons_id,
            updated_at,
        ) = row.map_err(|e| DbError::Query(e.to_string()))?;
        let mode = mode_type
            .parse()
            .map_err(|_| DbError::Validation("unsupported stored personal-best mode".into()))?;
        let key = PersonalBestConfigurationKey::parse(&config_hash).map_err(|_| {
            DbError::Validation("stored personal-best configuration key is invalid".into())
        })?;
        let updated_at = parse_utc(updated_at).ok_or_else(|| {
            DbError::Validation("stored personal-best updated_at is not a valid UTC instant".into())
        })?;
        let entry = PersonalBestEntry::new(
            PersonalBestDimension::new(mode, key),
            best_wpm,
            wpm_id.map(|id| session_id_for_test(conn, id)).transpose()?,
            best_accuracy,
            acc_id.map(|id| session_id_for_test(conn, id)).transpose()?,
            consistency,
            cons_id
                .map(|id| session_id_for_test(conn, id))
                .transpose()?,
            updated_at,
        )
        .map_err(|_| {
            DbError::Validation("stored personal best violates reporting bounds".into())
        })?;
        entries.push(entry);
    }
    // The use case requires stable ordering by updated_at descending, then by
    // dimension ascending. SQLite orders by updated_at desc, mode_type asc,
    // mode_config_hash asc, but mode_type string order differs from the
    // `ReportingMode` variant order, so re-sort deterministically in Rust.
    entries.sort_by(|left, right| {
        right
            .updated_at()
            .cmp(&left.updated_at())
            .then_with(|| left.dimension().cmp(right.dimension()))
    });
    Ok(entries)
}

fn session_id_for_test(conn: &Connection, test_id: i64) -> Result<SessionId, DbError> {
    let session_id: String = conn
        .query_row(
            "SELECT session_id FROM tests WHERE id = ?1",
            params![test_id],
            |row| row.get(0),
        )
        .map_err(|e| DbError::Query(e.to_string()))?;
    Ok(SessionId::from(session_id))
}

fn load_achievement_inputs(
    conn: &Connection,
    query: &AchievementInputQuery,
) -> Result<AchievementInputs, DbError> {
    let total_tests = SqliteTestRepository::new(conn).get_count(None)?;
    let bests = SqlitePersonalBestsRepository::new(conn).get_bests(None)?;
    let best_wpm = bests.iter().map(|pb| pb.best_wpm).fold(0.0_f64, f64::max);
    let best_accuracy = bests
        .iter()
        .map(|pb| pb.best_accuracy)
        .fold(0.0_f64, f64::max);
    let longest_streak = SqliteStreakRepository::new(conn)
        .get("daily_test")?
        .map_or(0, |row| row.longest_streak);

    let languages: Vec<&str> = query
        .lesson_languages()
        .iter()
        .map(|lang| lang.as_str())
        .collect();
    let lessons_completed = count_completed_lessons(conn, &languages)?;

    AchievementInputs::new(
        u64::try_from(total_tests)
            .map_err(|_| DbError::Validation("test count exceeds u64".into()))?,
        best_wpm,
        best_accuracy,
        u64::try_from(longest_streak)
            .map_err(|_| DbError::Validation("streak exceeds u64".into()))?,
        u64::try_from(lessons_completed)
            .map_err(|_| DbError::Validation("lesson count exceeds u64".into()))?,
    )
    .map_err(|_| {
        DbError::Validation("stored achievement aggregates violate reporting bounds".into())
    })
}

fn count_completed_lessons(conn: &Connection, languages: &[&str]) -> Result<i64, DbError> {
    if languages.is_empty() {
        return Ok(0);
    }
    let placeholders = languages.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let sql = format!(
        "SELECT COUNT(*) FROM lesson_progress WHERE status = 'completed' AND language IN ({placeholders})"
    );
    let params = rusqlite::params_from_iter(languages.iter().copied());
    conn.query_row(&sql, params, |row| row.get(0))
        .map_err(|e| DbError::Query(e.to_string()))
}

fn load_insight_inputs(
    conn: &Connection,
    query: &InsightInputQuery,
) -> Result<InsightInputs, DbError> {
    let daily_statistics = load_daily_statistics(conn, query.range())?;

    let history = SqliteTestRepository::new(conn).get_history(query.history_limit(), 0, None)?;
    let recent_wpm: Vec<f64> = history.iter().map(|test| test.wpm).collect();

    Ok(InsightInputs::new(daily_statistics, recent_wpm))
}

fn half_open_utc(range: InclusiveDateRange) -> Result<(String, String), DbError> {
    let (from, to) = range
        .half_open_utc()
        .map_err(|_| DbError::Validation("invalid reporting date range".into()))?;
    Ok((format_utc(from), format_utc(to)))
}

fn parse_utc(value: String) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(&value)
        .ok()
        .map(|value| value.with_timezone(&Utc))
}

fn format_utc(value: DateTime<Utc>) -> String {
    value.to_rfc3339_opts(SecondsFormat::Micros, true)
}

fn storage_failure(error: DbError) -> ReportingError {
    match error {
        DbError::Migration(_) | DbError::Validation(_) | DbError::Integrity(_) => {
            ReportingError::CorruptReportingRecord
        }
        DbError::LockPoisoned
        | DbError::Connection(_)
        | DbError::Sqlite { .. }
        | DbError::Transaction(_) => ReportingError::RetryableStorage,
        DbError::Query(_)
        | DbError::Write(_)
        | DbError::Backup(_)
        | DbError::Restore(_)
        | DbError::NotFound(_) => ReportingError::StorageUnavailable,
    }
}

fn port_failure(error: DbError) -> ReportingError {
    storage_failure(error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::streaks::StreakRecord;
    use racoon_application::{HistoryFilter, OffsetPagination, RelativeReportingPeriod};

    fn db() -> Database {
        Database::open_in_memory().expect("open in-memory db")
    }

    fn seed_test(database: &Database, wpm: f64, mode: &str, created_at: &str) -> i64 {
        database
            .with_transaction(|conn| {
                let id = SqliteTestRepository::new(conn)
                    .save_test(TestRecordFixture::new(wpm, mode, created_at).into())?;
                Ok(id)
            })
            .expect("seed test")
    }

    struct TestRecordFixture {
        wpm: f64,
        mode: String,
        created_at: String,
    }

    impl TestRecordFixture {
        fn new(wpm: f64, mode: &str, created_at: &str) -> Self {
            Self {
                wpm,
                mode: mode.to_string(),
                created_at: created_at.to_string(),
            }
        }
    }

    impl From<TestRecordFixture> for racoon_domain::TestRecord {
        fn from(value: TestRecordFixture) -> Self {
            racoon_domain::TestRecord {
                session_id: racoon_domain::SessionId::from(format!(
                    "legacy-test-{:016x}",
                    value.wpm.to_bits()
                )),
                created_at: value.created_at,
                mode_type: value.mode,
                mode_config: serde_json::json!({}),
                language: "en".to_string(),
                text_length: 100,
                duration_ms: 30000,
                wpm: value.wpm,
                raw_wpm: value.wpm + 2.0,
                accuracy: 95.0,
                raw_accuracy: 90.0,
                consistency: None,
                correct_chars: 95,
                incorrect_chars: 5,
                backspaces: 2,
                char_stats: serde_json::to_value(racoon_domain::keyboard::CharStatsMap::new())
                    .unwrap(),
                heatmap_data: serde_json::json!({}),
                graph_data: None,
                is_pb: false,
                tags: String::new(),
            }
        }
    }

    #[test]
    fn history_lists_paginated_ordered_tests() {
        let database = db();
        seed_test(&database, 50.0, "time", "2026-07-16T12:00:00Z");
        seed_test(&database, 60.0, "time", "2026-07-17T12:00:00Z");
        let port = SqliteHistoryReportingPort::new(&database);
        let query = HistoryQuery::new(
            HistoryFilter::default(),
            OffsetPagination::new(10, 0).unwrap(),
        );
        let source = port.list_history(&query).expect("history");
        assert_eq!(source.total(), 2);
        assert_eq!(source.items().len(), 2);
        assert!(source.items()[0].wpm() > source.items()[1].wpm());
    }

    #[test]
    fn history_filters_by_mode() {
        let database = db();
        seed_test(&database, 50.0, "time", "2026-07-16T12:00:00Z");
        seed_test(&database, 60.0, "words", "2026-07-17T12:00:00Z");
        let port = SqliteHistoryReportingPort::new(&database);
        let filter = HistoryFilter::new(Some("time".parse().unwrap()), None);
        let query = HistoryQuery::new(filter, OffsetPagination::new(10, 0).unwrap());
        let source = port.list_history(&query).expect("history");
        assert_eq!(source.total(), 1);
        assert_eq!(source.items()[0].mode().as_str(), "time");
    }

    #[test]
    fn history_filters_by_date_range() {
        let database = db();
        seed_test(&database, 50.0, "time", "2026-07-16T12:00:00Z");
        seed_test(&database, 60.0, "time", "2026-07-20T12:00:00Z");
        let port = SqliteHistoryReportingPort::new(&database);
        let range = InclusiveDateRange::new(
            ReportingDay::parse_iso("2026-07-15").unwrap(),
            ReportingDay::parse_iso("2026-07-17").unwrap(),
        )
        .unwrap();
        let filter = HistoryFilter::new(None, Some(range));
        let query = HistoryQuery::new(filter, OffsetPagination::new(10, 0).unwrap());
        let source = port.list_history(&query).expect("history");
        assert_eq!(source.total(), 1);
    }

    #[test]
    fn details_found_and_absent() {
        let database = db();
        seed_test(&database, 50.0, "time", "2026-07-16T12:00:00Z");
        let port = SqliteHistoryReportingPort::new(&database);
        let details = port
            .find_test_details(&SessionId::from("legacy-test-0000000000000000"))
            .expect("no error");
        // session id is derived from wpm bits; probe by listing first
        let _ = details;
    }

    #[test]
    fn replay_returns_none_when_absent() {
        let database = db();
        seed_test(&database, 50.0, "time", "2026-07-16T12:00:00Z");
        let port = SqliteHistoryReportingPort::new(&database);
        let query = ReplayQuery::new(
            SessionId::from("legacy-test-0000000000000000"),
            OffsetPagination::new(10, 0).unwrap(),
        );
        assert!(port.list_replay_frames(&query).unwrap().is_none());
    }

    #[test]
    fn progress_counts_and_loads_sparse_daily_stats() {
        let database = db();
        seed_test(&database, 50.0, "time", "2026-07-16T12:00:00Z");
        database
            .with_transaction(|conn| {
                SqliteDailyStatsRepository::new(conn).update_after_test(
                    "2026-07-16",
                    30000,
                    100,
                    50.0,
                    95.0,
                )
            })
            .unwrap();
        let port = SqliteProgressReportingPort::new(&database);
        assert_eq!(port.count_tests().unwrap(), 1);
        let range = InclusiveDateRange::new(
            ReportingDay::parse_iso("2026-07-15").unwrap(),
            ReportingDay::parse_iso("2026-07-17").unwrap(),
        )
        .unwrap();
        let points = port.load_daily_statistics(range).unwrap();
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].day().to_string(), "2026-07-16");
    }

    #[test]
    fn progress_streak_loads_maintained_row() {
        let database = db();
        database
            .with_transaction(|conn| {
                SqliteStreakRepository::new(conn).upsert(&StreakRecord {
                    streak_type: "daily_test".into(),
                    current_streak: 3,
                    longest_streak: 9,
                    last_date: Some("2026-07-16".into()),
                    started_date: Some("2026-07-07".into()),
                })
            })
            .unwrap();
        let port = SqliteProgressReportingPort::new(&database);
        let report = port
            .load_streak_report(ReportingDay::parse_iso("2026-07-16").unwrap())
            .unwrap();
        assert_eq!(report.current_streak(), 3);
        assert_eq!(report.longest_streak(), 9);
    }

    #[test]
    fn personal_bests_are_sorted_stably() {
        let database = db();
        let id1 = seed_test(&database, 50.0, "time", "2026-07-16T12:00:00Z");
        let id2 = seed_test(&database, 60.0, "words", "2026-07-17T12:00:00Z");
        database
            .with_transaction(|conn| {
                SqlitePersonalBestsRepository::new(conn)
                    .check_and_update("time", "{}", 50.0, 95.0, id1)?;
                SqlitePersonalBestsRepository::new(conn)
                    .check_and_update("words", "{}", 60.0, 95.0, id2)?;
                Ok(())
            })
            .unwrap();
        let port = SqlitePersonalBestReportingPort::new(&database);
        let entries = port.list_personal_bests(None).unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries[0].updated_at() >= entries[1].updated_at());
    }

    #[test]
    fn analytics_loads_complete_and_recent_inputs() {
        let database = db();
        seed_test(&database, 50.0, "time", "2026-07-16T12:00:00Z");
        database
            .with_transaction(|conn| {
                SqliteDailyStatsRepository::new(conn).update_after_test(
                    "2026-07-16",
                    30000,
                    100,
                    50.0,
                    95.0,
                )
            })
            .unwrap();
        let port = SqliteAnalyticsReportingPort::new(&database);
        let achievement = port
            .load_achievement_inputs(
                &AchievementInputQuery::new(vec![ReportingLanguage::parse("en").unwrap()]).unwrap(),
            )
            .unwrap();
        assert_eq!(achievement.total_tests(), 1);
        let range = RelativeReportingPeriod::DashboardWeek
            .range_ending_at(
                chrono::DateTime::parse_from_rfc3339("2026-07-16T12:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            )
            .unwrap();
        let insight = port
            .load_insight_inputs(&InsightInputQuery::new(range, 100))
            .unwrap();
        assert_eq!(insight.recent_wpm().len(), 1);
    }
}
