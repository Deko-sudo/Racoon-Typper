//! E2E integration tests — full test lifecycle.

use racoon_core::{CoreEngine, CustomMode, KeyResult, QuoteMode, TimeMode, WordsMode};

#[test]
fn e2e_start_and_complete_time_test() {
    let mut engine = CoreEngine::new();
    let mode = Box::new(TimeMode::new("hello".to_string(), "en".to_string(), 30));
    let info = engine.start_test_mode("s1".to_string(), mode);

    assert_eq!(info.text, "hello");
    assert_eq!(info.mode_type, "time");
    assert_eq!(info.language, "en");

    // Type all characters
    for ch in "hello".chars() {
        let key_event = racoon_core::KeyEvent {
            key: ch.to_string(),
            code: format!("Key{}", ch.to_uppercase()),
            timestamp: 100,
        };
        let output = engine.process_key(&key_event);
        assert_ne!(output.key_result, KeyResult::Noop);
    }

    assert!(engine.is_complete());
}

#[test]
fn e2e_start_and_complete_words_test() {
    let mut engine = CoreEngine::new();
    let mode = Box::new(WordsMode::new("test word".to_string(), "en".to_string(), 2));
    let info = engine.start_test_mode("s2".to_string(), mode);

    assert_eq!(info.mode_type, "words");
    assert_eq!(info.text, "test word");

    // Type all characters
    for ch in "test word".chars() {
        let key_event = racoon_core::KeyEvent {
            key: ch.to_string(),
            code: "KeyA".to_string(),
            timestamp: 100,
        };
        engine.process_key(&key_event);
    }

    assert!(engine.is_complete());
}

#[test]
fn e2e_start_and_complete_quote_test() {
    let mut engine = CoreEngine::new();
    let mode = Box::new(QuoteMode::new(
        "Hello world".to_string(),
        "en".to_string(),
        Some(0),
    ));
    let info = engine.start_test_mode("s3".to_string(), mode);

    assert_eq!(info.mode_type, "quote");

    for ch in "Hello world".chars() {
        let key_event = racoon_core::KeyEvent {
            key: ch.to_string(),
            code: "KeyA".to_string(),
            timestamp: 100,
        };
        engine.process_key(&key_event);
    }

    assert!(engine.is_complete());
}

#[test]
fn e2e_start_and_complete_custom_test() {
    let mut engine = CoreEngine::new();
    let mode = Box::new(CustomMode::new("custom text".to_string(), "en".to_string()));
    let info = engine.start_test_mode("s4".to_string(), mode);

    assert_eq!(info.mode_type, "custom");

    for ch in "custom text".chars() {
        let key_event = racoon_core::KeyEvent {
            key: ch.to_string(),
            code: "KeyA".to_string(),
            timestamp: 100,
        };
        engine.process_key(&key_event);
    }

    assert!(engine.is_complete());
}

#[test]
fn e2e_save_result_to_db() {
    use racoon_data::repository::{SqliteTestRepository, TestRepository};
    use racoon_data::Database;
    use racoon_domain::TestRecord;

    let db = Database::open_in_memory().unwrap();
    let conn = db.conn();
    let repo = SqliteTestRepository::new(&conn);

    let record = TestRecord {
        created_at: chrono::Utc::now().to_rfc3339(),
        mode_type: "time".to_string(),
        mode_config: serde_json::json!({"duration": 30}),
        language: "en".to_string(),
        text_length: 50,
        duration_ms: 30000,
        wpm: 45.5,
        raw_wpm: 50.0,
        accuracy: 95.0,
        raw_accuracy: 90.0,
        consistency: None,
        correct_chars: 95,
        incorrect_chars: 5,
        backspaces: 2,
        char_stats: serde_json::json!({}),
        heatmap_data: serde_json::json!({}),
        graph_data: None,
        is_pb: false,
        tags: "".to_string(),
    };

    let id = repo.save_test(record).unwrap();
    assert!(id > 0);

    let history = repo.get_history(10, 0, None).unwrap();
    assert_eq!(history.len(), 1);
    assert!((history[0].wpm - 45.5).abs() < 0.01);
}

#[test]
fn e2e_create_and_start_custom_text() {
    use racoon_data::repository::{CustomTextRepository, SqliteCustomTextRepository};
    use racoon_data::Database;

    let db = Database::open_in_memory().unwrap();
    let conn = db.conn();
    let repo = SqliteCustomTextRepository::new(&conn);

    // Create
    let id = repo.save("My Text", "The quick brown fox").unwrap();
    assert!(id > 0);

    // Retrieve
    let ct = repo.get_by_id(id).unwrap();
    assert_eq!(ct.name, "My Text");
    assert_eq!(ct.text, "The quick brown fox");

    // Increment use
    repo.increment_use(id).unwrap();
    let ct = repo.get_by_id(id).unwrap();
    assert_eq!(ct.use_count, 1);

    // Start test with this text
    let mut engine = CoreEngine::new();
    let mode = Box::new(CustomMode::new(ct.text.clone(), "en".to_string()));
    let info = engine.start_test_mode("s5".to_string(), mode);
    assert_eq!(info.text, "The quick brown fox");
    assert_eq!(info.mode_type, "custom");
}

#[test]
fn e2e_personal_best_update() {
    use racoon_data::repository::{
        PersonalBestsRepository, SqlitePersonalBestsRepository, SqliteTestRepository,
        TestRepository,
    };
    use racoon_data::Database;
    use racoon_domain::TestRecord;

    let db = Database::open_in_memory().unwrap();
    let conn = db.conn();
    let test_repo = SqliteTestRepository::new(&conn);
    let pb_repo = SqlitePersonalBestsRepository::new(&conn);

    // Save first test
    let record = TestRecord {
        created_at: chrono::Utc::now().to_rfc3339(),
        mode_type: "time".to_string(),
        mode_config: serde_json::json!({"duration": 30}),
        language: "en".to_string(),
        text_length: 50,
        duration_ms: 30000,
        wpm: 40.0,
        raw_wpm: 45.0,
        accuracy: 95.0,
        raw_accuracy: 90.0,
        consistency: None,
        correct_chars: 95,
        incorrect_chars: 5,
        backspaces: 2,
        char_stats: serde_json::json!({}),
        heatmap_data: serde_json::json!({}),
        graph_data: None,
        is_pb: false,
        tags: "".to_string(),
    };
    let test_id = test_repo.save_test(record).unwrap();

    // Check PB — first time
    let updates = pb_repo
        .check_and_update("time", r#"{"duration":30}"#, 40.0, 95.0, test_id)
        .unwrap();
    assert_eq!(updates.len(), 2); // wpm + accuracy (first record)

    // Save better test
    let record2 = TestRecord {
        created_at: chrono::Utc::now().to_rfc3339(),
        mode_type: "time".to_string(),
        mode_config: serde_json::json!({"duration": 30}),
        language: "en".to_string(),
        text_length: 50,
        duration_ms: 30000,
        wpm: 50.0,
        raw_wpm: 55.0,
        accuracy: 90.0,
        raw_accuracy: 85.0,
        consistency: None,
        correct_chars: 90,
        incorrect_chars: 10,
        backspaces: 3,
        char_stats: serde_json::json!({}),
        heatmap_data: serde_json::json!({}),
        graph_data: None,
        is_pb: false,
        tags: "".to_string(),
    };
    let test_id2 = test_repo.save_test(record2).unwrap();

    // Check PB — better WPM
    let updates = pb_repo
        .check_and_update("time", r#"{"duration":30}"#, 50.0, 90.0, test_id2)
        .unwrap();
    assert_eq!(updates.len(), 1); // only wpm improved
    assert_eq!(updates[0].metric, "wpm");
    assert_eq!(updates[0].previous, Some(40.0));
    assert_eq!(updates[0].new, 50.0);

    // Verify PB in DB
    let bests = pb_repo.get_bests(None).unwrap();
    assert_eq!(bests.len(), 1);
    assert!((bests[0].best_wpm - 50.0).abs() < 0.01);
}

#[test]
fn e2e_abort_and_restart() {
    let mut engine = CoreEngine::new();
    let mode = Box::new(TimeMode::new("hello".to_string(), "en".to_string(), 30));
    engine.start_test_mode("s1".to_string(), mode);

    // Type one char
    let key_event = racoon_core::KeyEvent {
        key: "h".to_string(),
        code: "KeyH".to_string(),
        timestamp: 100,
    };
    engine.process_key(&key_event);
    assert_eq!(engine.caret_position(), 1);

    // Abort
    engine.abort();
    assert!(!engine.is_active());

    // Start new test
    let mode = Box::new(TimeMode::new("world".to_string(), "en".to_string(), 30));
    let info = engine.start_test_mode("s2".to_string(), mode);
    assert_eq!(info.text, "world");
    assert_eq!(engine.caret_position(), 0);
}

#[test]
fn e2e_backspace_during_test() {
    let mut engine = CoreEngine::new();
    let mode = Box::new(TimeMode::new("hi".to_string(), "en".to_string(), 30));
    engine.start_test_mode("s1".to_string(), mode);

    // Type 'h'
    engine.process_key(&racoon_core::KeyEvent {
        key: "h".to_string(),
        code: "KeyH".to_string(),
        timestamp: 100,
    });
    assert_eq!(engine.caret_position(), 1);

    // Backspace
    let output = engine.process_key(&racoon_core::KeyEvent {
        key: "Backspace".to_string(),
        code: "Backspace".to_string(),
        timestamp: 200,
    });
    assert_eq!(engine.caret_position(), 0);
    assert_eq!(output.key_result, KeyResult::UndoneCorrect);

    // Type 'h' again
    engine.process_key(&racoon_core::KeyEvent {
        key: "h".to_string(),
        code: "KeyH".to_string(),
        timestamp: 300,
    });

    // Type 'i'
    engine.process_key(&racoon_core::KeyEvent {
        key: "i".to_string(),
        code: "KeyI".to_string(),
        timestamp: 400,
    });

    assert!(engine.is_complete());
}
