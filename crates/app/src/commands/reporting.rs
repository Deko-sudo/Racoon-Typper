//! Tauri adapters for persisted history, dashboards, analytics, exports, and replay.

use chrono::{Duration, Utc};
use racoon_core::analytics::{Achievement, Insight};
use racoon_core::consistency::ConsistencyReport;
use racoon_data::repository::{
    DailyStats, DailyStatsRepository, LessonRepository, PersonalBestsRepository, ReplayRepository,
    SqliteDailyStatsRepository, SqliteLessonRepository, SqlitePersonalBestsRepository,
    SqliteReplayRepository, SqliteTestRepository, TestRepository,
};
use racoon_domain::{PersonalBest, TestSummary};
use tauri::State;

use crate::commands::contracts::{DashboardStatsResponse, ProgressPoint, StatsHistoryResponse};
use crate::commands::with_db;
use crate::error::AppError;
use crate::state::AppState;
use crate::validation::{
    validate_export_format, validate_mode_filter, validate_page_limit, validate_page_offset,
    validate_positive_id, validate_progress_days, MAX_PAGE_LIMIT,
};

#[tauri::command]
pub(crate) fn get_stats_history(
    state: State<'_, AppState>,
    limit: Option<usize>,
    offset: Option<usize>,
    mode_filter: Option<String>,
) -> Result<StatsHistoryResponse, AppError> {
    let limit = validate_page_limit(limit.unwrap_or(50))?;
    let offset = validate_page_offset(offset.unwrap_or(0))?;
    let mode_filter = mode_filter
        .as_deref()
        .map(validate_mode_filter)
        .transpose()?;

    with_db(&state, |conn| {
        let repository = SqliteTestRepository::new(conn);
        let tests = repository.get_history(limit, offset, mode_filter)?;
        let total = repository.get_count(mode_filter)?;
        Ok(StatsHistoryResponse { tests, total })
    })
}

#[tauri::command]
pub(crate) fn get_personal_bests(
    state: State<'_, AppState>,
    mode_filter: Option<String>,
) -> Result<Vec<PersonalBest>, AppError> {
    let mode_filter = mode_filter
        .as_deref()
        .map(validate_mode_filter)
        .transpose()?;
    with_db(&state, |conn| {
        SqlitePersonalBestsRepository::new(conn).get_bests(mode_filter)
    })
}

#[tauri::command]
pub(crate) fn get_dashboard_stats(
    state: State<'_, AppState>,
) -> Result<DashboardStatsResponse, AppError> {
    let (week_ago, today) = utc_date_range(7);
    with_db(&state, |conn| {
        let test_repository = SqliteTestRepository::new(conn);
        let daily_repository = SqliteDailyStatsRepository::new(conn);
        let total_tests = test_repository.get_count(None)?;
        let today_stats = daily_repository.get_day(&today)?;
        let tests_today = today_stats.as_ref().map_or(0, |stats| stats.total_tests);
        let daily_goal_met = today_stats
            .as_ref()
            .is_some_and(|stats| stats.daily_goal_met);
        let week_stats = daily_repository.get_range(&week_ago, &today)?;
        let history = test_repository.get_history(MAX_PAGE_LIMIT, 0, None)?;
        let (current_streak, longest_streak) =
            racoon_core::StreakEngine::streak_from_dates(&test_dates(&history));

        Ok(DashboardStatsResponse {
            current_streak,
            longest_streak,
            avg_wpm: weighted_daily_average(&week_stats, |stats| stats.avg_wpm),
            avg_accuracy: weighted_daily_average(&week_stats, |stats| stats.avg_accuracy),
            tests_today,
            tests_this_week: week_stats.iter().map(|stats| stats.total_tests).sum(),
            total_tests,
            daily_goal_met,
        })
    })
}

#[tauri::command]
pub(crate) fn get_progress_history(
    state: State<'_, AppState>,
    days: Option<u32>,
) -> Result<Vec<ProgressPoint>, AppError> {
    let days = validate_progress_days(days.unwrap_or(30))?;
    let (from, to) = utc_date_range(i64::from(days));
    with_db(&state, |conn| {
        Ok(SqliteDailyStatsRepository::new(conn)
            .get_range(&from, &to)?
            .iter()
            .map(|stats| ProgressPoint {
                date: stats.date.clone(),
                wpm: stats.avg_wpm,
                accuracy: stats.avg_accuracy,
                tests: stats.total_tests,
            })
            .collect())
    })
}

/// The nested collection preserves the pre-existing `[[achievement...]]` wire
/// shape while making its element type explicit. Flattening it is Phase 3 IPC
/// contract work.
#[tauri::command]
pub(crate) fn get_achievements(
    state: State<'_, AppState>,
) -> Result<Vec<Vec<Achievement>>, AppError> {
    with_db(&state, |conn| {
        let test_repository = SqliteTestRepository::new(conn);
        let lesson_repository = SqliteLessonRepository::new(conn);
        let total_tests = test_repository.get_count(None)?;
        let history = test_repository.get_history(500, 0, None)?;
        let best_wpm = history.iter().map(|test| test.wpm).fold(0.0_f64, f64::max);
        let best_accuracy = history
            .iter()
            .map(|test| test.accuracy)
            .fold(0.0_f64, f64::max);
        let (_, longest_streak) =
            racoon_core::StreakEngine::streak_from_dates(&test_dates(&history));
        let lessons_completed = ["en", "ru"]
            .into_iter()
            .map(|language| {
                lesson_repository.get_progress(language).map(|progress| {
                    progress
                        .iter()
                        .filter(|lesson| lesson.status == "completed")
                        .count() as i64
                })
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .sum();

        Ok(vec![racoon_core::analytics::check_achievements(
            total_tests,
            best_wpm,
            best_accuracy,
            0,
            longest_streak,
            lessons_completed,
        )])
    })
}

/// The nested collection preserves the pre-existing `[[insight...]]` wire
/// shape while making its element type explicit. Flattening it is Phase 3 IPC
/// contract work.
#[tauri::command]
pub(crate) fn get_insights(state: State<'_, AppState>) -> Result<Vec<Vec<Insight>>, AppError> {
    let (week_ago, today) = utc_date_range(7);
    with_db(&state, |conn| {
        let daily_repository = SqliteDailyStatsRepository::new(conn);
        let test_repository = SqliteTestRepository::new(conn);
        let week_stats = daily_repository.get_range(&week_ago, &today)?;
        let history = test_repository.get_history(100, 0, None)?;
        let wpm_samples: Vec<f64> = history.iter().map(|test| test.wpm).collect();
        let consistency = racoon_core::consistency::calc_consistency(&wpm_samples);

        Ok(vec![racoon_core::analytics::generate_insights(
            weighted_daily_average(&week_stats, |stats| stats.avg_wpm),
            weighted_daily_average(&week_stats, |stats| stats.avg_accuracy),
            consistency.score,
            0,
            0,
        )])
    })
}

#[tauri::command]
pub(crate) fn get_consistency(state: State<'_, AppState>) -> Result<ConsistencyReport, AppError> {
    with_db(&state, |conn| {
        let history = SqliteTestRepository::new(conn).get_history(100, 0, None)?;
        let samples: Vec<f64> = history.iter().map(|test| test.wpm).collect();
        Ok(racoon_core::consistency::calc_consistency(&samples))
    })
}

#[tauri::command]
pub(crate) fn export_data(state: State<'_, AppState>, format: String) -> Result<String, AppError> {
    validate_export_format(&format)?;
    let history = with_db(&state, |conn| {
        SqliteTestRepository::new(conn).get_history(MAX_PAGE_LIMIT, 0, None)
    })?;

    match format.as_str() {
        "json" => {
            let data = serde_json::json!({
                "tests": history.iter().map(|test| serde_json::json!({
                    "session_id": test.session_id.to_string(),
                    "date": test.created_at,
                    "mode": test.mode_type,
                    "wpm": test.wpm,
                    "accuracy": test.accuracy,
                    "duration_ms": test.duration_ms,
                })).collect::<Vec<_>>(),
            });
            Ok(racoon_core::analytics::export_json(&data))
        }
        "csv" => {
            let mut rows = vec![vec![
                "Session_id".to_string(),
                "Date".to_string(),
                "Mode".to_string(),
                "WPM".to_string(),
                "Accuracy".to_string(),
                "Duration_ms".to_string(),
            ]];
            for test in &history {
                rows.push(vec![
                    test.session_id.to_string(),
                    test.created_at.clone(),
                    test.mode_type.clone(),
                    format!("{:.1}", test.wpm),
                    format!("{:.1}", test.accuracy),
                    test.duration_ms.to_string(),
                ]);
            }
            Ok(racoon_core::analytics::export_csv(&rows))
        }
        _ => Err(AppError::InvalidConfig(format!(
            "Unknown export format: {format}"
        ))),
    }
}

#[tauri::command]
pub(crate) fn get_replay(
    state: State<'_, AppState>,
    test_id: i64,
) -> Result<Vec<racoon_data::repository::ReplayFrame>, AppError> {
    validate_positive_id(test_id, "test")?;
    with_db(&state, |conn| {
        SqliteReplayRepository::new(conn).load_replay(test_id)
    })
}

fn utc_date_range(days: i64) -> (String, String) {
    let now = Utc::now();
    (
        (now - Duration::days(days)).format("%Y-%m-%d").to_string(),
        now.format("%Y-%m-%d").to_string(),
    )
}

fn test_dates(history: &[TestSummary]) -> Vec<String> {
    history
        .iter()
        .filter_map(|test| {
            test.created_at
                .split_once('T')
                .map(|(date, _)| date.to_string())
        })
        .filter(|date| !date.is_empty())
        .collect()
}

fn weighted_daily_average(stats: &[DailyStats], metric: impl Fn(&DailyStats) -> f64) -> f64 {
    let total_tests: i64 = stats.iter().map(|stat| stat.total_tests).sum();
    if total_tests <= 0 {
        return 0.0;
    }
    stats
        .iter()
        .map(|stat| metric(stat) * stat.total_tests as f64)
        .sum::<f64>()
        / total_tests as f64
}
