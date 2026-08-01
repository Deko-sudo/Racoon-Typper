//! SQLite adapter for the application-owned durable session recovery ledger.
//!
//! This module owns SQL and transaction boundaries only. Completion payload
//! schema, header interpretation, recovery policy, and fingerprint validation
//! remain in `racoon-application`.

use chrono::SecondsFormat;
use racoon_application::recovery::CompletionIntentConflict;
use racoon_application::{
    classify_finalization_claim, validate_durable_state_transition,
    validate_sanitized_session_descriptor, CompletionIntent, CompletionIntentFingerprint,
    CompletionIntentLoadError, CompletionIntentLoadOutcome, CompletionIntentMetadata,
    DurableSessionState, DurableStateTransitionOutcome, FinalizationClaimOutcome,
    InterruptionReason, LedgerConflict, LedgerMutationOutcome, QuarantineReason, RecoveryCandidate,
    RecoveryPermanentFailure, RecoveryPortFailure, SessionRecoveryLedger, StartedSession,
    StoredCompletionIntentHeader, StoredHeaderValue, MAX_COMPLETION_INTENT_PAYLOAD_BYTES,
    MAX_SESSION_DESCRIPTOR_BYTES,
};
use racoon_domain::SessionId;
use rusqlite::{params, types::ValueRef, Connection, ErrorCode, OptionalExtension, Row};
use serde_json::Value;

use crate::{Database, DbError};

const SQLITE_UTC_NOW: &str = "strftime('%Y-%m-%dT%H:%M:%fZ', 'now')";
const MAX_MODE_TYPE_BYTES: usize = 64;
const MAX_LANGUAGE_BYTES: usize = 64;

/// Inward-facing SQLite implementation of [`SessionRecoveryLedger`].
///
/// It borrows the existing [`Database`], so it uses the project's sole
/// connection lock and `IMMEDIATE` transaction helper rather than introducing
/// another connection owner or repository registry.
pub struct SqliteSessionRecoveryLedger<'a> {
    database: &'a Database,
}

impl<'a> SqliteSessionRecoveryLedger<'a> {
    pub fn new(database: &'a Database) -> Self {
        Self { database }
    }
}

impl SessionRecoveryLedger for SqliteSessionRecoveryLedger<'_> {
    fn record_started(
        &self,
        session: &StartedSession,
    ) -> Result<LedgerMutationOutcome, RecoveryPortFailure> {
        let start = validated_start(session)?;
        self.database
            .with_transaction(|connection| {
                let existing = load_started_record(connection, session.session_id())?;
                let Some(existing) = existing else {
                    connection
                        .execute(
                            &format!(
                                "INSERT INTO session_ledger (
                                    session_id, state, mode_type, mode_descriptor, language,
                                    created_at, updated_at
                                 ) VALUES (?1, 'running', ?2, ?3, ?4, ?5, {SQLITE_UTC_NOW})"
                            ),
                            params![
                                session.session_id().as_str(),
                                start.mode_type,
                                start.mode_descriptor,
                                start.language,
                                start.started_at,
                            ],
                        )
                        .map_err(write_error)?;
                    return Ok(LedgerMutationOutcome::Created);
                };

                let state = parse_durable_state(&existing.state);
                if state == Some(DurableSessionState::Running)
                    && existing.mode_type == start.mode_type
                    && existing.language == start.language
                    && existing.started_at == start.started_at
                    && descriptor_matches(&existing.mode_descriptor, session.mode_descriptor())
                {
                    return Ok(LedgerMutationOutcome::AlreadyExistsIdentical);
                }

                if state.is_none() {
                    return Ok(LedgerMutationOutcome::Quarantined(
                        QuarantineReason::InvalidStateRecord,
                    ));
                }

                Ok(LedgerMutationOutcome::Conflicting(
                    LedgerConflict::SessionStart(session.session_id().clone()),
                ))
            })
            .map_err(port_failure)
    }

    fn record_completion_intent(
        &self,
        intent: &CompletionIntent,
    ) -> Result<LedgerMutationOutcome, RecoveryPortFailure> {
        validate_incoming_intent(intent)?;
        self.database
            .with_transaction(|connection| {
                let Some(state_text) = load_state(connection, intent.payload().session_id())? else {
                    return Ok(LedgerMutationOutcome::NotFound);
                };

                if let Some(existing) = load_stored_intent(connection, intent.payload().session_id())? {
                    return compare_stored_intent(&existing, intent);
                }

                match parse_durable_state(&state_text) {
                    Some(DurableSessionState::Running) => {
                        insert_intent(connection, intent)?;
                        let changed = connection
                            .execute(
                                &format!(
                                    "UPDATE session_ledger
                                     SET state = 'awaiting_persistence', updated_at = {SQLITE_UTC_NOW}
                                     WHERE session_id = ?1 AND state = 'running'"
                                ),
                                params![intent.payload().session_id().as_str()],
                            )
                            .map_err(write_error)?;
                        if changed != 1 {
                            return Err(DbError::Integrity(
                                "session state changed during completion intent recording".into(),
                            ));
                        }
                        Ok(LedgerMutationOutcome::Created)
                    }
                    Some(DurableSessionState::AwaitingPersistence)
                    | Some(DurableSessionState::FinalizationPending) => {
                        quarantine_row(
                            connection,
                            intent.payload().session_id(),
                            QuarantineReason::MissingCompletionIntent,
                        )?;
                        Ok(LedgerMutationOutcome::Quarantined(
                            QuarantineReason::MissingCompletionIntent,
                        ))
                    }
                    Some(DurableSessionState::Finalized)
                    | Some(DurableSessionState::Aborted)
                    | Some(DurableSessionState::Interrupted)
                    | Some(DurableSessionState::Quarantined) => Ok(
                        LedgerMutationOutcome::Quarantined(QuarantineReason::InvalidStateRecord),
                    ),
                    None => {
                        quarantine_row(
                            connection,
                            intent.payload().session_id(),
                            QuarantineReason::InvalidStateRecord,
                        )?;
                        Ok(LedgerMutationOutcome::Quarantined(
                            QuarantineReason::InvalidStateRecord,
                        ))
                    }
                }
            })
            .map_err(port_failure)
    }

    fn claim_completion_for_finalization(
        &self,
        session_id: &SessionId,
        expected_fingerprint: &CompletionIntentFingerprint,
    ) -> Result<FinalizationClaimOutcome, RecoveryPortFailure> {
        validate_session_id(session_id)?;
        self.database
            .with_transaction(|connection| {
                let candidate = load_candidate(connection, session_id)?;
                let outcome = classify_finalization_claim(candidate.as_ref(), expected_fingerprint);
                match outcome {
                    FinalizationClaimOutcome::Claimed => {
                        let changed = connection
                            .execute(
                                &format!(
                                    "UPDATE session_ledger
                                     SET state = 'finalization_pending', updated_at = {SQLITE_UTC_NOW}
                                     WHERE session_id = ?1 AND state = 'awaiting_persistence'"
                                ),
                                params![session_id.as_str()],
                            )
                            .map_err(write_error)?;
                        if changed != 1 {
                            return Err(DbError::Integrity(
                                "finalization claim did not update its awaiting session".into(),
                            ));
                        }
                    }
                    FinalizationClaimOutcome::Quarantined(reason) => {
                        quarantine_row(connection, session_id, reason)?;
                    }
                    FinalizationClaimOutcome::AlreadyPending
                    | FinalizationClaimOutcome::AlreadyFinalized
                    | FinalizationClaimOutcome::NotFound
                    | FinalizationClaimOutcome::Conflict(_)
                    | FinalizationClaimOutcome::RejectedTerminal { .. } => {}
                }
                Ok(outcome)
            })
            .map_err(port_failure)
    }

    fn list_recovery_candidates(&self) -> Result<Vec<RecoveryCandidate>, RecoveryPortFailure> {
        self.database
            .with_connection(|connection| {
                // Deliberately omits `canonical_payload`: candidate scanning reads
                // only ledger state and optional immutable-intent headers.
                let mut statement = connection
                    .prepare(
                        "SELECT
                            ledger.session_id,
                            ledger.state,
                            intent.session_id,
                            intent.canonicalization_version,
                            intent.payload_version,
                            intent.fingerprint
                         FROM session_ledger AS ledger
                         LEFT JOIN session_completion_intents AS intent
                           ON intent.session_id = ledger.session_id
                         ORDER BY ledger.created_at ASC, ledger.session_id ASC",
                    )
                    .map_err(query_error)?;
                let mut rows = statement.query([]).map_err(query_error)?;
                let mut candidates = Vec::new();
                while let Some(row) = rows.next().map_err(query_error)? {
                    candidates.push(candidate_from_row(row)?);
                }
                Ok(candidates)
            })
            .map_err(port_failure)
    }

    fn load_completion_intent(
        &self,
        session_id: &SessionId,
    ) -> Result<CompletionIntentLoadOutcome, RecoveryPortFailure> {
        validate_session_id(session_id)?;
        self.database
            .with_connection(|connection| {
                let Some(stored) = load_stored_intent(connection, session_id)? else {
                    return Ok(CompletionIntentLoadOutcome::NotFound);
                };
                let metadata = stored.metadata();
                let fingerprint = match metadata {
                    CompletionIntentMetadata::Present { .. } => metadata
                        .fingerprint()
                        .cloned()
                        .ok_or_else(|| DbError::Integrity("missing parsed fingerprint".into()))?,
                    CompletionIntentMetadata::UnsupportedCanonicalizationVersion { version } => {
                        return Ok(
                            CompletionIntentLoadOutcome::UnsupportedCanonicalizationVersion {
                                version,
                            },
                        )
                    }
                    CompletionIntentMetadata::UnsupportedVersion { version } => {
                        return Ok(CompletionIntentLoadOutcome::UnsupportedVersion { version })
                    }
                    CompletionIntentMetadata::Missing | CompletionIntentMetadata::Corrupt => {
                        return Ok(CompletionIntentLoadOutcome::Corrupt)
                    }
                };

                if metadata.session_id() != Some(session_id) {
                    return Ok(CompletionIntentLoadOutcome::Quarantined(
                        QuarantineReason::InconsistentDurableMetadata,
                    ));
                }
                let Some(payload) = stored.payload_bytes() else {
                    return Ok(CompletionIntentLoadOutcome::Corrupt);
                };
                let Some(recorded_length) = stored.payload_length() else {
                    return Ok(CompletionIntentLoadOutcome::Corrupt);
                };
                if recorded_length != payload.len()
                    || payload.len() > MAX_COMPLETION_INTENT_PAYLOAD_BYTES
                {
                    return Ok(CompletionIntentLoadOutcome::Corrupt);
                }

                match CompletionIntent::from_stored_payload(payload, &fingerprint) {
                    Ok(intent) if intent.payload().session_id() == session_id => {
                        Ok(CompletionIntentLoadOutcome::Found(Box::new(intent)))
                    }
                    Ok(_) => Ok(CompletionIntentLoadOutcome::Quarantined(
                        QuarantineReason::InconsistentDurableMetadata,
                    )),
                    Err(CompletionIntentLoadError::UnsupportedCanonicalizationVersion(error)) => {
                        Ok(
                            CompletionIntentLoadOutcome::UnsupportedCanonicalizationVersion {
                                version: error.version,
                            },
                        )
                    }
                    Err(CompletionIntentLoadError::UnsupportedVersion(error)) => {
                        Ok(CompletionIntentLoadOutcome::UnsupportedVersion {
                            version: error.version,
                        })
                    }
                    Err(
                        CompletionIntentLoadError::CorruptCanonicalPayload
                        | CompletionIntentLoadError::PayloadTooLarge
                        | CompletionIntentLoadError::FingerprintMismatch,
                    ) => Ok(CompletionIntentLoadOutcome::Corrupt),
                }
            })
            .map_err(port_failure)
    }

    fn mark_interrupted(
        &self,
        session_id: &SessionId,
        reason: InterruptionReason,
    ) -> Result<LedgerMutationOutcome, RecoveryPortFailure> {
        validate_session_id(session_id)?;
        self.database
            .with_transaction(|connection| {
                transition_with_reason(
                    connection,
                    session_id,
                    DurableSessionState::Interrupted,
                    Some(interruption_reason_name(reason)),
                    None,
                    None,
                )
            })
            .map_err(port_failure)
    }

    fn mark_aborted(
        &self,
        session_id: &SessionId,
    ) -> Result<LedgerMutationOutcome, RecoveryPortFailure> {
        validate_session_id(session_id)?;
        self.database
            .with_transaction(|connection| {
                transition_with_reason(
                    connection,
                    session_id,
                    DurableSessionState::Aborted,
                    None,
                    Some("explicit_abort"),
                    None,
                )
            })
            .map_err(port_failure)
    }

    fn quarantine(
        &self,
        session_id: &SessionId,
        reason: QuarantineReason,
    ) -> Result<LedgerMutationOutcome, RecoveryPortFailure> {
        validate_session_id(session_id)?;
        self.database
            .with_transaction(|connection| {
                transition_with_reason(
                    connection,
                    session_id,
                    DurableSessionState::Quarantined,
                    None,
                    None,
                    Some(reason),
                )
            })
            .map_err(port_failure)
    }
}

struct ValidatedStart {
    mode_type: String,
    mode_descriptor: String,
    language: String,
    started_at: String,
}

struct StartedRecord {
    state: String,
    mode_type: String,
    mode_descriptor: String,
    language: String,
    started_at: String,
}

#[derive(Clone)]
enum RawSqlValue {
    Null,
    Integer(i64),
    Text(String),
    Blob(Vec<u8>),
    Real,
}

struct StoredIntentRow {
    session_id: RawSqlValue,
    canonicalization_version: RawSqlValue,
    payload_version: RawSqlValue,
    fingerprint: RawSqlValue,
    canonical_payload: RawSqlValue,
    payload_byte_length: RawSqlValue,
}

impl StoredIntentRow {
    fn metadata(&self) -> CompletionIntentMetadata {
        CompletionIntentMetadata::from_stored_header(StoredCompletionIntentHeader::Present {
            session_id: header_text(&self.session_id),
            canonicalization_version: header_integer(&self.canonicalization_version),
            payload_version: header_integer(&self.payload_version),
            fingerprint: header_text(&self.fingerprint),
        })
    }

    fn payload_bytes(&self) -> Option<&[u8]> {
        match &self.canonical_payload {
            RawSqlValue::Blob(value) => Some(value),
            RawSqlValue::Null
            | RawSqlValue::Integer(_)
            | RawSqlValue::Text(_)
            | RawSqlValue::Real => None,
        }
    }

    fn payload_length(&self) -> Option<usize> {
        match self.payload_byte_length {
            RawSqlValue::Integer(value) => usize::try_from(value).ok(),
            RawSqlValue::Null | RawSqlValue::Text(_) | RawSqlValue::Blob(_) | RawSqlValue::Real => {
                None
            }
        }
    }
}

fn validated_start(session: &StartedSession) -> Result<ValidatedStart, RecoveryPortFailure> {
    validate_session_id(session.session_id())?;
    validate_sanitized_session_descriptor(session.mode_descriptor())
        .map_err(|_| invalid_contract())?;
    if session.mode_type().is_empty()
        || session.mode_type().len() > MAX_MODE_TYPE_BYTES
        || session.language().is_empty()
        || session.language().len() > MAX_LANGUAGE_BYTES
    {
        return Err(invalid_contract());
    }
    let mode_descriptor =
        serde_json::to_string(session.mode_descriptor()).map_err(|_| invalid_contract())?;
    if mode_descriptor.len() > MAX_SESSION_DESCRIPTOR_BYTES {
        return Err(invalid_contract());
    }
    Ok(ValidatedStart {
        mode_type: session.mode_type().to_string(),
        mode_descriptor,
        language: session.language().to_string(),
        started_at: session
            .started_at()
            .to_rfc3339_opts(SecondsFormat::Micros, true),
    })
}

fn validate_incoming_intent(intent: &CompletionIntent) -> Result<(), RecoveryPortFailure> {
    validate_session_id(intent.payload().session_id())?;
    let restored =
        CompletionIntent::from_stored_payload(intent.canonical_payload(), intent.fingerprint())
            .map_err(|_| invalid_contract())?;
    if restored != *intent || restored.payload().session_id() != intent.payload().session_id() {
        return Err(invalid_contract());
    }
    Ok(())
}

fn validate_session_id(session_id: &SessionId) -> Result<(), RecoveryPortFailure> {
    SessionId::parse(session_id.as_str())
        .map(|_| ())
        .map_err(|_| invalid_contract())
}

fn invalid_contract() -> RecoveryPortFailure {
    RecoveryPortFailure::PermanentFailure(RecoveryPermanentFailure::InvalidContract)
}

fn load_started_record(
    connection: &Connection,
    session_id: &SessionId,
) -> Result<Option<StartedRecord>, DbError> {
    connection
        .query_row(
            "SELECT state, mode_type, mode_descriptor, language, created_at
             FROM session_ledger WHERE session_id = ?1",
            params![session_id.as_str()],
            |row| {
                Ok(StartedRecord {
                    state: row.get(0)?,
                    mode_type: row.get(1)?,
                    mode_descriptor: row.get(2)?,
                    language: row.get(3)?,
                    started_at: row.get(4)?,
                })
            },
        )
        .optional()
        .map_err(query_error)
}

fn load_state(connection: &Connection, session_id: &SessionId) -> Result<Option<String>, DbError> {
    connection
        .query_row(
            "SELECT state FROM session_ledger WHERE session_id = ?1",
            params![session_id.as_str()],
            |row| row.get(0),
        )
        .optional()
        .map_err(query_error)
}

fn load_candidate(
    connection: &Connection,
    session_id: &SessionId,
) -> Result<Option<RecoveryCandidate>, DbError> {
    let mut statement = connection
        .prepare(
            "SELECT
                ledger.session_id,
                ledger.state,
                intent.session_id,
                intent.canonicalization_version,
                intent.payload_version,
                intent.fingerprint
             FROM session_ledger AS ledger
             LEFT JOIN session_completion_intents AS intent
               ON intent.session_id = ledger.session_id
             WHERE ledger.session_id = ?1",
        )
        .map_err(query_error)?;
    let mut rows = statement
        .query(params![session_id.as_str()])
        .map_err(query_error)?;
    match rows.next().map_err(query_error)? {
        Some(row) => candidate_from_row(row).map(Some),
        None => Ok(None),
    }
}

fn candidate_from_row(row: &Row<'_>) -> Result<RecoveryCandidate, DbError> {
    let ledger_session_id = required_text(row, 0)?;
    let session_id = SessionId::parse(ledger_session_id)
        .map_err(|_| DbError::Integrity("invalid ledger session identity".into()))?;
    let state = required_text(row, 1).ok().and_then(parse_durable_state);
    if let Some(state) = state {
        let intent_session_id = value_ref(row, 2)?;
        let metadata = match intent_session_id {
            ValueRef::Null => {
                CompletionIntentMetadata::from_stored_header(StoredCompletionIntentHeader::Missing)
            }
            value => CompletionIntentMetadata::from_stored_header(
                StoredCompletionIntentHeader::Present {
                    session_id: header_text_value(value),
                    canonicalization_version: header_integer_value(value_ref(row, 3)?),
                    payload_version: header_integer_value(value_ref(row, 4)?),
                    fingerprint: header_text_value(value_ref(row, 5)?),
                },
            ),
        };
        Ok(RecoveryCandidate::new(session_id, state, metadata))
    } else {
        // An unknown or wrongly typed state cannot be represented directly by
        // the application enum. Represent it as a non-terminal corrupt record
        // so existing pure policy requests durable quarantine instead of
        // silently skipping it or resuming it.
        Ok(RecoveryCandidate::new(
            session_id,
            DurableSessionState::Running,
            CompletionIntentMetadata::Corrupt,
        ))
    }
}

fn load_stored_intent(
    connection: &Connection,
    session_id: &SessionId,
) -> Result<Option<StoredIntentRow>, DbError> {
    let mut statement = connection
        .prepare(
            "SELECT session_id, canonicalization_version, payload_version,
                    fingerprint, canonical_payload, payload_byte_length
             FROM session_completion_intents WHERE session_id = ?1",
        )
        .map_err(query_error)?;
    let mut rows = statement
        .query(params![session_id.as_str()])
        .map_err(query_error)?;
    let Some(row) = rows.next().map_err(query_error)? else {
        return Ok(None);
    };
    Ok(Some(StoredIntentRow {
        session_id: raw_value(row, 0)?,
        canonicalization_version: raw_value(row, 1)?,
        payload_version: raw_value(row, 2)?,
        fingerprint: raw_value(row, 3)?,
        canonical_payload: raw_value(row, 4)?,
        payload_byte_length: raw_value(row, 5)?,
    }))
}

fn compare_stored_intent(
    stored: &StoredIntentRow,
    incoming: &CompletionIntent,
) -> Result<LedgerMutationOutcome, DbError> {
    let metadata = stored.metadata();
    if !matches!(&metadata, CompletionIntentMetadata::Present { .. }) {
        return Ok(LedgerMutationOutcome::Quarantined(
            metadata_quarantine_reason(&metadata),
        ));
    }
    let Some(existing_session_id) = metadata.session_id() else {
        return Ok(LedgerMutationOutcome::Quarantined(
            QuarantineReason::CorruptCompletionPayload,
        ));
    };
    let Some(existing_fingerprint) = metadata.fingerprint() else {
        return Ok(LedgerMutationOutcome::Quarantined(
            QuarantineReason::CorruptCompletionPayload,
        ));
    };
    let Some(existing_payload) = stored.payload_bytes() else {
        return Ok(LedgerMutationOutcome::Quarantined(
            QuarantineReason::CorruptCompletionPayload,
        ));
    };
    let Some(existing_length) = stored.payload_length() else {
        return Ok(LedgerMutationOutcome::Quarantined(
            QuarantineReason::CorruptCompletionPayload,
        ));
    };
    let stored_intent = if existing_length != existing_payload.len() {
        None
    } else {
        CompletionIntent::from_stored_payload(existing_payload, existing_fingerprint).ok()
    };
    let Some(stored_intent) = stored_intent else {
        return Ok(LedgerMutationOutcome::Quarantined(
            QuarantineReason::CorruptCompletionPayload,
        ));
    };
    if existing_session_id != incoming.payload().session_id()
        || stored_intent.payload().session_id() != incoming.payload().session_id()
    {
        return Ok(LedgerMutationOutcome::Quarantined(
            QuarantineReason::InconsistentDurableMetadata,
        ));
    }
    if existing_fingerprint != incoming.fingerprint() {
        return Ok(LedgerMutationOutcome::Conflicting(
            LedgerConflict::CompletionIntent(CompletionIntentConflict {
                existing_session_id: existing_session_id.clone(),
                incoming_session_id: incoming.payload().session_id().clone(),
                existing_fingerprint: existing_fingerprint.clone(),
                incoming_fingerprint: incoming.fingerprint().clone(),
            }),
        ));
    }
    if existing_payload != incoming.canonical_payload() {
        return Ok(LedgerMutationOutcome::Quarantined(
            QuarantineReason::CorruptCompletionPayload,
        ));
    }
    Ok(LedgerMutationOutcome::AlreadyExistsIdentical)
}

fn insert_intent(connection: &Connection, intent: &CompletionIntent) -> Result<(), DbError> {
    connection
        .execute(
            &format!(
                "INSERT INTO session_completion_intents (
                    session_id, canonicalization_version, payload_version,
                    fingerprint, canonical_payload, payload_byte_length, created_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, {SQLITE_UTC_NOW})"
            ),
            params![
                intent.payload().session_id().as_str(),
                i64::from(intent.canonicalization_version().as_u16()),
                i64::from(intent.payload_version().as_u16()),
                intent.fingerprint().as_str(),
                intent.canonical_payload(),
                i64::try_from(intent.canonical_payload().len()).map_err(
                    |_| DbError::Validation("payload length overflows SQLite integer".into())
                )?,
            ],
        )
        .map_err(write_error)?;
    Ok(())
}

fn transition_with_reason(
    connection: &Connection,
    session_id: &SessionId,
    target: DurableSessionState,
    interruption_reason: Option<&str>,
    abort_reason: Option<&str>,
    quarantine_reason: Option<QuarantineReason>,
) -> Result<LedgerMutationOutcome, DbError> {
    let Some(current) = load_state(connection, session_id)? else {
        return Ok(LedgerMutationOutcome::NotFound);
    };
    let Some(current) = parse_durable_state(&current) else {
        quarantine_row(connection, session_id, QuarantineReason::InvalidStateRecord)?;
        return Ok(LedgerMutationOutcome::Quarantined(
            QuarantineReason::InvalidStateRecord,
        ));
    };
    match validate_durable_state_transition(Some(current), target) {
        DurableStateTransitionOutcome::Idempotent => {
            Ok(LedgerMutationOutcome::AlreadyExistsIdentical)
        }
        DurableStateTransitionOutcome::Valid => {
            connection
                .execute(
                    &format!(
                        "UPDATE session_ledger
                         SET state = ?2,
                             interruption_reason = ?3,
                             abort_reason = ?4,
                             quarantine_reason = ?5,
                             updated_at = {SQLITE_UTC_NOW}
                         WHERE session_id = ?1"
                    ),
                    params![
                        session_id.as_str(),
                        state_name(target),
                        interruption_reason,
                        abort_reason,
                        quarantine_reason.map(quarantine_reason_name),
                    ],
                )
                .map_err(write_error)?;
            Ok(LedgerMutationOutcome::Created)
        }
        DurableStateTransitionOutcome::Invalid { .. }
        | DurableStateTransitionOutcome::ForbiddenFromTerminal { .. } => Ok(
            LedgerMutationOutcome::Quarantined(QuarantineReason::InvalidStateRecord),
        ),
    }
}

fn quarantine_row(
    connection: &Connection,
    session_id: &SessionId,
    reason: QuarantineReason,
) -> Result<(), DbError> {
    connection
        .execute(
            &format!(
                "UPDATE session_ledger
                 SET state = 'quarantined', quarantine_reason = ?2, updated_at = {SQLITE_UTC_NOW}
                 WHERE session_id = ?1
                   AND state NOT IN ('finalized', 'aborted', 'interrupted', 'quarantined')"
            ),
            params![session_id.as_str(), quarantine_reason_name(reason)],
        )
        .map_err(write_error)?;
    Ok(())
}

fn descriptor_matches(stored: &str, incoming: &Value) -> bool {
    serde_json::from_str::<Value>(stored)
        .map(|descriptor| descriptor == *incoming)
        .unwrap_or(false)
}

fn parse_durable_state(value: &str) -> Option<DurableSessionState> {
    serde_json::from_value(Value::String(value.to_owned())).ok()
}

fn state_name(state: DurableSessionState) -> &'static str {
    match state {
        DurableSessionState::Running => "running",
        DurableSessionState::AwaitingPersistence => "awaiting_persistence",
        DurableSessionState::FinalizationPending => "finalization_pending",
        DurableSessionState::Finalized => "finalized",
        DurableSessionState::Aborted => "aborted",
        DurableSessionState::Interrupted => "interrupted",
        DurableSessionState::Quarantined => "quarantined",
    }
}

fn interruption_reason_name(reason: InterruptionReason) -> &'static str {
    match reason {
        InterruptionReason::ProcessRestart => "process_restart",
    }
}

fn quarantine_reason_name(reason: QuarantineReason) -> &'static str {
    match reason {
        QuarantineReason::UnsupportedCanonicalizationVersion => {
            "unsupported_canonicalization_version"
        }
        QuarantineReason::UnsupportedIntentVersion => "unsupported_intent_version",
        QuarantineReason::CorruptCompletionPayload => "corrupt_completion_payload",
        QuarantineReason::MissingCompletionIntent => "missing_completion_intent",
        QuarantineReason::ConflictingCompletionIntent => "conflicting_completion_intent",
        QuarantineReason::InvalidStateRecord => "invalid_state_record",
        QuarantineReason::FingerprintMismatch => "fingerprint_mismatch",
        QuarantineReason::InconsistentDurableMetadata => "inconsistent_durable_metadata",
    }
}

fn metadata_quarantine_reason(metadata: &CompletionIntentMetadata) -> QuarantineReason {
    match metadata {
        CompletionIntentMetadata::Missing => QuarantineReason::MissingCompletionIntent,
        CompletionIntentMetadata::UnsupportedCanonicalizationVersion { .. } => {
            QuarantineReason::UnsupportedCanonicalizationVersion
        }
        CompletionIntentMetadata::UnsupportedVersion { .. } => {
            QuarantineReason::UnsupportedIntentVersion
        }
        CompletionIntentMetadata::Corrupt => QuarantineReason::CorruptCompletionPayload,
        CompletionIntentMetadata::Present { .. } => QuarantineReason::InconsistentDurableMetadata,
    }
}

fn raw_value(row: &Row<'_>, index: usize) -> Result<RawSqlValue, DbError> {
    Ok(match value_ref(row, index)? {
        ValueRef::Null => RawSqlValue::Null,
        ValueRef::Integer(value) => RawSqlValue::Integer(value),
        ValueRef::Real(_) => RawSqlValue::Real,
        ValueRef::Text(value) => match std::str::from_utf8(value) {
            Ok(value) => RawSqlValue::Text(value.to_owned()),
            Err(_) => RawSqlValue::Real,
        },
        ValueRef::Blob(value) => RawSqlValue::Blob(value.to_vec()),
    })
}

fn value_ref<'a>(row: &'a Row<'_>, index: usize) -> Result<ValueRef<'a>, DbError> {
    row.get_ref(index).map_err(query_error)
}

fn required_text<'a>(row: &'a Row<'_>, index: usize) -> Result<&'a str, DbError> {
    match value_ref(row, index)? {
        ValueRef::Text(value) => std::str::from_utf8(value)
            .map_err(|_| DbError::Integrity("invalid UTF-8 in required text column".into())),
        _ => Err(DbError::Integrity(
            "invalid required text column type".into(),
        )),
    }
}

fn header_text(value: &RawSqlValue) -> StoredHeaderValue<&str> {
    match value {
        RawSqlValue::Null => StoredHeaderValue::Missing,
        RawSqlValue::Text(value) => StoredHeaderValue::Value(value),
        RawSqlValue::Integer(_) | RawSqlValue::Blob(_) | RawSqlValue::Real => {
            StoredHeaderValue::Invalid
        }
    }
}

fn header_integer(value: &RawSqlValue) -> StoredHeaderValue<i64> {
    match value {
        RawSqlValue::Null => StoredHeaderValue::Missing,
        RawSqlValue::Integer(value) => StoredHeaderValue::Value(*value),
        RawSqlValue::Text(_) | RawSqlValue::Blob(_) | RawSqlValue::Real => {
            StoredHeaderValue::Invalid
        }
    }
}

fn header_text_value(value: ValueRef<'_>) -> StoredHeaderValue<&str> {
    match value {
        ValueRef::Null => StoredHeaderValue::Missing,
        ValueRef::Text(value) => std::str::from_utf8(value)
            .map(StoredHeaderValue::Value)
            .unwrap_or(StoredHeaderValue::Invalid),
        ValueRef::Integer(_) | ValueRef::Real(_) | ValueRef::Blob(_) => StoredHeaderValue::Invalid,
    }
}

fn header_integer_value(value: ValueRef<'_>) -> StoredHeaderValue<i64> {
    match value {
        ValueRef::Null => StoredHeaderValue::Missing,
        ValueRef::Integer(value) => StoredHeaderValue::Value(value),
        ValueRef::Real(_) | ValueRef::Text(_) | ValueRef::Blob(_) => StoredHeaderValue::Invalid,
    }
}

fn query_error(error: rusqlite::Error) -> DbError {
    DbError::from_sqlite("session ledger query", error)
}

fn write_error(error: rusqlite::Error) -> DbError {
    DbError::from_sqlite("session ledger write", error)
}

fn port_failure(error: DbError) -> RecoveryPortFailure {
    match error {
        DbError::Migration(_) => {
            RecoveryPortFailure::PermanentFailure(RecoveryPermanentFailure::UnsupportedSchema)
        }
        DbError::Integrity(_) | DbError::Validation(_) | DbError::NotFound(_) => {
            RecoveryPortFailure::PermanentFailure(RecoveryPermanentFailure::IntegrityFailure)
        }
        DbError::Sqlite { code, .. } => {
            if matches!(code, ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked) {
                RecoveryPortFailure::RetryableFailure
            } else {
                RecoveryPortFailure::PermanentFailure(RecoveryPermanentFailure::IntegrityFailure)
            }
        }
        DbError::Connection(_)
        | DbError::Query(_)
        | DbError::Write(_)
        | DbError::Transaction(_)
        | DbError::Backup(_)
        | DbError::Restore(_) => {
            RecoveryPortFailure::PermanentFailure(RecoveryPermanentFailure::IntegrityFailure)
        }
        DbError::LockPoisoned => RecoveryPortFailure::RetryableFailure,
    }
}
