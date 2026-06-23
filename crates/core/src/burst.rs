//! Burst Detection — определение всплесков скорости печати.
//! Burst = последовательность быстрых правильных нажатий.

/// Результат burst анализа.
#[derive(Debug, Clone, serde::Serialize)]
pub struct BurstReport {
    pub burst_count: usize,
    pub max_burst_length: usize,
    pub avg_burst_length: f64,
    pub total_burst_chars: usize,
}

/// Интервал между нажатиями (мс).
#[derive(Debug, Clone)]
pub struct KeystrokeInterval {
    pub interval_ms: u64,
    pub is_correct: bool,
}

/// Анализирует keystroke intervals и находит bursts.
/// Burst = 3+ подряд правильных нажатий с интервалом < threshold_ms.
pub fn detect_bursts(intervals: &[KeystrokeInterval], threshold_ms: u64) -> BurstReport {
    let mut burst_count = 0;
    let mut max_burst_length = 0;
    let mut total_burst_chars = 0;
    let mut current_burst = 0;
    let mut burst_lengths = Vec::new();

    for ks in intervals {
        if ks.is_correct && ks.interval_ms <= threshold_ms {
            current_burst += 1;
        } else {
            if current_burst >= 3 {
                burst_count += 1;
                total_burst_chars += current_burst;
                max_burst_length = max_burst_length.max(current_burst);
                burst_lengths.push(current_burst);
            }
            current_burst = 0;
        }
    }

    // Don't forget trailing burst
    if current_burst >= 3 {
        burst_count += 1;
        total_burst_chars += current_burst;
        max_burst_length = max_burst_length.max(current_burst);
        burst_lengths.push(current_burst);
    }

    let avg_burst_length = if burst_lengths.is_empty() {
        0.0
    } else {
        burst_lengths.iter().sum::<usize>() as f64 / burst_lengths.len() as f64
    };

    BurstReport {
        burst_count,
        max_burst_length,
        avg_burst_length,
        total_burst_chars,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ks(ms: u64, correct: bool) -> KeystrokeInterval {
        KeystrokeInterval {
            interval_ms: ms,
            is_correct: correct,
        }
    }

    #[test]
    fn empty_intervals() {
        let r = detect_bursts(&[], 200);
        assert_eq!(r.burst_count, 0);
    }

    #[test]
    fn single_burst() {
        let intervals = vec![ks(100, true), ks(100, true), ks(100, true)];
        let r = detect_bursts(&intervals, 200);
        assert_eq!(r.burst_count, 1);
        assert_eq!(r.max_burst_length, 3);
    }

    #[test]
    fn no_burst_short_sequence() {
        let intervals = vec![ks(100, true), ks(100, true)];
        let r = detect_bursts(&intervals, 200);
        assert_eq!(r.burst_count, 0);
    }

    #[test]
    fn burst_broken_by_incorrect() {
        let intervals = vec![
            ks(100, true),
            ks(100, true),
            ks(100, true),
            ks(100, false),
            ks(100, true),
            ks(100, true),
            ks(100, true),
        ];
        let r = detect_bursts(&intervals, 200);
        assert_eq!(r.burst_count, 2);
    }

    #[test]
    fn burst_broken_by_slow_interval() {
        let intervals = vec![
            ks(100, true),
            ks(100, true),
            ks(100, true),
            ks(500, true), // slow
            ks(100, true),
            ks(100, true),
            ks(100, true),
        ];
        let r = detect_bursts(&intervals, 200);
        assert_eq!(r.burst_count, 2);
    }

    #[test]
    fn max_burst_length() {
        let intervals = vec![
            ks(100, true),
            ks(100, true),
            ks(100, true),
            ks(100, true),
            ks(100, true),
            ks(500, false),
            ks(100, true),
            ks(100, true),
            ks(100, true),
        ];
        let r = detect_bursts(&intervals, 200);
        assert_eq!(r.max_burst_length, 5);
        assert_eq!(r.burst_count, 2);
    }

    #[test]
    fn avg_burst_length() {
        let intervals = vec![
            ks(100, true),
            ks(100, true),
            ks(100, true),
            ks(500, false),
            ks(100, true),
            ks(100, true),
            ks(100, true),
            ks(100, true),
        ];
        let r = detect_bursts(&intervals, 200);
        assert!((r.avg_burst_length - 3.5).abs() < 0.01);
    }

    #[test]
    fn total_burst_chars() {
        let intervals = vec![
            ks(100, true),
            ks(100, true),
            ks(100, true),
            ks(500, false),
            ks(100, true),
            ks(100, true),
            ks(100, true),
        ];
        let r = detect_bursts(&intervals, 200);
        assert_eq!(r.total_burst_chars, 6);
    }

    #[test]
    fn threshold_filters() {
        let intervals = vec![ks(250, true), ks(250, true), ks(250, true)];
        let r = detect_bursts(&intervals, 200);
        assert_eq!(r.burst_count, 0);
    }

    #[test]
    fn all_incorrect_no_burst() {
        let intervals = vec![ks(100, false), ks(100, false), ks(100, false)];
        let r = detect_bursts(&intervals, 200);
        assert_eq!(r.burst_count, 0);
    }

    #[test]
    fn long_burst_at_end() {
        let intervals = vec![
            ks(100, false),
            ks(100, true),
            ks(100, true),
            ks(100, true),
            ks(100, true),
        ];
        let r = detect_bursts(&intervals, 200);
        assert_eq!(r.burst_count, 1);
        assert_eq!(r.max_burst_length, 4);
    }
}
