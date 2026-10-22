use std::time::{Duration, Instant};

use racoon_core::{KeyAction, KeyClassifier, StatisticsEngine, TextBuffer};

#[test]
fn audit_live_raw_wpm_retains_corrected_error_presses() {
    let mut buffer = TextBuffer::new("abc");
    let mut stats = StatisticsEngine::new();
    let result = buffer.process_print('x', 0);
    stats.on_key_processed_at(&result, &buffer, 0);
    let result = buffer.process_backspace();
    stats.on_key_processed_at(&result, &buffer, 100);
    let result = buffer.process_print('a', 200);
    stats.on_key_processed_at(&result, &buffer, 200);
    let result = buffer.process_print('b', 300);
    stats.on_key_processed_at(&result, &buffer, 300);
    stats.tracker.start_time = Some(Instant::now() - Duration::from_secs(60));
    let live = stats.live_stats(&buffer);
    let final_stats = stats.finalize(&buffer, 60_000);
    assert!(
        (live.raw_wpm - final_stats.raw_wpm).abs() < 0.01,
        "live {}, final {}",
        live.raw_wpm,
        final_stats.raw_wpm
    );
}

#[test]
fn audit_consistency_excludes_incomplete_last_bucket() {
    let mut buffer = TextBuffer::new(&"a".repeat(30));
    let mut stats = StatisticsEngine::new();
    for index in 0..=20 {
        let result = buffer.process_print('a', index * 100);
        stats.on_key_processed_at(&result, &buffer, index * 100);
    }
    let score = stats.finalize(&buffer, 2000).consistency.expect("score");
    assert!(
        (score - 100.0).abs() < 0.01,
        "uniform typing scored {score}"
    );
}

#[test]
fn audit_consistency_does_not_treat_backspace_as_a_print() {
    let score = |backspace| {
        let mut buffer = TextBuffer::new("aaaaa");
        let mut stats = StatisticsEngine::new();
        for timestamp in [0, 1000, 2000, 3000] {
            let result = buffer.process_print('a', timestamp);
            stats.on_key_processed_at(&result, &buffer, timestamp);
            if backspace && timestamp == 1000 {
                let result = buffer.process_backspace();
                stats.on_key_processed_at(&result, &buffer, 1500);
            }
        }
        stats.finalize(&buffer, 3000).consistency.expect("score")
    };
    assert!(
        (score(false) - score(true)).abs() < 0.01,
        "same print cadence: without {}, with {}",
        score(false),
        score(true)
    );
}

#[test]
fn audit_consistency_counts_silence_before_time_completion() {
    let mut buffer = TextBuffer::new("aaaaa");
    let mut stats = StatisticsEngine::new();
    for timestamp in [0, 100, 200] {
        let result = buffer.process_print('a', timestamp);
        stats.on_key_processed_at(&result, &buffer, timestamp);
    }
    let score = stats.finalize(&buffer, 10_000).consistency.expect("score");
    assert!(score < 50.0, "nine seconds of silence scored {score}");
}

#[test]
fn audit_named_keys_and_multiscalar_values_are_not_truncated() {
    for key in [
        "ArrowLeft",
        "Delete",
        "Dead",
        "Unidentified",
        "F1",
        "ab",
        "e\u{301}",
    ] {
        assert_eq!(
            KeyClassifier::classify(key, "Unidentified"),
            KeyAction::Ignore,
            "key {key} must not become its first letter"
        );
    }
    for ch in ['ф', '字', '🦝', ' '] {
        assert_eq!(
            KeyClassifier::classify(&ch.to_string(), "Unidentified"),
            KeyAction::Print(ch)
        );
    }
}
