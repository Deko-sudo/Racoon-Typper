//! Analytics — WPM/Accuracy/Error timelines, achievements, finger stats, insights, export.

use std::collections::HashMap;

// ── Timelines ──

/// Точка временной линии.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TimelinePoint {
    pub timestamp_ms: u64,
    pub wpm: f64,
    pub accuracy: f64,
    pub errors: usize,
}

/// Строит WPM/Accuracy/Error timeline из keystroke timestamps.
pub fn build_timeline(
    keystroke_times: &[u64],
    correct_count: &[bool],
    window_ms: u64,
) -> Vec<TimelinePoint> {
    if keystroke_times.is_empty() {
        return Vec::new();
    }

    let mut points = Vec::new();
    let start = keystroke_times[0];
    let end = *keystroke_times.last().unwrap_or(&0);

    let mut window_start = start;
    while window_start < end {
        let window_end = window_start + window_ms;
        let mut correct = 0usize;
        let mut total = 0usize;

        for (i, &t) in keystroke_times.iter().enumerate() {
            if t >= window_start && t < window_end {
                total += 1;
                if i < correct_count.len() && correct_count[i] {
                    correct += 1;
                }
            }
        }

        if total > 0 {
            let elapsed_min = window_ms as f64 / 60000.0;
            let wpm = (correct as f64 / 5.0) / elapsed_min;
            let accuracy = (correct as f64 / total as f64) * 100.0;
            let errors = total - correct;
            points.push(TimelinePoint {
                timestamp_ms: window_start,
                wpm,
                accuracy,
                errors,
            });
        }

        window_start = window_end;
    }

    points
}

// ── Achievements ──

/// Достижение.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub unlocked: bool,
    pub unlocked_at: Option<String>,
}

/// Проверяет достижения на основе статистики.
///
/// `now` — ISO-8601/RFC-3339 метка, проставляемая в `unlocked_at` для всех
/// разблокированных достижений. Вызывающая сторона формирует её (обычно
/// `chrono::Utc::now().to_rfc3339()`), чтобы чистая core-логика не зависела
/// от системного времени и оставалась детерминированной в тестах.
pub fn check_achievements(
    total_tests: i64,
    best_wpm: f64,
    best_accuracy: f64,
    _current_streak: i64,
    longest_streak: i64,
    lessons_completed: i64,
    now: String,
) -> Vec<Achievement> {
    // Локальный хелпер: строит Achievement с корректным unlocked_at.
    let mk = |id: &str, name: &str, description: &str, unlocked: bool| Achievement {
        id: id.to_string(),
        name: name.to_string(),
        description: description.to_string(),
        unlocked,
        unlocked_at: if unlocked { Some(now.clone()) } else { None },
    };

    let achievements = vec![
        mk(
            "first_test",
            "First Steps",
            "Complete your first test",
            total_tests >= 1,
        ),
        mk("5_tests", "Warm Up", "Complete 5 tests", total_tests >= 5),
        mk(
            "10_tests",
            "Getting Started",
            "Complete 10 tests",
            total_tests >= 10,
        ),
        mk(
            "25_tests",
            "Regular",
            "Complete 25 tests",
            total_tests >= 25,
        ),
        mk(
            "50_tests",
            "Dedicated",
            "Complete 50 tests",
            total_tests >= 50,
        ),
        mk(
            "100_tests",
            "Centurion",
            "Complete 100 tests",
            total_tests >= 100,
        ),
        mk(
            "250_tests",
            "Quarter Milestone",
            "Complete 250 tests",
            total_tests >= 250,
        ),
        mk(
            "500_tests",
            "Half a Thousand",
            "Complete 500 tests",
            total_tests >= 500,
        ),
        mk(
            "1000_tests",
            "Typing Veteran",
            "Complete 1,000 tests",
            total_tests >= 1000,
        ),
        mk("wpm_20", "Beginner", "Reach 20 WPM", best_wpm >= 20.0),
        mk("wpm_30", "Speed Runner", "Reach 30 WPM", best_wpm >= 30.0),
        mk("wpm_50", "Fast Fingers", "Reach 50 WPM", best_wpm >= 50.0),
        mk("wpm_60", "Quick", "Reach 60 WPM", best_wpm >= 60.0),
        mk("wpm_80", "Lightning", "Reach 80 WPM", best_wpm >= 80.0),
        mk("wpm_100", "Speed Demon", "Reach 100 WPM", best_wpm >= 100.0),
        mk("wpm_120", "Blur", "Reach 120 WPM", best_wpm >= 120.0),
        mk("wpm_150", "Mach Speed", "Reach 150 WPM", best_wpm >= 150.0),
        mk("wpm_200", "Impossible", "Reach 200 WPM", best_wpm >= 200.0),
        mk(
            "acc_85",
            "Steady",
            "Reach 85% accuracy",
            best_accuracy >= 85.0,
        ),
        mk(
            "acc_90",
            "Reliable",
            "Reach 90% accuracy",
            best_accuracy >= 90.0,
        ),
        mk(
            "acc_95",
            "Sharpshooter",
            "Reach 95% accuracy",
            best_accuracy >= 95.0,
        ),
        mk(
            "acc_97",
            "Surgical",
            "Reach 97% accuracy",
            best_accuracy >= 97.0,
        ),
        mk(
            "acc_99",
            "Perfect Precision",
            "Reach 99% accuracy",
            best_accuracy >= 99.0,
        ),
        mk(
            "acc_100",
            "Flawless",
            "Reach 100% accuracy",
            best_accuracy >= 100.0,
        ),
        mk("streak_1", "First Day", "1-day streak", longest_streak >= 1),
        mk("streak_3", "On a Roll", "3-day streak", longest_streak >= 3),
        mk(
            "streak_7",
            "Week Warrior",
            "7-day streak",
            longest_streak >= 7,
        ),
        mk(
            "streak_14",
            "Fortnight",
            "14-day streak",
            longest_streak >= 14,
        ),
        mk(
            "streak_30",
            "Unstoppable",
            "30-day streak",
            longest_streak >= 30,
        ),
        mk(
            "streak_60",
            "Two Months Strong",
            "60-day streak",
            longest_streak >= 60,
        ),
        mk(
            "streak_100",
            "Century Streak",
            "100-day streak",
            longest_streak >= 100,
        ),
        mk(
            "lessons_1",
            "First Lesson",
            "Complete your first lesson",
            lessons_completed >= 1,
        ),
        mk(
            "lessons_2",
            "Apprentice",
            "Complete 2 lessons",
            lessons_completed >= 2,
        ),
        mk(
            "lessons_5",
            "Student",
            "Complete 5 lessons",
            lessons_completed >= 5,
        ),
        mk(
            "lessons_20",
            "Scholar",
            "Complete 20 lessons",
            lessons_completed >= 20,
        ),
        mk(
            "lessons_50",
            "Mentor",
            "Complete 50 lessons",
            lessons_completed >= 50,
        ),
        mk(
            "lessons_100",
            "Master",
            "Complete 100 lessons",
            lessons_completed >= 100,
        ),
        mk(
            "lessons_200",
            "Grandmaster",
            "Complete 200 lessons",
            lessons_completed >= 200,
        ),
        mk(
            "balanced_50_95",
            "Balanced",
            "Reach 50 WPM with 95% accuracy",
            best_wpm >= 50.0 && best_accuracy >= 95.0,
        ),
        mk(
            "elite_80_95",
            "Elite",
            "Reach 80 WPM with 95% accuracy",
            best_wpm >= 80.0 && best_accuracy >= 95.0,
        ),
        mk(
            "marathon_100_80",
            "Marathon",
            "Complete 100 tests and reach 80 WPM",
            total_tests >= 100 && best_wpm >= 80.0,
        ),
        mk(
            "legend_500_100",
            "Legend",
            "Complete 500 tests and reach 100 WPM",
            total_tests >= 500 && best_wpm >= 100.0,
        ),
        mk(
            "consistent_7_20",
            "Consistent",
            "Hold a 7-day streak and complete 20 lessons",
            longest_streak >= 7 && lessons_completed >= 20,
        ),
        mk(
            "flawless_99_50",
            "Perfectionist",
            "Reach 99% accuracy across 50 tests",
            best_accuracy >= 99.0 && total_tests >= 50,
        ),
        mk("wpm_40", "Cruising", "Reach 40 WPM", best_wpm >= 40.0),
        mk("wpm_70", "Swift", "Reach 70 WPM", best_wpm >= 70.0),
        mk(
            "acc_92",
            "Precise",
            "Reach 92% accuracy",
            best_accuracy >= 92.0,
        ),
        mk(
            "streak_21",
            "Three Weeks",
            "21-day streak",
            longest_streak >= 21,
        ),
        mk(
            "lessons_10",
            "Diligent",
            "Complete 10 lessons",
            lessons_completed >= 10,
        ),
        mk(
            "tests_75",
            "Persistent",
            "Complete 75 tests",
            total_tests >= 75,
        ),
        mk("wpm_90", "Turbo", "Reach 90 WPM", best_wpm >= 90.0),
        mk(
            "acc_98",
            "Virtuoso",
            "Reach 98% accuracy",
            best_accuracy >= 98.0,
        ),
    ];

    // Сортируем: разблокированные — сверху, по убыванию id-порога визуально ок.
    achievements
}

// ── Finger Statistics ──

/// Статистика по пальцам.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FingerStat {
    pub finger: String,
    pub total: usize,
    pub correct: usize,
    pub incorrect: usize,
    pub accuracy: f64,
    pub avg_interval_ms: f64,
}

/// Вычисляет статистику по пальцам из keystroke data.
pub fn compute_finger_stats(
    keystrokes: &[(char, bool, u64)], // (char, correct, interval_ms)
    is_russian: bool,
) -> Vec<FingerStat> {
    use crate::finger_map::finger_for_char;

    let mut per_finger: HashMap<String, (usize, usize, usize, u64)> = HashMap::new();

    for &(ch, correct, interval_ms) in keystrokes {
        let finger = finger_for_char(ch, is_russian);
        let entry = per_finger
            .entry(finger.display_name().to_string())
            .or_insert((0, 0, 0, 0u64));
        entry.0 += 1; // total
        if correct {
            entry.1 += 1;
        } else {
            entry.2 += 1;
        }
        entry.3 += interval_ms;
    }

    let mut stats: Vec<FingerStat> = per_finger
        .iter()
        .map(
            |(finger, &(total, correct, incorrect, total_interval))| FingerStat {
                finger: finger.clone(),
                total,
                correct,
                incorrect,
                accuracy: if total > 0 {
                    (correct as f64 / total as f64) * 100.0
                } else {
                    100.0
                },
                avg_interval_ms: if total > 0 {
                    total_interval as f64 / total as f64
                } else {
                    0.0
                },
            },
        )
        .collect();

    stats.sort_by_key(|b| std::cmp::Reverse(b.total));
    stats
}

// ── Personal Insights ──

/// Персональная рекомендация.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Insight {
    pub level: String, // info, warning, success
    pub title: String,
    pub message: String,
}

/// Генерирует персональные рекомендации.
pub fn generate_insights(
    avg_wpm: f64,
    avg_accuracy: f64,
    consistency_score: f64,
    weak_key_count: usize,
    streak: i64,
) -> Vec<Insight> {
    let mut insights = Vec::new();

    if avg_accuracy < 90.0 {
        insights.push(Insight {
            level: "warning".to_string(),
            title: "Focus on Accuracy".to_string(),
            message: format!(
                "Your accuracy is {:.1}%. Slow down slightly and focus on hitting the right keys.",
                avg_accuracy
            ),
        });
    }

    if avg_accuracy >= 95.0 && avg_wpm < 40.0 {
        insights.push(Insight {
            level: "info".to_string(),
            title: "Ready for Speed".to_string(),
            message: "Your accuracy is excellent! Try to type faster while maintaining precision."
                .to_string(),
        });
    }

    if consistency_score < 60.0 {
        insights.push(Insight {
            level: "warning".to_string(),
            title: "Inconsistent Speed".to_string(),
            message: format!(
                "Your consistency is {:.0}%. Try to maintain a steady rhythm.",
                consistency_score
            ),
        });
    }

    if weak_key_count > 3 {
        insights.push(Insight {
            level: "info".to_string(),
            title: "Weak Keys Detected".to_string(),
            message: format!(
                "You have {} problematic keys. Use Weak Keys training to improve.",
                weak_key_count
            ),
        });
    }

    if streak >= 3 {
        insights.push(Insight {
            level: "success".to_string(),
            title: "Streak Active".to_string(),
            message: format!("{} days in a row! Keep it up!", streak),
        });
    }

    if avg_wpm > 60.0 && avg_accuracy > 95.0 {
        insights.push(Insight {
            level: "success".to_string(),
            title: "Excellent Performance".to_string(),
            message: "You're typing fast and accurately. Great job!".to_string(),
        });
    }

    if insights.is_empty() {
        insights.push(Insight {
            level: "info".to_string(),
            title: "Keep Practicing".to_string(),
            message: "Complete more tests to get personalized insights.".to_string(),
        });
    }

    insights
}

// ── Export ──

/// Экспорт данных в JSON.
pub fn export_json(data: &serde_json::Value) -> String {
    serde_json::to_string_pretty(data).unwrap_or_default()
}

/// Экспорт данных в Markdown: таблица + сводка.
pub fn export_markdown(rows: &[Vec<String>], summary: &[(&str, String)]) -> String {
    let mut out = String::new();
    out.push_str("# Racoon Typper export\n\n");

    if !summary.is_empty() {
        out.push_str("## Summary\n\n");
        out.push_str("| Metric | Value |\n|---|---|\n");
        for (label, value) in summary {
            out.push_str(&format!(
                "| {} | {} |\n",
                escape_markdown_cell(label),
                escape_markdown_cell(value)
            ));
        }
        out.push('\n');
    }

    if let Some(header) = rows.first() {
        out.push_str("## Tests\n\n");
        out.push_str("| ");
        out.push_str(
            &header
                .iter()
                .map(|cell| escape_markdown_cell(cell))
                .collect::<Vec<_>>()
                .join(" | "),
        );
        out.push_str(" |\n");
        out.push_str(&format!("|{}|\n", "---|".repeat(header.len())));
        for row in &rows[1..] {
            out.push_str("| ");
            out.push_str(
                &row.iter()
                    .map(|cell| escape_markdown_cell(cell))
                    .collect::<Vec<_>>()
                    .join(" | "),
            );
            out.push_str(" |\n");
        }
        out.push('\n');
    }

    out
}

fn escape_markdown_cell(cell: &str) -> String {
    cell.replace('|', "\\|").replace('\n', " ")
}

/// Экспорт данных в CSV.
pub fn export_csv(rows: &[Vec<String>]) -> String {
    rows.iter()
        .map(|row| {
            row.iter()
                .map(|cell| {
                    if cell.contains(',') || cell.contains('"') || cell.contains('\n') {
                        format!("\"{}\"", cell.replace('"', "\"\""))
                    } else {
                        cell.clone()
                    }
                })
                .collect::<Vec<_>>()
                .join(",")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// ── Session Replay ──

/// Событие replay — одно нажатие клавиши.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReplayEvent {
    pub timestamp_ms: u64,
    pub key: String,
    pub expected: String,
    pub correct: bool,
}

/// Полный replay сессии.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SessionReplay {
    pub events: Vec<ReplayEvent>,
    pub total_duration_ms: u64,
    pub text: String,
}

impl SessionReplay {
    pub fn new(text: String) -> Self {
        Self {
            events: Vec::new(),
            total_duration_ms: 0,
            text,
        }
    }

    pub fn add_event(&mut self, timestamp_ms: u64, key: &str, expected: &str, correct: bool) {
        self.events.push(ReplayEvent {
            timestamp_ms,
            key: key.to_string(),
            expected: expected.to_string(),
            correct,
        });
        if timestamp_ms > self.total_duration_ms {
            self.total_duration_ms = timestamp_ms;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Timeline tests ──

    #[test]
    fn timeline_empty() {
        let t = build_timeline(&[], &[], 5000);
        assert!(t.is_empty());
    }

    #[test]
    fn timeline_single_window() {
        let times = vec![0, 100, 200, 300, 400];
        let correct = vec![true, true, true, false, true];
        let t = build_timeline(&times, &correct, 5000);
        assert_eq!(t.len(), 1);
        assert!((t[0].accuracy - 80.0).abs() < 0.01); // 4/5
    }

    #[test]
    fn timeline_multiple_windows() {
        let times = vec![0, 100, 6000, 6100];
        let correct = vec![true, true, true, true];
        let t = build_timeline(&times, &correct, 5000);
        assert_eq!(t.len(), 2);
    }

    #[test]
    fn timeline_calculates_wpm() {
        let times: Vec<u64> = (0..5).map(|i| i * 100).collect();
        let correct = vec![true; 5];
        let t = build_timeline(&times, &correct, 5000);
        // 5 correct chars in 5s = 1/5 = 0.2 chars per 5s
        // window = 5000ms = 5s = 1/12 min
        // WPM = (5/5) / (5/60000) = 1 * 12000 = 12
        assert!(t[0].wpm > 0.0);
    }

    #[test]
    fn timeline_counts_errors() {
        let times = vec![0, 100, 200, 300, 400];
        let correct = vec![true, false, false, true, true];
        let t = build_timeline(&times, &correct, 5000);
        assert_eq!(t[0].errors, 2);
    }

    // ── Achievement tests ──

    #[test]
    fn achievements_empty() {
        let a = check_achievements(0, 0.0, 0.0, 0, 0, 0, "2026-01-01T00:00:00Z".to_string());
        assert!(!a[0].unlocked);
        // Unlocked achievements must carry a timestamp; locked ones must not.
        assert!(a.iter().all(|x| x.unlocked == x.unlocked_at.is_some()));
    }

    #[test]
    fn achievements_first_test() {
        let a = check_achievements(1, 0.0, 0.0, 0, 0, 0, "2026-01-01T00:00:00Z".to_string());
        assert!(a.iter().any(|x| x.id == "first_test" && x.unlocked));
    }

    #[test]
    fn achievements_wpm_50() {
        let a = check_achievements(10, 55.0, 90.0, 0, 0, 0, "2026-01-01T00:00:00Z".to_string());
        assert!(a.iter().any(|x| x.id == "wpm_50" && x.unlocked));
        assert!(!a.iter().any(|x| x.id == "wpm_80" && x.unlocked));
    }

    #[test]
    fn achievements_acc_95() {
        let a = check_achievements(10, 40.0, 96.0, 0, 0, 0, "2026-01-01T00:00:00Z".to_string());
        assert!(a.iter().any(|x| x.id == "acc_95" && x.unlocked));
    }

    #[test]
    fn achievements_streak_7() {
        let a = check_achievements(10, 40.0, 90.0, 5, 7, 0, "2026-01-01T00:00:00Z".to_string());
        assert!(a.iter().any(|x| x.id == "streak_7" && x.unlocked));
    }

    #[test]
    fn achievements_lessons_5() {
        let a = check_achievements(10, 40.0, 90.0, 0, 0, 5, "2026-01-01T00:00:00Z".to_string());
        assert!(a.iter().any(|x| x.id == "lessons_5" && x.unlocked));
    }

    #[test]
    fn achievements_all_unlocked() {
        let a = check_achievements(
            1000,
            200.0,
            100.0,
            100,
            100,
            200,
            "2026-01-01T00:00:00Z".to_string(),
        );
        assert!(a.iter().all(|x| x.unlocked));
    }

    #[test]
    fn achievements_count() {
        let a = check_achievements(0, 0.0, 0.0, 0, 0, 0, "2026-01-01T00:00:00Z".to_string());
        assert_eq!(a.len(), 52);
    }

    // ── Finger stats tests ──

    #[test]
    fn finger_stats_basic() {
        let keystrokes = vec![('a', true, 100), ('a', true, 120), ('a', false, 150)];
        let stats = compute_finger_stats(&keystrokes, false);
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].finger, "Left Pinky");
        assert_eq!(stats[0].total, 3);
        assert_eq!(stats[0].correct, 2);
        assert_eq!(stats[0].incorrect, 1);
    }

    #[test]
    fn finger_stats_multiple_fingers() {
        let keystrokes = vec![
            ('a', true, 100), // Left Pinky
            ('f', true, 100), // Left Index
            ('j', true, 100), // Right Index
        ];
        let stats = compute_finger_stats(&keystrokes, false);
        assert_eq!(stats.len(), 3);
    }

    #[test]
    fn finger_stats_accuracy() {
        let keystrokes = vec![('a', true, 100), ('a', false, 100)];
        let stats = compute_finger_stats(&keystrokes, false);
        assert!((stats[0].accuracy - 50.0).abs() < 0.01);
    }

    #[test]
    fn finger_stats_empty() {
        let stats = compute_finger_stats(&[], false);
        assert!(stats.is_empty());
    }

    #[test]
    fn finger_stats_sorted_by_usage() {
        let keystrokes = vec![
            ('a', true, 100),
            ('a', true, 100),
            ('a', true, 100),
            ('f', true, 100),
        ];
        let stats = compute_finger_stats(&keystrokes, false);
        assert_eq!(stats[0].finger, "Left Pinky"); // 3 uses
        assert_eq!(stats[1].finger, "Left Index"); // 1 use
    }

    // ── Insights tests ──

    #[test]
    fn insights_low_accuracy() {
        let i = generate_insights(40.0, 85.0, 80.0, 1, 0);
        assert!(i.iter().any(|x| x.title == "Focus on Accuracy"));
    }

    #[test]
    fn insights_high_accuracy_low_wpm() {
        let i = generate_insights(30.0, 96.0, 80.0, 0, 0);
        assert!(i.iter().any(|x| x.title == "Ready for Speed"));
    }

    #[test]
    fn insights_low_consistency() {
        let i = generate_insights(40.0, 90.0, 50.0, 0, 0);
        assert!(i.iter().any(|x| x.title == "Inconsistent Speed"));
    }

    #[test]
    fn insights_weak_keys() {
        let i = generate_insights(40.0, 90.0, 80.0, 5, 0);
        assert!(i.iter().any(|x| x.title == "Weak Keys Detected"));
    }

    #[test]
    fn insights_streak_active() {
        let i = generate_insights(40.0, 90.0, 80.0, 0, 5);
        assert!(i.iter().any(|x| x.title == "Streak Active"));
    }

    #[test]
    fn insights_excellent() {
        let i = generate_insights(70.0, 96.0, 80.0, 0, 0);
        assert!(i.iter().any(|x| x.title == "Excellent Performance"));
    }

    #[test]
    fn insights_default_when_no_data() {
        let i = generate_insights(0.0, 100.0, 100.0, 0, 0);
        assert!(!i.is_empty());
    }

    // ── Export tests ──

    #[test]
    fn export_json_valid() {
        let data = serde_json::json!({"wpm": 40.0, "accuracy": 95.0});
        let json = export_json(&data);
        assert!(json.contains("40.0"));
        assert!(json.contains("95.0"));
    }

    #[test]
    fn export_csv_basic() {
        let rows = vec![
            vec![
                "Date".to_string(),
                "WPM".to_string(),
                "Accuracy".to_string(),
            ],
            vec!["2026-06-01".to_string(), "40".to_string(), "95".to_string()],
        ];
        let csv = export_csv(&rows);
        assert!(csv.contains("Date,WPM,Accuracy"));
        assert!(csv.contains("2026-06-01,40,95"));
    }

    #[test]
    fn export_csv_with_commas() {
        let rows = vec![vec!["Name, with comma".to_string(), "Value".to_string()]];
        let csv = export_csv(&rows);
        assert!(csv.contains("\"Name, with comma\""));
    }

    #[test]
    fn export_markdown_renders_table_and_summary() {
        let rows = vec![
            vec!["Date".to_string(), "WPM".to_string()],
            vec!["2026-06-01".to_string(), "40".to_string()],
        ];
        let summary = vec![("Total tests", "1".to_string())];
        let md = export_markdown(&rows, &summary);
        assert!(md.contains("# Racoon Typper export"));
        assert!(md.contains("| Total tests | 1 |"));
        assert!(md.contains("| Date | WPM |"));
        assert!(md.contains("| 2026-06-01 | 40 |"));
    }

    #[test]
    fn export_markdown_escapes_pipes() {
        let rows = vec![vec!["A|B".to_string()]];
        let md = export_markdown(&rows, &[]);
        assert!(md.contains("A\\|B"));
    }

    #[test]
    fn export_markdown_empty_rows_omits_table() {
        let md = export_markdown(&[], &[]);
        assert!(md.contains("# Racoon Typper export"));
        assert!(!md.contains("## Tests"));
    }

    // ── Session Replay tests ──

    #[test]
    fn replay_basic() {
        let mut r = SessionReplay::new("hello".to_string());
        r.add_event(0, "h", "h", true);
        r.add_event(100, "e", "e", true);
        r.add_event(200, "x", "l", false);
        assert_eq!(r.events.len(), 3);
        assert_eq!(r.total_duration_ms, 200);
    }

    #[test]
    fn replay_empty() {
        let r = SessionReplay::new("test".to_string());
        assert_eq!(r.events.len(), 0);
    }

    #[test]
    fn replay_tracks_correctness() {
        let mut r = SessionReplay::new("hi".to_string());
        r.add_event(0, "h", "h", true);
        r.add_event(100, "x", "i", false);
        assert!(r.events[0].correct);
        assert!(!r.events[1].correct);
    }
}
