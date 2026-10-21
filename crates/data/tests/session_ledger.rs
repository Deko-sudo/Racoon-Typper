//! Temporary-database integration coverage for the V006–V008 durable ledgers.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

use chrono::{DateTime, Utc};
use racoon_application::{
    CompletionIntent, CompletionIntentFingerprint, CompletionIntentLoadOutcome,
    CompletionIntentMetadata, CompletionPolicySnapshot, DurableSessionState,
    FinalizationClaimOutcome, FinalizationCommitOutcome, FinalizationLedger,
    FinalizationLedgerClaimOutcome, FinalizationLedgerState, FinalizationLoadOutcome,
    FinalizationQuarantineReason, LedgerMutationOutcome, QuarantineReason, SessionCompletion,
    SessionFinalizer, SessionRecoveryLedger, StartedSession, MAX_COMPLETION_INTENT_PAYLOAD_BYTES,
};
use racoon_core::ReplayFrame;
#[cfg(feature = "test-support")]
use racoon_data::repository::session_finalizer::FinalizerFailurePoint;
use racoon_data::{
    Database, SqliteFinalizationLedger, SqliteSessionFinalizer, SqliteSessionRecoveryLedger,
};
use racoon_domain::{CharStatus, FinalStats, SessionId};
use refinery::{Migration, Runner};
use rusqlite::{params, Connection};
use serde_json::{json, Value};

static NEXT_TEMP_DB: AtomicU64 = AtomicU64::new(0);

const SESSION_A: &str = "018f0c2e-7b8d-7abc-8def-0123456789aa";
const SESSION_B: &str = "018f0c2e-7b8d-7abc-8def-0123456789ab";
const SESSION_C: &str = "018f0c2e-7b8d-7abc-8def-0123456789ac";
const SESSION_D: &str = "018f0c2e-7b8d-7abc-8def-0123456789ad";
const SESSION_E: &str = "018f0c2e-7b8d-7abc-8def-0123456789ae";
const SESSION_F: &str = "018f0c2e-7b8d-7abc-8def-0123456789af";

fn timestamp(value: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(value)
        .expect("fixture timestamp")
        .with_timezone(&Utc)
}

fn session_id(value: &str) -> SessionId {
    SessionId::parse(value).expect("fixture UUIDv7")
}

fn started(session_id: SessionId, started_at: &str) -> StartedSession {
    StartedSession::new(
        session_id,
        "custom",
        json!({"language": "en", "kind": "sanitized"}),
        "en",
        timestamp(started_at),
    )
    .expect("fixture start")
}

fn intent(session_id: SessionId, language: &str, mode_config: Value) -> CompletionIntent {
    intent_with_policy(
        session_id,
        language,
        mode_config,
        CompletionPolicySnapshot::time(15.0),
    )
}

fn intent_with_policy(
    session_id: SessionId,
    language: &str,
    mode_config: Value,
    policy: CompletionPolicySnapshot,
) -> CompletionIntent {
    intent_with_policy_and_duration(session_id, language, mode_config, policy, 1_000)
}

fn intent_with_policy_and_duration(
    session_id: SessionId,
    language: &str,
    mode_config: Value,
    policy: CompletionPolicySnapshot,
    duration_ms: u64,
) -> CompletionIntent {
    CompletionIntent::from_completion(
        &SessionCompletion {
            session_id,
            completed_at: timestamp("2026-07-16T12:00:00Z"),
            final_stats: FinalStats {
                wpm: 60.0,
                raw_wpm: 61.0,
                accuracy: 0.98,
                raw_accuracy: 0.99,
                consistency: Some(0.9),
                correct_chars: 5,
                incorrect_chars: 0,
                backspaces: 0,
                char_stats: json!({"a": {"correct": 1, "incorrect": 0}}),
                heatmap: json!({"a": {"count": 1}}),
                graph_data: Some(json!([60.0])),
                duration_ms,
            },
            mode_type: "custom".to_string(),
            mode_config,
            language: language.to_string(),
            text_length: 5,
            replay_frames: vec![ReplayFrame {
                timestamp_ms: 10,
                key: "typed-secret-key".to_string(),
                caret_pos: 1,
                char_status: CharStatus::Correct,
                expected_char: 'a',
                typed_char: Some('a'),
            }],
            lesson_id: None,
        },
        policy,
    )
    .expect("fixture completion intent")
}

fn prepare_finalization_pending(database: &Database, id: SessionId) -> CompletionIntent {
    prepare_finalization_pending_with_policy(database, id, CompletionPolicySnapshot::time(15.0))
}

fn prepare_finalization_pending_with_policy(
    database: &Database,
    id: SessionId,
    policy: CompletionPolicySnapshot,
) -> CompletionIntent {
    prepare_finalization_pending_with_policy_and_duration(database, id, policy, 1_000)
}

fn prepare_finalization_pending_with_policy_and_duration(
    database: &Database,
    id: SessionId,
    policy: CompletionPolicySnapshot,
    duration_ms: u64,
) -> CompletionIntent {
    let recovery = SqliteSessionRecoveryLedger::new(database);
    let completion = intent_with_policy_and_duration(
        id.clone(),
        "en",
        json!({"language": "en"}),
        policy,
        duration_ms,
    );
    recovery
        .record_started(&started(id.clone(), "2026-07-16T10:00:00Z"))
        .expect("start");
    recovery
        .record_completion_intent(&completion)
        .expect("intent");
    assert_eq!(
        recovery.claim_completion_for_finalization(&id, completion.fingerprint()),
        Ok(FinalizationClaimOutcome::Claimed)
    );
    completion
}

fn prepare_lesson_finalization_pending(database: &Database, id: SessionId) -> CompletionIntent {
    let recovery = SqliteSessionRecoveryLedger::new(database);
    recovery
        .record_started(&started(id.clone(), "2026-07-16T10:00:00Z"))
        .expect("start");
    let completion = CompletionIntent::from_completion(
        &SessionCompletion {
            session_id: id.clone(),
            completed_at: timestamp("2026-07-16T12:00:00Z"),
            final_stats: FinalStats {
                wpm: 60.0,
                raw_wpm: 61.0,
                accuracy: 0.98,
                raw_accuracy: 0.99,
                consistency: Some(0.9),
                correct_chars: 5,
                incorrect_chars: 0,
                backspaces: 0,
                char_stats: json!({}),
                heatmap: json!({}),
                graph_data: None,
                duration_ms: 1_000,
            },
            mode_type: "lesson".to_owned(),
            mode_config: json!({"language": "en", "lesson_id": "en_m1_l1"}),
            language: "en".to_owned(),
            text_length: 5,
            replay_frames: vec![],
            lesson_id: Some("en_m1_l1".to_owned()),
        },
        CompletionPolicySnapshot::time(15.0),
    )
    .expect("lesson intent");
    recovery
        .record_completion_intent(&completion)
        .expect("intent");
    assert_eq!(
        recovery.claim_completion_for_finalization(&id, completion.fingerprint()),
        Ok(FinalizationClaimOutcome::Claimed)
    );
    completion
}

fn finalization_snapshot(
    database: &Database,
    session_id: &SessionId,
) -> (
    String,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
) {
    let connection = database.conn();
    connection
        .query_row(
            "SELECT session_id, fingerprint, state, claimed_at, committed_at, quarantine_reason
             FROM session_finalizations WHERE session_id = ?1",
            params![session_id.as_str()],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .expect("finalization snapshot")
}

fn temporary_database_path(name: &str) -> PathBuf {
    let sequence = NEXT_TEMP_DB.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "racoon-session-ledger-{name}-{}-{sequence}.db",
        std::process::id()
    ))
}

fn remove_database(path: &PathBuf) {
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(path.with_extension("db-wal"));
    let _ = std::fs::remove_file(path.with_extension("db-shm"));
}

fn ledger_snapshot(
    database: &Database,
    session_id: &SessionId,
) -> (String, String, String, String, String, String, String) {
    let connection = database.conn();
    connection
        .query_row(
            "SELECT state, mode_type, mode_descriptor, language, created_at, updated_at,
                    coalesce(quarantine_reason, '')
             FROM session_ledger WHERE session_id = ?1",
            params![session_id.as_str()],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                ))
            },
        )
        .expect("ledger snapshot")
}

fn intent_snapshot(
    database: &Database,
    session_id: &SessionId,
) -> (i64, i64, String, Vec<u8>, i64, String) {
    let connection = database.conn();
    connection
        .query_row(
            "SELECT canonicalization_version, payload_version, fingerprint, canonical_payload,
                    payload_byte_length, created_at
             FROM session_completion_intents WHERE session_id = ?1",
            params![session_id.as_str()],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .expect("intent snapshot")
}

fn historical_refinery_migrations(version: usize) -> Vec<Migration> {
    [
        (
            "V001__initial.sql",
            include_str!("../migrations/V001__initial.sql"),
        ),
        (
            "V002__lesson_language.sql",
            include_str!("../migrations/V002__lesson_language.sql"),
        ),
        (
            "V003__replays.sql",
            include_str!("../migrations/V003__replays.sql"),
        ),
        (
            "V004__custom_text_language.sql",
            include_str!("../migrations/V004__custom_text_language.sql"),
        ),
        (
            "V005__session_identity.sql",
            include_str!("../migrations/V005__session_identity.sql"),
        ),
        (
            "V006__session_ledger.sql",
            include_str!("../migrations/V006__session_ledger.sql"),
        ),
        (
            "V007__session_completion_intents.sql",
            include_str!("../migrations/V007__session_completion_intents.sql"),
        ),
    ]
    .into_iter()
    .take(version)
    .map(|(name, sql)| Migration::unapplied(name, sql).expect("valid historical migration"))
    .collect()
}

fn insert_historical_test(connection: &Connection, has_session_identity: bool) {
    if has_session_identity {
        connection
            .execute(
                "INSERT INTO tests (
                    session_id, created_at, mode_type, mode_config, language, text_length,
                    duration_ms, wpm, raw_wpm, accuracy, raw_accuracy, consistency,
                    correct_chars, incorrect_chars, backspaces, char_stats, heatmap_data,
                    graph_data, is_pb, tags
                 ) VALUES (
                    'legacy-test-0000000000000001', '2026-01-01T00:00:00Z', 'time', '{}',
                    'en', 1, 1, 1.0, 1.0, 1.0, 1.0, NULL, 1, 0, 0, '{}', '{}', NULL, 0, ''
                 )",
                [],
            )
            .expect("V005 historical test");
    } else {
        connection
            .execute(
                "INSERT INTO tests (
                    created_at, mode_type, mode_config, language, text_length, duration_ms,
                    wpm, raw_wpm, accuracy, raw_accuracy, consistency, correct_chars,
                    incorrect_chars, backspaces, char_stats, heatmap_data, graph_data, is_pb, tags
                 ) VALUES (
                    '2026-01-01T00:00:00Z', 'time', '{}', 'en', 1, 1,
                    1.0, 1.0, 1.0, 1.0, NULL, 1, 0, 0, '{}', '{}', NULL, 0, ''
                 )",
                [],
            )
            .expect("pre-V005 historical test");
    }
}

fn insert_raw_intent(
    database: &Database,
    session_id: &SessionId,
    canonicalization_version: i64,
    payload_version: i64,
    fingerprint: &str,
    payload: &[u8],
    payload_length: i64,
) {
    let connection = database.conn();
    connection
        .execute(
            "INSERT INTO session_completion_intents (
                session_id, canonicalization_version, payload_version, fingerprint,
                canonical_payload, payload_byte_length, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, '2026-07-16T12:00:00Z')",
            params![
                session_id.as_str(),
                canonicalization_version,
                payload_version,
                fingerprint,
                payload,
                payload_length,
            ],
        )
        .expect("raw fixture intent");
}

fn replace_intent_table_with_corruption_fixture(database: &Database) {
    let connection = database.conn();
    connection
        .pragma_update(None, "foreign_keys", "OFF")
        .expect("disable foreign keys for corruption fixture");
    connection
        .execute_batch(
            "DROP TRIGGER session_completion_intents_immutable;
             DROP TABLE session_completion_intents;
             CREATE TABLE session_completion_intents (
                session_id,
                canonicalization_version,
                payload_version,
                fingerprint,
                canonical_payload,
                payload_byte_length,
                created_at
             );",
        )
        .expect("corruption fixture table");
    connection
        .pragma_update(None, "foreign_keys", "ON")
        .expect("restore foreign keys after corruption fixture");
}

fn insert_corrupt_intent_header(database: &Database, session_id: &SessionId, fingerprint: &str) {
    let connection = database.conn();
    connection
        .execute(
            "INSERT INTO session_completion_intents VALUES (
                ?1, 1, 1, ?2, X'7B7D', 2, '2026-07-16T12:00:00Z'
             )",
            params![session_id.as_str(), fingerprint],
        )
        .expect("corrupt intent header fixture");
}

fn replace_finalization_table_with_corruption_fixture(database: &Database) {
    let connection = database.conn();
    connection
        .execute_batch(
            "DROP TRIGGER session_finalizations_initial_state_pending;
             DROP TRIGGER session_finalizations_not_replaceable;
             DROP TRIGGER session_finalizations_not_deletable;
             DROP TRIGGER session_finalizations_identity_immutable;
             DROP TRIGGER session_finalizations_transition_guard;
             DROP TABLE session_finalizations;
             CREATE TABLE session_finalizations (
                session_id,
                fingerprint,
                state,
                claimed_at,
                committed_at,
                quarantine_reason
             );",
        )
        .expect("corruption fixture finalization table");
}

#[test]
fn clean_schema_contains_v006_v007_and_v008_tables_and_indexes() {
    let database = Database::open_in_memory().expect("clean database");
    let connection = database.conn();
    for table in [
        "session_ledger",
        "session_completion_intents",
        "session_finalizations",
    ] {
        let count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                params![table],
                |row| row.get(0),
            )
            .expect("table lookup");
        assert_eq!(count, 1, "missing {table}");
    }
    for index in [
        "idx_session_ledger_recovery_order",
        "idx_session_ledger_state_recovery_order",
        "idx_session_completion_intents_session_fingerprint",
        "idx_session_finalizations_diagnostic_order",
    ] {
        let count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = ?1",
                params![index],
                |row| row.get(0),
            )
            .expect("index lookup");
        assert_eq!(count, 1, "missing {index}");
    }
}

#[test]
fn every_historical_migration_start_reaches_v008_without_backfill() {
    let migrations = [
        include_str!("../migrations/V001__initial.sql"),
        include_str!("../migrations/V002__lesson_language.sql"),
        include_str!("../migrations/V003__replays.sql"),
        include_str!("../migrations/V004__custom_text_language.sql"),
        include_str!("../migrations/V005__session_identity.sql"),
        include_str!("../migrations/V006__session_ledger.sql"),
        include_str!("../migrations/V007__session_completion_intents.sql"),
        include_str!("../migrations/V008__session_finalizations.sql"),
    ];

    for starting_version in 1..=7 {
        let connection = rusqlite::Connection::open_in_memory().expect("temporary SQLite");
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .expect("foreign keys");
        for migration in &migrations[..starting_version] {
            connection
                .execute_batch(migration)
                .expect("historical migration");
        }
        for migration in &migrations[starting_version..] {
            connection
                .execute_batch(migration)
                .expect("forward migration");
        }
        let ledger_rows: i64 = connection
            .query_row("SELECT COUNT(*) FROM session_ledger", [], |row| row.get(0))
            .expect("ledger count");
        assert_eq!(
            ledger_rows, 0,
            "V{starting_version} must not backfill tests"
        );
        let foreign_keys: i64 = connection
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .expect("foreign key pragma");
        assert_eq!(foreign_keys, 1);
    }
}

#[test]
fn v005_history_is_preserved_without_creating_ledger_rows() {
    let connection = rusqlite::Connection::open_in_memory().expect("temporary SQLite");
    for migration in [
        include_str!("../migrations/V001__initial.sql"),
        include_str!("../migrations/V002__lesson_language.sql"),
        include_str!("../migrations/V003__replays.sql"),
        include_str!("../migrations/V004__custom_text_language.sql"),
    ] {
        connection
            .execute_batch(migration)
            .expect("historical migration");
    }
    connection
        .execute(
            "INSERT INTO tests (
                created_at, mode_type, mode_config, language, text_length, duration_ms,
                wpm, raw_wpm, accuracy, raw_accuracy, consistency, correct_chars,
                incorrect_chars, backspaces, char_stats, heatmap_data, graph_data, is_pb, tags
             ) VALUES ('2026-01-01T00:00:00Z', 'custom', '{}', 'en', 1, 1,
                1.0, 1.0, 1.0, 1.0, NULL, 1, 0, 0, '{}', '{}', NULL, 0, '')",
            [],
        )
        .expect("historical test");
    for migration in [
        include_str!("../migrations/V005__session_identity.sql"),
        include_str!("../migrations/V006__session_ledger.sql"),
        include_str!("../migrations/V007__session_completion_intents.sql"),
        include_str!("../migrations/V008__session_finalizations.sql"),
    ] {
        connection
            .execute_batch(migration)
            .expect("forward migration");
    }
    let historical_session_id: String = connection
        .query_row("SELECT session_id FROM tests WHERE id = 1", [], |row| {
            row.get(0)
        })
        .expect("legacy session id");
    assert_eq!(historical_session_id, "legacy-test-0000000000000001");
    let ledger_rows: i64 = connection
        .query_row("SELECT COUNT(*) FROM session_ledger", [], |row| row.get(0))
        .expect("ledger count");
    assert_eq!(ledger_rows, 0);
}

#[test]
fn records_starts_and_immutable_intents_with_reopen_safe_retries() {
    let database = Database::open_in_memory().expect("database");
    let ledger = SqliteSessionRecoveryLedger::new(&database);
    let id = session_id(SESSION_A);
    let start = started(id.clone(), "2026-07-16T10:00:00Z");
    assert_eq!(
        ledger.record_started(&start),
        Ok(LedgerMutationOutcome::Created)
    );
    assert_eq!(
        ledger.record_started(&start),
        Ok(LedgerMutationOutcome::AlreadyExistsIdentical)
    );

    let conflicting_start = StartedSession::new(
        id.clone(),
        "custom",
        json!({"language": "fr", "kind": "sanitized"}),
        "fr",
        timestamp("2026-07-16T10:00:00Z"),
    )
    .expect("conflicting fixture");
    assert!(matches!(
        ledger.record_started(&conflicting_start),
        Ok(LedgerMutationOutcome::Conflicting(_))
    ));

    let first = intent(id.clone(), "en", json!({"language": "en"}));
    assert_eq!(
        ledger.record_completion_intent(&first),
        Ok(LedgerMutationOutcome::Created)
    );
    assert_eq!(
        ledger.record_completion_intent(&first),
        Ok(LedgerMutationOutcome::AlreadyExistsIdentical)
    );
    let conflicting = intent(id.clone(), "fr", json!({"language": "fr"}));
    assert!(matches!(
        ledger.record_completion_intent(&conflicting),
        Ok(LedgerMutationOutcome::Conflicting(_))
    ));

    assert_eq!(ledger.mark_aborted(&id), Ok(LedgerMutationOutcome::Created));
    assert!(matches!(
        ledger.record_started(&start),
        Ok(LedgerMutationOutcome::Conflicting(_))
    ));
}

#[test]
fn exact_payload_bound_is_stored_and_one_byte_over_is_rejected_by_schema() {
    let database = Database::open_in_memory().expect("database");
    let ledger = SqliteSessionRecoveryLedger::new(&database);
    let id = session_id(SESSION_A);
    let start = started(id.clone(), "2026-07-16T10:00:00Z");
    ledger.record_started(&start).expect("start");

    let empty = intent(id.clone(), "en", Value::String(String::new()));
    let exact_characters = MAX_COMPLETION_INTENT_PAYLOAD_BYTES - empty.canonical_payload().len();
    let exact = intent(
        id.clone(),
        "en",
        Value::String("x".repeat(exact_characters)),
    );
    assert_eq!(
        exact.canonical_payload().len(),
        MAX_COMPLETION_INTENT_PAYLOAD_BYTES
    );
    assert_eq!(
        ledger.record_completion_intent(&exact),
        Ok(LedgerMutationOutcome::Created)
    );

    let oversize_id = session_id(SESSION_B);
    let oversize_start = started(oversize_id.clone(), "2026-07-16T10:00:01Z");
    ledger
        .record_started(&oversize_start)
        .expect("second start");
    let oversize = vec![0_u8; MAX_COMPLETION_INTENT_PAYLOAD_BYTES + 1];
    let fingerprint = "a".repeat(64);
    let connection = database.conn();
    let result = connection.execute(
        "INSERT INTO session_completion_intents (
            session_id, canonicalization_version, payload_version, fingerprint,
            canonical_payload, payload_byte_length, created_at
         ) VALUES (?1, 1, 1, ?2, ?3, ?4, '2026-07-16T10:00:01Z')",
        params![
            oversize_id.as_str(),
            fingerprint,
            oversize,
            (MAX_COMPLETION_INTENT_PAYLOAD_BYTES + 1) as i64,
        ],
    );
    assert!(
        result.is_err(),
        "V007 must reject payloads larger than 8 MiB"
    );
}

#[test]
fn candidates_are_metadata_only_ordered_and_full_load_stays_separate_after_reopen() {
    let path = temporary_database_path("reopen");
    remove_database(&path);
    let first_id = session_id(SESSION_A);
    let second_id = session_id(SESSION_B);
    let first = intent(first_id.clone(), "en", json!({"language": "en"}));

    {
        let database = Database::open(&path).expect("database");
        let ledger = SqliteSessionRecoveryLedger::new(&database);
        ledger
            .record_started(&started(first_id.clone(), "2026-07-16T10:00:00Z"))
            .expect("first start");
        ledger
            .record_started(&started(second_id.clone(), "2026-07-16T10:00:00Z"))
            .expect("second start");
        ledger.record_completion_intent(&first).expect("intent");
    }

    {
        let database = Database::open(&path).expect("reopened database");
        let ledger = SqliteSessionRecoveryLedger::new(&database);
        let candidates = ledger.list_recovery_candidates().expect("candidates");
        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.session_id().as_str())
                .collect::<Vec<_>>(),
            vec![SESSION_A, SESSION_B]
        );
        assert!(matches!(
            candidates[0].intent_metadata(),
            CompletionIntentMetadata::Present { .. }
        ));
        assert_eq!(
            candidates[1].intent_metadata(),
            &CompletionIntentMetadata::Missing
        );
        assert_eq!(
            ledger.load_completion_intent(&first_id),
            Ok(CompletionIntentLoadOutcome::Found(Box::new(first.clone())))
        );
    }
    remove_database(&path);
}

#[test]
fn corrupt_headers_and_payloads_are_isolated_without_candidate_payload_loading() {
    let database = Database::open_in_memory().expect("database");
    let ledger = SqliteSessionRecoveryLedger::new(&database);
    let id = session_id(SESSION_A);
    ledger
        .record_started(&started(id.clone(), "2026-07-16T10:00:00Z"))
        .expect("start");
    let fingerprint = "a".repeat(64);
    insert_raw_intent(&database, &id, 2, 1, &fingerprint, b"not-json", 8);

    let candidates = ledger.list_recovery_candidates().expect("metadata scan");
    assert!(matches!(
        candidates[0].intent_metadata(),
        CompletionIntentMetadata::UnsupportedCanonicalizationVersion { version: 2 }
    ));
    assert_eq!(
        ledger.load_completion_intent(&id),
        Ok(CompletionIntentLoadOutcome::UnsupportedCanonicalizationVersion { version: 2 })
    );

    let other = session_id(SESSION_B);
    ledger
        .record_started(&started(other.clone(), "2026-07-16T10:00:01Z"))
        .expect("second start");
    let valid_fingerprint = CompletionIntentFingerprint::from_canonical_bytes(b"expected");
    insert_raw_intent(
        &database,
        &other,
        1,
        1,
        valid_fingerprint.as_str(),
        b"not-json",
        8,
    );
    assert_eq!(
        ledger.load_completion_intent(&other),
        Ok(CompletionIntentLoadOutcome::Corrupt)
    );
}

#[test]
fn metadata_listing_classifies_partial_and_wrongly_typed_headers_without_payload_access() {
    let database = Database::open_in_memory().expect("database");
    let ledger = SqliteSessionRecoveryLedger::new(&database);
    for (session, offset) in [
        (SESSION_C, "00"),
        (SESSION_D, "01"),
        (SESSION_E, "02"),
        (SESSION_F, "03"),
    ] {
        ledger
            .record_started(&started(
                session_id(session),
                &format!("2026-07-16T10:00:{offset}Z"),
            ))
            .expect("start");
    }
    replace_intent_table_with_corruption_fixture(&database);
    let fingerprint = "a".repeat(64);
    let connection = database.conn();
    connection
        .execute(
            "INSERT INTO session_completion_intents VALUES (?1, NULL, NULL, NULL, X'00', 1, ?2)",
            params![SESSION_C, "2026-07-16T12:00:00Z"],
        )
        .expect("partial header");
    connection
        .execute(
            "INSERT INTO session_completion_intents VALUES (?1, 'wrong-type', 1, ?2, X'00', 1, ?3)",
            params![SESSION_D, fingerprint, "2026-07-16T12:00:00Z"],
        )
        .expect("wrong type header");
    connection
        .execute(
            "INSERT INTO session_completion_intents VALUES (?1, 1, 2, ?2, X'00', 1, ?3)",
            params![SESSION_E, fingerprint, "2026-07-16T12:00:00Z"],
        )
        .expect("unsupported payload version");
    connection
        .execute(
            "INSERT INTO session_completion_intents VALUES (?1, 1, 1, 'NOT-LOWERCASE', X'00', 1, ?2)",
            params![SESSION_F, "2026-07-16T12:00:00Z"],
        )
        .expect("invalid fingerprint");
    drop(connection);

    let candidates = ledger
        .list_recovery_candidates()
        .expect("candidate listing");
    assert!(matches!(
        candidates
            .iter()
            .find(|candidate| candidate.session_id().as_str() == SESSION_C)
            .expect("partial candidate")
            .intent_metadata(),
        CompletionIntentMetadata::Corrupt
    ));
    assert!(matches!(
        candidates
            .iter()
            .find(|candidate| candidate.session_id().as_str() == SESSION_D)
            .expect("typed candidate")
            .intent_metadata(),
        CompletionIntentMetadata::Corrupt
    ));
    assert!(matches!(
        candidates
            .iter()
            .find(|candidate| candidate.session_id().as_str() == SESSION_E)
            .expect("unsupported candidate")
            .intent_metadata(),
        CompletionIntentMetadata::UnsupportedVersion { version: 2 }
    ));
    assert!(matches!(
        candidates
            .iter()
            .find(|candidate| candidate.session_id().as_str() == SESSION_F)
            .expect("fingerprint candidate")
            .intent_metadata(),
        CompletionIntentMetadata::Corrupt
    ));
}

#[test]
fn full_load_detects_fingerprint_and_length_corruption_and_invalid_ledger_ids_block_safely() {
    let database = Database::open_in_memory().expect("database");
    let ledger = SqliteSessionRecoveryLedger::new(&database);
    let id = session_id(SESSION_A);
    let valid = intent(id.clone(), "en", json!({"language": "en"}));
    ledger
        .record_started(&started(id.clone(), "2026-07-16T10:00:00Z"))
        .expect("start");
    let mismatched = CompletionIntentFingerprint::from_canonical_bytes(b"different");
    insert_raw_intent(
        &database,
        &id,
        1,
        1,
        mismatched.as_str(),
        valid.canonical_payload(),
        valid.canonical_payload().len() as i64,
    );
    assert_eq!(
        ledger.load_completion_intent(&id),
        Ok(CompletionIntentLoadOutcome::Corrupt)
    );

    let second = session_id(SESSION_B);
    ledger
        .record_started(&started(second.clone(), "2026-07-16T10:00:01Z"))
        .expect("second start");
    replace_intent_table_with_corruption_fixture(&database);
    let connection = database.conn();
    connection
        .execute(
            "INSERT INTO session_completion_intents VALUES (?1, 1, 1, ?2, ?3, 1, ?4)",
            params![
                second.as_str(),
                valid.fingerprint().as_str(),
                valid.canonical_payload(),
                "2026-07-16T12:00:00Z",
            ],
        )
        .expect("length mismatch fixture");
    drop(connection);
    assert_eq!(
        ledger.load_completion_intent(&second),
        Ok(CompletionIntentLoadOutcome::Corrupt)
    );

    let invalid_ledger = Database::open_in_memory().expect("separate database");
    {
        let connection = invalid_ledger.conn();
        connection
            .execute(
                "INSERT INTO session_ledger (
                    session_id, state, mode_type, mode_descriptor, language, created_at, updated_at
                 ) VALUES ('not-a-session-id', 'running', 'custom', '{}', 'en',
                    '2026-07-16T10:00:00Z', '2026-07-16T10:00:00Z')",
                [],
            )
            .expect("invalid identity corruption fixture");
    }
    assert!(matches!(
        SqliteSessionRecoveryLedger::new(&invalid_ledger).list_recovery_candidates(),
        Err(racoon_application::RecoveryPortFailure::PermanentFailure(_))
    ));
}

#[test]
fn unknown_state_is_presented_as_corrupt_and_invalid_terminal_transitions_do_not_reopen_rows() {
    let database = Database::open_in_memory().expect("database");
    let ledger = SqliteSessionRecoveryLedger::new(&database);
    let id = session_id(SESSION_A);
    ledger
        .record_started(&started(id.clone(), "2026-07-16T10:00:00Z"))
        .expect("start");
    {
        let connection = database.conn();
        connection
            .pragma_update(None, "ignore_check_constraints", "ON")
            .expect("ignore checks");
        connection
            .execute(
                "UPDATE session_ledger SET state = 'unknown_state' WHERE session_id = ?1",
                params![id.as_str()],
            )
            .expect("invalid state fixture");
        connection
            .pragma_update(None, "ignore_check_constraints", "OFF")
            .expect("restore checks");
    }
    let candidate = ledger
        .list_recovery_candidates()
        .expect("candidate listing")
        .into_iter()
        .next()
        .expect("candidate");
    assert_eq!(candidate.state(), DurableSessionState::Running);
    assert_eq!(
        candidate.intent_metadata(),
        &CompletionIntentMetadata::Corrupt
    );

    let normal = session_id(SESSION_B);
    ledger
        .record_started(&started(normal.clone(), "2026-07-16T10:00:01Z"))
        .expect("normal start");
    assert_eq!(
        ledger.mark_aborted(&normal),
        Ok(LedgerMutationOutcome::Created)
    );
    assert_eq!(
        ledger.mark_interrupted(
            &normal,
            racoon_application::InterruptionReason::ProcessRestart
        ),
        Ok(LedgerMutationOutcome::Quarantined(
            QuarantineReason::InvalidStateRecord
        ))
    );
    let candidates = ledger.list_recovery_candidates().expect("candidates");
    assert!(candidates.iter().any(|candidate| {
        candidate.session_id() == &normal && candidate.state() == DurableSessionState::Aborted
    }));
}

#[test]
fn claims_are_atomic_idempotent_and_quarantine_invalid_records() {
    let database = Database::open_in_memory().expect("database");
    let ledger = SqliteSessionRecoveryLedger::new(&database);
    let id = session_id(SESSION_A);
    let completion = intent(id.clone(), "en", json!({"language": "en"}));
    ledger
        .record_started(&started(id.clone(), "2026-07-16T10:00:00Z"))
        .expect("start");
    ledger
        .record_completion_intent(&completion)
        .expect("intent");
    assert_eq!(
        ledger.claim_completion_for_finalization(&id, completion.fingerprint()),
        Ok(FinalizationClaimOutcome::Claimed)
    );
    assert_eq!(
        ledger.claim_completion_for_finalization(&id, completion.fingerprint()),
        Ok(FinalizationClaimOutcome::AlreadyPending)
    );
    let conflicting = CompletionIntentFingerprint::from_canonical_bytes(b"different");
    assert!(matches!(
        ledger.claim_completion_for_finalization(&id, &conflicting),
        Ok(FinalizationClaimOutcome::Conflict(_))
    ));

    {
        let connection = database.conn();
        connection
            .execute(
                "UPDATE session_ledger SET state = 'finalized' WHERE session_id = ?1",
                params![id.as_str()],
            )
            .expect("fixture finalized state");
    }
    assert_eq!(
        ledger.claim_completion_for_finalization(&id, completion.fingerprint()),
        Ok(FinalizationClaimOutcome::AlreadyFinalized)
    );

    let aborted_id = session_id(SESSION_C);
    ledger
        .record_started(&started(aborted_id.clone(), "2026-07-16T10:00:00Z"))
        .expect("aborted start");
    ledger.mark_aborted(&aborted_id).expect("abort");
    assert_eq!(
        ledger.claim_completion_for_finalization(&aborted_id, completion.fingerprint()),
        Ok(FinalizationClaimOutcome::RejectedTerminal {
            state: DurableSessionState::Aborted
        })
    );
    assert_eq!(
        ledger.claim_completion_for_finalization(&session_id(SESSION_D), completion.fingerprint()),
        Ok(FinalizationClaimOutcome::NotFound)
    );

    let missing_intent_id = session_id(SESSION_E);
    ledger
        .record_started(&started(missing_intent_id.clone(), "2026-07-16T10:00:00Z"))
        .expect("missing intent start");
    {
        let connection = database.conn();
        connection
            .execute(
                "UPDATE session_ledger SET state = 'awaiting_persistence' WHERE session_id = ?1",
                params![missing_intent_id.as_str()],
            )
            .expect("fixture awaiting state");
    }
    assert_eq!(
        ledger.claim_completion_for_finalization(&missing_intent_id, completion.fingerprint()),
        Ok(FinalizationClaimOutcome::Quarantined(
            QuarantineReason::MissingCompletionIntent
        ))
    );

    let unsupported_id = session_id(SESSION_F);
    ledger
        .record_started(&started(unsupported_id.clone(), "2026-07-16T10:00:00Z"))
        .expect("unsupported start");
    insert_raw_intent(
        &database,
        &unsupported_id,
        2,
        1,
        completion.fingerprint().as_str(),
        completion.canonical_payload(),
        completion.canonical_payload().len() as i64,
    );
    {
        let connection = database.conn();
        connection
            .execute(
                "UPDATE session_ledger SET state = 'awaiting_persistence' WHERE session_id = ?1",
                params![unsupported_id.as_str()],
            )
            .expect("unsupported awaiting state");
    }
    assert_eq!(
        ledger.claim_completion_for_finalization(&unsupported_id, completion.fingerprint()),
        Ok(FinalizationClaimOutcome::Quarantined(
            QuarantineReason::UnsupportedCanonicalizationVersion
        ))
    );

    let running_id = session_id(SESSION_B);
    ledger
        .record_started(&started(running_id.clone(), "2026-07-16T10:00:01Z"))
        .expect("running start");
    assert_eq!(
        ledger.claim_completion_for_finalization(&running_id, completion.fingerprint()),
        Ok(FinalizationClaimOutcome::Quarantined(
            QuarantineReason::InvalidStateRecord
        ))
    );
    let candidates = ledger.list_recovery_candidates().expect("candidates");
    assert!(candidates.iter().any(|candidate| {
        candidate.session_id() == &running_id
            && candidate.state() == DurableSessionState::Quarantined
    }));
}

#[test]
fn foreign_keys_and_failed_claim_updates_rollback_without_sensitive_ledger_content() {
    let database = Database::open_in_memory().expect("database");
    let unknown = session_id(SESSION_A);
    let connection = database.conn();
    let foreign_key = connection.execute(
        "INSERT INTO session_completion_intents (
            session_id, canonicalization_version, payload_version, fingerprint,
            canonical_payload, payload_byte_length, created_at
         ) VALUES (?1, 1, 1, ?2, X'7B7D', 2, '2026-07-16T10:00:00Z')",
        params![unknown.as_str(), "a".repeat(64)],
    );
    assert!(foreign_key.is_err(), "V007 must reject orphan intents");
    drop(connection);

    let ledger = SqliteSessionRecoveryLedger::new(&database);
    let start = started(unknown.clone(), "2026-07-16T10:00:00Z");
    ledger.record_started(&start).expect("start");
    {
        let connection = database.conn();
        connection
            .execute_batch(
                "CREATE TRIGGER force_completion_intent_rollback
                 BEFORE UPDATE OF state ON session_ledger
                 WHEN NEW.state = 'awaiting_persistence'
                 BEGIN SELECT RAISE(ABORT, 'forced rollback'); END;",
            )
            .expect("rollback trigger");
    }
    let completion = intent(unknown.clone(), "en", json!({"language": "en"}));
    assert!(ledger.record_completion_intent(&completion).is_err());
    let connection = database.conn();
    let stored_intents: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM session_completion_intents WHERE session_id = ?1",
            params![unknown.as_str()],
            |row| row.get(0),
        )
        .expect("intent count");
    assert_eq!(
        stored_intents, 0,
        "failed state update must roll back intent insert"
    );
    let (descriptor, interruption, abort, quarantine): (
        String,
        Option<String>,
        Option<String>,
        Option<String>,
    ) = connection
        .query_row(
            "SELECT mode_descriptor, interruption_reason, abort_reason, quarantine_reason
             FROM session_ledger WHERE session_id = ?1",
            params![unknown.as_str()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .expect("ledger row");
    let ledger_text = format!("{descriptor}{interruption:?}{abort:?}{quarantine:?}");
    assert!(!ledger_text.contains("typed-secret-key"));
    assert!(!ledger_text.contains("\"replay_frames\""));
}

#[test]
fn terminal_ledger_rows_resist_update_delete_and_replacement() {
    let database = Database::open_in_memory().expect("database");
    let ledger = SqliteSessionRecoveryLedger::new(&database);
    let id = session_id(SESSION_A);
    ledger
        .record_started(&started(id.clone(), "2026-07-16T10:00:00Z"))
        .expect("start");
    assert_eq!(
        ledger.mark_aborted(&id).expect("abort"),
        LedgerMutationOutcome::Created
    );
    let before = ledger_snapshot(&database, &id);

    let connection = database.conn();
    assert!(connection
        .execute(
            "UPDATE session_ledger SET state = 'running' WHERE session_id = ?1",
            params![id.as_str()],
        )
        .is_err());
    assert!(connection
        .execute(
            "DELETE FROM session_ledger WHERE session_id = ?1",
            params![id.as_str()],
        )
        .is_err());
    for state in ["running", "interrupted"] {
        assert!(connection
            .execute(
                "INSERT OR REPLACE INTO session_ledger (
                    session_id, state, mode_type, mode_descriptor, language, created_at, updated_at
                 ) VALUES (?1, ?2, 'custom', '{}', 'en',
                    '2026-07-16T11:00:00Z', '2026-07-16T11:00:00Z')",
                params![id.as_str(), state],
            )
            .is_err());
    }
    drop(connection);
    assert_eq!(ledger_snapshot(&database, &id), before);

    let non_terminal = session_id(SESSION_B);
    ledger
        .record_started(&started(non_terminal.clone(), "2026-07-16T10:00:01Z"))
        .expect("second start");
    assert_eq!(
        ledger.mark_interrupted(
            &non_terminal,
            racoon_application::InterruptionReason::ProcessRestart
        ),
        Ok(LedgerMutationOutcome::Created)
    );
}

#[test]
fn completion_intents_resist_update_delete_replace_and_parent_replacement() {
    let database = Database::open_in_memory().expect("database");
    let ledger = SqliteSessionRecoveryLedger::new(&database);
    let id = session_id(SESSION_A);
    let completion = intent(id.clone(), "en", json!({"language": "en"}));
    ledger
        .record_started(&started(id.clone(), "2026-07-16T10:00:00Z"))
        .expect("start");
    ledger
        .record_completion_intent(&completion)
        .expect("intent");
    let before = intent_snapshot(&database, &id);

    let connection = database.conn();
    for update in [
        "canonicalization_version = 9",
        "payload_version = 9",
        "fingerprint = 'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb'",
        "canonical_payload = X'5B5D'",
        "payload_byte_length = 2",
        "created_at = '2026-07-16T13:00:00Z'",
    ] {
        assert!(connection
            .execute(
                &format!("UPDATE session_completion_intents SET {update} WHERE session_id = ?1"),
                params![id.as_str()],
            )
            .is_err());
    }
    assert!(connection
        .execute(
            "DELETE FROM session_completion_intents WHERE session_id = ?1",
            params![id.as_str()],
        )
        .is_err());
    assert!(connection
        .execute(
            "INSERT OR REPLACE INTO session_completion_intents (
                session_id, canonicalization_version, payload_version, fingerprint,
                canonical_payload, payload_byte_length, created_at
             ) VALUES (?1, 1, 1,
                'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',
                X'5B5D', 2, '2026-07-16T13:00:00Z')",
            params![id.as_str()],
        )
        .is_err());
    assert!(connection
        .execute(
            "INSERT OR REPLACE INTO session_ledger (
                session_id, state, mode_type, mode_descriptor, language, created_at, updated_at
             ) VALUES (?1, 'running', 'custom', '{}', 'en',
                '2026-07-16T13:00:00Z', '2026-07-16T13:00:00Z')",
            params![id.as_str()],
        )
        .is_err());
    drop(connection);

    assert_eq!(intent_snapshot(&database, &id), before);
}

#[test]
fn descriptor_privacy_rejection_prevents_any_ledger_write() {
    let database = Database::open_in_memory().expect("database");
    let id = session_id(SESSION_A);
    let secret = "raw-typed-descriptor-secret";
    let rejected = StartedSession::new(
        id.clone(),
        "custom",
        json!({"nested": {"typed-text": secret}}),
        "en",
        timestamp("2026-07-16T10:00:00Z"),
    );
    let error = rejected.expect_err("typed content must be rejected before persistence");
    assert!(!error.to_string().contains(secret));
    let connection = database.conn();
    let rows: i64 = connection
        .query_row("SELECT COUNT(*) FROM session_ledger", [], |row| row.get(0))
        .expect("ledger count");
    assert_eq!(rows, 0);
}

#[test]
fn refinery_history_fixtures_upgrade_through_v009_and_reopen_idempotently() {
    for starting_version in 1..=7 {
        let path = temporary_database_path(&format!("refinery-v{starting_version}"));
        remove_database(&path);
        {
            let mut connection = Connection::open(&path).expect("historical fixture database");
            let migrations = historical_refinery_migrations(starting_version);
            Runner::new(&migrations)
                .run(&mut connection)
                .expect("historical Refinery migration path");
            insert_historical_test(&connection, starting_version >= 5);
        }

        {
            let database = Database::open(&path).expect("production upgrade");
            let connection = database.conn();
            let versions: Vec<i64> = connection
                .prepare("SELECT version FROM refinery_schema_history ORDER BY version")
                .expect("history query")
                .query_map([], |row| row.get(0))
                .expect("history rows")
                .collect::<Result<_, _>>()
                .expect("history values");
            assert_eq!(versions, (1..=9).collect::<Vec<_>>());
            let session_identity: String = connection
                .query_row("SELECT session_id FROM tests WHERE id = 1", [], |row| {
                    row.get(0)
                })
                .expect("preserved session identity");
            assert_eq!(session_identity, "legacy-test-0000000000000001");
            let ledger_rows: i64 = connection
                .query_row("SELECT COUNT(*) FROM session_ledger", [], |row| row.get(0))
                .expect("ledger count");
            assert_eq!(ledger_rows, 0, "historical tests are not backfilled");
            let foreign_keys: i64 = connection
                .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
                .expect("foreign key pragma");
            assert_eq!(foreign_keys, 1);
            for trigger in [
                "session_ledger_terminal_row_not_replaceable",
                "session_ledger_terminal_row_not_deletable",
                "session_completion_intents_not_replaceable",
                "session_completion_intents_not_deletable",
                "session_finalizations_not_replaceable",
                "session_finalizations_not_deletable",
            ] {
                let count: i64 = connection
                    .query_row(
                        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'trigger' AND name = ?1",
                        params![trigger],
                        |row| row.get(0),
                    )
                    .expect("trigger lookup");
                assert_eq!(count, 1, "missing {trigger}");
            }
        }
        Database::open(&path).expect("idempotent production reopen");
        remove_database(&path);
    }
}

#[test]
fn a_structured_sqlite_busy_error_is_retryable_after_real_lock_contention() {
    let path = temporary_database_path("busy");
    remove_database(&path);
    let database = Database::open(&path).expect("database");
    {
        let connection = database.conn();
        connection
            .busy_timeout(Duration::from_millis(20))
            .expect("short test timeout");
    }
    let lock_connection = Connection::open(&path).expect("second connection");
    lock_connection
        .execute_batch("BEGIN IMMEDIATE")
        .expect("hold write reservation");

    let ledger = SqliteSessionRecoveryLedger::new(&database);
    let start = started(session_id(SESSION_A), "2026-07-16T10:00:00Z");
    assert_eq!(
        ledger.record_started(&start),
        Err(racoon_application::RecoveryPortFailure::RetryableFailure)
    );

    lock_connection
        .execute_batch("COMMIT")
        .expect("release lock");
    assert_eq!(
        ledger.record_started(&start),
        Ok(LedgerMutationOutcome::Created)
    );
    drop(lock_connection);
    drop(database);
    remove_database(&path);
}

#[test]
fn finalization_claim_commit_and_identical_retries_are_durable_and_effect_free() {
    let database = Database::open_in_memory().expect("database");
    let id = session_id(SESSION_A);
    let completion = prepare_finalization_pending(&database, id.clone());
    let finalizations = SqliteFinalizationLedger::new(&database);
    let claimed_at = timestamp("2026-07-16T12:01:00.123456Z");
    let committed_at = timestamp("2026-07-16T12:02:00.654321Z");

    assert_eq!(
        finalizations.claim_finalization(&id, completion.fingerprint(), claimed_at),
        Ok(FinalizationLedgerClaimOutcome::Claimed)
    );
    let before_retry = finalization_snapshot(&database, &id);
    assert_eq!(
        finalizations.claim_finalization(
            &id,
            completion.fingerprint(),
            timestamp("2026-07-16T12:03:00Z"),
        ),
        Ok(FinalizationLedgerClaimOutcome::AlreadyPending)
    );
    assert_eq!(finalization_snapshot(&database, &id), before_retry);
    assert_eq!(
        finalizations.mark_finalization_committed(&id, completion.fingerprint(), committed_at),
        Ok(FinalizationCommitOutcome::Committed)
    );
    let committed = finalization_snapshot(&database, &id);
    assert_eq!(
        finalizations.mark_finalization_committed(
            &id,
            completion.fingerprint(),
            timestamp("2026-07-16T12:04:00Z"),
        ),
        Ok(FinalizationCommitOutcome::AlreadyCommitted)
    );
    assert_eq!(finalization_snapshot(&database, &id), committed);
    assert!(matches!(
        finalizations.load_finalization(&id),
        Ok(FinalizationLoadOutcome::Found(record))
            if record.state() == FinalizationLedgerState::Committed
                && record.claimed_at() == claimed_at
                && record.committed_at() == Some(committed_at)
    ));

    let connection = database.conn();
    let tests: i64 = connection
        .query_row("SELECT COUNT(*) FROM tests", [], |row| row.get(0))
        .expect("test count");
    let replays: i64 = connection
        .query_row("SELECT COUNT(*) FROM test_replays", [], |row| row.get(0))
        .expect("replay count");
    assert_eq!(tests, 0, "finalization ledger must not insert tests");
    assert_eq!(replays, 0, "finalization ledger must not insert replays");
}

#[test]
fn finalization_rejects_missing_intents_conflicts_and_quarantines_pending_records() {
    let database = Database::open_in_memory().expect("database");
    let finalizations = SqliteFinalizationLedger::new(&database);
    let missing = session_id(SESSION_A);
    let expected = CompletionIntentFingerprint::try_from_hex("a".repeat(64)).expect("fingerprint");
    assert_eq!(
        finalizations.claim_finalization(&missing, &expected, timestamp("2026-07-16T12:00:00Z")),
        Ok(FinalizationLedgerClaimOutcome::NotFound)
    );

    let missing_intent = session_id(SESSION_B);
    let recovery = SqliteSessionRecoveryLedger::new(&database);
    recovery
        .record_started(&started(missing_intent.clone(), "2026-07-16T10:00:00Z"))
        .expect("start");
    database
        .conn()
        .execute(
            "UPDATE session_ledger SET state = 'finalization_pending' WHERE session_id = ?1",
            params![missing_intent.as_str()],
        )
        .expect("fixture state");
    assert_eq!(
        finalizations.claim_finalization(
            &missing_intent,
            &expected,
            timestamp("2026-07-16T12:00:00Z"),
        ),
        Ok(FinalizationLedgerClaimOutcome::MissingCompletionIntent)
    );

    let id = session_id(SESSION_C);
    let completion = prepare_finalization_pending(&database, id.clone());
    let conflicting =
        CompletionIntentFingerprint::try_from_hex("b".repeat(64)).expect("fingerprint");
    assert!(matches!(
        finalizations.claim_finalization(&id, &conflicting, timestamp("2026-07-16T12:00:00Z")),
        Ok(FinalizationLedgerClaimOutcome::Conflict(_))
    ));
    assert_eq!(
        finalizations.claim_finalization(
            &id,
            completion.fingerprint(),
            timestamp("2026-07-16T12:00:00Z")
        ),
        Ok(FinalizationLedgerClaimOutcome::Claimed)
    );
    assert_eq!(
        finalizations.quarantine_finalization(
            &id,
            completion.fingerprint(),
            FinalizationQuarantineReason::CorruptDurableMetadata,
        ),
        Ok(FinalizationCommitOutcome::Quarantined(
            FinalizationQuarantineReason::CorruptDurableMetadata
        ))
    );
    assert_eq!(
        finalizations.mark_finalization_committed(
            &id,
            completion.fingerprint(),
            timestamp("2026-07-16T12:01:00Z"),
        ),
        Ok(FinalizationCommitOutcome::Quarantined(
            FinalizationQuarantineReason::CorruptDurableMetadata
        ))
    );
}

#[test]
fn finalization_quarantine_prioritizes_v007_v008_fingerprint_mismatch() {
    let database = Database::open_in_memory().expect("database");
    let id = session_id(SESSION_A);
    let completion = prepare_finalization_pending(&database, id.clone());
    let finalizations = SqliteFinalizationLedger::new(&database);
    finalizations
        .claim_finalization(
            &id,
            completion.fingerprint(),
            timestamp("2026-07-16T12:00:00Z"),
        )
        .expect("claim");
    let before = finalization_snapshot(&database, &id);

    replace_intent_table_with_corruption_fixture(&database);
    insert_corrupt_intent_header(&database, &id, &"a".repeat(64));

    assert_eq!(
        finalizations.quarantine_finalization(
            &id,
            completion.fingerprint(),
            FinalizationQuarantineReason::InvalidFinalizationState,
        ),
        Ok(FinalizationCommitOutcome::Quarantined(
            FinalizationQuarantineReason::FingerprintMismatch
        ))
    );
    let after = finalization_snapshot(&database, &id);
    assert_eq!(after.0, before.0);
    assert_eq!(after.1, before.1);
    assert_eq!(after.2, "quarantined");
    assert_eq!(after.3, before.3);
    assert_eq!(after.4, None);
    assert_eq!(after.5.as_deref(), Some("fingerprint_mismatch"));
}

#[test]
fn finalization_quarantine_prioritizes_missing_or_corrupt_v007_metadata() {
    let database = Database::open_in_memory().expect("corrupt intent database");
    let corrupt_id = session_id(SESSION_A);
    let corrupt_completion = prepare_finalization_pending(&database, corrupt_id.clone());
    let finalizations = SqliteFinalizationLedger::new(&database);
    finalizations
        .claim_finalization(
            &corrupt_id,
            corrupt_completion.fingerprint(),
            timestamp("2026-07-16T12:00:00Z"),
        )
        .expect("claim");
    replace_intent_table_with_corruption_fixture(&database);
    insert_corrupt_intent_header(&database, &corrupt_id, "NOT-LOWERCASE");

    assert_eq!(
        finalizations.quarantine_finalization(
            &corrupt_id,
            corrupt_completion.fingerprint(),
            FinalizationQuarantineReason::InvalidFinalizationState,
        ),
        Ok(FinalizationCommitOutcome::Quarantined(
            FinalizationQuarantineReason::CorruptDurableMetadata
        ))
    );
    assert_eq!(
        finalization_snapshot(&database, &corrupt_id).5.as_deref(),
        Some("corrupt_durable_metadata")
    );

    let missing_database = Database::open_in_memory().expect("missing intent database");
    let missing_id = session_id(SESSION_B);
    let missing_completion = prepare_finalization_pending(&missing_database, missing_id.clone());
    let missing_finalizations = SqliteFinalizationLedger::new(&missing_database);
    missing_finalizations
        .claim_finalization(
            &missing_id,
            missing_completion.fingerprint(),
            timestamp("2026-07-16T12:00:00Z"),
        )
        .expect("claim");
    replace_intent_table_with_corruption_fixture(&missing_database);

    assert_eq!(
        missing_finalizations.quarantine_finalization(
            &missing_id,
            missing_completion.fingerprint(),
            FinalizationQuarantineReason::InvalidFinalizationState,
        ),
        Ok(FinalizationCommitOutcome::Quarantined(
            FinalizationQuarantineReason::MissingCompletionIntent
        ))
    );
    assert_eq!(
        finalization_snapshot(&missing_database, &missing_id)
            .5
            .as_deref(),
        Some("missing_completion_intent")
    );
}

#[test]
fn finalization_quarantine_conflicts_without_mutation_and_preserves_terminals() {
    let database = Database::open_in_memory().expect("database");
    let id = session_id(SESSION_A);
    let completion = prepare_finalization_pending(&database, id.clone());
    let finalizations = SqliteFinalizationLedger::new(&database);
    finalizations
        .claim_finalization(
            &id,
            completion.fingerprint(),
            timestamp("2026-07-16T12:00:00Z"),
        )
        .expect("claim");
    let before_conflict = finalization_snapshot(&database, &id);
    let conflicting =
        CompletionIntentFingerprint::try_from_hex("b".repeat(64)).expect("fingerprint");
    assert!(matches!(
        finalizations.quarantine_finalization(
            &id,
            &conflicting,
            FinalizationQuarantineReason::InvalidFinalizationState,
        ),
        Ok(FinalizationCommitOutcome::Conflict(_))
    ));
    assert_eq!(finalization_snapshot(&database, &id), before_conflict);

    finalizations
        .quarantine_finalization(
            &id,
            completion.fingerprint(),
            FinalizationQuarantineReason::CorruptDurableMetadata,
        )
        .expect("terminal quarantine");
    let terminal = finalization_snapshot(&database, &id);
    replace_intent_table_with_corruption_fixture(&database);
    insert_corrupt_intent_header(&database, &id, &"a".repeat(64));
    assert_eq!(
        finalizations.quarantine_finalization(
            &id,
            completion.fingerprint(),
            FinalizationQuarantineReason::InvalidFinalizationState,
        ),
        Ok(FinalizationCommitOutcome::Quarantined(
            FinalizationQuarantineReason::CorruptDurableMetadata
        ))
    );
    assert_eq!(finalization_snapshot(&database, &id), terminal);
}

#[test]
fn session_finalizer_commits_effects_and_terminal_markers_exactly_once() {
    let database = Database::open_in_memory().expect("database");
    let id = session_id(SESSION_A);
    let completion = prepare_finalization_pending(&database, id.clone());
    SqliteFinalizationLedger::new(&database)
        .claim_finalization(
            &id,
            completion.fingerprint(),
            timestamp("2026-07-16T12:00:00Z"),
        )
        .expect("V008 claim");

    let finalizer = SqliteSessionFinalizer::new(&database);
    assert_eq!(
        finalizer.finalize_completion(&id, completion.fingerprint()),
        Ok(racoon_application::FinalizationOutcome::NewlyFinalized)
    );
    let connection = database.conn();
    let test_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM tests WHERE session_id = ?1",
            params![id.as_str()],
            |row| row.get(0),
        )
        .expect("test count");
    let replay_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM test_replays WHERE test_id = (SELECT id FROM tests WHERE session_id = ?1)",
            params![id.as_str()],
            |row| row.get(0),
        )
        .expect("replay count");
    let state: String = connection
        .query_row(
            "SELECT state FROM session_ledger WHERE session_id = ?1",
            params![id.as_str()],
            |row| row.get(0),
        )
        .expect("ledger state");
    let personal_best_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM personal_bests", [], |row| row.get(0))
        .expect("personal-best count");
    let daily: (i64, i64, i64) = connection
        .query_row(
            "SELECT total_tests, total_time_ms, daily_goal_met FROM daily_stats WHERE date = '2026-07-16'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("daily statistics");
    let streak: (i64, i64, String) = connection
        .query_row(
            "SELECT current_streak, longest_streak, last_date FROM streaks WHERE type = 'daily_test'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("daily streak");
    drop(connection);
    assert_eq!(test_count, 1);
    assert_eq!(
        replay_count,
        completion.payload().replay_frames().len() as i64
    );
    assert_eq!(personal_best_count, 1);
    assert_eq!(daily, (1, 1_000, 0));
    assert_eq!(streak, (1, 1, "2026-07-16".to_owned()));
    assert_eq!(state, "finalized");
    assert_eq!(finalization_snapshot(&database, &id).2, "committed");
    assert_eq!(
        finalizer.finalize_completion(&id, completion.fingerprint()),
        Ok(racoon_application::FinalizationOutcome::AlreadyFinalized)
    );
    let connection = database.conn();
    let repeated_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM tests WHERE session_id = ?1",
            params![id.as_str()],
            |row| row.get(0),
        )
        .expect("repeated test count");
    assert_eq!(repeated_count, 1);
}

fn finalizer_daily_goal_met(policy: CompletionPolicySnapshot, duration_ms: u64) -> bool {
    let database = Database::open_in_memory().expect("database");
    let id = session_id(SESSION_A);
    let completion = prepare_finalization_pending_with_policy_and_duration(
        &database,
        id.clone(),
        policy,
        duration_ms,
    );
    SqliteFinalizationLedger::new(&database)
        .claim_finalization(
            &id,
            completion.fingerprint(),
            timestamp("2026-07-16T12:00:00Z"),
        )
        .expect("V008 claim");
    assert_eq!(
        SqliteSessionFinalizer::new(&database).finalize_completion(&id, completion.fingerprint()),
        Ok(racoon_application::FinalizationOutcome::NewlyFinalized)
    );
    let connection = database.conn();
    let daily_goal_met = connection
        .query_row(
            "SELECT daily_goal_met FROM daily_stats WHERE date = '2026-07-16'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .expect("daily-goal state")
        == 1;
    drop(connection);
    daily_goal_met
}

#[test]
fn session_finalizer_daily_goal_policy_preserves_zero_and_fractional_time_boundaries() {
    // Zero/negative targets are unset goals and are never met — matching the
    // live-completion path and the WPM/accuracy zero rules.
    assert!(!finalizer_daily_goal_met(
        CompletionPolicySnapshot::time(0.0),
        0
    ));
    assert!(!finalizer_daily_goal_met(
        CompletionPolicySnapshot::time(-1.0),
        0
    ));
    assert!(!finalizer_daily_goal_met(
        CompletionPolicySnapshot::time(0.000_001),
        0
    ));
    assert!(finalizer_daily_goal_met(
        CompletionPolicySnapshot::time(0.000_001),
        1
    ));
    assert!(!finalizer_daily_goal_met(
        CompletionPolicySnapshot::time(1_000_000.0),
        1_000
    ));

    for (target, below, exact, above) in [
        (0.5, 29_999, 30_000, 30_001),
        (1.25, 74_999, 75_000, 75_001),
        (1.0, 59_999, 60_000, 60_001),
    ] {
        assert!(
            !finalizer_daily_goal_met(CompletionPolicySnapshot::time(target), below),
            "{target} minutes below threshold"
        );
        assert!(
            finalizer_daily_goal_met(CompletionPolicySnapshot::time(target), exact),
            "{target} minutes at threshold"
        );
        assert!(
            finalizer_daily_goal_met(CompletionPolicySnapshot::time(target), above),
            "{target} minutes above threshold"
        );
    }
}

#[test]
fn session_finalizer_daily_goal_policy_preserves_wpm_and_accuracy_zero_rules() {
    assert!(!finalizer_daily_goal_met(
        CompletionPolicySnapshot::wpm(0.0),
        1_000
    ));
    assert!(finalizer_daily_goal_met(
        CompletionPolicySnapshot::wpm(60.0),
        1_000
    ));
    assert!(!finalizer_daily_goal_met(
        CompletionPolicySnapshot::wpm(61.0),
        1_000
    ));

    assert!(!finalizer_daily_goal_met(
        CompletionPolicySnapshot::accuracy(0.0),
        1_000
    ));
    assert!(finalizer_daily_goal_met(
        CompletionPolicySnapshot::accuracy(0.98),
        1_000
    ));
    assert!(!finalizer_daily_goal_met(
        CompletionPolicySnapshot::accuracy(0.99),
        1_000
    ));
}

#[test]
fn session_finalizer_rejects_conflicting_fingerprint_without_effects() {
    let database = Database::open_in_memory().expect("database");
    let id = session_id(SESSION_A);
    let completion = prepare_finalization_pending(&database, id.clone());
    SqliteFinalizationLedger::new(&database)
        .claim_finalization(
            &id,
            completion.fingerprint(),
            timestamp("2026-07-16T12:00:00Z"),
        )
        .expect("V008 claim");
    let conflicting =
        CompletionIntentFingerprint::try_from_hex("a".repeat(64)).expect("fingerprint");
    assert!(matches!(
        SqliteSessionFinalizer::new(&database).finalize_completion(&id, &conflicting),
        Ok(racoon_application::FinalizationOutcome::Conflict(_))
    ));
    let connection = database.conn();
    let test_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM tests WHERE session_id = ?1",
            params![id.as_str()],
            |row| row.get(0),
        )
        .expect("test count");
    assert_eq!(test_count, 0);
    drop(connection);
    assert_eq!(finalization_snapshot(&database, &id).2, "pending");
}

#[test]
fn session_finalizer_does_not_accept_terminal_markers_without_a_matching_test() {
    let database = Database::open_in_memory().expect("database");
    let id = session_id(SESSION_A);
    let completion = prepare_finalization_pending(&database, id.clone());
    SqliteFinalizationLedger::new(&database)
        .claim_finalization(
            &id,
            completion.fingerprint(),
            timestamp("2026-07-16T12:00:00Z"),
        )
        .expect("V008 claim");
    let finalizer = SqliteSessionFinalizer::new(&database);
    assert_eq!(
        finalizer.finalize_completion(&id, completion.fingerprint()),
        Ok(racoon_application::FinalizationOutcome::NewlyFinalized)
    );
    {
        let connection = database.conn();
        connection
            .execute(
                "UPDATE tests SET wpm = 1.0 WHERE session_id = ?1",
                params![id.as_str()],
            )
            .expect("corrupt test fixture");
    }
    assert_eq!(
        finalizer.finalize_completion(&id, completion.fingerprint()),
        Ok(racoon_application::FinalizationOutcome::Quarantined(
            QuarantineReason::InconsistentDurableMetadata
        ))
    );
    assert_eq!(finalization_snapshot(&database, &id).2, "committed");
}

#[test]
fn session_finalizer_terminal_proof_rejects_missing_or_changed_authoritative_effects() {
    let mutations = [
        (
            "modified character statistics",
            "UPDATE tests SET char_stats = '{\"changed\":true}' WHERE session_id = ?1",
        ),
        (
            "modified heatmap data",
            "UPDATE tests SET heatmap_data = '{\"changed\":true}' WHERE session_id = ?1",
        ),
        (
            "modified graph data",
            "UPDATE tests SET graph_data = '{\"changed\":true}' WHERE session_id = ?1",
        ),
        (
            "missing replay",
            "DELETE FROM test_replays WHERE test_id = (SELECT id FROM tests WHERE session_id = ?1)",
        ),
        (
            "modified replay timestamp",
            "UPDATE test_replays SET timestamp_ms = 99 WHERE test_id = (SELECT id FROM tests WHERE session_id = ?1)",
        ),
        (
            "modified replay position",
            "UPDATE test_replays SET position = 99 WHERE test_id = (SELECT id FROM tests WHERE session_id = ?1)",
        ),
        (
            "modified replay typed character",
            "UPDATE test_replays SET typed_char = 'x' WHERE test_id = (SELECT id FROM tests WHERE session_id = ?1)",
        ),
        (
            "modified replay correctness",
            "UPDATE test_replays SET correct = 0 WHERE test_id = (SELECT id FROM tests WHERE session_id = ?1)",
        ),
        (
            "extra replay",
            "INSERT INTO test_replays (test_id, frame_index, timestamp_ms, position, expected_char, typed_char, correct)
             VALUES ((SELECT id FROM tests WHERE session_id = ?1), 1, 11, 2, 'z', 'z', 1)",
        ),
    ];

    for (name, mutation) in mutations {
        let database = Database::open_in_memory().expect("database");
        let id = session_id(SESSION_A);
        let completion = prepare_finalization_pending(&database, id.clone());
        SqliteFinalizationLedger::new(&database)
            .claim_finalization(
                &id,
                completion.fingerprint(),
                timestamp("2026-07-16T12:00:00Z"),
            )
            .expect("V008 claim");
        let finalizer = SqliteSessionFinalizer::new(&database);
        assert_eq!(
            finalizer.finalize_completion(&id, completion.fingerprint()),
            Ok(racoon_application::FinalizationOutcome::NewlyFinalized),
            "{name} setup"
        );
        let ledger_before = ledger_snapshot(&database, &id);
        let finalization_before = finalization_snapshot(&database, &id);
        database
            .conn()
            .execute(mutation, params![id.as_str()])
            .expect(name);

        assert_eq!(
            finalizer.finalize_completion(&id, completion.fingerprint()),
            Ok(racoon_application::FinalizationOutcome::Quarantined(
                QuarantineReason::InconsistentDurableMetadata
            )),
            "{name} must not be accepted as finalized"
        );
        assert_eq!(ledger_snapshot(&database, &id), ledger_before, "{name}");
        assert_eq!(
            finalization_snapshot(&database, &id),
            finalization_before,
            "{name}"
        );
    }
}

#[test]
fn session_finalizer_terminal_proof_rejects_missing_primary_test_without_rewrite() {
    let database = Database::open_in_memory().expect("database");
    let id = session_id(SESSION_A);
    let completion = prepare_finalization_pending(&database, id.clone());
    SqliteFinalizationLedger::new(&database)
        .claim_finalization(
            &id,
            completion.fingerprint(),
            timestamp("2026-07-16T12:00:00Z"),
        )
        .expect("V008 claim");
    let finalizer = SqliteSessionFinalizer::new(&database);
    assert_eq!(
        finalizer.finalize_completion(&id, completion.fingerprint()),
        Ok(racoon_application::FinalizationOutcome::NewlyFinalized)
    );
    let ledger_before = ledger_snapshot(&database, &id);
    let finalization_before = finalization_snapshot(&database, &id);
    let connection = database.conn();
    connection
        .execute(
            "DELETE FROM test_replays WHERE test_id = (SELECT id FROM tests WHERE session_id = ?1)",
            params![id.as_str()],
        )
        .expect("remove replay fixture");
    // This is an impossible production state: foreign keys normally retain
    // the primary result. Disable them only long enough to prove a terminal
    // retry does not accept a missing result if storage is externally corrupt.
    connection
        .execute_batch("PRAGMA foreign_keys = OFF")
        .expect("disable fixture foreign keys");
    connection
        .execute(
            "DELETE FROM tests WHERE session_id = ?1",
            params![id.as_str()],
        )
        .expect("remove primary test fixture");
    connection
        .execute_batch("PRAGMA foreign_keys = ON")
        .expect("restore foreign keys");
    drop(connection);

    assert_eq!(
        finalizer.finalize_completion(&id, completion.fingerprint()),
        Ok(racoon_application::FinalizationOutcome::Quarantined(
            QuarantineReason::InconsistentDurableMetadata
        ))
    );
    assert_eq!(ledger_snapshot(&database, &id), ledger_before);
    assert_eq!(finalization_snapshot(&database, &id), finalization_before);
}

#[test]
fn session_finalizer_reopen_retry_uses_complete_durable_effect_evidence() {
    let path = temporary_database_path("session-finalizer-ambiguous-success");
    remove_database(&path);
    let id = session_id(SESSION_A);
    let completion = {
        let database = Database::open(&path).expect("database");
        let completion = prepare_finalization_pending(&database, id.clone());
        SqliteFinalizationLedger::new(&database)
            .claim_finalization(
                &id,
                completion.fingerprint(),
                timestamp("2026-07-16T12:00:00Z"),
            )
            .expect("V008 claim");
        assert_eq!(
            SqliteSessionFinalizer::new(&database)
                .finalize_completion(&id, completion.fingerprint()),
            Ok(racoon_application::FinalizationOutcome::NewlyFinalized)
        );
        completion
    };

    let database = Database::open(&path).expect("reopened database");
    assert_eq!(
        SqliteSessionFinalizer::new(&database).finalize_completion(&id, completion.fingerprint()),
        Ok(racoon_application::FinalizationOutcome::AlreadyFinalized)
    );
    let connection = database.conn();
    let effects: (i64, i64, i64, i64, i64, i64, String, String) = connection
        .query_row(
            "SELECT
                (SELECT COUNT(*) FROM tests WHERE session_id = ?1),
                (SELECT COUNT(*) FROM test_replays WHERE test_id = (SELECT id FROM tests WHERE session_id = ?1)),
                (SELECT total_tests FROM daily_stats WHERE date = '2026-07-16'),
                (SELECT total_time_ms FROM daily_stats WHERE date = '2026-07-16'),
                (SELECT COUNT(*) FROM personal_bests),
                (SELECT current_streak FROM streaks WHERE type = 'daily_test'),
                (SELECT state FROM session_finalizations WHERE session_id = ?1),
                (SELECT state FROM session_ledger WHERE session_id = ?1)",
            params![id.as_str()],
            |row| {
                Ok((
                    row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?,
                    row.get(6)?, row.get(7)?,
                ))
            },
        )
        .expect("durable effects");
    assert_eq!(
        effects,
        (1, 1, 1, 1_000, 1, 1, "committed".into(), "finalized".into())
    );
    drop(connection);
    drop(database);
    remove_database(&path);
}

#[test]
fn session_finalizer_rolls_back_effects_when_lesson_completion_fails() {
    let database = Database::open_in_memory().expect("database");
    let id = session_id(SESSION_A);
    let completion = prepare_lesson_finalization_pending(&database, id.clone());
    SqliteFinalizationLedger::new(&database)
        .claim_finalization(
            &id,
            completion.fingerprint(),
            timestamp("2026-07-16T12:00:00Z"),
        )
        .expect("V008 claim");

    assert!(matches!(
        SqliteSessionFinalizer::new(&database).finalize_completion(&id, completion.fingerprint()),
        Err(racoon_application::RecoveryPortFailure::PermanentFailure(_))
    ));
    let connection = database.conn();
    let test_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM tests WHERE session_id = ?1",
            params![id.as_str()],
            |row| row.get(0),
        )
        .expect("test count");
    let daily_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM daily_stats", [], |row| row.get(0))
        .expect("daily count");
    let lesson_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM lesson_progress", [], |row| row.get(0))
        .expect("lesson count");
    let remaining_effects: (i64, i64, i64, i64) = connection
        .query_row(
            "SELECT
                (SELECT COUNT(*) FROM test_replays),
                (SELECT COUNT(*) FROM personal_bests),
                (SELECT COUNT(*) FROM streaks),
                (SELECT coalesce(SUM(daily_goal_met), 0) FROM daily_stats)",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .expect("remaining effect count");
    let state: String = connection
        .query_row(
            "SELECT state FROM session_ledger WHERE session_id = ?1",
            params![id.as_str()],
            |row| row.get(0),
        )
        .expect("ledger state");
    drop(connection);
    assert_eq!((test_count, daily_count, lesson_count), (0, 0, 0));
    assert_eq!(remaining_effects, (0, 0, 0, 0));
    assert_eq!(state, "finalization_pending");
    assert_eq!(finalization_snapshot(&database, &id).2, "pending");
}

#[cfg(feature = "test-support")]
#[test]
fn session_finalizer_rolls_back_every_injected_effect_boundary() {
    for failure_point in [
        FinalizerFailurePoint::BeforeTestInsertion,
        FinalizerFailurePoint::AfterTestInsertion,
        FinalizerFailurePoint::AfterReplayInsertion,
        FinalizerFailurePoint::AfterPersonalBestUpdate,
        FinalizerFailurePoint::AfterDailyStatisticsUpdate,
        FinalizerFailurePoint::BeforeFinalizationCommit,
        FinalizerFailurePoint::AfterFinalizationCommit,
        FinalizerFailurePoint::AfterSessionFinalized,
    ] {
        let database = Database::open_in_memory().expect("database");
        let id = session_id(SESSION_A);
        let completion = prepare_finalization_pending(&database, id.clone());
        SqliteFinalizationLedger::new(&database)
            .claim_finalization(
                &id,
                completion.fingerprint(),
                timestamp("2026-07-16T12:00:00Z"),
            )
            .expect("V008 claim");
        assert!(matches!(
            SqliteSessionFinalizer::with_failure_injection(&database, failure_point)
                .finalize_completion(&id, completion.fingerprint()),
            Err(racoon_application::RecoveryPortFailure::PermanentFailure(_))
        ));
        let connection = database.conn();
        let effect_rows: (i64, i64, i64, i64, i64, i64, i64) = connection
            .query_row(
                "SELECT
                    (SELECT COUNT(*) FROM tests WHERE session_id = ?1),
                    (SELECT COUNT(*) FROM test_replays),
                    (SELECT COUNT(*) FROM personal_bests),
                    (SELECT COUNT(*) FROM daily_stats),
                    (SELECT COUNT(*) FROM streaks),
                    (SELECT COUNT(*) FROM lesson_progress),
                    (SELECT coalesce(SUM(daily_goal_met), 0) FROM daily_stats)",
                params![id.as_str()],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                    ))
                },
            )
            .expect("effect rollback snapshot");
        assert_eq!(effect_rows, (0, 0, 0, 0, 0, 0, 0), "{failure_point:?}");
        drop(connection);
        assert_eq!(ledger_snapshot(&database, &id).0, "finalization_pending");
        assert_eq!(finalization_snapshot(&database, &id).2, "pending");
        assert_eq!(
            SqliteSessionFinalizer::new(&database)
                .finalize_completion(&id, completion.fingerprint()),
            Ok(racoon_application::FinalizationOutcome::NewlyFinalized),
            "{failure_point:?} retry"
        );
    }
}

#[test]
fn session_finalizer_converges_for_real_same_session_concurrency() {
    let path = temporary_database_path("session-finalizer-same-session");
    remove_database(&path);
    let setup = Database::open(&path).expect("setup database");
    let id = session_id(SESSION_A);
    let completion = prepare_finalization_pending(&setup, id.clone());
    SqliteFinalizationLedger::new(&setup)
        .claim_finalization(
            &id,
            completion.fingerprint(),
            timestamp("2026-07-16T12:00:00Z"),
        )
        .expect("claim");
    drop(setup);

    let barrier = Arc::new(Barrier::new(2));
    let first_path = path.clone();
    let second_path = path.clone();
    let first_id = id.clone();
    let second_id = id.clone();
    let first_fingerprint = completion.fingerprint().clone();
    let second_fingerprint = completion.fingerprint().clone();
    let first_barrier = Arc::clone(&barrier);
    let second_barrier = Arc::clone(&barrier);
    let first = thread::spawn(move || {
        let database = Database::open(&first_path).expect("first database");
        first_barrier.wait();
        SqliteSessionFinalizer::new(&database).finalize_completion(&first_id, &first_fingerprint)
    });
    let second = thread::spawn(move || {
        let database = Database::open(&second_path).expect("second database");
        second_barrier.wait();
        SqliteSessionFinalizer::new(&database).finalize_completion(&second_id, &second_fingerprint)
    });
    let outcomes = [
        first.join().expect("first finalization"),
        second.join().expect("second finalization"),
    ];
    assert!(outcomes.contains(&Ok(racoon_application::FinalizationOutcome::NewlyFinalized)));
    assert!(outcomes.contains(&Ok(
        racoon_application::FinalizationOutcome::AlreadyFinalized
    )));

    let database = Database::open(&path).expect("reopen database");
    let connection = database.conn();
    let counts: (i64, i64, i64) = connection
        .query_row(
            "SELECT
                (SELECT COUNT(*) FROM tests WHERE session_id = ?1),
                (SELECT COUNT(*) FROM test_replays WHERE test_id = (SELECT id FROM tests WHERE session_id = ?1)),
                (SELECT total_tests FROM daily_stats WHERE date = '2026-07-16')",
            params![id.as_str()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("exactly-once effects");
    assert_eq!(counts, (1, 1, 1));
    drop(connection);
    drop(database);
    remove_database(&path);
}

#[test]
fn session_finalizer_keeps_daily_aggregates_correct_for_concurrent_sessions() {
    let path = temporary_database_path("session-finalizer-different-sessions");
    remove_database(&path);
    let setup = Database::open(&path).expect("setup database");
    let first_id = session_id(SESSION_A);
    let second_id = session_id(SESSION_B);
    let first = prepare_finalization_pending(&setup, first_id.clone());
    let second = prepare_finalization_pending(&setup, second_id.clone());
    let ledger = SqliteFinalizationLedger::new(&setup);
    ledger
        .claim_finalization(
            &first_id,
            first.fingerprint(),
            timestamp("2026-07-16T12:00:00Z"),
        )
        .expect("first claim");
    ledger
        .claim_finalization(
            &second_id,
            second.fingerprint(),
            timestamp("2026-07-16T12:00:00Z"),
        )
        .expect("second claim");
    drop(setup);

    let barrier = Arc::new(Barrier::new(2));
    let first_path = path.clone();
    let second_path = path.clone();
    let first_fingerprint = first.fingerprint().clone();
    let second_fingerprint = second.fingerprint().clone();
    let first_barrier = Arc::clone(&barrier);
    let second_barrier = Arc::clone(&barrier);
    let first_thread = thread::spawn(move || {
        let database = Database::open(&first_path).expect("first database");
        first_barrier.wait();
        SqliteSessionFinalizer::new(&database).finalize_completion(&first_id, &first_fingerprint)
    });
    let second_thread = thread::spawn(move || {
        let database = Database::open(&second_path).expect("second database");
        second_barrier.wait();
        SqliteSessionFinalizer::new(&database).finalize_completion(&second_id, &second_fingerprint)
    });
    assert_eq!(
        first_thread.join().expect("first finalization"),
        Ok(racoon_application::FinalizationOutcome::NewlyFinalized)
    );
    assert_eq!(
        second_thread.join().expect("second finalization"),
        Ok(racoon_application::FinalizationOutcome::NewlyFinalized)
    );

    let database = Database::open(&path).expect("reopen database");
    let connection = database.conn();
    let counts: (i64, i64, i64) = connection
        .query_row(
            "SELECT
                (SELECT COUNT(*) FROM tests WHERE session_id IN (?1, ?2)),
                (SELECT total_tests FROM daily_stats WHERE date = '2026-07-16'),
                (SELECT total_time_ms FROM daily_stats WHERE date = '2026-07-16')",
            params![SESSION_A, SESSION_B],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("concurrent aggregate result");
    assert_eq!(counts, (2, 2, 2_000));
    drop(connection);
    drop(database);
    remove_database(&path);
}

#[test]
fn finalization_schema_rejects_fingerprint_mismatch_mutation_deletion_and_replacement() {
    let database = Database::open_in_memory().expect("database");
    let id = session_id(SESSION_A);
    let completion = prepare_finalization_pending(&database, id.clone());
    let finalizations = SqliteFinalizationLedger::new(&database);
    finalizations
        .claim_finalization(
            &id,
            completion.fingerprint(),
            timestamp("2026-07-16T12:00:00Z"),
        )
        .expect("claim");
    let before = finalization_snapshot(&database, &id);
    let different = "b".repeat(64);

    let connection = database.conn();
    for mutation in [
        "fingerprint = 'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb'",
        "claimed_at = '2026-07-16T12:05:00Z'",
    ] {
        assert!(connection
            .execute(
                &format!("UPDATE session_finalizations SET {mutation} WHERE session_id = ?1"),
                params![id.as_str()],
            )
            .is_err());
    }
    assert!(connection
        .execute(
            "DELETE FROM session_finalizations WHERE session_id = ?1",
            params![id.as_str()],
        )
        .is_err());
    assert!(connection
        .execute(
            "INSERT OR REPLACE INTO session_finalizations (
                session_id, fingerprint, state, claimed_at
             ) VALUES (?1, ?2, 'pending', '2026-07-16T12:05:00Z')",
            params![id.as_str(), &different],
        )
        .is_err());
    assert!(connection
        .execute(
            "REPLACE INTO session_finalizations (session_id, fingerprint, state, claimed_at)
             VALUES (?1, ?2, 'pending', '2026-07-16T12:05:00Z')",
            params![id.as_str(), &different],
        )
        .is_err());
    assert!(
        connection
            .execute(
                "INSERT OR REPLACE INTO session_ledger (
                session_id, state, mode_type, mode_descriptor, language, created_at, updated_at
             ) VALUES (?1, 'running', 'custom', '{}', 'en',
                '2026-07-16T12:05:00Z', '2026-07-16T12:05:00Z')",
                params![id.as_str()],
            )
            .is_err(),
        "parent replacement must not remove finalization descendants"
    );
    drop(connection);
    assert_eq!(finalization_snapshot(&database, &id), before);

    let mismatch_id = session_id(SESSION_B);
    let mismatch_completion = prepare_finalization_pending(&database, mismatch_id.clone());
    let connection = database.conn();
    assert!(
        connection
            .execute(
                "INSERT INTO session_finalizations (session_id, fingerprint, state, claimed_at)
             VALUES (?1, ?2, 'pending', '2026-07-16T12:00:00Z')",
                params![mismatch_id.as_str(), &different],
            )
            .is_err(),
        "V008 composite foreign key must reject a mismatched intent fingerprint"
    );
    drop(connection);
    assert_eq!(
        finalizations.claim_finalization(
            &mismatch_id,
            mismatch_completion.fingerprint(),
            timestamp("2026-07-16T12:00:00Z"),
        ),
        Ok(FinalizationLedgerClaimOutcome::Claimed)
    );
    finalizations
        .mark_finalization_committed(
            &mismatch_id,
            mismatch_completion.fingerprint(),
            timestamp("2026-07-16T12:01:00Z"),
        )
        .expect("commit");
    let connection = database.conn();
    assert!(
        connection
            .execute(
                "UPDATE session_finalizations SET state = 'pending', committed_at = NULL
             WHERE session_id = ?1",
                params![mismatch_id.as_str()],
            )
            .is_err(),
        "committed finalization must not reopen"
    );
}

#[test]
fn finalization_schema_timestamp_invariants_and_corrupt_rows_are_isolated() {
    let database = Database::open_in_memory().expect("database");
    let id = session_id(SESSION_A);
    let completion = prepare_finalization_pending(&database, id.clone());
    let connection = database.conn();
    assert!(
        connection
            .execute(
                "INSERT INTO session_finalizations (
                session_id, fingerprint, state, claimed_at, committed_at
             ) VALUES (?1, ?2, 'pending', '2026-07-16T12:00:00Z', '2026-07-16T12:01:00Z')",
                params![id.as_str(), completion.fingerprint().as_str()],
            )
            .is_err(),
        "pending records cannot carry committed_at"
    );
    assert!(
        connection
            .execute(
                "INSERT INTO session_finalizations (session_id, fingerprint, state, claimed_at)
             VALUES (?1, ?2, 'committed', '2026-07-16T12:00:00Z')",
                params![id.as_str(), completion.fingerprint().as_str()],
            )
            .is_err(),
        "initial finalization rows must be pending"
    );
    drop(connection);

    replace_finalization_table_with_corruption_fixture(&database);
    database
        .conn()
        .execute(
            "INSERT INTO session_finalizations VALUES (?1, ?2, 'unknown', 'invalid', NULL, NULL)",
            params![id.as_str(), completion.fingerprint().as_str()],
        )
        .expect("corrupt fixture row");
    assert_eq!(
        SqliteFinalizationLedger::new(&database).load_finalization(&id),
        Ok(FinalizationLoadOutcome::Corrupt)
    );

    let intent_corruption = Database::open_in_memory().expect("intent corruption database");
    let corrupt_id = session_id(SESSION_B);
    let recovery = SqliteSessionRecoveryLedger::new(&intent_corruption);
    recovery
        .record_started(&started(corrupt_id.clone(), "2026-07-16T10:00:00Z"))
        .expect("start");
    intent_corruption
        .conn()
        .execute(
            "UPDATE session_ledger SET state = 'finalization_pending' WHERE session_id = ?1",
            params![corrupt_id.as_str()],
        )
        .expect("fixture state");
    replace_intent_table_with_corruption_fixture(&intent_corruption);
    intent_corruption
        .conn()
        .execute(
            "INSERT INTO session_completion_intents VALUES (?1, 1, 1, 'NOT-LOWERCASE', X'7B7D', 2, '2026-07-16T12:00:00Z')",
            params![corrupt_id.as_str()],
        )
        .expect("corrupt intent fixture");
    let expected = CompletionIntentFingerprint::try_from_hex("a".repeat(64)).expect("fingerprint");
    assert_eq!(
        SqliteFinalizationLedger::new(&intent_corruption).claim_finalization(
            &corrupt_id,
            &expected,
            timestamp("2026-07-16T12:00:00Z"),
        ),
        Ok(FinalizationLedgerClaimOutcome::Quarantined(
            FinalizationQuarantineReason::CorruptDurableMetadata
        ))
    );
}

#[test]
fn finalization_claims_converge_under_real_concurrency_and_retry_after_lock_release() {
    let path = temporary_database_path("finalization-concurrency");
    remove_database(&path);
    let setup = Database::open(&path).expect("setup database");
    let id = session_id(SESSION_A);
    let completion = prepare_finalization_pending(&setup, id.clone());
    drop(setup);

    let barrier = Arc::new(Barrier::new(2));
    let first_path = path.clone();
    let second_path = path.clone();
    let first_id = id.clone();
    let second_id = id.clone();
    let first_fingerprint = completion.fingerprint().clone();
    let second_fingerprint = completion.fingerprint().clone();
    let first_barrier = Arc::clone(&barrier);
    let second_barrier = Arc::clone(&barrier);
    let first = thread::spawn(move || {
        let database = Database::open(&first_path).expect("first database");
        first_barrier.wait();
        SqliteFinalizationLedger::new(&database).claim_finalization(
            &first_id,
            &first_fingerprint,
            timestamp("2026-07-16T12:00:00Z"),
        )
    });
    let second = thread::spawn(move || {
        let database = Database::open(&second_path).expect("second database");
        second_barrier.wait();
        SqliteFinalizationLedger::new(&database).claim_finalization(
            &second_id,
            &second_fingerprint,
            timestamp("2026-07-16T12:00:01Z"),
        )
    });
    let outcomes = [
        first.join().expect("first claim"),
        second.join().expect("second claim"),
    ];
    assert!(outcomes.contains(&Ok(FinalizationLedgerClaimOutcome::Claimed)));
    assert!(outcomes.contains(&Ok(FinalizationLedgerClaimOutcome::AlreadyPending)));

    let database = Database::open(&path).expect("reopen database");
    {
        let connection = database.conn();
        connection
            .busy_timeout(Duration::from_millis(20))
            .expect("short busy timeout");
    }
    let lock_connection = Connection::open(&path).expect("lock connection");
    lock_connection
        .execute_batch("BEGIN IMMEDIATE")
        .expect("hold write reservation");
    let finalizations = SqliteFinalizationLedger::new(&database);
    assert_eq!(
        finalizations.mark_finalization_committed(
            &id,
            completion.fingerprint(),
            timestamp("2026-07-16T12:02:00Z"),
        ),
        Err(racoon_application::RecoveryPortFailure::RetryableFailure)
    );
    lock_connection
        .execute_batch("COMMIT")
        .expect("release lock");
    assert_eq!(
        finalizations.mark_finalization_committed(
            &id,
            completion.fingerprint(),
            timestamp("2026-07-16T12:02:00Z"),
        ),
        Ok(FinalizationCommitOutcome::Committed)
    );

    let conflict_id = session_id(SESSION_B);
    let conflict_completion = prepare_finalization_pending(&database, conflict_id.clone());
    let conflict_barrier = Arc::new(Barrier::new(2));
    let correct_path = path.clone();
    let conflicting_path = path.clone();
    let correct_id = conflict_id.clone();
    let conflicting_id = conflict_id.clone();
    let correct_fingerprint = conflict_completion.fingerprint().clone();
    let conflicting_fingerprint =
        CompletionIntentFingerprint::try_from_hex("b".repeat(64)).expect("fingerprint");
    let correct_barrier = Arc::clone(&conflict_barrier);
    let conflicting_barrier = Arc::clone(&conflict_barrier);
    let correct = thread::spawn(move || {
        let database = Database::open(&correct_path).expect("correct database");
        correct_barrier.wait();
        SqliteFinalizationLedger::new(&database).claim_finalization(
            &correct_id,
            &correct_fingerprint,
            timestamp("2026-07-16T12:03:00Z"),
        )
    });
    let conflicting = thread::spawn(move || {
        let database = Database::open(&conflicting_path).expect("conflicting database");
        conflicting_barrier.wait();
        SqliteFinalizationLedger::new(&database).claim_finalization(
            &conflicting_id,
            &conflicting_fingerprint,
            timestamp("2026-07-16T12:03:00Z"),
        )
    });
    let conflict_outcomes = [
        correct.join().expect("correct claim"),
        conflicting.join().expect("conflicting claim"),
    ];
    assert!(conflict_outcomes
        .iter()
        .any(|outcome| matches!(outcome, Ok(FinalizationLedgerClaimOutcome::Claimed))));
    assert!(conflict_outcomes
        .iter()
        .any(|outcome| { matches!(outcome, Ok(FinalizationLedgerClaimOutcome::Conflict(_))) }));
    drop(lock_connection);
    drop(database);
    remove_database(&path);
}
