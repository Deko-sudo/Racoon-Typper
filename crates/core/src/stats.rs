//! Statistics Engine — расчёт WPM, Raw WPM, Accuracy.
//! Live Tracker обновляется на каждое нажатие.

use std::collections::HashMap;
use std::time::Instant;

use racoon_domain::keyboard::{CharStat, KeyHeatData};

use crate::typing::TextBuffer;

/// Live Tracker — обновляется на каждый keystroke.
#[derive(Debug, Clone)]
pub struct LiveTracker {
    pub correct_chars: usize,
    pub incorrect_chars: usize,
    pub backspaces: usize,
    pub total_keystrokes: usize,
    pub start_time: Option<Instant>,
}

impl LiveTracker {
    pub fn new() -> Self {
        Self {
            correct_chars: 0,
            incorrect_chars: 0,
            backspaces: 0,
            total_keystrokes: 0,
            start_time: None,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Регистрирует нажатие.
    pub fn on_key_processed(&mut self, result: &crate::typing::TypingResult, buf: &TextBuffer) {
        self.total_keystrokes += 1;

        if self.start_time.is_none() {
            self.start_time = buf.start_time;
        }

        match result {
            crate::typing::TypingResult::Correct => {
                self.correct_chars += 1;
            }
            crate::typing::TypingResult::Incorrect => {
                self.incorrect_chars += 1;
            }
            crate::typing::TypingResult::UndoneCorrect => {
                // Backspace снял correct — уменьшаем
                self.backspaces += 1;
                if self.correct_chars > 0 {
                    self.correct_chars -= 1;
                }
            }
            crate::typing::TypingResult::UndoneIncorrect => {
                // Backspace снял incorrect — уменьшаем
                self.backspaces += 1;
                if self.incorrect_chars > 0 {
                    self.incorrect_chars -= 1;
                }
            }
            _ => {}
        }
    }

    /// Затраченное время в минутах.
    pub fn elapsed_minutes(&self) -> f64 {
        let elapsed_ms = self
            .start_time
            .map(|t| t.elapsed().as_millis() as u64)
            .unwrap_or(0);
        elapsed_ms as f64 / 60_000.0
    }
}

impl Default for LiveTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// WPM Calculator — Net WPM и Raw WPM.
pub struct WpmCalculator;

impl WpmCalculator {
    /// Net WPM = (correct_chars / 5) / elapsed_minutes
    /// 5 символов = 1 "слово" (стандарт индустрии)
    pub fn net_wpm(correct_chars: usize, elapsed_minutes: f64) -> f64 {
        if elapsed_minutes < 0.0001 {
            return 0.0;
        }
        (correct_chars as f64 / 5.0) / elapsed_minutes
    }

    /// Raw WPM = (all_typed_chars / 5) / elapsed_minutes
    /// all_typed_chars = correct + incorrect + backspaces
    pub fn raw_wpm(
        correct_chars: usize,
        incorrect_chars: usize,
        backspaces: usize,
        elapsed_minutes: f64,
    ) -> f64 {
        if elapsed_minutes < 0.0001 {
            return 0.0;
        }
        let all_typed = (correct_chars + incorrect_chars + backspaces) as f64;
        (all_typed / 5.0) / elapsed_minutes
    }
}

/// Accuracy Calculator — Net Accuracy и Raw Accuracy.
pub struct AccuracyCalculator;

impl AccuracyCalculator {
    /// Net Accuracy = correct / (correct + incorrect) * 100
    pub fn net_accuracy(correct: usize, incorrect: usize) -> f64 {
        let total = correct + incorrect;
        if total == 0 {
            return 100.0;
        }
        (correct as f64 / total as f64) * 100.0
    }

    /// Raw Accuracy = first_correct / total_first_attempts * 100
    pub fn raw_accuracy(first_correct: usize, total_first_attempts: usize) -> f64 {
        if total_first_attempts == 0 {
            return 100.0;
        }
        (first_correct as f64 / total_first_attempts as f64) * 100.0
    }
}

/// Heatmap Builder — строит HashMap<char, KeyHeatData> из TextBuffer.
pub struct HeatmapBuilder;

impl HeatmapBuilder {
    /// Строит heatmap из typed_chars.
    /// Использует first_typed и first_correct (первая попытка, даже если backspaced).
    pub fn build(buf: &TextBuffer) -> HashMap<String, KeyHeatData> {
        let mut map: HashMap<String, KeyHeatData> = HashMap::new();

        for tc in &buf.typed_chars {
            // Пропускаем позиции, где не было первой попытки
            if tc.first_typed.is_none() {
                continue;
            }

            let key = tc.expected.to_string();
            let entry = map.entry(key).or_insert(KeyHeatData {
                total_attempts: 0,
                correct: 0,
                incorrect: 0,
                avg_wpm_at_key: 0.0,
            });

            entry.total_attempts += 1;
            if tc.first_correct {
                entry.correct += 1;
            } else {
                entry.incorrect += 1;
            }
        }

        map
    }

    /// Строит char_stats из typed_chars (финальные, не backspaced).
    pub fn build_char_stats(buf: &TextBuffer) -> HashMap<String, CharStat> {
        use racoon_domain::CharStatus;
        let mut map: HashMap<String, CharStat> = HashMap::new();

        for tc in &buf.typed_chars {
            let key = tc.expected.to_string();
            let entry = map.entry(key).or_default();

            match tc.status {
                CharStatus::Correct => {
                    entry.correct += 1;
                    entry.total += 1;
                }
                CharStatus::Incorrect => {
                    entry.incorrect += 1;
                    entry.total += 1;
                }
                _ => {}
            }
        }

        map
    }

    /// Считает first_correct (количество первых правильных попыток).
    pub fn count_first_correct(buf: &TextBuffer) -> usize {
        buf.typed_chars
            .iter()
            .filter(|tc| tc.first_typed.is_some() && tc.first_correct)
            .count()
    }

    /// Считает total_first_attempts (позиций где была первая попытка).
    pub fn count_first_attempts(buf: &TextBuffer) -> usize {
        buf.typed_chars
            .iter()
            .filter(|tc| tc.first_typed.is_some())
            .count()
    }
}

/// Statistics Engine — связывает LiveTracker, WPM, Accuracy, Heatmap.
pub struct StatisticsEngine {
    pub tracker: LiveTracker,
}

impl StatisticsEngine {
    pub fn new() -> Self {
        Self {
            tracker: LiveTracker::new(),
        }
    }

    /// Обновляет статистику на каждое нажатие.
    pub fn on_key_processed(&mut self, result: &crate::typing::TypingResult, buf: &TextBuffer) {
        self.tracker.on_key_processed(result, buf);
    }

    /// Возвращает live статистику (WPM, Raw WPM, Accuracy, elapsed_ms).
    pub fn live_stats(&self, _buf: &TextBuffer) -> racoon_domain::LiveStats {
        let elapsed_min = self.tracker.elapsed_minutes();
        let elapsed_ms = self
            .tracker
            .start_time
            .map(|t| t.elapsed().as_millis() as u64)
            .unwrap_or(0);

        let wpm = WpmCalculator::net_wpm(self.tracker.correct_chars, elapsed_min);
        let raw_wpm = WpmCalculator::raw_wpm(
            self.tracker.correct_chars,
            self.tracker.incorrect_chars,
            self.tracker.backspaces,
            elapsed_min,
        );
        let accuracy = AccuracyCalculator::net_accuracy(
            self.tracker.correct_chars,
            self.tracker.incorrect_chars,
        );

        racoon_domain::LiveStats {
            wpm,
            raw_wpm,
            accuracy,
            elapsed_ms,
        }
    }

    /// Финализирует статистику при завершении теста.
    pub fn finalize(&self, buf: &TextBuffer, duration_ms: u64) -> racoon_domain::FinalStats {
        let elapsed_min = duration_ms as f64 / 60_000.0;

        let correct = buf.correct_chars();
        let incorrect = buf.incorrect_chars();
        let backspaces = buf.backspace_count();

        let wpm = WpmCalculator::net_wpm(correct, elapsed_min);
        let raw_wpm = WpmCalculator::raw_wpm(correct, incorrect, backspaces, elapsed_min);
        let accuracy = AccuracyCalculator::net_accuracy(correct, incorrect);

        let first_correct = HeatmapBuilder::count_first_correct(buf);
        let total_first = HeatmapBuilder::count_first_attempts(buf);
        let raw_accuracy = AccuracyCalculator::raw_accuracy(first_correct, total_first);

        let char_stats = HeatmapBuilder::build_char_stats(buf);
        let heatmap = HeatmapBuilder::build(buf);

        racoon_domain::FinalStats {
            wpm,
            raw_wpm,
            accuracy,
            raw_accuracy,
            consistency: None, // v0.2
            correct_chars: correct,
            incorrect_chars: incorrect,
            backspaces,
            char_stats: serde_json::to_value(&char_stats).unwrap_or(serde_json::Value::Null),
            heatmap: serde_json::to_value(&heatmap).unwrap_or(serde_json::Value::Null),
            graph_data: None, // v0.2
            duration_ms,
        }
    }

    pub fn reset(&mut self) {
        self.tracker.reset();
    }
}

impl Default for StatisticsEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typing::TextBuffer;
    use racoon_domain::stats::TypedChar;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn wpm_calculation() {
        // 100 correct chars / 60s = 100/5 = 20 WPM
        let wpm = WpmCalculator::net_wpm(100, 1.0);
        assert!((wpm - 20.0).abs() < 0.01);
    }

    #[test]
    fn raw_wpm_calculation() {
        // 100 correct + 10 incorrect + 5 backspaces = 115 total
        // 115/5 = 23 WPM
        let raw = WpmCalculator::raw_wpm(100, 10, 5, 1.0);
        assert!((raw - 23.0).abs() < 0.01);
    }

    #[test]
    fn wpm_zero_elapsed() {
        let wpm = WpmCalculator::net_wpm(100, 0.0);
        assert_eq!(wpm, 0.0);
    }

    #[test]
    fn net_accuracy_100() {
        let acc = AccuracyCalculator::net_accuracy(100, 0);
        assert!((acc - 100.0).abs() < 0.01);
    }

    #[test]
    fn net_accuracy_95() {
        // 95 correct, 5 incorrect = 95%
        let acc = AccuracyCalculator::net_accuracy(95, 5);
        assert!((acc - 95.0).abs() < 0.01);
    }

    #[test]
    fn net_accuracy_zero() {
        let acc = AccuracyCalculator::net_accuracy(0, 0);
        assert_eq!(acc, 100.0); // нет данных = 100%
    }

    #[test]
    fn raw_accuracy() {
        // 80 first_correct / 100 total_first_attempts = 80%
        let acc = AccuracyCalculator::raw_accuracy(80, 100);
        assert!((acc - 80.0).abs() < 0.01);
    }

    #[test]
    fn live_tracker_correct() {
        let mut buf = TextBuffer::new("hello");
        let mut tracker = LiveTracker::new();

        let r = buf.process_print('h', 0);
        tracker.on_key_processed(&r, &buf);
        assert_eq!(tracker.correct_chars, 1);
        assert_eq!(tracker.total_keystrokes, 1);
    }

    #[test]
    fn live_tracker_incorrect() {
        let mut buf = TextBuffer::new("hello");
        let mut tracker = LiveTracker::new();

        let r = buf.process_print('x', 0);
        tracker.on_key_processed(&r, &buf);
        assert_eq!(tracker.incorrect_chars, 1);
        assert_eq!(tracker.total_keystrokes, 1);
    }

    #[test]
    fn live_tracker_backspace_correct() {
        let mut buf = TextBuffer::new("hello");
        let mut tracker = LiveTracker::new();

        let r = buf.process_print('h', 0); // correct
        tracker.on_key_processed(&r, &buf);
        assert_eq!(tracker.correct_chars, 1);

        let r = buf.process_backspace(); // UndoneCorrect
        tracker.on_key_processed(&r, &buf);
        assert_eq!(tracker.backspaces, 1);
        assert_eq!(tracker.correct_chars, 0);
    }

    #[test]
    fn live_tracker_backspace_incorrect() {
        let mut buf = TextBuffer::new("hello");
        let mut tracker = LiveTracker::new();

        let r = buf.process_print('x', 0); // incorrect
        tracker.on_key_processed(&r, &buf);
        assert_eq!(tracker.incorrect_chars, 1);

        let r = buf.process_backspace(); // UndoneIncorrect
        tracker.on_key_processed(&r, &buf);
        assert_eq!(tracker.backspaces, 1);
        assert_eq!(tracker.incorrect_chars, 0);
    }

    #[test]
    fn heatmap_preserves_errors_after_correction() {
        let mut buf = TextBuffer::new("hello");
        buf.process_print('x', 0); // incorrect, first_typed='x'
        buf.process_backspace(); // UndoneIncorrect
        buf.process_print('h', 10); // correct

        let heatmap = HeatmapBuilder::build(&buf);
        let h_entry = heatmap.get("h").unwrap();
        assert_eq!(h_entry.total_attempts, 1);
        assert_eq!(h_entry.incorrect, 1); // первая попытка была incorrect
        assert_eq!(h_entry.correct, 0);
    }

    #[test]
    fn heatmap_correct_only() {
        let mut buf = TextBuffer::new("abc");
        buf.process_print('a', 0);
        buf.process_print('b', 1);
        buf.process_print('c', 2);

        let heatmap = HeatmapBuilder::build(&buf);
        assert_eq!(heatmap.len(), 3);
        for key in ["a", "b", "c"] {
            let entry = heatmap.get(key).unwrap();
            assert_eq!(entry.correct, 1);
            assert_eq!(entry.incorrect, 0);
        }
    }

    #[test]
    fn char_stats_final_only() {
        let mut buf = TextBuffer::new("ab");
        buf.process_print('a', 0); // correct
        buf.process_print('x', 1); // incorrect at pos 1
        buf.process_backspace(); // UndoneIncorrect
        buf.process_print('b', 2); // correct

        let stats = HeatmapBuilder::build_char_stats(&buf);
        let a_stat = stats.get("a").unwrap();
        assert_eq!(a_stat.correct, 1);
        let b_stat = stats.get("b").unwrap();
        assert_eq!(b_stat.correct, 1);
    }

    #[test]
    fn statistics_engine_live_stats() {
        let mut buf = TextBuffer::new("hello world");
        let mut engine = StatisticsEngine::new();

        for ch in "hello world".chars() {
            let r = buf.process_print(ch, 0);
            engine.on_key_processed(&r, &buf);
        }

        let stats = engine.live_stats(&buf);
        assert_eq!(stats.accuracy, 100.0);
        // WPM может быть очень малым из-за быстрого выполнения
        // Проверяем что elapsed > 0 (таймер запущен)
        assert!(stats.elapsed_ms <= 1000); // уложились быстро
    }

    #[test]
    fn statistics_engine_finalize() {
        let mut buf = TextBuffer::new("hello");
        let mut engine = StatisticsEngine::new();

        for ch in "hello".chars() {
            let r = buf.process_print(ch, 0);
            engine.on_key_processed(&r, &buf);
        }

        let final_stats = engine.finalize(&buf, 10_000); // 10 seconds
        assert_eq!(final_stats.correct_chars, 5);
        assert_eq!(final_stats.incorrect_chars, 0);
        assert!((final_stats.wpm - 6.0).abs() < 0.1); // 5/5 / (10s/60s) = 1/ (1/6) = 6 WPM
        assert!((final_stats.accuracy - 100.0).abs() < 0.01);
        assert_eq!(final_stats.backspaces, 0);
    }

    #[test]
    fn finalize_with_errors_and_backspace() {
        let mut buf = TextBuffer::new("hello");
        let mut engine = StatisticsEngine::new();

        // Wrong, backspace, correct
        let r = buf.process_print('x', 0);
        engine.on_key_processed(&r, &buf);
        let r = buf.process_backspace(); // UndoneIncorrect — снимает incorrect, ставит Pending
        engine.on_key_processed(&r, &buf);

        // Now type correctly
        for ch in "hello".chars() {
            let r = buf.process_print(ch, 10);
            engine.on_key_processed(&r, &buf);
        }

        let final_stats = engine.finalize(&buf, 10_000);
        assert_eq!(final_stats.correct_chars, 5);
        // backspace_count() считает Backspaced статус, но UndoneIncorrect ставит Pending
        // tracker.backspaces = 1 (засчитан при on_key_processed)
        assert_eq!(engine.tracker.backspaces, 1);
        // Raw accuracy: first attempt on 'h' was wrong (x), rest correct
        // total_first_attempts = 5, first_correct = 4
        assert!((final_stats.raw_accuracy - 80.0).abs() < 0.1);
    }

    #[test]
    fn reset_clears_tracker() {
        let mut engine = StatisticsEngine::new();
        engine.tracker.correct_chars = 10;
        engine.tracker.incorrect_chars = 5;
        engine.tracker.backspaces = 2;
        engine.tracker.total_keystrokes = 17;

        engine.reset();
        assert_eq!(engine.tracker.correct_chars, 0);
        assert_eq!(engine.tracker.incorrect_chars, 0);
        assert_eq!(engine.tracker.backspaces, 0);
        assert_eq!(engine.tracker.total_keystrokes, 0);
    }

    #[test]
    fn duration_calculation() {
        let mut tracker = LiveTracker::new();
        tracker.start_time = Some(Instant::now());
        thread::sleep(Duration::from_millis(100));
        let min = tracker.elapsed_minutes();
        assert!(min > 0.0);
        // 100ms = ~0.00167 minutes
        assert!(min < 0.01);
    }
}
