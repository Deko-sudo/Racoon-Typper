// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

use racoon_data::profile_transfer::{
    apply_profile_import, export_profile, parse_profile_export, plan_profile_import,
    ProfileImportPolicy, PROFILE_FORMAT, PROFILE_SCHEMA_VERSION,
};
use racoon_data::repository::{
    CustomTextRepository, SqliteCustomTextRepository, SqliteTestRepository, TestRepository,
};
use racoon_data::Database;
use racoon_domain::{SessionId, TestRecord};

fn empty_document() -> String {
    format!(
        r#"{{"format":"{PROFILE_FORMAT}","schema_version":{PROFILE_SCHEMA_VERSION},"exported_at":"2026-08-05T12:00:00Z","application_version":"1.1.0","profile":{{"tests":[],"personal_bests":[],"daily_stats":[],"streaks":[],"custom_texts":[],"lesson_progress":[]}}}}"#
    )
}

#[test]
fn parses_the_current_versioned_empty_profile() {
    let profile = parse_profile_export(empty_document().as_bytes()).expect("valid profile export");
    assert_eq!(profile.format, PROFILE_FORMAT);
    assert!(profile.profile.tests.is_empty());
}

#[test]
fn rejects_unknown_fields_and_oversized_documents() {
    let document = empty_document().replace(
        "\"lesson_progress\":[]",
        "\"lesson_progress\":[],\"surprise\":true",
    );
    assert!(parse_profile_export(document.as_bytes()).is_err());
    assert!(parse_profile_export(&vec![
        b' ';
        racoon_data::profile_transfer::MAX_PROFILE_BYTES + 1
    ])
    .is_err());
}

#[test]
fn rejects_unsupported_versions_and_invalid_custom_texts() {
    assert!(parse_profile_export(
        empty_document()
            .replace("\"schema_version\":1", "\"schema_version\":99")
            .as_bytes()
    )
    .is_err());
    let document = empty_document().replace(
        "\"custom_texts\":[]",
        "\"custom_texts\":[{\"name\":\"\",\"text\":\"body\",\"language\":\"en\"}]",
    );
    assert!(parse_profile_export(document.as_bytes()).is_err());
}

fn record(session_id: &str) -> TestRecord {
    TestRecord {
        session_id: SessionId::from(session_id),
        created_at: "2026-08-05T12:00:00Z".to_string(),
        mode_type: "time".to_string(),
        mode_config: serde_json::json!({"duration": 30}),
        language: "en".to_string(),
        text_length: 50,
        duration_ms: 30_000,
        wpm: 42.0,
        raw_wpm: 45.0,
        accuracy: 96.0,
        raw_accuracy: 94.0,
        consistency: Some(90.0),
        correct_chars: 48,
        incorrect_chars: 2,
        backspaces: 1,
        char_stats: serde_json::json!({}),
        heatmap_data: serde_json::json!({}),
        graph_data: None,
        is_pb: false,
        tags: "import".to_string(),
    }
}

#[test]
fn profile_transfer_plans_without_writes_then_merges_or_replaces_atomically() {
    let source = Database::open_in_memory().expect("source database");
    source
        .with_transaction(|tx| {
            SqliteTestRepository::new(tx).save_test(record("exported-session"))?;
            SqliteCustomTextRepository::new(tx).save_with_language("Exported", "body", "en")?;
            Ok(())
        })
        .expect("seed source");
    let document = serde_json::to_vec(
        &export_profile(&source, "1.1.0", "2026-08-05T12:00:00Z").expect("export profile"),
    )
    .expect("serialize export");

    let target = Database::open_in_memory().expect("target database");
    target
        .with_transaction(|tx| SqliteTestRepository::new(tx).save_test(record("local-session")))
        .expect("seed target");

    let merge_plan =
        plan_profile_import(&target, &document, ProfileImportPolicy::Merge).expect("plan merge");
    assert_eq!(merge_plan.tests.to_insert, 1);
    assert_eq!(merge_plan.tests.existing, 0);
    assert_eq!(
        target
            .with_connection(|conn| SqliteTestRepository::new(conn).get_count(None))
            .expect("count after dry run"),
        1,
        "planning must not write"
    );

    let mut invalid_export: serde_json::Value =
        serde_json::from_slice(&document).expect("parse exported profile");
    invalid_export["application_version"] = serde_json::Value::String(String::new());
    let invalid_document = serde_json::to_vec(&invalid_export).expect("serialize invalid profile");
    assert!(
        apply_profile_import(&target, &invalid_document, ProfileImportPolicy::Replace).is_err()
    );
    assert_eq!(
        target
            .with_connection(|conn| SqliteTestRepository::new(conn).get_count(None))
            .expect("count after rejected import"),
        1,
        "semantic validation must run before replace deletes existing data"
    );

    apply_profile_import(&target, &document, ProfileImportPolicy::Merge).expect("merge import");
    assert_eq!(
        target
            .with_connection(|conn| SqliteTestRepository::new(conn).get_count(None))
            .expect("count after merge"),
        2
    );

    let replace_plan = apply_profile_import(&target, &document, ProfileImportPolicy::Replace)
        .expect("replace import");
    assert_eq!(replace_plan.tests.to_insert, 1);
    assert_eq!(
        target
            .with_connection(|conn| SqliteTestRepository::new(conn).get_count(None))
            .expect("count after replace"),
        1
    );
}
