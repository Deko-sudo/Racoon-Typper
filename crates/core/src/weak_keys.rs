//! WeakKeysAnalyzer — анализ heatmap, построение отчёта о проблемных клавишах.

use std::collections::HashMap;

use racoon_domain::keyboard::CharStat;

/// Одна проблемная клавиша с рангом.
#[derive(Debug, Clone, serde::Serialize)]
pub struct WeakKey {
    pub ch: char,
    pub error_count: usize,
    pub total: usize,
    pub accuracy: f64,
    pub rank: usize,
}

/// Полный отчёт о проблемных клавишах.
#[derive(Debug, Clone, serde::Serialize)]
pub struct WeakKeysReport {
    pub weak_keys: Vec<WeakKey>,
    pub total_chars_analyzed: usize,
    pub overall_accuracy: f64,
    pub critical_count: usize,
}

impl WeakKeysReport {
    /// Критические клавиши (accuracy < 70%).
    pub fn critical_keys(&self) -> Vec<&WeakKey> {
        self.weak_keys
            .iter()
            .filter(|k| k.accuracy < 70.0)
            .collect()
    }

    /// Топ-N проблемных клавиш.
    pub fn top_n(&self, n: usize) -> Vec<&WeakKey> {
        self.weak_keys.iter().take(n).collect()
    }

    /// Есть ли критические клавиши.
    pub fn has_critical(&self) -> bool {
        self.critical_count > 0
    }
}

/// WeakKeysAnalyzer — анализирует char_stats и строит WeakKeysReport.
pub struct WeakKeysAnalyzer;

impl WeakKeysAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Анализирует char_stats и возвращает отсортированный отчёт.
    pub fn analyze(&self, char_stats: &HashMap<String, CharStat>) -> WeakKeysReport {
        let mut weak: Vec<WeakKey> = char_stats
            .iter()
            .filter(|(_, s)| s.total > 0 && s.incorrect > 0)
            .map(|(ch, s)| {
                let accuracy = (s.correct as f64 / s.total as f64) * 100.0;
                WeakKey {
                    ch: ch.chars().next().unwrap_or(' '),
                    error_count: s.incorrect,
                    total: s.total,
                    accuracy,
                    rank: 0,
                }
            })
            .collect();

        // Сортировка по error_count descending
        weak.sort_by_key(|b| std::cmp::Reverse(b.error_count));

        // Назначаем ранги
        for (i, k) in weak.iter_mut().enumerate() {
            k.rank = i + 1;
        }

        let total_chars: usize = char_stats.values().map(|s| s.total).sum();

        let total_correct: usize = char_stats.values().map(|s| s.correct).sum();

        let overall = if total_chars > 0 {
            (total_correct as f64 / total_chars as f64) * 100.0
        } else {
            100.0
        };

        let critical_count = weak.iter().filter(|k| k.accuracy < 70.0).count();

        WeakKeysReport {
            weak_keys: weak,
            total_chars_analyzed: total_chars,
            overall_accuracy: overall,
            critical_count,
        }
    }
}

impl Default for WeakKeysAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stat(correct: usize, incorrect: usize) -> CharStat {
        CharStat {
            correct,
            incorrect,
            total: correct + incorrect,
        }
    }

    fn make_stats() -> HashMap<String, CharStat> {
        let mut m = HashMap::new();
        m.insert("a".to_string(), stat(3, 7)); // 30% — critical
        m.insert("e".to_string(), stat(6, 4)); // 60% — critical
        m.insert("i".to_string(), stat(9, 1)); // 90% — not critical
        m.insert("o".to_string(), stat(20, 0)); // 100% — no errors
        m
    }

    #[test]
    fn analyze_returns_sorted_by_errors() {
        let a = WeakKeysAnalyzer::new();
        let report = a.analyze(&make_stats());
        assert_eq!(report.weak_keys[0].ch, 'a'); // 7 errors
        assert_eq!(report.weak_keys[1].ch, 'e'); // 4 errors
        assert_eq!(report.weak_keys[2].ch, 'i'); // 1 error
    }

    #[test]
    fn analyze_assigns_ranks() {
        let a = WeakKeysAnalyzer::new();
        let report = a.analyze(&make_stats());
        assert_eq!(report.weak_keys[0].rank, 1);
        assert_eq!(report.weak_keys[1].rank, 2);
        assert_eq!(report.weak_keys[2].rank, 3);
    }

    #[test]
    fn analyze_excludes_no_error_chars() {
        let a = WeakKeysAnalyzer::new();
        let report = a.analyze(&make_stats());
        // 'o' has 0 errors, should not appear
        assert!(!report.weak_keys.iter().any(|k| k.ch == 'o'));
    }

    #[test]
    fn analyze_calculates_overall_accuracy() {
        let a = WeakKeysAnalyzer::new();
        let report = a.analyze(&make_stats());
        // total: (3+6+9+20) / (10+10+10+20) = 38/50 = 76%
        assert!((report.overall_accuracy - 76.0).abs() < 0.1);
    }

    #[test]
    fn analyze_counts_critical() {
        let a = WeakKeysAnalyzer::new();
        let report = a.analyze(&make_stats());
        assert_eq!(report.critical_count, 2); // a (30%), e (60%)
    }

    #[test]
    fn critical_keys_filter() {
        let a = WeakKeysAnalyzer::new();
        let report = a.analyze(&make_stats());
        let crit = report.critical_keys();
        assert_eq!(crit.len(), 2);
        assert!(crit.iter().all(|k| k.accuracy < 70.0));
    }

    #[test]
    fn top_n_filter() {
        let a = WeakKeysAnalyzer::new();
        let report = a.analyze(&make_stats());
        let top2 = report.top_n(2);
        assert_eq!(top2.len(), 2);
        assert_eq!(top2[0].ch, 'a');
        assert_eq!(top2[1].ch, 'e');
    }

    #[test]
    fn has_critical_true() {
        let a = WeakKeysAnalyzer::new();
        let report = a.analyze(&make_stats());
        assert!(report.has_critical());
    }

    #[test]
    fn has_critical_false() {
        let a = WeakKeysAnalyzer::new();
        let mut m = HashMap::new();
        m.insert("x".to_string(), stat(8, 1)); // 88% — not critical
        let report = a.analyze(&m);
        assert!(!report.has_critical());
    }

    #[test]
    fn empty_stats() {
        let a = WeakKeysAnalyzer::new();
        let report = a.analyze(&HashMap::new());
        assert!(report.weak_keys.is_empty());
        assert_eq!(report.total_chars_analyzed, 0);
        assert!((report.overall_accuracy - 100.0).abs() < 0.01);
    }

    #[test]
    fn all_correct_no_weak() {
        let a = WeakKeysAnalyzer::new();
        let mut m = HashMap::new();
        m.insert("a".to_string(), stat(10, 0));
        m.insert("b".to_string(), stat(20, 0));
        let report = a.analyze(&m);
        assert!(report.weak_keys.is_empty());
    }

    #[test]
    fn accuracy_calculation() {
        let a = WeakKeysAnalyzer::new();
        let report = a.analyze(&make_stats());
        let a_key = report.weak_keys.iter().find(|k| k.ch == 'a').unwrap();
        assert!((a_key.accuracy - 30.0).abs() < 0.1);
    }

    #[test]
    fn single_char_analysis() {
        let a = WeakKeysAnalyzer::new();
        let mut m = HashMap::new();
        m.insert("z".to_string(), stat(0, 5)); // 0% accuracy
        let report = a.analyze(&m);
        assert_eq!(report.weak_keys.len(), 1);
        assert_eq!(report.weak_keys[0].ch, 'z');
        assert_eq!(report.weak_keys[0].rank, 1);
        assert!((report.weak_keys[0].accuracy).abs() < 0.01);
    }
}
