//! SQLite adapter for the durable, effect-free finalization ledger.
//!
//! This adapter records only a fingerprint-bound claim and committed marker.
//! It never writes tests, replays, statistics, lessons, or other completion
//! effects; later orchestration must compose those effects atomically.

use chrono::{DateTime, SecondsFormat, Utc};
use racoon_application::{
    CompletionIntentFingerprint, DurableSessionState, FinalizationCommitOutcome,
    FinalizationConflict, FinalizationLedger, FinalizationLedgerClaimOutcome,
    FinalizationLedgerState, FinalizationLoadOutcome, FinalizationQuarantineReason,
    FinalizationRecord, RecoveryPermanentFailure, RecoveryPortFailure,
};
use racoon_domain::SessionId;
use rusqlite::{params, types::ValueRef, Connection, ErrorCode, OptionalExtension};

use crate::{Database, DbError};

/// Inward-facing SQLite implementation of the application finalization ledger.
pub struct SqliteFinalizationLedger<'a> {
    database: &'a Database,
}

impl<'a> SqliteFinalizationLedger<'a> {
    pub fn new(database: &'a Database) -> Self {
        Self { database }
    }
}

impl FinalizationLedger for SqliteFinalizationLedger<'_> {
    fn claim_finalization(
        &self,
        session_id: &SessionId,
        expected_fingerprint: &CompletionIntentFingerprint,
        claimed_at: DateTime<Utc>,
    ) -> Result<FinalizationLedgerClaimOutcome, RecoveryPortFailure> {
        validate_session_id(session_id)?;
        self.database
            .with_transaction(|connection| {
                let Some(session_state) = load_session_state(connection, session_id)? else {
                    return Ok(FinalizationLedgerClaimOutcome::NotFound);
                };
                if session_state != Some(DurableSessionState::FinalizationPending) {
                    return Ok(FinalizationLedgerClaimOutcome::Quarantined(
                        FinalizationQuarantineReason::InvalidFinalizationState,
                    ));
                }

                let intent_fingerprint = match load_intent_fingerprint(connection, session_id)? {
                    IntentFingerprint::Missing => {
                        return Ok(FinalizationLedgerClaimOutcome::MissingCompletionIntent)
                    }
                    IntentFingerprint::Corrupt => {
                        return Ok(FinalizationLedgerClaimOutcome::Quarantined(
                            FinalizationQuarantineReason::CorruptDurableMetadata,
                        ))
                    }
                    IntentFingerprint::Found(fingerprint) => fingerprint,
                };
                if intent_fingerprint != *expected_fingerprint {
                    return Ok(FinalizationLedgerClaimOutcome::Conflict(conflict(
                        session_id,
                        expected_fingerprint,
                        &intent_fingerprint,
                    )));
                }

                match load_finalization_record(connection, session_id)? {
                    StoredFinalization::Missing => {
                        connection
                            .execute(
                                "INSERT INTO session_finalizations (
                                    session_id, fingerprint, state, claimed_at
                                 ) VALUES (?1, ?2, 'pending', ?3)",
                                params![
                                    session_id.as_str(),
                                    expected_fingerprint.as_str(),
                                    format_utc(claimed_at),
                                ],
                            )
                            .map_err(write_error)?;
                        Ok(FinalizationLedgerClaimOutcome::Claimed)
                    }
                    StoredFinalization::Corrupt => Ok(FinalizationLedgerClaimOutcome::Corrupt),
                    StoredFinalization::Found(record) => {
                        if record.fingerprint() != expected_fingerprint {
                            return Ok(FinalizationLedgerClaimOutcome::Conflict(conflict(
                                session_id,
                                expected_fingerprint,
                                record.fingerprint(),
                            )));
                        }
                        if record.fingerprint() != &intent_fingerprint {
                            return quarantine_pending_or_report(
                                connection,
                                &record,
                                FinalizationQuarantineReason::FingerprintMismatch,
                            )
                            .map(FinalizationLedgerClaimOutcome::Quarantined);
                        }
                        match record.state() {
                            FinalizationLedgerState::Pending => {
                                Ok(FinalizationLedgerClaimOutcome::AlreadyPending)
                            }
                            FinalizationLedgerState::Committed => {
                                Ok(FinalizationLedgerClaimOutcome::AlreadyCommitted)
                            }
                            FinalizationLedgerState::Quarantined => {
                                Ok(FinalizationLedgerClaimOutcome::Quarantined(
                                    record.quarantine_reason().expect("validated record"),
                                ))
                            }
                        }
                    }
                }
            })
            .map_err(port_failure)
    }

    fn mark_finalization_committed(
        &self,
        session_id: &SessionId,
        expected_fingerprint: &CompletionIntentFingerprint,
        committed_at: DateTime<Utc>,
    ) -> Result<FinalizationCommitOutcome, RecoveryPortFailure> {
        validate_session_id(session_id)?;
        self.database
            .with_transaction(|connection| {
                let Some(session_state) = load_session_state(connection, session_id)? else {
                    return Ok(FinalizationCommitOutcome::NotFound);
                };
                if session_state != Some(DurableSessionState::FinalizationPending) {
                    return Ok(FinalizationCommitOutcome::Quarantined(
                        FinalizationQuarantineReason::InvalidFinalizationState,
                    ));
                }
                let intent_fingerprint = match load_intent_fingerprint(connection, session_id)? {
                    IntentFingerprint::Missing => {
                        return Ok(FinalizationCommitOutcome::Quarantined(
                            FinalizationQuarantineReason::MissingCompletionIntent,
                        ))
                    }
                    IntentFingerprint::Corrupt => {
                        return Ok(FinalizationCommitOutcome::Quarantined(
                            FinalizationQuarantineReason::CorruptDurableMetadata,
                        ))
                    }
                    IntentFingerprint::Found(fingerprint) => fingerprint,
                };
                if intent_fingerprint != *expected_fingerprint {
                    return Ok(FinalizationCommitOutcome::Conflict(conflict(
                        session_id,
                        expected_fingerprint,
                        &intent_fingerprint,
                    )));
                }
                match load_finalization_record(connection, session_id)? {
                    StoredFinalization::Missing => Ok(FinalizationCommitOutcome::NotFound),
                    StoredFinalization::Corrupt => Ok(FinalizationCommitOutcome::Corrupt),
                    StoredFinalization::Found(record) => {
                        if record.fingerprint() != expected_fingerprint {
                            return Ok(FinalizationCommitOutcome::Conflict(conflict(
                                session_id,
                                expected_fingerprint,
                                record.fingerprint(),
                            )));
                        }
                        if record.fingerprint() != &intent_fingerprint {
                            return quarantine_pending_or_report(
                                connection,
                                &record,
                                FinalizationQuarantineReason::FingerprintMismatch,
                            )
                            .map(FinalizationCommitOutcome::Quarantined);
                        }
                        match record.state() {
                            FinalizationLedgerState::Pending => {
                                let changed = connection
                                    .execute(
                                        "UPDATE session_finalizations
                                         SET state = 'committed', committed_at = ?2
                                         WHERE session_id = ?1 AND state = 'pending'",
                                        params![session_id.as_str(), format_utc(committed_at)],
                                    )
                                    .map_err(write_error)?;
                                if changed != 1 {
                                    return Err(DbError::Integrity(
                                        "finalization commit lost its pending record".into(),
                                    ));
                                }
                                Ok(FinalizationCommitOutcome::Committed)
                            }
                            FinalizationLedgerState::Committed => {
                                Ok(FinalizationCommitOutcome::AlreadyCommitted)
                            }
                            FinalizationLedgerState::Quarantined => {
                                Ok(FinalizationCommitOutcome::Quarantined(
                                    record.quarantine_reason().expect("validated record"),
                                ))
                            }
                        }
                    }
                }
            })
            .map_err(port_failure)
    }

    fn load_finalization(
        &self,
        session_id: &SessionId,
    ) -> Result<FinalizationLoadOutcome, RecoveryPortFailure> {
        validate_session_id(session_id)?;
        self.database
            .with_connection(
                |connection| match load_finalization_record(connection, session_id)? {
                    StoredFinalization::Missing => Ok(FinalizationLoadOutcome::NotFound),
                    StoredFinalization::Corrupt => Ok(FinalizationLoadOutcome::Corrupt),
                    StoredFinalization::Found(record) => {
                        match load_intent_fingerprint(connection, session_id)? {
                            IntentFingerprint::Found(fingerprint)
                                if fingerprint == *record.fingerprint() =>
                            {
                                Ok(FinalizationLoadOutcome::Found(record))
                            }
                            IntentFingerprint::Missing
                            | IntentFingerprint::Corrupt
                            | IntentFingerprint::Found(_) => Ok(FinalizationLoadOutcome::Corrupt),
                        }
                    }
                },
            )
            .map_err(port_failure)
    }

    fn quarantine_finalization(
        &self,
        session_id: &SessionId,
        expected_fingerprint: &CompletionIntentFingerprint,
        reason: FinalizationQuarantineReason,
    ) -> Result<FinalizationCommitOutcome, RecoveryPortFailure> {
        validate_session_id(session_id)?;
        self.database
            .with_transaction(|connection| {
                match load_finalization_record(connection, session_id)? {
                    StoredFinalization::Missing => Ok(FinalizationCommitOutcome::NotFound),
                    StoredFinalization::Corrupt => Ok(FinalizationCommitOutcome::Corrupt),
                    StoredFinalization::Found(record) => {
                        if record.fingerprint() != expected_fingerprint {
                            return Ok(FinalizationCommitOutcome::Conflict(conflict(
                                session_id,
                                expected_fingerprint,
                                record.fingerprint(),
                            )));
                        }
                        let durable_reason = match load_intent_fingerprint(connection, session_id)?
                        {
                            IntentFingerprint::Missing => {
                                Some(FinalizationQuarantineReason::MissingCompletionIntent)
                            }
                            IntentFingerprint::Corrupt => {
                                Some(FinalizationQuarantineReason::CorruptDurableMetadata)
                            }
                            IntentFingerprint::Found(fingerprint)
                                if fingerprint != *record.fingerprint() =>
                            {
                                Some(FinalizationQuarantineReason::FingerprintMismatch)
                            }
                            IntentFingerprint::Found(_) => None,
                        };

                        let reason = durable_reason.unwrap_or(reason);
                        quarantine_record_or_preserve_terminal(connection, &record, reason)
                    }
                }
            })
            .map_err(port_failure)
    }
}

enum IntentFingerprint {
    Missing,
    Found(CompletionIntentFingerprint),
    Corrupt,
}

enum StoredFinalization {
    Missing,
    Found(FinalizationRecord),
    Corrupt,
}

fn validate_session_id(session_id: &SessionId) -> Result<(), RecoveryPortFailure> {
    SessionId::parse(session_id.as_str())
        .map(|_| ())
        .map_err(|_| invalid_contract())
}

fn invalid_contract() -> RecoveryPortFailure {
    RecoveryPortFailure::PermanentFailure(RecoveryPermanentFailure::InvalidContract)
}

fn load_session_state(
    connection: &Connection,
    session_id: &SessionId,
) -> Result<Option<Option<DurableSessionState>>, DbError> {
    connection
        .query_row(
            "SELECT state FROM session_ledger WHERE session_id = ?1",
            params![session_id.as_str()],
            |row| {
                let value = row.get_ref(0)?;
                Ok(match value {
                    ValueRef::Text(value) => std::str::from_utf8(value)
                        .ok()
                        .and_then(parse_durable_state),
                    ValueRef::Null
                    | ValueRef::Integer(_)
                    | ValueRef::Real(_)
                    | ValueRef::Blob(_) => None,
                })
            },
        )
        .optional()
        .map_err(query_error)
}

fn load_intent_fingerprint(
    connection: &Connection,
    session_id: &SessionId,
) -> Result<IntentFingerprint, DbError> {
    let stored: Option<StoredText> = connection
        .query_row(
            "SELECT fingerprint FROM session_completion_intents WHERE session_id = ?1",
            params![session_id.as_str()],
            |row| Ok(stored_text(row.get_ref(0)?)),
        )
        .optional()
        .map_err(query_error)?;
    let stored = match stored {
        None => return Ok(IntentFingerprint::Missing),
        Some(StoredText::Text(value)) => value,
        Some(StoredText::Null | StoredText::Invalid) => return Ok(IntentFingerprint::Corrupt),
    };
    match CompletionIntentFingerprint::try_from_hex(&stored) {
        Ok(fingerprint) if fingerprint.as_str() == stored => {
            Ok(IntentFingerprint::Found(fingerprint))
        }
        Ok(_) | Err(_) => Ok(IntentFingerprint::Corrupt),
    }
}

fn load_finalization_record(
    connection: &Connection,
    session_id: &SessionId,
) -> Result<StoredFinalization, DbError> {
    let row = connection
        .query_row(
            "SELECT session_id, fingerprint, state, claimed_at, committed_at, quarantine_reason
             FROM session_finalizations WHERE session_id = ?1",
            params![session_id.as_str()],
            |row| {
                Ok((
                    stored_text(row.get_ref(0)?),
                    stored_text(row.get_ref(1)?),
                    stored_text(row.get_ref(2)?),
                    stored_text(row.get_ref(3)?),
                    stored_text(row.get_ref(4)?),
                    stored_text(row.get_ref(5)?),
                ))
            },
        )
        .optional()
        .map_err(query_error)?;
    let Some((stored_session_id, fingerprint, state, claimed_at, committed_at, reason)) = row
    else {
        return Ok(StoredFinalization::Missing);
    };
    let StoredText::Text(stored_session_id) = stored_session_id else {
        return Ok(StoredFinalization::Corrupt);
    };
    let Ok(stored_session_id) = SessionId::parse(&stored_session_id) else {
        return Ok(StoredFinalization::Corrupt);
    };
    if &stored_session_id != session_id {
        return Ok(StoredFinalization::Corrupt);
    }
    let StoredText::Text(fingerprint) = fingerprint else {
        return Ok(StoredFinalization::Corrupt);
    };
    let fingerprint = match CompletionIntentFingerprint::try_from_hex(&fingerprint) {
        Ok(value) if value.as_str() == fingerprint => value,
        Ok(_) | Err(_) => return Ok(StoredFinalization::Corrupt),
    };
    let StoredText::Text(state) = state else {
        return Ok(StoredFinalization::Corrupt);
    };
    let Some(state) = FinalizationLedgerState::from_storage_name(&state) else {
        return Ok(StoredFinalization::Corrupt);
    };
    let StoredText::Text(claimed_at) = claimed_at else {
        return Ok(StoredFinalization::Corrupt);
    };
    let Some(claimed_at) = parse_utc(claimed_at) else {
        return Ok(StoredFinalization::Corrupt);
    };
    let committed_at = match committed_at {
        StoredText::Null => None,
        StoredText::Text(value) => match parse_utc(value) {
            Some(value) => Some(value),
            None => return Ok(StoredFinalization::Corrupt),
        },
        StoredText::Invalid => return Ok(StoredFinalization::Corrupt),
    };
    let reason = match reason {
        StoredText::Null => None,
        StoredText::Text(value) => match FinalizationQuarantineReason::from_storage_name(&value) {
            Some(value) => Some(value),
            None => return Ok(StoredFinalization::Corrupt),
        },
        StoredText::Invalid => return Ok(StoredFinalization::Corrupt),
    };
    match FinalizationRecord::new(
        stored_session_id,
        fingerprint,
        state,
        claimed_at,
        committed_at,
        reason,
    ) {
        Ok(record) => Ok(StoredFinalization::Found(record)),
        Err(_) => Ok(StoredFinalization::Corrupt),
    }
}

enum StoredText {
    Null,
    Text(String),
    Invalid,
}

fn stored_text(value: ValueRef<'_>) -> StoredText {
    match value {
        ValueRef::Null => StoredText::Null,
        ValueRef::Text(value) => std::str::from_utf8(value)
            .map(|value| StoredText::Text(value.to_owned()))
            .unwrap_or(StoredText::Invalid),
        ValueRef::Integer(_) | ValueRef::Real(_) | ValueRef::Blob(_) => StoredText::Invalid,
    }
}

fn parse_utc(value: String) -> Option<DateTime<Utc>> {
    value
        .ends_with('Z')
        .then(|| {
            DateTime::parse_from_rfc3339(&value)
                .ok()
                .map(|value| value.with_timezone(&Utc))
        })
        .flatten()
}

fn parse_durable_state(value: &str) -> Option<DurableSessionState> {
    serde_json::from_value(serde_json::Value::String(value.to_owned())).ok()
}

fn format_utc(value: DateTime<Utc>) -> String {
    value.to_rfc3339_opts(SecondsFormat::Micros, true)
}

fn conflict(
    session_id: &SessionId,
    expected: &CompletionIntentFingerprint,
    stored: &CompletionIntentFingerprint,
) -> FinalizationConflict {
    FinalizationConflict {
        session_id: session_id.clone(),
        expected_fingerprint: expected.clone(),
        stored_fingerprint: stored.clone(),
    }
}

fn quarantine_pending_or_report(
    connection: &Connection,
    record: &FinalizationRecord,
    reason: FinalizationQuarantineReason,
) -> Result<FinalizationQuarantineReason, DbError> {
    if record.state() == FinalizationLedgerState::Pending {
        let changed = connection
            .execute(
                "UPDATE session_finalizations
                 SET state = 'quarantined', quarantine_reason = ?2
                 WHERE session_id = ?1 AND state = 'pending'",
                params![record.session_id().as_str(), reason.storage_name()],
            )
            .map_err(write_error)?;
        if changed != 1 {
            return Err(DbError::Integrity(
                "finalization corruption quarantine lost its pending record".into(),
            ));
        }
    }
    Ok(reason)
}

/// Applies a quarantine reason to a pending row only. Terminal records retain
/// their durable metadata even when an adjacent V007 row is missing or corrupt.
fn quarantine_record_or_preserve_terminal(
    connection: &Connection,
    record: &FinalizationRecord,
    reason: FinalizationQuarantineReason,
) -> Result<FinalizationCommitOutcome, DbError> {
    match record.state() {
        FinalizationLedgerState::Pending => {
            quarantine_pending_or_report(connection, record, reason)
                .map(FinalizationCommitOutcome::Quarantined)
        }
        FinalizationLedgerState::Committed => Ok(FinalizationCommitOutcome::NotPending {
            state: FinalizationLedgerState::Committed,
        }),
        FinalizationLedgerState::Quarantined => Ok(FinalizationCommitOutcome::Quarantined(
            record.quarantine_reason().expect("validated record"),
        )),
    }
}

fn query_error(error: rusqlite::Error) -> DbError {
    DbError::from_sqlite("finalization ledger query", error)
}

fn write_error(error: rusqlite::Error) -> DbError {
    DbError::from_sqlite("finalization ledger write", error)
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
        | DbError::Transaction(_) => {
            RecoveryPortFailure::PermanentFailure(RecoveryPermanentFailure::IntegrityFailure)
        }
        DbError::LockPoisoned => RecoveryPortFailure::RetryableFailure,
    }
}
