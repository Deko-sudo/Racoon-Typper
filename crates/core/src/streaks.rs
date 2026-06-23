//! StreakEngine — логика серий (streaks) ежедневной практики.

/// Состояние streak.
#[derive(Debug, Clone, serde::Serialize)]
pub struct StreakInfo {
    pub streak_type: String,
    pub current: i64,
    pub longest: i64,
    pub last_date: Option<String>,
    pub started_date: Option<String>,
    pub is_active: bool,
}

/// StreakEngine — вычисляет и обновляет streaks.
pub struct StreakEngine;

impl StreakEngine {
    pub fn new() -> Self {
        Self
    }

    /// Вычисляет новый streak на основе текущего и даты последнего теста.
    /// today: YYYY-MM-DD
    /// last_date: YYYY-MM-DD предыдущего теста
    pub fn compute_streak(
        current_streak: i64,
        longest_streak: i64,
        last_date: Option<&str>,
        today: &str,
    ) -> (i64, i64, bool) {
        match last_date {
            None => {
                // Первый тест — streak = 1
                let new_current = 1;
                let new_longest = longest_streak.max(new_current);
                (new_current, new_longest, true)
            }
            Some(last) => {
                let days_diff = Self::days_between(last, today);

                if days_diff == 0 {
                    // Тот же день — streak не меняется
                    (current_streak, longest_streak, true)
                } else if days_diff == 1 {
                    // Следующий день — streak++
                    let new_current = current_streak + 1;
                    let new_longest = longest_streak.max(new_current);
                    (new_current, new_longest, true)
                } else {
                    // Пропущен день(и) — reset
                    let new_current = 1;
                    let new_longest = longest_streak;
                    (new_current, new_longest, true)
                }
            }
        }
    }

    /// Проверяет, активен ли streak (последний тест был сегодня или вчера).
    pub fn is_streak_active(last_date: Option<&str>, today: &str) -> bool {
        match last_date {
            None => false,
            Some(last) => {
                let diff = Self::days_between(last, today);
                diff <= 1
            }
        }
    }

    /// Разница в днях между двумя датами YYYY-MM-DD.
    pub fn days_between(from: &str, to: &str) -> i64 {
        let from_date = Self::parse_date(from);
        let to_date = Self::parse_date(to);
        match (from_date, to_date) {
            (Some(f), Some(t)) => (t - f).num_days(),
            _ => 0,
        }
    }

    fn parse_date(s: &str) -> Option<chrono::NaiveDate> {
        chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
    }

    /// Вычисляет streak из списка дат тестов.
    /// Возвращает (current, longest).
    pub fn streak_from_dates(dates: &[String]) -> (i64, i64) {
        if dates.is_empty() {
            return (0, 0);
        }

        // Уникальные даты, отсортированные
        let mut unique: Vec<String> = dates.to_vec();
        unique.sort();
        unique.dedup();

        let mut current = 1i64;
        let mut longest = 1i64;

        for i in 1..unique.len() {
            let diff = Self::days_between(&unique[i - 1], &unique[i]);
            if diff == 1 {
                current += 1;
                longest = longest.max(current);
            } else if diff > 1 {
                current = 1;
            }
            // diff == 0 — тот же день, не меняем
        }

        // Проверяем, активен ли streak сейчас
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let last = unique.last().unwrap();
        let diff_to_today = Self::days_between(last, &today);
        if diff_to_today > 1 {
            current = 0;
        }

        (current, longest)
    }
}

impl Default for StreakEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_test_starts_streak() {
        let (curr, longest, active) = StreakEngine::compute_streak(0, 0, None, "2026-06-01");
        assert_eq!(curr, 1);
        assert_eq!(longest, 1);
        assert!(active);
    }

    #[test]
    fn consecutive_day_increments() {
        let (curr, longest, _) =
            StreakEngine::compute_streak(3, 5, Some("2026-06-01"), "2026-06-02");
        assert_eq!(curr, 4);
        assert_eq!(longest, 5);
    }

    #[test]
    fn consecutive_day_updates_longest() {
        let (curr, longest, _) =
            StreakEngine::compute_streak(5, 5, Some("2026-06-01"), "2026-06-02");
        assert_eq!(curr, 6);
        assert_eq!(longest, 6);
    }

    #[test]
    fn same_day_no_change() {
        let (curr, longest, _) =
            StreakEngine::compute_streak(3, 5, Some("2026-06-01"), "2026-06-01");
        assert_eq!(curr, 3);
        assert_eq!(longest, 5);
    }

    #[test]
    fn skipped_day_resets() {
        let (curr, longest, _) =
            StreakEngine::compute_streak(5, 7, Some("2026-06-01"), "2026-06-03");
        assert_eq!(curr, 1);
        assert_eq!(longest, 7);
    }

    #[test]
    fn skipped_multiple_days_resets() {
        let (curr, longest, _) =
            StreakEngine::compute_streak(10, 15, Some("2026-06-01"), "2026-06-10");
        assert_eq!(curr, 1);
        assert_eq!(longest, 15);
    }

    #[test]
    fn is_active_today() {
        assert!(StreakEngine::is_streak_active(
            Some("2026-06-01"),
            "2026-06-01"
        ));
    }

    #[test]
    fn is_active_yesterday() {
        assert!(StreakEngine::is_streak_active(
            Some("2026-06-01"),
            "2026-06-02"
        ));
    }

    #[test]
    fn is_inactive_two_days_ago() {
        assert!(!StreakEngine::is_streak_active(
            Some("2026-06-01"),
            "2026-06-03"
        ));
    }

    #[test]
    fn is_inactive_no_last_date() {
        assert!(!StreakEngine::is_streak_active(None, "2026-06-01"));
    }

    #[test]
    fn days_between_same_day() {
        assert_eq!(StreakEngine::days_between("2026-06-01", "2026-06-01"), 0);
    }

    #[test]
    fn days_between_one_day() {
        assert_eq!(StreakEngine::days_between("2026-06-01", "2026-06-02"), 1);
    }

    #[test]
    fn days_between_week() {
        assert_eq!(StreakEngine::days_between("2026-06-01", "2026-06-08"), 7);
    }

    #[test]
    fn streak_from_dates_consecutive() {
        let dates = vec![
            "2026-06-01".to_string(),
            "2026-06-02".to_string(),
            "2026-06-03".to_string(),
        ];
        let (curr, longest) = StreakEngine::streak_from_dates(&dates);
        // Если последние даты не сегодня, current = 0
        // Но longest должен быть 3
        assert_eq!(longest, 3);
    }

    #[test]
    fn streak_from_dates_with_gap() {
        let dates = vec![
            "2026-06-01".to_string(),
            "2026-06-02".to_string(),
            "2026-06-05".to_string(),
        ];
        let (curr, longest) = StreakEngine::streak_from_dates(&dates);
        assert_eq!(longest, 2);
    }

    #[test]
    fn streak_from_dates_empty() {
        let (curr, longest) = StreakEngine::streak_from_dates(&[]);
        assert_eq!(curr, 0);
        assert_eq!(longest, 0);
    }

    #[test]
    fn streak_from_dates_same_day_multiple() {
        let dates = vec![
            "2026-06-01".to_string(),
            "2026-06-01".to_string(),
            "2026-06-01".to_string(),
        ];
        let (_, longest) = StreakEngine::streak_from_dates(&dates);
        assert_eq!(longest, 1);
    }
}
