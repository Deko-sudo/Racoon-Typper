//! Application-owned reporting contracts and deterministic use cases.
//!
//! This module deliberately contains reporting policy, typed requests, and
//! privacy-minimized projections only. Storage execution and transport
//! conversion remain adapter responsibilities.

use std::{fmt, str::FromStr};

use chrono::{DateTime, Duration, NaiveDate, Utc};
use racoon_core::{
    analytics::{check_achievements, generate_insights, Achievement, Insight},
    consistency::{calc_consistency, ConsistencyReport},
};
use racoon_domain::SessionId;

use crate::ports::{
    AnalyticsReportingPort, HistoryReportingPort, PersonalBestReportingPort, ProgressReportingPort,
    SessionWallClock,
};

/// The largest page returned by an application reporting query.
pub const MAX_REPORTING_PAGE_LIMIT: usize = 1_000;

/// The largest offset accepted by an application reporting query.
pub const MAX_REPORTING_PAGE_OFFSET: usize = 1_000_000;

/// Existing history callers default to this page size.
pub const DEFAULT_HISTORY_PAGE_LIMIT: usize = 50;

/// Existing export callers request one maximum-size page.
pub const DEFAULT_EXPORT_PAGE_LIMIT: usize = MAX_REPORTING_PAGE_LIMIT;

/// Existing insight and consistency calculations inspect this many tests.
pub const ANALYTICS_HISTORY_LIMIT: usize = 100;

/// Bounded application errors for reporting requests and reporting ports.
///
/// These variants deliberately carry no storage diagnostics, paths, payloads,
/// or user-entered content. Adapters classify their own failures into this
/// vocabulary before they cross the application boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportingError {
    InvalidDateRange,
    DateArithmeticOverflow,
    InvalidPagination,
    UnsupportedMode,
    TestNotFound,
    ReplayUnavailable,
    CorruptReportingRecord,
    RetryableStorage,
    StorageUnavailable,
    InvariantViolation,
}

impl fmt::Display for ReportingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidDateRange => "invalid reporting date range",
            Self::DateArithmeticOverflow => "reporting date arithmetic overflowed",
            Self::InvalidPagination => "invalid reporting pagination",
            Self::UnsupportedMode => "unsupported reporting mode",
            Self::TestNotFound => "test was not found",
            Self::ReplayUnavailable => "replay is unavailable",
            Self::CorruptReportingRecord => "stored reporting record is corrupt",
            Self::RetryableStorage => "reporting storage is temporarily unavailable",
            Self::StorageUnavailable => "reporting storage is unavailable",
            Self::InvariantViolation => "reporting invariant was violated",
        })
    }
}

impl std::error::Error for ReportingError {}

/// A UTC calendar day used by all reporting requests and projections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReportingDay(NaiveDate);

impl ReportingDay {
    pub fn new(year: i32, month: u32, day: u32) -> Result<Self, ReportingError> {
        NaiveDate::from_ymd_opt(year, month, day)
            .map(Self)
            .ok_or(ReportingError::InvalidDateRange)
    }

    pub fn parse_iso(value: &str) -> Result<Self, ReportingError> {
        NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .map(Self)
            .map_err(|_| ReportingError::InvalidDateRange)
    }

    pub fn from_utc(timestamp: DateTime<Utc>) -> Self {
        Self(timestamp.date_naive())
    }

    pub const fn as_naive_date(self) -> NaiveDate {
        self.0
    }

    pub fn start_utc(self) -> Result<DateTime<Utc>, ReportingError> {
        self.0
            .and_hms_opt(0, 0, 0)
            .map(|timestamp| DateTime::from_naive_utc_and_offset(timestamp, Utc))
            .ok_or(ReportingError::DateArithmeticOverflow)
    }

    pub fn next_day(self) -> Result<Self, ReportingError> {
        self.0
            .checked_add_signed(Duration::days(1))
            .map(Self)
            .ok_or(ReportingError::DateArithmeticOverflow)
    }

    pub fn days_before(self, days: i64) -> Result<Self, ReportingError> {
        self.0
            .checked_sub_signed(Duration::days(days))
            .map(Self)
            .ok_or(ReportingError::DateArithmeticOverflow)
    }

    pub fn signed_days_since(self, earlier: Self) -> i64 {
        (self.0 - earlier.0).num_days()
    }
}

impl fmt::Display for ReportingDay {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.format("%Y-%m-%d").fmt(formatter)
    }
}

/// A public UTC date range with inclusive calendar-day endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InclusiveDateRange {
    start: ReportingDay,
    end: ReportingDay,
}

impl InclusiveDateRange {
    pub fn new(start: ReportingDay, end: ReportingDay) -> Result<Self, ReportingError> {
        if start > end {
            return Err(ReportingError::InvalidDateRange);
        }
        Ok(Self { start, end })
    }

    pub const fn single(day: ReportingDay) -> Self {
        Self {
            start: day,
            end: day,
        }
    }

    pub const fn start(self) -> ReportingDay {
        self.start
    }

    pub const fn end(self) -> ReportingDay {
        self.end
    }

    /// Converts the closed day range to a deterministic half-open UTC range.
    pub fn half_open_utc(self) -> Result<(DateTime<Utc>, DateTime<Utc>), ReportingError> {
        Ok((self.start.start_utc()?, self.end.next_day()?.start_utc()?))
    }

    pub fn contains(self, day: ReportingDay) -> bool {
        self.start <= day && day <= self.end
    }
}

/// The currently supported reporting modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReportingMode {
    Time,
    Words,
    Quote,
    Custom,
    Lesson,
}

impl ReportingMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Time => "time",
            Self::Words => "words",
            Self::Quote => "quote",
            Self::Custom => "custom",
            Self::Lesson => "lesson",
        }
    }
}

impl fmt::Display for ReportingMode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for ReportingMode {
    type Err = ReportingError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "time" => Ok(Self::Time),
            "words" => Ok(Self::Words),
            "quote" => Ok(Self::Quote),
            "custom" => Ok(Self::Custom),
            "lesson" => Ok(Self::Lesson),
            _ => Err(ReportingError::UnsupportedMode),
        }
    }
}

/// A validated optional reporting-mode constraint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReportingModeFilter(ReportingMode);

impl ReportingModeFilter {
    pub const fn new(mode: ReportingMode) -> Self {
        Self(mode)
    }

    pub const fn mode(self) -> ReportingMode {
        self.0
    }
}

impl FromStr for ReportingModeFilter {
    type Err = ReportingError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value.parse().map(Self)
    }
}

/// A bounded language label present in reporting data when it is meaningful.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReportingLanguage(String);

impl ReportingLanguage {
    pub fn parse(value: impl AsRef<str>) -> Result<Self, ReportingError> {
        let value = value.as_ref();
        if value.is_empty()
            || value.chars().count() > 16
            || !value
                .chars()
                .all(|character| character.is_ascii_lowercase() || character == '-')
        {
            return Err(ReportingError::CorruptReportingRecord);
        }
        Ok(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ReportingLanguage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// A bounded lesson reference that does not include lesson content.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReportingLessonId(String);

impl ReportingLessonId {
    pub fn parse(value: impl AsRef<str>) -> Result<Self, ReportingError> {
        let value = value.as_ref();
        if value.is_empty()
            || value.chars().count() > 128
            || !value.chars().all(|character| {
                character.is_ascii_alphanumeric() || character == '_' || character == '-'
            })
        {
            return Err(ReportingError::CorruptReportingRecord);
        }
        Ok(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Opaque grouping material for a personal-best mode configuration.
///
/// This value is intentionally not the source configuration JSON. The data
/// adapter will later map its stored, stable grouping value into this bounded
/// application value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PersonalBestConfigurationKey(String);

impl PersonalBestConfigurationKey {
    pub fn parse(value: impl AsRef<str>) -> Result<Self, ReportingError> {
        let value = value.as_ref();
        if value.is_empty()
            || value.chars().count() > 128
            || !value.chars().all(|character| character.is_ascii_hexdigit())
        {
            return Err(ReportingError::CorruptReportingRecord);
        }
        Ok(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Bounded offset pagination shared by history, replay, and export requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OffsetPagination {
    limit: usize,
    offset: usize,
}

impl OffsetPagination {
    pub fn new(limit: usize, offset: usize) -> Result<Self, ReportingError> {
        if !(1..=MAX_REPORTING_PAGE_LIMIT).contains(&limit) || offset > MAX_REPORTING_PAGE_OFFSET {
            return Err(ReportingError::InvalidPagination);
        }
        Ok(Self { limit, offset })
    }

    pub const fn history_default() -> Self {
        Self {
            limit: DEFAULT_HISTORY_PAGE_LIMIT,
            offset: 0,
        }
    }

    pub const fn export_default() -> Self {
        Self {
            limit: DEFAULT_EXPORT_PAGE_LIMIT,
            offset: 0,
        }
    }

    pub const fn limit(self) -> usize {
        self.limit
    }

    pub const fn offset(self) -> usize {
        self.offset
    }

    pub fn has_more(self, total: u64, returned: usize) -> Result<bool, ReportingError> {
        if returned > self.limit {
            return Err(ReportingError::InvariantViolation);
        }
        let consumed = u64::try_from(self.offset)
            .map_err(|_| ReportingError::InvariantViolation)?
            .checked_add(u64::try_from(returned).map_err(|_| ReportingError::InvariantViolation)?)
            .ok_or(ReportingError::InvariantViolation)?;
        Ok(consumed < total)
    }

    /// Returns the next valid offset when the current response has more data.
    /// A result outside the accepted offset bound has no representable next
    /// request and is returned as `None` rather than overflowing.
    pub fn next_offset(self, total: u64, returned: usize) -> Result<Option<usize>, ReportingError> {
        if !self.has_more(total, returned)? || returned == 0 {
            return Ok(None);
        }
        let next = self
            .offset
            .checked_add(returned)
            .ok_or(ReportingError::InvariantViolation)?;
        Ok((next <= MAX_REPORTING_PAGE_OFFSET).then_some(next))
    }
}

/// Accepted relative periods used by existing reporting views.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelativeReportingPeriod {
    DashboardWeek,
    ProgressSevenDays,
    ProgressThirtyDays,
    ProgressNinetyDays,
}

impl RelativeReportingPeriod {
    pub const fn days_back(self) -> i64 {
        match self {
            Self::DashboardWeek | Self::ProgressSevenDays => 7,
            Self::ProgressThirtyDays => 30,
            Self::ProgressNinetyDays => 90,
        }
    }

    /// Preserves the existing inclusive behavior: a seven-day lookback covers
    /// today plus the preceding seven calendar days.
    pub fn range_ending_at(self, now: DateTime<Utc>) -> Result<InclusiveDateRange, ReportingError> {
        let end = ReportingDay::from_utc(now);
        InclusiveDateRange::new(end.days_before(self.days_back())?, end)
    }
}

/// Optional constraints supported by current history and export behavior.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HistoryFilter {
    mode: Option<ReportingModeFilter>,
    date_range: Option<InclusiveDateRange>,
}

impl HistoryFilter {
    pub const fn new(
        mode: Option<ReportingModeFilter>,
        date_range: Option<InclusiveDateRange>,
    ) -> Self {
        Self { mode, date_range }
    }

    pub const fn mode(&self) -> Option<ReportingModeFilter> {
        self.mode
    }

    pub const fn date_range(&self) -> Option<InclusiveDateRange> {
        self.date_range
    }
}

/// A validated history-port request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryQuery {
    filter: HistoryFilter,
    pagination: OffsetPagination,
}

impl HistoryQuery {
    pub const fn new(filter: HistoryFilter, pagination: OffsetPagination) -> Self {
        Self { filter, pagination }
    }

    pub const fn filter(&self) -> &HistoryFilter {
        &self.filter
    }

    pub const fn pagination(&self) -> OffsetPagination {
        self.pagination
    }
}

/// An application request for a history page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListTestHistoryRequest(HistoryQuery);

impl ListTestHistoryRequest {
    pub const fn new(filter: HistoryFilter, pagination: OffsetPagination) -> Self {
        Self(HistoryQuery::new(filter, pagination))
    }

    pub const fn query(&self) -> &HistoryQuery {
        &self.0
    }
}

impl Default for ListTestHistoryRequest {
    fn default() -> Self {
        Self::new(
            HistoryFilter::default(),
            OffsetPagination::history_default(),
        )
    }
}

/// Privacy-minimized summary of one completed typing session.
#[derive(Debug, Clone, PartialEq)]
pub struct HistoryItem {
    session_id: SessionId,
    completed_at: DateTime<Utc>,
    mode: ReportingMode,
    language: Option<ReportingLanguage>,
    duration_ms: u64,
    characters: u64,
    wpm: f64,
    accuracy: f64,
    is_personal_best: bool,
    replay_available: bool,
}

impl HistoryItem {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        session_id: SessionId,
        completed_at: DateTime<Utc>,
        mode: ReportingMode,
        language: Option<ReportingLanguage>,
        duration_ms: u64,
        characters: u64,
        wpm: f64,
        accuracy: f64,
        is_personal_best: bool,
        replay_available: bool,
    ) -> Result<Self, ReportingError> {
        validate_score(wpm)?;
        validate_percentage(accuracy)?;
        Ok(Self {
            session_id,
            completed_at,
            mode,
            language,
            duration_ms,
            characters,
            wpm,
            accuracy,
            is_personal_best,
            replay_available,
        })
    }

    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    pub const fn completed_at(&self) -> DateTime<Utc> {
        self.completed_at
    }

    pub const fn mode(&self) -> ReportingMode {
        self.mode
    }

    pub fn language(&self) -> Option<&ReportingLanguage> {
        self.language.as_ref()
    }

    pub const fn duration_ms(&self) -> u64 {
        self.duration_ms
    }

    pub const fn characters(&self) -> u64 {
        self.characters
    }

    pub const fn wpm(&self) -> f64 {
        self.wpm
    }

    pub const fn accuracy(&self) -> f64 {
        self.accuracy
    }

    pub const fn is_personal_best(&self) -> bool {
        self.is_personal_best
    }

    pub const fn replay_available(&self) -> bool {
        self.replay_available
    }
}

/// A page result returned by a history reporting port before envelope policy.
#[derive(Debug, Clone, PartialEq)]
pub struct HistoryPageSource {
    items: Vec<HistoryItem>,
    total: u64,
}

impl HistoryPageSource {
    pub fn new(items: Vec<HistoryItem>, total: u64) -> Self {
        Self { items, total }
    }

    pub fn items(&self) -> &[HistoryItem] {
        &self.items
    }

    pub const fn total(&self) -> u64 {
        self.total
    }

    pub fn into_parts(self) -> (Vec<HistoryItem>, u64) {
        (self.items, self.total)
    }
}

/// A stable, bounded history response.
#[derive(Debug, Clone, PartialEq)]
pub struct HistoryPage {
    items: Vec<HistoryItem>,
    offset: usize,
    limit: usize,
    total: u64,
    has_more: bool,
}

impl HistoryPage {
    fn new(
        items: Vec<HistoryItem>,
        pagination: OffsetPagination,
        total: u64,
    ) -> Result<Self, ReportingError> {
        let has_more = pagination.has_more(total, items.len())?;
        Ok(Self {
            items,
            offset: pagination.offset(),
            limit: pagination.limit(),
            total,
            has_more,
        })
    }

    pub fn items(&self) -> &[HistoryItem] {
        &self.items
    }

    pub const fn offset(&self) -> usize {
        self.offset
    }

    pub const fn limit(&self) -> usize {
        self.limit
    }

    pub const fn total(&self) -> u64 {
        self.total
    }

    pub const fn has_more(&self) -> bool {
        self.has_more
    }
}

/// Details intentionally exclude typed text, replay frames, heatmaps, and
/// graph data. Those are separate, explicit reporting capabilities.
#[derive(Debug, Clone, PartialEq)]
pub struct TestDetails {
    session_id: SessionId,
    completed_at: DateTime<Utc>,
    mode: ReportingMode,
    language: Option<ReportingLanguage>,
    lesson_id: Option<ReportingLessonId>,
    characters: u64,
    duration_ms: u64,
    wpm: f64,
    raw_wpm: f64,
    accuracy: f64,
    raw_accuracy: f64,
    consistency: Option<f64>,
    correct_characters: u64,
    incorrect_characters: u64,
    backspaces: u64,
    is_personal_best: bool,
    replay_available: bool,
}

impl TestDetails {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        session_id: SessionId,
        completed_at: DateTime<Utc>,
        mode: ReportingMode,
        language: Option<ReportingLanguage>,
        lesson_id: Option<ReportingLessonId>,
        characters: u64,
        duration_ms: u64,
        wpm: f64,
        raw_wpm: f64,
        accuracy: f64,
        raw_accuracy: f64,
        consistency: Option<f64>,
        correct_characters: u64,
        incorrect_characters: u64,
        backspaces: u64,
        is_personal_best: bool,
        replay_available: bool,
    ) -> Result<Self, ReportingError> {
        validate_score(wpm)?;
        validate_score(raw_wpm)?;
        validate_percentage(accuracy)?;
        validate_percentage(raw_accuracy)?;
        if consistency.is_some_and(|value| !value.is_finite() || !(0.0..=100.0).contains(&value)) {
            return Err(ReportingError::CorruptReportingRecord);
        }
        Ok(Self {
            session_id,
            completed_at,
            mode,
            language,
            lesson_id,
            characters,
            duration_ms,
            wpm,
            raw_wpm,
            accuracy,
            raw_accuracy,
            consistency,
            correct_characters,
            incorrect_characters,
            backspaces,
            is_personal_best,
            replay_available,
        })
    }

    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    pub const fn completed_at(&self) -> DateTime<Utc> {
        self.completed_at
    }

    pub const fn mode(&self) -> ReportingMode {
        self.mode
    }

    pub fn language(&self) -> Option<&ReportingLanguage> {
        self.language.as_ref()
    }

    pub fn lesson_id(&self) -> Option<&ReportingLessonId> {
        self.lesson_id.as_ref()
    }

    pub const fn characters(&self) -> u64 {
        self.characters
    }

    pub const fn duration_ms(&self) -> u64 {
        self.duration_ms
    }

    pub const fn wpm(&self) -> f64 {
        self.wpm
    }

    pub const fn raw_wpm(&self) -> f64 {
        self.raw_wpm
    }

    pub const fn accuracy(&self) -> f64 {
        self.accuracy
    }

    pub const fn raw_accuracy(&self) -> f64 {
        self.raw_accuracy
    }

    pub const fn consistency(&self) -> Option<f64> {
        self.consistency
    }

    pub const fn correct_characters(&self) -> u64 {
        self.correct_characters
    }

    pub const fn incorrect_characters(&self) -> u64 {
        self.incorrect_characters
    }

    pub const fn backspaces(&self) -> u64 {
        self.backspaces
    }

    pub const fn is_personal_best(&self) -> bool {
        self.is_personal_best
    }

    pub const fn replay_available(&self) -> bool {
        self.replay_available
    }
}

/// An application-owned replay frame. Its `Debug` implementation redacts the
/// expected and typed characters because replay content is sensitive.
#[derive(Clone, PartialEq, Eq)]
pub struct ReplayFrame {
    frame_index: u64,
    timestamp_ms: u64,
    position: u64,
    expected_char: char,
    typed_char: Option<char>,
    correct: bool,
}

impl ReplayFrame {
    pub const fn new(
        frame_index: u64,
        timestamp_ms: u64,
        position: u64,
        expected_char: char,
        typed_char: Option<char>,
        correct: bool,
    ) -> Self {
        Self {
            frame_index,
            timestamp_ms,
            position,
            expected_char,
            typed_char,
            correct,
        }
    }

    pub const fn frame_index(&self) -> u64 {
        self.frame_index
    }

    pub const fn timestamp_ms(&self) -> u64 {
        self.timestamp_ms
    }

    pub const fn position(&self) -> u64 {
        self.position
    }

    pub const fn expected_char(&self) -> char {
        self.expected_char
    }

    pub const fn typed_char(&self) -> Option<char> {
        self.typed_char
    }

    pub const fn correct(&self) -> bool {
        self.correct
    }
}

impl fmt::Debug for ReplayFrame {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ReplayFrame")
            .field("frame_index", &self.frame_index)
            .field("timestamp_ms", &self.timestamp_ms)
            .field("position", &self.position)
            .field("typed_char_present", &self.typed_char.is_some())
            .field("correct", &self.correct)
            .finish()
    }
}

/// A validated replay-port request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayQuery {
    session_id: SessionId,
    pagination: OffsetPagination,
}

impl ReplayQuery {
    pub fn new(session_id: SessionId, pagination: OffsetPagination) -> Self {
        Self {
            session_id,
            pagination,
        }
    }

    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    pub const fn pagination(&self) -> OffsetPagination {
        self.pagination
    }
}

/// A replay page supplied by a reporting port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayPageSource {
    frames: Vec<ReplayFrame>,
    has_more: bool,
    total: Option<u64>,
}

impl ReplayPageSource {
    pub fn new(frames: Vec<ReplayFrame>, has_more: bool, total: Option<u64>) -> Self {
        Self {
            frames,
            has_more,
            total,
        }
    }

    pub fn frames(&self) -> &[ReplayFrame] {
        &self.frames
    }

    pub const fn has_more(&self) -> bool {
        self.has_more
    }

    pub const fn total(&self) -> Option<u64> {
        self.total
    }

    pub fn into_parts(self) -> (Vec<ReplayFrame>, bool, Option<u64>) {
        (self.frames, self.has_more, self.total)
    }
}

/// A stable, explicitly paginated replay result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayPage {
    session_id: SessionId,
    frames: Vec<ReplayFrame>,
    offset: usize,
    limit: usize,
    returned: usize,
    has_more: bool,
    total: Option<u64>,
}

impl ReplayPage {
    fn new(
        session_id: SessionId,
        pagination: OffsetPagination,
        frames: Vec<ReplayFrame>,
        has_more: bool,
        total: Option<u64>,
    ) -> Self {
        Self {
            session_id,
            returned: frames.len(),
            frames,
            offset: pagination.offset(),
            limit: pagination.limit(),
            has_more,
            total,
        }
    }

    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    pub fn frames(&self) -> &[ReplayFrame] {
        &self.frames
    }

    pub const fn offset(&self) -> usize {
        self.offset
    }

    pub const fn limit(&self) -> usize {
        self.limit
    }

    pub const fn returned(&self) -> usize {
        self.returned
    }

    pub const fn has_more(&self) -> bool {
        self.has_more
    }

    pub const fn total(&self) -> Option<u64> {
        self.total
    }
}

/// One persisted daily aggregate. Missing days remain absent by design.
#[derive(Debug, Clone, PartialEq)]
pub struct DailyStatisticsPoint {
    day: ReportingDay,
    total_tests: u64,
    total_duration_ms: u64,
    total_characters: u64,
    best_wpm: f64,
    average_wpm: f64,
    average_accuracy: f64,
    lessons_completed: u64,
    daily_goal_met: bool,
}

impl DailyStatisticsPoint {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        day: ReportingDay,
        total_tests: u64,
        total_duration_ms: u64,
        total_characters: u64,
        best_wpm: f64,
        average_wpm: f64,
        average_accuracy: f64,
        lessons_completed: u64,
        daily_goal_met: bool,
    ) -> Result<Self, ReportingError> {
        validate_score(best_wpm)?;
        validate_score(average_wpm)?;
        validate_percentage(average_accuracy)?;
        Ok(Self {
            day,
            total_tests,
            total_duration_ms,
            total_characters,
            best_wpm,
            average_wpm,
            average_accuracy,
            lessons_completed,
            daily_goal_met,
        })
    }

    pub const fn day(&self) -> ReportingDay {
        self.day
    }

    pub const fn total_tests(&self) -> u64 {
        self.total_tests
    }

    pub const fn total_duration_ms(&self) -> u64 {
        self.total_duration_ms
    }

    pub const fn total_characters(&self) -> u64 {
        self.total_characters
    }

    pub const fn best_wpm(&self) -> f64 {
        self.best_wpm
    }

    pub const fn average_wpm(&self) -> f64 {
        self.average_wpm
    }

    pub const fn average_accuracy(&self) -> f64 {
        self.average_accuracy
    }

    pub const fn lessons_completed(&self) -> u64 {
        self.lessons_completed
    }

    pub const fn daily_goal_met(&self) -> bool {
        self.daily_goal_met
    }
}

/// A sparse daily-statistics response for one inclusive day range.
#[derive(Debug, Clone, PartialEq)]
pub struct DailyStatisticsRange {
    range: InclusiveDateRange,
    points: Vec<DailyStatisticsPoint>,
}

impl DailyStatisticsRange {
    fn new(range: InclusiveDateRange, points: Vec<DailyStatisticsPoint>) -> Self {
        Self { range, points }
    }

    pub const fn range(&self) -> InclusiveDateRange {
        self.range
    }

    pub fn points(&self) -> &[DailyStatisticsPoint] {
        &self.points
    }
}

/// Deterministic streak state relative to an explicit UTC calendar day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreakReport {
    current_streak: u64,
    longest_streak: u64,
    as_of: ReportingDay,
}

impl StreakReport {
    pub const fn new(current_streak: u64, longest_streak: u64, as_of: ReportingDay) -> Self {
        Self {
            current_streak,
            longest_streak,
            as_of,
        }
    }

    pub const fn current_streak(&self) -> u64 {
        self.current_streak
    }

    pub const fn longest_streak(&self) -> u64 {
        self.longest_streak
    }

    pub const fn as_of(&self) -> ReportingDay {
        self.as_of
    }
}

/// Dashboard-oriented reporting summary with no presentation formatting.
#[derive(Debug, Clone, PartialEq)]
pub struct ReportingSummary {
    period: InclusiveDateRange,
    current_streak: u64,
    longest_streak: u64,
    average_wpm: f64,
    average_accuracy: f64,
    tests_today: u64,
    tests_in_period: u64,
    total_tests: u64,
    daily_goal_met: bool,
}

impl ReportingSummary {
    #[allow(clippy::too_many_arguments)]
    fn new(
        period: InclusiveDateRange,
        streak: StreakReport,
        average_wpm: f64,
        average_accuracy: f64,
        tests_today: u64,
        tests_in_period: u64,
        total_tests: u64,
        daily_goal_met: bool,
    ) -> Self {
        Self {
            period,
            current_streak: streak.current_streak,
            longest_streak: streak.longest_streak,
            average_wpm,
            average_accuracy,
            tests_today,
            tests_in_period,
            total_tests,
            daily_goal_met,
        }
    }

    pub const fn period(&self) -> InclusiveDateRange {
        self.period
    }

    pub const fn current_streak(&self) -> u64 {
        self.current_streak
    }

    pub const fn longest_streak(&self) -> u64 {
        self.longest_streak
    }

    pub const fn average_wpm(&self) -> f64 {
        self.average_wpm
    }

    pub const fn average_accuracy(&self) -> f64 {
        self.average_accuracy
    }

    pub const fn tests_today(&self) -> u64 {
        self.tests_today
    }

    pub const fn tests_in_period(&self) -> u64 {
        self.tests_in_period
    }

    pub const fn total_tests(&self) -> u64 {
        self.total_tests
    }

    pub const fn daily_goal_met(&self) -> bool {
        self.daily_goal_met
    }
}

/// Stable personal-best grouping dimensions without raw configuration data.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PersonalBestDimension {
    mode: ReportingMode,
    configuration_key: PersonalBestConfigurationKey,
}

impl PersonalBestDimension {
    pub fn new(mode: ReportingMode, configuration_key: PersonalBestConfigurationKey) -> Self {
        Self {
            mode,
            configuration_key,
        }
    }

    pub const fn mode(&self) -> ReportingMode {
        self.mode
    }

    pub fn configuration_key(&self) -> &PersonalBestConfigurationKey {
        &self.configuration_key
    }
}

/// A personal-best projection. Source references use session identities only.
#[derive(Debug, Clone, PartialEq)]
pub struct PersonalBestEntry {
    dimension: PersonalBestDimension,
    best_wpm: f64,
    best_wpm_session_id: Option<SessionId>,
    best_accuracy: f64,
    best_accuracy_session_id: Option<SessionId>,
    best_consistency: Option<f64>,
    best_consistency_session_id: Option<SessionId>,
    updated_at: DateTime<Utc>,
}

impl PersonalBestEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        dimension: PersonalBestDimension,
        best_wpm: f64,
        best_wpm_session_id: Option<SessionId>,
        best_accuracy: f64,
        best_accuracy_session_id: Option<SessionId>,
        best_consistency: Option<f64>,
        best_consistency_session_id: Option<SessionId>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, ReportingError> {
        validate_score(best_wpm)?;
        validate_percentage(best_accuracy)?;
        if best_consistency
            .is_some_and(|value| !value.is_finite() || !(0.0..=100.0).contains(&value))
        {
            return Err(ReportingError::CorruptReportingRecord);
        }
        Ok(Self {
            dimension,
            best_wpm,
            best_wpm_session_id,
            best_accuracy,
            best_accuracy_session_id,
            best_consistency,
            best_consistency_session_id,
            updated_at,
        })
    }

    pub fn dimension(&self) -> &PersonalBestDimension {
        &self.dimension
    }

    pub const fn best_wpm(&self) -> f64 {
        self.best_wpm
    }

    pub fn best_wpm_session_id(&self) -> Option<&SessionId> {
        self.best_wpm_session_id.as_ref()
    }

    pub const fn best_accuracy(&self) -> f64 {
        self.best_accuracy
    }

    pub fn best_accuracy_session_id(&self) -> Option<&SessionId> {
        self.best_accuracy_session_id.as_ref()
    }

    pub const fn best_consistency(&self) -> Option<f64> {
        self.best_consistency
    }

    pub fn best_consistency_session_id(&self) -> Option<&SessionId> {
        self.best_consistency_session_id.as_ref()
    }

    pub const fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

/// One bounded metric sample used by analytics policy.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReportingMetricSample {
    completed_at: DateTime<Utc>,
    wpm: f64,
    accuracy: f64,
}

/// The selection needed by the achievements policy.
///
/// Achievement values are maintained aggregates, rather than a bounded history
/// window. This keeps milestone eligibility correct for long-lived profiles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AchievementInputQuery {
    lesson_languages: Vec<ReportingLanguage>,
}

impl AchievementInputQuery {
    pub fn new(lesson_languages: Vec<ReportingLanguage>) -> Result<Self, ReportingError> {
        if lesson_languages.is_empty() || lesson_languages.len() > 16 {
            return Err(ReportingError::InvariantViolation);
        }
        Ok(Self { lesson_languages })
    }

    pub fn lesson_languages(&self) -> &[ReportingLanguage] {
        &self.lesson_languages
    }
}

/// The bounded input selection needed by the existing insights and
/// consistency policies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InsightInputQuery {
    range: InclusiveDateRange,
    history_limit: usize,
}

impl InsightInputQuery {
    pub const fn new(range: InclusiveDateRange, history_limit: usize) -> Self {
        Self {
            range,
            history_limit,
        }
    }

    pub const fn range(&self) -> InclusiveDateRange {
        self.range
    }

    pub const fn history_limit(&self) -> usize {
        self.history_limit
    }
}

impl ReportingMetricSample {
    pub fn new(
        completed_at: DateTime<Utc>,
        wpm: f64,
        accuracy: f64,
    ) -> Result<Self, ReportingError> {
        validate_score(wpm)?;
        validate_percentage(accuracy)?;
        Ok(Self {
            completed_at,
            wpm,
            accuracy,
        })
    }

    pub const fn completed_at(&self) -> DateTime<Utc> {
        self.completed_at
    }

    pub const fn wpm(&self) -> f64 {
        self.wpm
    }

    pub const fn accuracy(&self) -> f64 {
        self.accuracy
    }
}

/// Complete maintained aggregates required by the achievements calculation.
#[derive(Debug, Clone, PartialEq)]
pub struct AchievementInputs {
    total_tests: u64,
    best_wpm: f64,
    best_accuracy: f64,
    longest_streak: u64,
    lessons_completed: u64,
}

impl AchievementInputs {
    pub fn new(
        total_tests: u64,
        best_wpm: f64,
        best_accuracy: f64,
        longest_streak: u64,
        lessons_completed: u64,
    ) -> Result<Self, ReportingError> {
        validate_score(best_wpm)?;
        validate_percentage(best_accuracy)?;
        Ok(Self {
            total_tests,
            best_wpm,
            best_accuracy,
            longest_streak,
            lessons_completed,
        })
    }

    pub const fn total_tests(&self) -> u64 {
        self.total_tests
    }

    pub const fn best_wpm(&self) -> f64 {
        self.best_wpm
    }

    pub const fn best_accuracy(&self) -> f64 {
        self.best_accuracy
    }

    pub const fn longest_streak(&self) -> u64 {
        self.longest_streak
    }

    pub const fn lessons_completed(&self) -> u64 {
        self.lessons_completed
    }
}

/// Bounded analytics input for existing insights and consistency calculations.
#[derive(Debug, Clone, PartialEq)]
pub struct InsightInputs {
    daily_statistics: Vec<DailyStatisticsPoint>,
    recent_wpm: Vec<f64>,
}

impl InsightInputs {
    pub fn new(daily_statistics: Vec<DailyStatisticsPoint>, recent_wpm: Vec<f64>) -> Self {
        Self {
            daily_statistics,
            recent_wpm,
        }
    }

    pub fn daily_statistics(&self) -> &[DailyStatisticsPoint] {
        &self.daily_statistics
    }

    pub fn recent_wpm(&self) -> &[f64] {
        &self.recent_wpm
    }
}

/// Application-owned output for the three existing analytics calculations.
#[derive(Debug, Clone)]
pub struct AnalyticsSnapshot {
    achievements: Vec<Achievement>,
    insights: Vec<Insight>,
    consistency: ConsistencyReport,
}

impl AnalyticsSnapshot {
    fn new(
        achievements: Vec<Achievement>,
        insights: Vec<Insight>,
        consistency: ConsistencyReport,
    ) -> Self {
        Self {
            achievements,
            insights,
            consistency,
        }
    }

    pub fn achievements(&self) -> &[Achievement] {
        &self.achievements
    }

    pub fn insights(&self) -> &[Insight] {
        &self.insights
    }

    pub fn consistency(&self) -> &ConsistencyReport {
        &self.consistency
    }
}

/// A validated export-port request. Export serialization is intentionally not
/// part of this application use case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportQuery {
    filter: HistoryFilter,
    pagination: OffsetPagination,
}

impl ExportQuery {
    pub const fn new(filter: HistoryFilter, pagination: OffsetPagination) -> Self {
        Self { filter, pagination }
    }

    pub const fn filter(&self) -> &HistoryFilter {
        &self.filter
    }

    pub const fn pagination(&self) -> OffsetPagination {
        self.pagination
    }
}

/// Application request for a privacy-minimized history export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildTestHistoryExportRequest(ExportQuery);

impl BuildTestHistoryExportRequest {
    pub const fn new(filter: HistoryFilter, pagination: OffsetPagination) -> Self {
        Self(ExportQuery::new(filter, pagination))
    }

    pub const fn query(&self) -> &ExportQuery {
        &self.0
    }
}

impl Default for BuildTestHistoryExportRequest {
    fn default() -> Self {
        Self::new(HistoryFilter::default(), OffsetPagination::export_default())
    }
}

/// One application-approved export row, selected without typed text or replay
/// data.
#[derive(Debug, Clone, PartialEq)]
pub struct ExportRow {
    session_id: SessionId,
    completed_at: DateTime<Utc>,
    mode: ReportingMode,
    wpm: f64,
    accuracy: f64,
    duration_ms: u64,
}

impl ExportRow {
    pub fn new(
        session_id: SessionId,
        completed_at: DateTime<Utc>,
        mode: ReportingMode,
        wpm: f64,
        accuracy: f64,
        duration_ms: u64,
    ) -> Result<Self, ReportingError> {
        validate_score(wpm)?;
        validate_percentage(accuracy)?;
        Ok(Self {
            session_id,
            completed_at,
            mode,
            wpm,
            accuracy,
            duration_ms,
        })
    }

    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    pub const fn completed_at(&self) -> DateTime<Utc> {
        self.completed_at
    }

    pub const fn mode(&self) -> ReportingMode {
        self.mode
    }

    pub const fn wpm(&self) -> f64 {
        self.wpm
    }

    pub const fn accuracy(&self) -> f64 {
        self.accuracy
    }

    pub const fn duration_ms(&self) -> u64 {
        self.duration_ms
    }
}

/// Export rows supplied by a reporting port before envelope policy.
#[derive(Debug, Clone, PartialEq)]
pub struct ExportDatasetSource {
    rows: Vec<ExportRow>,
    total: u64,
}

impl ExportDatasetSource {
    pub fn new(rows: Vec<ExportRow>, total: u64) -> Self {
        Self { rows, total }
    }

    pub fn rows(&self) -> &[ExportRow] {
        &self.rows
    }

    pub const fn total(&self) -> u64 {
        self.total
    }

    pub fn into_parts(self) -> (Vec<ExportRow>, u64) {
        (self.rows, self.total)
    }
}

/// Typed export data ready for a later pure serializer or adapter.
#[derive(Debug, Clone, PartialEq)]
pub struct ExportDataset {
    rows: Vec<ExportRow>,
    offset: usize,
    limit: usize,
    total: u64,
    has_more: bool,
}

impl ExportDataset {
    fn new(
        rows: Vec<ExportRow>,
        pagination: OffsetPagination,
        total: u64,
    ) -> Result<Self, ReportingError> {
        let has_more = pagination.has_more(total, rows.len())?;
        Ok(Self {
            rows,
            offset: pagination.offset(),
            limit: pagination.limit(),
            total,
            has_more,
        })
    }

    pub fn rows(&self) -> &[ExportRow] {
        &self.rows
    }

    pub const fn offset(&self) -> usize {
        self.offset
    }

    pub const fn limit(&self) -> usize {
        self.limit
    }

    pub const fn total(&self) -> u64 {
        self.total
    }

    pub const fn has_more(&self) -> bool {
        self.has_more
    }
}

/// Lists a privacy-minimized history page using a single reporting port.
pub struct ListTestHistory<'a, P: HistoryReportingPort + ?Sized> {
    port: &'a P,
}

impl<'a, P: HistoryReportingPort + ?Sized> ListTestHistory<'a, P> {
    pub const fn new(port: &'a P) -> Self {
        Self { port }
    }

    pub fn execute(&self, request: ListTestHistoryRequest) -> Result<HistoryPage, ReportingError> {
        let query = request.query();
        let source = self.port.list_history(query)?;
        let (items, total) = source.into_parts();
        if u64::try_from(items.len()).map_err(|_| ReportingError::InvariantViolation)? > total {
            return Err(ReportingError::InvariantViolation);
        }
        validate_history_page(&items, query)?;
        HistoryPage::new(items, query.pagination(), total)
    }
}

/// Loads one detail projection by durable session identity.
pub struct GetTestDetails<'a, P: HistoryReportingPort + ?Sized> {
    port: &'a P,
}

impl<'a, P: HistoryReportingPort + ?Sized> GetTestDetails<'a, P> {
    pub const fn new(port: &'a P) -> Self {
        Self { port }
    }

    pub fn execute(&self, session_id: &SessionId) -> Result<TestDetails, ReportingError> {
        let details = self
            .port
            .find_test_details(session_id)?
            .ok_or(ReportingError::TestNotFound)?;
        if details.session_id() != session_id {
            return Err(ReportingError::InvariantViolation);
        }
        Ok(details)
    }
}

/// Loads one bounded replay page by durable session identity.
pub struct GetTestReplayPage<'a, P: HistoryReportingPort + ?Sized> {
    port: &'a P,
}

impl<'a, P: HistoryReportingPort + ?Sized> GetTestReplayPage<'a, P> {
    pub const fn new(port: &'a P) -> Self {
        Self { port }
    }

    pub fn execute(
        &self,
        session_id: SessionId,
        pagination: OffsetPagination,
    ) -> Result<ReplayPage, ReportingError> {
        let query = ReplayQuery::new(session_id.clone(), pagination);
        let source = self
            .port
            .list_replay_frames(&query)?
            .ok_or(ReportingError::ReplayUnavailable)?;
        let (frames, has_more, total) = source.into_parts();
        if frames.len() > pagination.limit() || !is_strictly_ordered_frames(&frames) {
            return Err(ReportingError::InvariantViolation);
        }
        if let Some(total) = total {
            let returned =
                u64::try_from(frames.len()).map_err(|_| ReportingError::InvariantViolation)?;
            let consumed = u64::try_from(pagination.offset())
                .map_err(|_| ReportingError::InvariantViolation)?
                .checked_add(returned)
                .ok_or(ReportingError::InvariantViolation)?;
            // `has_more` must agree with `consumed` versus `total`. The port
            // supplies `has_more` from outside this use case, so both
            // contradictions are rejected: claiming more remain when nothing
            // does, and claiming none remain when frames are still owed.
            if consumed > total
                || (has_more && consumed >= total)
                || (!has_more && consumed < total)
            {
                return Err(ReportingError::InvariantViolation);
            }
        }
        Ok(ReplayPage::new(
            session_id, pagination, frames, has_more, total,
        ))
    }
}

/// Produces the current dashboard summary using an injected UTC clock.
pub struct GetReportingSummary<'a, P: ProgressReportingPort + ?Sized, C: SessionWallClock + ?Sized>
{
    port: &'a P,
    clock: &'a C,
}

impl<'a, P: ProgressReportingPort + ?Sized, C: SessionWallClock + ?Sized>
    GetReportingSummary<'a, P, C>
{
    pub const fn new(port: &'a P, clock: &'a C) -> Self {
        Self { port, clock }
    }

    pub fn execute(&self) -> Result<ReportingSummary, ReportingError> {
        let now = self.clock.utc_now();
        let as_of = ReportingDay::from_utc(now);
        let period = RelativeReportingPeriod::DashboardWeek.range_ending_at(now)?;
        let daily_statistics = self.port.load_daily_statistics(period)?;
        validate_daily_statistics(&daily_statistics, period)?;
        let total_tests = self.port.count_tests()?;
        let streak = self.port.load_streak_report(as_of)?;
        let today = daily_statistics
            .iter()
            .find(|statistics| statistics.day() == as_of);

        Ok(ReportingSummary::new(
            period,
            streak,
            weighted_daily_average(&daily_statistics, DailyStatisticsPoint::average_wpm)?,
            weighted_daily_average(&daily_statistics, DailyStatisticsPoint::average_accuracy)?,
            today.map_or(0, DailyStatisticsPoint::total_tests),
            daily_statistics
                .iter()
                .try_fold(0_u64, |total, statistics| {
                    total
                        .checked_add(statistics.total_tests())
                        .ok_or(ReportingError::InvariantViolation)
                })?,
            total_tests,
            today.is_some_and(DailyStatisticsPoint::daily_goal_met),
        ))
    }
}

/// Lists sparse persisted daily aggregates for an explicit inclusive range.
pub struct ListDailyStatistics<'a, P: ProgressReportingPort + ?Sized> {
    port: &'a P,
}

impl<'a, P: ProgressReportingPort + ?Sized> ListDailyStatistics<'a, P> {
    pub const fn new(port: &'a P) -> Self {
        Self { port }
    }

    pub fn execute(
        &self,
        range: InclusiveDateRange,
    ) -> Result<DailyStatisticsRange, ReportingError> {
        let points = self.port.load_daily_statistics(range)?;
        validate_daily_statistics(&points, range)?;
        Ok(DailyStatisticsRange::new(range, points))
    }
}

/// Calculates deterministic current and longest streak values from recent UTC
/// activity days supplied by a reporting port.
pub struct GetStreakReport<'a, P: ProgressReportingPort + ?Sized, C: SessionWallClock + ?Sized> {
    port: &'a P,
    clock: &'a C,
}

impl<'a, P: ProgressReportingPort + ?Sized, C: SessionWallClock + ?Sized>
    GetStreakReport<'a, P, C>
{
    pub const fn new(port: &'a P, clock: &'a C) -> Self {
        Self { port, clock }
    }

    pub fn execute(&self) -> Result<StreakReport, ReportingError> {
        let now = self.clock.utc_now();
        let as_of = ReportingDay::from_utc(now);
        self.port.load_streak_report(as_of)
    }
}

/// Lists the stored personal-best projections without recomputing them.
pub struct ListPersonalBests<'a, P: PersonalBestReportingPort + ?Sized> {
    port: &'a P,
}

impl<'a, P: PersonalBestReportingPort + ?Sized> ListPersonalBests<'a, P> {
    pub const fn new(port: &'a P) -> Self {
        Self { port }
    }

    pub fn execute(
        &self,
        mode: Option<ReportingModeFilter>,
    ) -> Result<Vec<PersonalBestEntry>, ReportingError> {
        let entries = self.port.list_personal_bests(mode)?;
        if !is_stably_ordered_personal_bests(&entries) {
            return Err(ReportingError::InvariantViolation);
        }
        Ok(entries)
    }
}

/// Produces complete achievement aggregates plus bounded insights and consistency
/// calculations without reading a system clock directly.
pub struct GetAnalyticsSnapshot<
    'a,
    P: AnalyticsReportingPort + ?Sized,
    C: SessionWallClock + ?Sized,
> {
    port: &'a P,
    clock: &'a C,
}

impl<'a, P: AnalyticsReportingPort + ?Sized, C: SessionWallClock + ?Sized>
    GetAnalyticsSnapshot<'a, P, C>
{
    pub const fn new(port: &'a P, clock: &'a C) -> Self {
        Self { port, clock }
    }

    pub fn execute(&self) -> Result<AnalyticsSnapshot, ReportingError> {
        let now = self.clock.utc_now();
        let achievement_query = AchievementInputQuery::new(vec![
            ReportingLanguage::parse("en")?,
            ReportingLanguage::parse("ru")?,
        ])?;
        let achievement_inputs = self.port.load_achievement_inputs(&achievement_query)?;
        validate_achievement_inputs(&achievement_inputs)?;
        let insight_period = RelativeReportingPeriod::DashboardWeek.range_ending_at(now)?;
        let insight_inputs = self.port.load_insight_inputs(&InsightInputQuery::new(
            insight_period,
            ANALYTICS_HISTORY_LIMIT,
        ))?;
        validate_insight_inputs(&insight_inputs, insight_period)?;

        let consistency = calc_consistency(insight_inputs.recent_wpm());

        Ok(AnalyticsSnapshot::new(
            check_achievements(
                i64::try_from(achievement_inputs.total_tests())
                    .map_err(|_| ReportingError::InvariantViolation)?,
                achievement_inputs.best_wpm(),
                achievement_inputs.best_accuracy(),
                0,
                i64::try_from(achievement_inputs.longest_streak())
                    .map_err(|_| ReportingError::InvariantViolation)?,
                i64::try_from(achievement_inputs.lessons_completed())
                    .map_err(|_| ReportingError::InvariantViolation)?,
                Utc::now().to_rfc3339(),
            ),
            generate_insights(
                weighted_daily_average(
                    insight_inputs.daily_statistics(),
                    DailyStatisticsPoint::average_wpm,
                )?,
                weighted_daily_average(
                    insight_inputs.daily_statistics(),
                    DailyStatisticsPoint::average_accuracy,
                )?,
                consistency.score,
                0,
                0,
            ),
            consistency,
        ))
    }
}

/// Builds typed, privacy-minimized export data without performing I/O.
pub struct BuildTestHistoryExport<'a, P: HistoryReportingPort + ?Sized> {
    port: &'a P,
}

impl<'a, P: HistoryReportingPort + ?Sized> BuildTestHistoryExport<'a, P> {
    pub const fn new(port: &'a P) -> Self {
        Self { port }
    }

    pub fn execute(
        &self,
        request: BuildTestHistoryExportRequest,
    ) -> Result<ExportDataset, ReportingError> {
        let query = request.query();
        let source = self.port.list_export_rows(query)?;
        let (rows, total) = source.into_parts();
        if u64::try_from(rows.len()).map_err(|_| ReportingError::InvariantViolation)? > total {
            return Err(ReportingError::InvariantViolation);
        }
        validate_export_rows(&rows, query)?;
        ExportDataset::new(rows, query.pagination(), total)
    }
}

fn validate_score(value: f64) -> Result<(), ReportingError> {
    if value.is_finite() && value >= 0.0 {
        Ok(())
    } else {
        Err(ReportingError::CorruptReportingRecord)
    }
}

fn validate_percentage(value: f64) -> Result<(), ReportingError> {
    if value.is_finite() && (0.0..=100.0).contains(&value) {
        Ok(())
    } else {
        Err(ReportingError::CorruptReportingRecord)
    }
}

fn validate_history_page(
    items: &[HistoryItem],
    query: &HistoryQuery,
) -> Result<(), ReportingError> {
    if items.len() > query.pagination().limit() || !is_stably_ordered_history(items) {
        return Err(ReportingError::InvariantViolation);
    }
    if let Some(mode) = query.filter().mode() {
        if items.iter().any(|item| item.mode() != mode.mode()) {
            return Err(ReportingError::InvariantViolation);
        }
    }
    if let Some(range) = query.filter().date_range() {
        if items
            .iter()
            .any(|item| !range.contains(ReportingDay::from_utc(item.completed_at())))
        {
            return Err(ReportingError::InvariantViolation);
        }
    }
    Ok(())
}

fn is_stably_ordered_history(items: &[HistoryItem]) -> bool {
    items.windows(2).all(|window| {
        let current = &window[0];
        let next = &window[1];
        current.completed_at() > next.completed_at()
            || (current.completed_at() == next.completed_at()
                && current.session_id() > next.session_id())
    })
}

fn is_strictly_ordered_frames(frames: &[ReplayFrame]) -> bool {
    frames
        .windows(2)
        .all(|window| window[0].frame_index() < window[1].frame_index())
}

fn validate_daily_statistics(
    points: &[DailyStatisticsPoint],
    range: InclusiveDateRange,
) -> Result<(), ReportingError> {
    let mut previous = None;
    for point in points {
        if !range.contains(point.day()) || previous.is_some_and(|day| day >= point.day()) {
            return Err(ReportingError::InvariantViolation);
        }
        previous = Some(point.day());
    }
    Ok(())
}

fn weighted_daily_average(
    statistics: &[DailyStatisticsPoint],
    metric: impl Fn(&DailyStatisticsPoint) -> f64,
) -> Result<f64, ReportingError> {
    let total_tests = statistics.iter().try_fold(0_u64, |total, point| {
        total
            .checked_add(point.total_tests())
            .ok_or(ReportingError::InvariantViolation)
    })?;
    if total_tests == 0 {
        return Ok(0.0);
    }
    Ok(statistics
        .iter()
        .map(|point| metric(point) * point.total_tests() as f64)
        .sum::<f64>()
        / total_tests as f64)
}

fn is_stably_ordered_personal_bests(entries: &[PersonalBestEntry]) -> bool {
    entries.windows(2).all(|window| {
        let current = &window[0];
        let next = &window[1];
        current.updated_at() > next.updated_at()
            || (current.updated_at() == next.updated_at() && current.dimension() < next.dimension())
    })
}

fn validate_achievement_inputs(inputs: &AchievementInputs) -> Result<(), ReportingError> {
    validate_score(inputs.best_wpm())?;
    validate_percentage(inputs.best_accuracy())
}

fn validate_insight_inputs(
    inputs: &InsightInputs,
    range: InclusiveDateRange,
) -> Result<(), ReportingError> {
    if inputs.recent_wpm().len() > ANALYTICS_HISTORY_LIMIT
        || inputs
            .recent_wpm()
            .iter()
            .any(|value| !value.is_finite() || *value < 0.0)
    {
        return Err(ReportingError::CorruptReportingRecord);
    }
    validate_daily_statistics(inputs.daily_statistics(), range)
}

fn validate_export_rows(rows: &[ExportRow], query: &ExportQuery) -> Result<(), ReportingError> {
    if rows.len() > query.pagination().limit()
        || rows.windows(2).any(|window| {
            window[0].completed_at() < window[1].completed_at()
                || (window[0].completed_at() == window[1].completed_at()
                    && window[0].session_id() <= window[1].session_id())
        })
    {
        return Err(ReportingError::InvariantViolation);
    }
    if let Some(mode) = query.filter().mode() {
        if rows.iter().any(|row| row.mode() != mode.mode()) {
            return Err(ReportingError::InvariantViolation);
        }
    }
    if let Some(range) = query.filter().date_range() {
        if rows
            .iter()
            .any(|row| !range.contains(ReportingDay::from_utc(row.completed_at())))
        {
            return Err(ReportingError::InvariantViolation);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
