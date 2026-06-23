//! Consistency Engine — вычисление consistency метрики.
//! Consistency = насколько ровно пользователь печатает (low variance = high consistency).

/// Результат анализа consistency.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConsistencyReport {
    pub score: f64, // 0-100, higher = more consistent
    pub mean_wpm: f64,
    pub std_dev: f64,
    pub cv: f64, // coefficient of variation
    pub samples: usize,
}

/// Вычисляет consistency из массива WPM значений.
pub fn calc_consistency(wpm_samples: &[f64]) -> ConsistencyReport {
    let n = wpm_samples.len();
    if n == 0 {
        return ConsistencyReport {
            score: 100.0,
            mean_wpm: 0.0,
            std_dev: 0.0,
            cv: 0.0,
            samples: 0,
        };
    }

    let mean = wpm_samples.iter().sum::<f64>() / n as f64;
    let variance = wpm_samples.iter().map(|w| (w - mean).powi(2)).sum::<f64>() / n as f64;
    let std_dev = variance.sqrt();
    let cv = if mean > 0.0 { std_dev / mean } else { 0.0 };

    // Score: 100 - cv*100, clamped 0-100
    let score = (100.0 - cv * 100.0).clamp(0.0, 100.0);

    ConsistencyReport {
        score,
        mean_wpm: mean,
        std_dev,
        cv,
        samples: n,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_samples() {
        let r = calc_consistency(&[]);
        assert_eq!(r.samples, 0);
        assert!((r.score - 100.0).abs() < 0.01);
    }

    #[test]
    fn perfect_consistency() {
        let r = calc_consistency(&[40.0, 40.0, 40.0, 40.0]);
        assert!((r.score - 100.0).abs() < 0.01);
        assert!((r.std_dev - 0.0).abs() < 0.01);
    }

    #[test]
    fn low_consistency() {
        let r = calc_consistency(&[20.0, 80.0, 10.0, 90.0]);
        assert!(r.score < 50.0);
        assert!(r.std_dev > 30.0);
    }

    #[test]
    fn single_sample() {
        let r = calc_consistency(&[50.0]);
        assert!((r.score - 100.0).abs() < 0.01);
    }

    #[test]
    fn moderate_consistency() {
        let r = calc_consistency(&[35.0, 40.0, 45.0, 42.0, 38.0]);
        assert!(r.score > 80.0);
    }

    #[test]
    fn mean_calculation() {
        let r = calc_consistency(&[30.0, 40.0, 50.0]);
        assert!((r.mean_wpm - 40.0).abs() < 0.01);
    }

    #[test]
    fn std_dev_calculation() {
        let r = calc_consistency(&[40.0, 50.0, 60.0]);
        assert!((r.std_dev - 8.165).abs() < 0.1);
    }

    #[test]
    fn cv_zero_mean() {
        let r = calc_consistency(&[0.0, 0.0, 0.0]);
        assert!((r.cv - 0.0).abs() < 0.01);
    }

    #[test]
    fn score_clamped() {
        let r = calc_consistency(&[1.0, 100.0, 1.0, 100.0]);
        assert!(r.score >= 0.0);
        assert!(r.score <= 100.0);
    }

    #[test]
    fn score_increases_with_stability() {
        let unstable = calc_consistency(&[10.0, 90.0]);
        let stable = calc_consistency(&[45.0, 55.0]);
        assert!(stable.score > unstable.score);
    }
}
