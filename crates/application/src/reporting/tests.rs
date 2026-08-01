use std::cell::RefCell;

use chrono::{DateTime, NaiveDate, Utc};
use racoon_domain::SessionId;

use super::*;
use crate::ports::{
    AnalyticsReportingPort, HistoryReportingPort, PersonalBestReportingPort, ProgressReportingPort,
    SessionWallClock,
};

const SESSION_A: &str = "018f0c2e-7b8d-7abc-8def-0123456789aa";

fn session_id(value: &str) -> SessionId {
    SessionId::from(value)
}

fn timestamp(value: &str) -> DateTime<Utc> {
    value.parse().expect("valid fixed timestamp")
}

fn day(year: i32, month: u32, day: u32) -> ReportingDay {
    ReportingDay::new(year, month, day).expect("valid fixed day")
}

fn language() -> ReportingLanguage {
    ReportingLanguage::parse("en").expect("fixture language")
}

fn history_item(session_id: SessionId, completed_at: DateTime<Utc>) -> HistoryItem {
    HistoryItem::new(
        session_id,
        completed_at,
        ReportingMode::Time,
        Some(language()),
        1_000,
        5,
        60.0,
        98.0,
        true,
        true,
    )
    .expect("valid history fixture")
}

fn details(session_id: SessionId) -> TestDetails {
    TestDetails::new(
        session_id,
        timestamp("2026-07-16T12:00:00Z"),
        ReportingMode::Lesson,
        None,
        None,
        5,
        1_000,
        60.0,
        62.0,
        98.0,
        96.0,
        None,
        5,
        0,
        0,
        true,
        false,
    )
    .expect("valid detail fixture")
}

fn daily(day: ReportingDay, total_tests: u64, wpm: f64, accuracy: f64) -> DailyStatisticsPoint {
    DailyStatisticsPoint::new(
        day,
        total_tests,
        total_tests * 1_000,
        total_tests * 5,
        wpm,
        wpm,
        accuracy,
        0,
        false,
    )
    .expect("valid daily fixture")
}

fn frame(index: u64) -> ReplayFrame {
    ReplayFrame::new(index, index * 10, index, 'a', Some('a'), true)
}

fn metric(completed_at: &str, wpm: f64, accuracy: f64) -> ReportingMetricSample {
    ReportingMetricSample::new(timestamp(completed_at), wpm, accuracy).expect("valid metric")
}

fn personal_best(updated_at: &str) -> PersonalBestEntry {
    PersonalBestEntry::new(
        PersonalBestDimension::new(
            ReportingMode::Time,
            PersonalBestConfigurationKey::parse("0123456789abcdef").expect("valid key"),
        ),
        60.0,
        Some(session_id(SESSION_A)),
        98.0,
        Some(session_id(SESSION_A)),
        Some(90.0),
        Some(session_id(SESSION_A)),
        timestamp(updated_at),
    )
    .expect("valid personal best")
}

struct FixedClock(DateTime<Utc>);

impl SessionWallClock for FixedClock {
    fn utc_now(&self) -> DateTime<Utc> {
        self.0
    }
}

struct FakeHistoryPort {
    history: Result<HistoryPageSource, ReportingError>,
    details: Result<Option<TestDetails>, ReportingError>,
    replay: Result<Option<ReplayPageSource>, ReportingError>,
    export: Result<ExportDatasetSource, ReportingError>,
    history_queries: RefCell<Vec<HistoryQuery>>,
    replay_queries: RefCell<Vec<ReplayQuery>>,
    export_queries: RefCell<Vec<ExportQuery>>,
}

impl FakeHistoryPort {
    fn empty() -> Self {
        Self {
            history: Ok(HistoryPageSource::new(vec![], 0)),
            details: Ok(None),
            replay: Ok(None),
            export: Ok(ExportDatasetSource::new(vec![], 0)),
            history_queries: RefCell::new(vec![]),
            replay_queries: RefCell::new(vec![]),
            export_queries: RefCell::new(vec![]),
        }
    }
}

impl HistoryReportingPort for FakeHistoryPort {
    fn list_history(&self, query: &HistoryQuery) -> Result<HistoryPageSource, ReportingError> {
        self.history_queries.borrow_mut().push(query.clone());
        self.history.clone()
    }

    fn find_test_details(&self, _: &SessionId) -> Result<Option<TestDetails>, ReportingError> {
        self.details.clone()
    }

    fn list_replay_frames(
        &self,
        query: &ReplayQuery,
    ) -> Result<Option<ReplayPageSource>, ReportingError> {
        self.replay_queries.borrow_mut().push(query.clone());
        self.replay.clone()
    }

    fn list_export_rows(&self, query: &ExportQuery) -> Result<ExportDatasetSource, ReportingError> {
        self.export_queries.borrow_mut().push(query.clone());
        self.export.clone()
    }
}

struct FakeProgressPort {
    total: Result<u64, ReportingError>,
    daily: Result<Vec<DailyStatisticsPoint>, ReportingError>,
    activity_days: Result<Vec<ReportingDay>, ReportingError>,
    daily_ranges: RefCell<Vec<InclusiveDateRange>>,
    activity_limits: RefCell<Vec<usize>>,
}

impl FakeProgressPort {
    fn empty() -> Self {
        Self {
            total: Ok(0),
            daily: Ok(vec![]),
            activity_days: Ok(vec![]),
            daily_ranges: RefCell::new(vec![]),
            activity_limits: RefCell::new(vec![]),
        }
    }
}

impl ProgressReportingPort for FakeProgressPort {
    fn count_tests(&self) -> Result<u64, ReportingError> {
        self.total
    }

    fn load_daily_statistics(
        &self,
        range: InclusiveDateRange,
    ) -> Result<Vec<DailyStatisticsPoint>, ReportingError> {
        self.daily_ranges.borrow_mut().push(range);
        self.daily.clone()
    }

    fn load_recent_activity_days(
        &self,
        history_limit: usize,
    ) -> Result<Vec<ReportingDay>, ReportingError> {
        self.activity_limits.borrow_mut().push(history_limit);
        self.activity_days.clone()
    }
}

struct FakePersonalBestPort {
    entries: Result<Vec<PersonalBestEntry>, ReportingError>,
    modes: RefCell<Vec<Option<ReportingModeFilter>>>,
}

impl PersonalBestReportingPort for FakePersonalBestPort {
    fn list_personal_bests(
        &self,
        mode: Option<ReportingModeFilter>,
    ) -> Result<Vec<PersonalBestEntry>, ReportingError> {
        self.modes.borrow_mut().push(mode);
        self.entries.clone()
    }
}

struct FakeAnalyticsPort {
    achievement: Result<AchievementInputs, ReportingError>,
    insight: Result<InsightInputs, ReportingError>,
    achievement_queries: RefCell<Vec<AchievementInputQuery>>,
    insight_queries: RefCell<Vec<InsightInputQuery>>,
}

impl AnalyticsReportingPort for FakeAnalyticsPort {
    fn load_achievement_inputs(
        &self,
        query: &AchievementInputQuery,
    ) -> Result<AchievementInputs, ReportingError> {
        self.achievement_queries.borrow_mut().push(query.clone());
        self.achievement.clone()
    }

    fn load_insight_inputs(
        &self,
        query: &InsightInputQuery,
    ) -> Result<InsightInputs, ReportingError> {
        self.insight_queries.borrow_mut().push(*query);
        self.insight.clone()
    }
}

#[test]
fn reporting_day_and_closed_range_use_explicit_utc_boundaries() {
    let start = day(2026, 7, 10);
    let end = day(2026, 7, 12);
    let range = InclusiveDateRange::new(start, end).expect("ordered range");

    assert!(range.contains(day(2026, 7, 11)));
    assert_eq!(range.start().to_string(), "2026-07-10");
    assert_eq!(range.end().to_string(), "2026-07-12");
    let (from, to) = range.half_open_utc().expect("safe bounds");
    assert_eq!(from, timestamp("2026-07-10T00:00:00Z"));
    assert_eq!(to, timestamp("2026-07-13T00:00:00Z"));
}

#[test]
fn reporting_ranges_reject_inversion_and_next_day_overflow() {
    assert_eq!(
        InclusiveDateRange::new(day(2026, 7, 2), day(2026, 7, 1)),
        Err(ReportingError::InvalidDateRange)
    );
    assert_eq!(
        ReportingDay(NaiveDate::MAX).next_day(),
        Err(ReportingError::DateArithmeticOverflow)
    );
    assert_eq!(
        "unsupported".parse::<ReportingModeFilter>(),
        Err(ReportingError::UnsupportedMode)
    );
}

#[test]
fn relative_periods_preserve_existing_inclusive_lookback_behavior() {
    let now = timestamp("2026-07-16T23:59:59Z");
    let seven = RelativeReportingPeriod::ProgressSevenDays
        .range_ending_at(now)
        .expect("seven-day range");
    let thirty = RelativeReportingPeriod::ProgressThirtyDays
        .range_ending_at(now)
        .expect("thirty-day range");
    let ninety = RelativeReportingPeriod::ProgressNinetyDays
        .range_ending_at(now)
        .expect("ninety-day range");

    assert_eq!(seven.start().to_string(), "2026-07-09");
    assert_eq!(seven.end().to_string(), "2026-07-16");
    assert_eq!(thirty.start().to_string(), "2026-06-16");
    assert_eq!(ninety.start().to_string(), "2026-04-17");
}

#[test]
fn pagination_is_bounded_and_calculates_safe_page_metadata() {
    assert_eq!(
        OffsetPagination::new(0, 0),
        Err(ReportingError::InvalidPagination)
    );
    assert!(OffsetPagination::new(1, 0).is_ok());
    assert!(OffsetPagination::new(MAX_REPORTING_PAGE_LIMIT, 0).is_ok());
    assert_eq!(
        OffsetPagination::new(MAX_REPORTING_PAGE_LIMIT + 1, 0),
        Err(ReportingError::InvalidPagination)
    );
    assert!(OffsetPagination::new(1, MAX_REPORTING_PAGE_OFFSET).is_ok());
    assert_eq!(
        OffsetPagination::new(1, MAX_REPORTING_PAGE_OFFSET + 1),
        Err(ReportingError::InvalidPagination)
    );

    let page = OffsetPagination::new(10, 10).expect("valid page");
    assert!(page.has_more(25, 10).expect("page metadata"));
    assert_eq!(page.next_offset(25, 10).expect("next page"), Some(20));
    assert!(!page.has_more(20, 10).expect("final page"));
    assert_eq!(page.next_offset(20, 10).expect("no next page"), None);
    assert_eq!(
        page.has_more(25, 11),
        Err(ReportingError::InvariantViolation)
    );
}

#[test]
fn history_forwards_validated_mode_and_date_filters_and_returns_stable_envelope() {
    let item = history_item(session_id(SESSION_A), timestamp("2026-07-16T12:00:00Z"));
    let mut port = FakeHistoryPort::empty();
    port.history = Ok(HistoryPageSource::new(vec![item], 2));
    let filter = HistoryFilter::new(
        Some("time".parse().expect("supported mode")),
        Some(InclusiveDateRange::new(day(2026, 7, 10), day(2026, 7, 16)).expect("range")),
    );
    let request = ListTestHistoryRequest::new(filter.clone(), OffsetPagination::new(1, 0).unwrap());

    let page = ListTestHistory::new(&port)
        .execute(request)
        .expect("history page");
    assert_eq!(page.items().len(), 1);
    assert_eq!(page.total(), 2);
    assert!(page.has_more());
    let queries = port.history_queries.borrow();
    assert_eq!(queries.len(), 1);
    assert_eq!(queries[0].filter(), &filter);
    assert_eq!(queries[0].pagination().limit(), 1);
}

#[test]
fn history_empty_and_storage_errors_are_typed() {
    let port = FakeHistoryPort::empty();
    let page = ListTestHistory::new(&port)
        .execute(ListTestHistoryRequest::default())
        .expect("empty history");
    assert!(page.items().is_empty());
    assert_eq!(page.total(), 0);
    assert!(!page.has_more());

    let mut unavailable = FakeHistoryPort::empty();
    unavailable.history = Err(ReportingError::StorageUnavailable);
    assert_eq!(
        ListTestHistory::new(&unavailable).execute(ListTestHistoryRequest::default()),
        Err(ReportingError::StorageUnavailable)
    );
}

#[test]
fn history_rejects_port_results_that_break_filter_or_stable_order_contracts() {
    let filter = HistoryFilter::new(Some(ReportingModeFilter::new(ReportingMode::Lesson)), None);
    let mut wrong_mode = FakeHistoryPort::empty();
    wrong_mode.history = Ok(HistoryPageSource::new(
        vec![history_item(
            session_id(SESSION_A),
            timestamp("2026-07-16T12:00:00Z"),
        )],
        1,
    ));
    assert_eq!(
        ListTestHistory::new(&wrong_mode).execute(ListTestHistoryRequest::new(
            filter,
            OffsetPagination::new(10, 0).unwrap(),
        )),
        Err(ReportingError::InvariantViolation)
    );

    let mut unordered = FakeHistoryPort::empty();
    unordered.history = Ok(HistoryPageSource::new(
        vec![
            history_item(session_id(SESSION_A), timestamp("2026-07-15T12:00:00Z")),
            history_item(
                session_id("018f0c2e-7b8d-7abc-8def-0123456789ac"),
                timestamp("2026-07-16T12:00:00Z"),
            ),
        ],
        2,
    ));
    assert_eq!(
        ListTestHistory::new(&unordered).execute(ListTestHistoryRequest::default()),
        Err(ReportingError::InvariantViolation)
    );
}

#[test]
fn history_projection_does_not_include_custom_text_or_raw_mode_configuration() {
    let item = history_item(session_id(SESSION_A), timestamp("2026-07-16T12:00:00Z"));
    let diagnostic = format!("{item:?}");
    assert!(!diagnostic.contains("typed-secret-content"));
    assert!(!diagnostic.contains("mode_config"));
    assert_eq!(item.session_id().as_str(), SESSION_A);
}

#[test]
fn details_distinguish_found_not_found_optional_absence_and_corruption() {
    let mut port = FakeHistoryPort::empty();
    port.details = Ok(Some(details(session_id(SESSION_A))));
    let result = GetTestDetails::new(&port)
        .execute(&session_id(SESSION_A))
        .expect("found details");
    assert!(result.language().is_none());
    assert!(result.lesson_id().is_none());
    assert!(result.consistency().is_none());
    assert_eq!(result.session_id().as_str(), SESSION_A);

    let absent = FakeHistoryPort::empty();
    assert_eq!(
        GetTestDetails::new(&absent).execute(&session_id(SESSION_A)),
        Err(ReportingError::TestNotFound)
    );

    let mut corrupt = FakeHistoryPort::empty();
    corrupt.details = Err(ReportingError::CorruptReportingRecord);
    assert_eq!(
        GetTestDetails::new(&corrupt).execute(&session_id(SESSION_A)),
        Err(ReportingError::CorruptReportingRecord)
    );

    let mut mismatched = FakeHistoryPort::empty();
    mismatched.details = Ok(Some(details(session_id("different-session"))));
    assert_eq!(
        GetTestDetails::new(&mismatched).execute(&session_id(SESSION_A)),
        Err(ReportingError::InvariantViolation)
    );
}

#[test]
fn replay_pages_are_bounded_but_total_replay_length_is_not() {
    let frames = (0..MAX_REPORTING_PAGE_LIMIT as u64)
        .map(frame)
        .collect::<Vec<_>>();
    let mut port = FakeHistoryPort::empty();
    port.replay = Ok(Some(ReplayPageSource::new(
        frames,
        true,
        Some((MAX_REPORTING_PAGE_LIMIT + 1) as u64),
    )));
    let pagination = OffsetPagination::new(MAX_REPORTING_PAGE_LIMIT, 0).unwrap();

    let page = GetTestReplayPage::new(&port)
        .execute(session_id(SESSION_A), pagination)
        .expect("first replay page");
    assert_eq!(page.returned(), MAX_REPORTING_PAGE_LIMIT);
    assert!(page.has_more());
    assert_eq!(page.total(), Some(1_001));
    assert_eq!(
        pagination
            .next_offset(page.total().unwrap(), page.returned())
            .unwrap(),
        Some(MAX_REPORTING_PAGE_LIMIT)
    );
}

#[test]
fn replay_distinguishes_optional_absence_from_empty_page_and_rejects_unordered_frames() {
    let absent = FakeHistoryPort::empty();
    assert_eq!(
        GetTestReplayPage::new(&absent)
            .execute(session_id(SESSION_A), OffsetPagination::new(10, 0).unwrap()),
        Err(ReportingError::ReplayUnavailable)
    );

    let mut empty = FakeHistoryPort::empty();
    empty.replay = Ok(Some(ReplayPageSource::new(vec![], false, Some(0))));
    assert!(GetTestReplayPage::new(&empty)
        .execute(session_id(SESSION_A), OffsetPagination::new(10, 0).unwrap())
        .expect("empty replay page")
        .frames()
        .is_empty());

    let mut unordered = FakeHistoryPort::empty();
    unordered.replay = Ok(Some(ReplayPageSource::new(
        vec![frame(1), frame(0)],
        false,
        Some(2),
    )));
    assert_eq!(
        GetTestReplayPage::new(&unordered)
            .execute(session_id(SESSION_A), OffsetPagination::new(10, 0).unwrap()),
        Err(ReportingError::InvariantViolation)
    );
}

#[test]
fn reporting_summary_uses_fixed_utc_clock_weighted_daily_averages_and_streak_policy() {
    let port = FakeProgressPort {
        total: Ok(4),
        daily: Ok(vec![
            daily(day(2026, 7, 15), 1, 10.0, 90.0),
            DailyStatisticsPoint::new(day(2026, 7, 16), 3, 3_000, 15, 30.0, 30.0, 96.0, 0, true)
                .unwrap(),
        ]),
        activity_days: Ok(vec![day(2026, 7, 14), day(2026, 7, 15), day(2026, 7, 16)]),
        daily_ranges: RefCell::new(vec![]),
        activity_limits: RefCell::new(vec![]),
    };
    let clock = FixedClock(timestamp("2026-07-16T12:00:00Z"));

    let summary = GetReportingSummary::new(&port, &clock)
        .execute()
        .expect("summary");
    assert_eq!(summary.period().start().to_string(), "2026-07-09");
    assert_eq!(summary.tests_today(), 3);
    assert_eq!(summary.tests_in_period(), 4);
    assert_eq!(summary.total_tests(), 4);
    assert_eq!(summary.current_streak(), 3);
    assert_eq!(summary.longest_streak(), 3);
    assert!((summary.average_wpm() - 25.0).abs() < f64::EPSILON);
    assert!((summary.average_accuracy() - 94.5).abs() < f64::EPSILON);
    assert!(summary.daily_goal_met());
    assert_eq!(
        port.activity_limits.borrow().as_slice(),
        &[DASHBOARD_ACTIVITY_HISTORY_LIMIT]
    );
}

#[test]
fn summary_and_streak_handle_empty_and_missed_activity_deterministically() {
    let empty = FakeProgressPort::empty();
    let clock = FixedClock(timestamp("2026-07-16T12:00:00Z"));
    let summary = GetReportingSummary::new(&empty, &clock).execute().unwrap();
    assert_eq!(summary.current_streak(), 0);
    assert_eq!(summary.longest_streak(), 0);
    assert_eq!(summary.average_wpm(), 0.0);

    let missed = FakeProgressPort {
        activity_days: Ok(vec![day(2026, 7, 10), day(2026, 7, 11)]),
        ..FakeProgressPort::empty()
    };
    let streak = GetStreakReport::new(&missed, &clock).execute().unwrap();
    assert_eq!(streak.current_streak(), 0);
    assert_eq!(streak.longest_streak(), 2);
    assert_eq!(streak.as_of().to_string(), "2026-07-16");
}

#[test]
fn daily_statistics_stay_sparse_validate_range_and_propagate_port_errors() {
    let range = InclusiveDateRange::new(day(2026, 7, 10), day(2026, 7, 16)).unwrap();
    let port = FakeProgressPort {
        daily: Ok(vec![daily(day(2026, 7, 12), 1, 50.0, 95.0)]),
        ..FakeProgressPort::empty()
    };
    let result = ListDailyStatistics::new(&port).execute(range).unwrap();
    assert_eq!(result.points().len(), 1);
    assert_eq!(result.points()[0].day().to_string(), "2026-07-12");

    let unavailable = FakeProgressPort {
        daily: Err(ReportingError::RetryableStorage),
        ..FakeProgressPort::empty()
    };
    assert_eq!(
        ListDailyStatistics::new(&unavailable).execute(range),
        Err(ReportingError::RetryableStorage)
    );

    let empty = FakeProgressPort::empty();
    assert!(ListDailyStatistics::new(&empty)
        .execute(range)
        .expect("empty sparse range")
        .points()
        .is_empty());
}

#[test]
fn personal_bests_preserve_safe_dimensions_and_forward_mode_filter() {
    let entry = personal_best("2026-07-16T12:00:00Z");
    let port = FakePersonalBestPort {
        entries: Ok(vec![entry]),
        modes: RefCell::new(vec![]),
    };
    let filter = Some("time".parse().unwrap());
    let entries = ListPersonalBests::new(&port).execute(filter).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].dimension().mode(), ReportingMode::Time);
    assert_eq!(
        entries[0].best_wpm_session_id().unwrap().as_str(),
        SESSION_A
    );
    assert_eq!(port.modes.borrow().as_slice(), &[filter]);
}

#[test]
fn personal_bests_allow_empty_results_and_reject_unstable_dimension_ordering() {
    let empty = FakePersonalBestPort {
        entries: Ok(vec![]),
        modes: RefCell::new(vec![]),
    };
    assert!(ListPersonalBests::new(&empty)
        .execute(None)
        .unwrap()
        .is_empty());

    let newer = personal_best("2026-07-16T12:00:00Z");
    let older = personal_best("2026-07-15T12:00:00Z");
    let unordered = FakePersonalBestPort {
        entries: Ok(vec![older, newer]),
        modes: RefCell::new(vec![]),
    };
    assert_eq!(
        ListPersonalBests::new(&unordered).execute(None),
        Err(ReportingError::InvariantViolation)
    );
}

#[test]
fn analytics_preserve_existing_caps_and_are_deterministic_with_a_fixed_clock() {
    let port = FakeAnalyticsPort {
        achievement: Ok(AchievementInputs::new(
            1,
            vec![metric("2026-07-16T12:00:00Z", 60.0, 98.0)],
            vec![day(2026, 7, 16)],
            0,
        )),
        insight: Ok(InsightInputs::new(
            vec![daily(day(2026, 7, 16), 1, 60.0, 98.0)],
            vec![60.0],
        )),
        achievement_queries: RefCell::new(vec![]),
        insight_queries: RefCell::new(vec![]),
    };
    let clock = FixedClock(timestamp("2026-07-16T12:00:00Z"));

    let snapshot = GetAnalyticsSnapshot::new(&port, &clock).execute().unwrap();
    assert_eq!(snapshot.consistency().samples, 1);
    assert!(snapshot
        .achievements()
        .iter()
        .any(|achievement| achievement.id == "first_test" && achievement.unlocked));
    assert!(!snapshot.insights().is_empty());
    assert_eq!(
        port.achievement_queries.borrow()[0].history_limit(),
        ACHIEVEMENT_HISTORY_LIMIT
    );
    assert_eq!(
        port.insight_queries.borrow()[0].history_limit(),
        ANALYTICS_HISTORY_LIMIT
    );
    assert_eq!(
        port.insight_queries.borrow()[0].range().start().to_string(),
        "2026-07-09"
    );
}

#[test]
fn analytics_rejects_unbounded_or_corrupt_inputs() {
    let too_many = (0..ACHIEVEMENT_HISTORY_LIMIT + 1)
        .map(|index| metric("2026-07-16T12:00:00Z", index as f64, 90.0))
        .collect();
    let port = FakeAnalyticsPort {
        achievement: Ok(AchievementInputs::new(1, too_many, vec![], 0)),
        insight: Ok(InsightInputs::new(vec![], vec![])),
        achievement_queries: RefCell::new(vec![]),
        insight_queries: RefCell::new(vec![]),
    };
    let clock = FixedClock(timestamp("2026-07-16T12:00:00Z"));
    assert!(matches!(
        GetAnalyticsSnapshot::new(&port, &clock).execute(),
        Err(ReportingError::InvariantViolation)
    ));
}

#[test]
fn analytics_empty_inputs_keep_existing_empty_data_semantics() {
    let port = FakeAnalyticsPort {
        achievement: Ok(AchievementInputs::new(0, vec![], vec![], 0)),
        insight: Ok(InsightInputs::new(vec![], vec![])),
        achievement_queries: RefCell::new(vec![]),
        insight_queries: RefCell::new(vec![]),
    };
    let clock = FixedClock(timestamp("2026-07-16T12:00:00Z"));
    let snapshot = GetAnalyticsSnapshot::new(&port, &clock).execute().unwrap();

    assert_eq!(snapshot.consistency().samples, 0);
    assert!(snapshot
        .achievements()
        .iter()
        .all(|achievement| !achievement.unlocked));
    assert!(!snapshot.insights().is_empty());
}

#[test]
fn export_forwards_filters_and_pagination_without_serializing_or_exposing_text() {
    let row = ExportRow::new(
        session_id(SESSION_A),
        timestamp("2026-07-16T12:00:00Z"),
        ReportingMode::Custom,
        60.0,
        98.0,
        1_000,
    )
    .unwrap();
    let mut port = FakeHistoryPort::empty();
    port.export = Ok(ExportDatasetSource::new(vec![row], 2));
    let filter = HistoryFilter::new(
        Some(ReportingModeFilter::new(ReportingMode::Custom)),
        Some(InclusiveDateRange::single(day(2026, 7, 16))),
    );
    let request =
        BuildTestHistoryExportRequest::new(filter.clone(), OffsetPagination::new(1, 0).unwrap());

    let dataset = BuildTestHistoryExport::new(&port).execute(request).unwrap();
    assert_eq!(dataset.rows().len(), 1);
    assert_eq!(dataset.total(), 2);
    assert!(dataset.has_more());
    assert_eq!(port.export_queries.borrow()[0].filter(), &filter);
    let diagnostic = format!("{dataset:?}");
    assert!(!diagnostic.contains("typed-secret-content"));
}

#[test]
fn export_rejects_unstable_ordering_and_excess_rows() {
    let mut port = FakeHistoryPort::empty();
    let later = ExportRow::new(
        session_id(SESSION_A),
        timestamp("2026-07-16T12:00:00Z"),
        ReportingMode::Time,
        60.0,
        98.0,
        1_000,
    )
    .unwrap();
    let earlier = ExportRow::new(
        session_id("018f0c2e-7b8d-7abc-8def-0123456789ac"),
        timestamp("2026-07-15T12:00:00Z"),
        ReportingMode::Time,
        50.0,
        95.0,
        1_000,
    )
    .unwrap();
    port.export = Ok(ExportDatasetSource::new(vec![earlier, later], 2));
    assert_eq!(
        BuildTestHistoryExport::new(&port).execute(BuildTestHistoryExportRequest::new(
            HistoryFilter::default(),
            OffsetPagination::new(10, 0).unwrap(),
        )),
        Err(ReportingError::InvariantViolation)
    );
}

#[test]
fn reporting_error_display_is_bounded_and_never_carries_storage_text() {
    assert_eq!(
        ReportingError::RetryableStorage.to_string(),
        "reporting storage is temporarily unavailable"
    );
    assert_eq!(
        ReportingError::CorruptReportingRecord.to_string(),
        "stored reporting record is corrupt"
    );
}
