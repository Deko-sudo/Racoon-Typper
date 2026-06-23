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
pub fn check_achievements(
    total_tests: i64,
    best_wpm: f64,
    best_accuracy: f64,
    _current_streak: i64,
    longest_streak: i64,
    lessons_completed: i64,
) -> Vec<Achievement> {
    let achievements = vec![
        Achievement {
            id: "first_test".to_string(),
            name: "First Steps".to_string(),
            description: "Complete your first test".to_string(),
            unlocked: total_tests >= 1,
            unlocked_at: None,
        },
        Achievement {
            id: "10_tests".to_string(),
            name: "Getting Started".to_string(),
            description: "Complete 10 tests".to_string(),
            unlocked: total_tests >= 10,
            unlocked_at: None,
        },
        Achievement {
            id: "50_tests".to_string(),
            name: "Dedicated".to_string(),
            description: "Complete 50 tests".to_string(),
            unlocked: total_tests >= 50,
            unlocked_at: None,
        },
        Achievement {
            id: "100_tests".to_string(),
            name: "Centurion".to_string(),
            description: "Complete 100 tests".to_string(),
            unlocked: total_tests >= 100,
            unlocked_at: None,
        },
        Achievement {
            id: "wpm_30".to_string(),
            name: "Speed Runner".to_string(),
            description: "Reach 30 WPM".to_string(),
            unlocked: best_wpm >= 30.0,
            unlocked_at: None,
        },
        Achievement {
            id: "wpm_50".to_string(),
            name: "Fast Fingers".to_string(),
            description: "Reach 50 WPM".to_string(),
            unlocked: best_wpm >= 50.0,
            unlocked_at: None,
        },
        Achievement {
            id: "wpm_80".to_string(),
            name: "Lightning".to_string(),
            description: "Reach 80 WPM".to_string(),
            unlocked: best_wpm >= 80.0,
            unlocked_at: None,
        },
        Achievement {
            id: "wpm_100".to_string(),
            name: "Speed Demon".to_string(),
            description: "Reach 100 WPM".to_string(),
            unlocked: best_wpm >= 100.0,
            unlocked_at: None,
        },
        Achievement {
            id: "acc_95".to_string(),
            name: "Sharpshooter".to_string(),
            description: "Reach 95% accuracy".to_string(),
            unlocked: best_accuracy >= 95.0,
            unlocked_at: None,
        },
        Achievement {
            id: "acc_99".to_string(),
            name: "Perfect Precision".to_string(),
            description: "Reach 99% accuracy".to_string(),
            unlocked: best_accuracy >= 99.0,
            unlocked_at: None,
        },
        Achievement {
            id: "streak_3".to_string(),
            name: "On a Roll".to_string(),
            description: "3-day streak".to_string(),
            unlocked: longest_streak >= 3,
            unlocked_at: None,
        },
        Achievement {
            id: "streak_7".to_string(),
            name: "Week Warrior".to_string(),
            description: "7-day streak".to_string(),
            unlocked: longest_streak >= 7,
            unlocked_at: None,
        },
        Achievement {
            id: "streak_30".to_string(),
            name: "Unstoppable".to_string(),
            description: "30-day streak".to_string(),
            unlocked: longest_streak >= 30,
            unlocked_at: None,
        },
        Achievement {
            id: "lessons_5".to_string(),
            name: "Student".to_string(),
            description: "Complete 5 lessons".to_string(),
            unlocked: lessons_completed >= 5,
            unlocked_at: None,
        },
        Achievement {
            id: "lessons_20".to_string(),
            name: "Scholar".to_string(),
            description: "Complete 20 lessons".to_string(),
            unlocked: lessons_completed >= 20,
            unlocked_at: None,
        },
    ];

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
        let a = check_achievements(0, 0.0, 0.0, 0, 0, 0);
        assert!(!a[0].unlocked);
    }

    #[test]
    fn achievements_first_test() {
        let a = check_achievements(1, 0.0, 0.0, 0, 0, 0);
        assert!(a.iter().any(|x| x.id == "first_test" && x.unlocked));
    }

    #[test]
    fn achievements_wpm_50() {
        let a = check_achievements(10, 55.0, 90.0, 0, 0, 0);
        assert!(a.iter().any(|x| x.id == "wpm_50" && x.unlocked));
        assert!(!a.iter().any(|x| x.id == "wpm_80" && x.unlocked));
    }

    #[test]
    fn achievements_acc_95() {
        let a = check_achievements(10, 40.0, 96.0, 0, 0, 0);
        assert!(a.iter().any(|x| x.id == "acc_95" && x.unlocked));
    }

    #[test]
    fn achievements_streak_7() {
        let a = check_achievements(10, 40.0, 90.0, 5, 7, 0);
        assert!(a.iter().any(|x| x.id == "streak_7" && x.unlocked));
    }

    #[test]
    fn achievements_lessons_5() {
        let a = check_achievements(10, 40.0, 90.0, 0, 0, 5);
        assert!(a.iter().any(|x| x.id == "lessons_5" && x.unlocked));
    }

    #[test]
    fn achievements_all_unlocked() {
        let a = check_achievements(100, 100.0, 99.0, 30, 30, 20);
        assert!(a.iter().all(|x| x.unlocked));
    }

    #[test]
    fn achievements_count() {
        let a = check_achievements(0, 0.0, 0.0, 0, 0, 0);
        assert_eq!(a.len(), 15);
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
