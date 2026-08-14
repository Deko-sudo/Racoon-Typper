//! Atomic, restart-safe SQLite implementation of the application finalizer.
//!
//! It deliberately remains unwired from the normal Tauri completion path. The
//! one `IMMEDIATE` transaction owns immutable-intent validation, all existing
//! completion effects, V008 commit, and the V006 terminal transition.

use chrono::Utc;
use racoon_application::{
    CompletionIntent, CompletionIntentFingerprint, CompletionIntentLoadError, DailyGoalPolicy,
    DurableSessionState, FinalizationConflict, FinalizationLedgerState, FinalizationOutcome,
    FinalizationRecord, FinalizationRecordError, QuarantineReason, RecoveryPermanentFailure,
    RecoveryPortFailure, SessionFinalizer,
};
use racoon_core::StreakEngine;
use racoon_domain::{CharStatus, SessionId, TestRecord};
use rusqlite::{params, Connection, ErrorCode, OptionalExtension};
#[cfg(feature = "crash-test-support")]
use std::io::Write;
#[cfg(feature = "crash-test-support")]
use std::path::PathBuf;

use crate::repository::{
    DailyStatsRepository, ReplayFrame, ReplayRepository, SqliteDailyStatsRepository,
    SqliteLessonRepository, SqlitePersonalBestsRepository, SqliteReplayRepository,
    SqliteStreakRepository, SqliteTestRepository, StreakRecord, StreakRepository, TestRepository,
};
use crate::{Database, DbError};

/// Infrastructure-side implementation of the application-owned finalizer.
pub struct SqliteSessionFinalizer<'a> {
    database: &'a Database,
    failure_point: Option<FailurePoint>,
    #[cfg(feature = "crash-test-support")]
    crash_checkpoint: Option<CrashCheckpointControl>,
}

/// Deterministic data-layer failure points used only by integration tests to
/// prove that the finalizer's single transaction rolls back every effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FailurePoint {
    BeforeTestInsertion,
    AfterTestInsertion,
    AfterReplayInsertion,
    AfterPersonalBestUpdate,
    AfterDailyStatisticsUpdate,
    BeforeFinalizationCommit,
    AfterFinalizationCommit,
    AfterSessionFinalized,
}

/// Deterministic test-only failure points for transaction rollback coverage.
///
/// This type is absent from the default production build. Integration tests
/// opt in through racoon-data's non-default `test-support` feature.
#[cfg(feature = "test-support")]
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinalizerFailurePoint {
    BeforeTestInsertion,
    AfterTestInsertion,
    AfterReplayInsertion,
    AfterPersonalBestUpdate,
    AfterDailyStatisticsUpdate,
    BeforeFinalizationCommit,
    AfterFinalizationCommit,
    AfterSessionFinalized,
}

#[cfg(feature = "test-support")]
impl From<FinalizerFailurePoint> for FailurePoint {
    fn from(value: FinalizerFailurePoint) -> Self {
        match value {
            FinalizerFailurePoint::BeforeTestInsertion => Self::BeforeTestInsertion,
            FinalizerFailurePoint::AfterTestInsertion => Self::AfterTestInsertion,
            FinalizerFailurePoint::AfterReplayInsertion => Self::AfterReplayInsertion,
            FinalizerFailurePoint::AfterPersonalBestUpdate => Self::AfterPersonalBestUpdate,
            FinalizerFailurePoint::AfterDailyStatisticsUpdate => Self::AfterDailyStatisticsUpdate,
            FinalizerFailurePoint::BeforeFinalizationCommit => Self::BeforeFinalizationCommit,
            FinalizerFailurePoint::AfterFinalizationCommit => Self::AfterFinalizationCommit,
            FinalizerFailurePoint::AfterSessionFinalized => Self::AfterSessionFinalized,
        }
    }
}

/// Bounded abrupt-termination checkpoints used only by the file-backed
/// process-crash recovery campaign. The enum and its constructor are absent
/// from normal production builds.
#[cfg(feature = "crash-test-support")]
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinalizerCrashCheckpoint {
    BeforeTestInsertion,
    AfterTestInsertion,
    AfterReplayInsertion,
    AfterPersonalBestUpdate,
    AfterDailyStatisticsUpdate,
    AfterStreakUpdate,
    AfterDailyGoalUpdate,
    AfterLessonUpdate,
    AfterV008CommittedUpdate,
    AfterV006FinalizedUpdate,
    AfterFinalizerTransactionCommit,
}

#[cfg(feature = "crash-test-support")]
impl FinalizerCrashCheckpoint {
    /// Stable, bounded name used by the test child protocol and marker file.
    pub const fn storage_name(self) -> &'static str {
        match self {
            Self::BeforeTestInsertion => "before_test_insertion",
            Self::AfterTestInsertion => "after_test_insertion",
            Self::AfterReplayInsertion => "after_replay_insertion",
            Self::AfterPersonalBestUpdate => "after_personal_best_update",
            Self::AfterDailyStatisticsUpdate => "after_daily_statistics_update",
            Self::AfterStreakUpdate => "after_streak_update",
            Self::AfterDailyGoalUpdate => "after_daily_goal_update",
            Self::AfterLessonUpdate => "after_lesson_update",
            Self::AfterV008CommittedUpdate => "after_v008_committed_update",
            Self::AfterV006FinalizedUpdate => "after_v006_finalized_update",
            Self::AfterFinalizerTransactionCommit => "after_finalizer_transaction_commit",
        }
    }

    /// Parses only the bounded names emitted by this test-only protocol.
    pub fn from_storage_name(value: &str) -> Option<Self> {
        match value {
            "before_test_insertion" => Some(Self::BeforeTestInsertion),
            "after_test_insertion" => Some(Self::AfterTestInsertion),
            "after_replay_insertion" => Some(Self::AfterReplayInsertion),
            "after_personal_best_update" => Some(Self::AfterPersonalBestUpdate),
            "after_daily_statistics_update" => Some(Self::AfterDailyStatisticsUpdate),
            "after_streak_update" => Some(Self::AfterStreakUpdate),
            "after_daily_goal_update" => Some(Self::AfterDailyGoalUpdate),
            "after_lesson_update" => Some(Self::AfterLessonUpdate),
            "after_v008_committed_update" => Some(Self::AfterV008CommittedUpdate),
            "after_v006_finalized_update" => Some(Self::AfterV006FinalizedUpdate),
            "after_finalizer_transaction_commit" => Some(Self::AfterFinalizerTransactionCommit),
            _ => None,
        }
    }
}

#[cfg(feature = "crash-test-support")]
struct CrashCheckpointControl {
    checkpoint: FinalizerCrashCheckpoint,
    marker_path: PathBuf,
}

impl<'a> SqliteSessionFinalizer<'a> {
    pub fn new(database: &'a Database) -> Self {
        Self {
            database,
            failure_point: None,
            #[cfg(feature = "crash-test-support")]
            crash_checkpoint: None,
        }
    }

    /// Test-only constructor for transaction rollback verification.
    #[cfg(feature = "test-support")]
    #[doc(hidden)]
    pub fn with_failure_injection(
        database: &'a Database,
        failure_point: FinalizerFailurePoint,
    ) -> Self {
        Self {
            database,
            failure_point: Some(failure_point.into()),
            #[cfg(feature = "crash-test-support")]
            crash_checkpoint: None,
        }
    }

    /// Test-only constructor for the real child-process crash campaign.
    ///
    /// When the selected checkpoint is reached, the child writes only the
    /// checkpoint's bounded name to `marker_path`, synchronizes that marker,
    /// and calls `std::process::abort()` without Rust stack unwinding.
    #[cfg(feature = "crash-test-support")]
    #[doc(hidden)]
    pub fn with_process_crash_checkpoint(
        database: &'a Database,
        checkpoint: FinalizerCrashCheckpoint,
        marker_path: impl Into<PathBuf>,
    ) -> Self {
        Self {
            database,
            failure_point: None,
            crash_checkpoint: Some(CrashCheckpointControl {
                checkpoint,
                marker_path: marker_path.into(),
            }),
        }
    }
}

impl SessionFinalizer for SqliteSessionFinalizer<'_> {
    fn finalize_completion(
        &self,
        session_id: &SessionId,
        expected_fingerprint: &CompletionIntentFingerprint,
    ) -> Result<FinalizationOutcome, RecoveryPortFailure> {
        SessionId::parse(session_id.as_str()).map_err(|_| invalid_contract())?;
        let outcome = self
            .database
            .with_transaction(|connection| {
                finalize_in_transaction(
                    connection,
                    session_id,
                    expected_fingerprint,
                    self.failure_point,
                    #[cfg(feature = "crash-test-support")]
                    self.crash_checkpoint.as_ref(),
                )
            })
            .map_err(port_failure)?;
        #[cfg(feature = "crash-test-support")]
        if matches!(&outcome, FinalizationOutcome::NewlyFinalized) {
            // The process-crash campaign defines this checkpoint precisely as
            // a newly applied finalization whose SQLite COMMIT has returned,
            // before the caller receives its successful business outcome.
            crash_at(
                self.crash_checkpoint.as_ref(),
                FinalizerCrashCheckpoint::AfterFinalizerTransactionCommit,
            );
        }
        Ok(outcome)
    }
}

fn finalize_in_transaction(
    connection: &Connection,
    session_id: &SessionId,
    expected_fingerprint: &CompletionIntentFingerprint,
    failure_point: Option<FailurePoint>,
    #[cfg(feature = "crash-test-support")] crash_checkpoint: Option<&CrashCheckpointControl>,
) -> Result<FinalizationOutcome, DbError> {
    let Some(session_state) = load_session_state(connection, session_id)? else {
        return Ok(FinalizationOutcome::NotFound);
    };
    let Some(intent) = load_intent(connection, session_id)? else {
        return Ok(FinalizationOutcome::Quarantined(
            QuarantineReason::MissingCompletionIntent,
        ));
    };
    let intent = match intent {
        StoredIntent::Found(intent) => *intent,
        StoredIntent::UnsupportedCanonicalization => {
            return Ok(FinalizationOutcome::Quarantined(
                QuarantineReason::UnsupportedCanonicalizationVersion,
            ))
        }
        StoredIntent::UnsupportedVersion => {
            return Ok(FinalizationOutcome::Quarantined(
                QuarantineReason::UnsupportedIntentVersion,
            ))
        }
        StoredIntent::Corrupt => {
            return Ok(FinalizationOutcome::Quarantined(
                QuarantineReason::CorruptCompletionPayload,
            ))
        }
    };
    if intent.fingerprint() != expected_fingerprint {
        return Ok(FinalizationOutcome::Conflict(conflict(
            session_id,
            expected_fingerprint,
            intent.fingerprint(),
        )));
    }

    let finalization = match load_finalization(connection, session_id)? {
        StoredFinalization::Missing | StoredFinalization::Corrupt => {
            return Ok(FinalizationOutcome::Quarantined(
                QuarantineReason::InconsistentDurableMetadata,
            ))
        }
        StoredFinalization::Found(record) => record,
    };
    if finalization.fingerprint() != expected_fingerprint {
        return Ok(FinalizationOutcome::Conflict(conflict(
            session_id,
            expected_fingerprint,
            finalization.fingerprint(),
        )));
    }

    match (session_state, finalization.state()) {
        (DurableSessionState::Finalized, FinalizationLedgerState::Committed) => {
            return if test_matches_intent(connection, &intent)? == Some(true) {
                Ok(FinalizationOutcome::AlreadyFinalized)
            } else {
                Ok(FinalizationOutcome::Quarantined(
                    QuarantineReason::InconsistentDurableMetadata,
                ))
            };
        }
        (DurableSessionState::FinalizationPending, FinalizationLedgerState::Pending) => {}
        (_, FinalizationLedgerState::Quarantined) => {
            return Ok(FinalizationOutcome::Quarantined(
                QuarantineReason::InconsistentDurableMetadata,
            ))
        }
        _ => {
            return Ok(FinalizationOutcome::Quarantined(
                QuarantineReason::InconsistentDurableMetadata,
            ))
        }
    }

    if test_id_for_session(connection, session_id)?.is_some() {
        return Ok(FinalizationOutcome::Quarantined(
            QuarantineReason::InconsistentDurableMetadata,
        ));
    }

    #[cfg(feature = "crash-test-support")]
    crash_at(
        crash_checkpoint,
        FinalizerCrashCheckpoint::BeforeTestInsertion,
    );
    inject(failure_point, FailurePoint::BeforeTestInsertion)?;
    apply_completion_effects(
        connection,
        &intent,
        failure_point,
        #[cfg(feature = "crash-test-support")]
        crash_checkpoint,
    )?;
    inject(failure_point, FailurePoint::BeforeFinalizationCommit)?;

    let committed = connection
        .execute(
            "UPDATE session_finalizations
             SET state = 'committed', committed_at = ?2
             WHERE session_id = ?1 AND state = 'pending'",
            params![
                session_id.as_str(),
                format_utc(*intent.payload().completed_at())
            ],
        )
        .map_err(write_error)?;
    if committed != 1 {
        return Err(DbError::Integrity(
            "finalization commit lost its pending row".into(),
        ));
    }
    #[cfg(feature = "crash-test-support")]
    crash_at(
        crash_checkpoint,
        FinalizerCrashCheckpoint::AfterV008CommittedUpdate,
    );
    inject(failure_point, FailurePoint::AfterFinalizationCommit)?;
    let finalized = connection
        .execute(
            "UPDATE session_ledger
             SET state = 'finalized', updated_at = ?2
             WHERE session_id = ?1 AND state = 'finalization_pending'",
            params![
                session_id.as_str(),
                format_utc(*intent.payload().completed_at())
            ],
        )
        .map_err(write_error)?;
    if finalized != 1 {
        return Err(DbError::Integrity(
            "session finalization lost its pending state".into(),
        ));
    }
    #[cfg(feature = "crash-test-support")]
    crash_at(
        crash_checkpoint,
        FinalizerCrashCheckpoint::AfterV006FinalizedUpdate,
    );
    inject(failure_point, FailurePoint::AfterSessionFinalized)?;
    Ok(FinalizationOutcome::NewlyFinalized)
}

fn apply_completion_effects(
    connection: &Connection,
    intent: &CompletionIntent,
    failure_point: Option<FailurePoint>,
    #[cfg(feature = "crash-test-support")] crash_checkpoint: Option<&CrashCheckpointControl>,
) -> Result<(), DbError> {
    let payload = intent.payload();
    let stats = payload.final_stats();
    let test_id = SqliteTestRepository::new(connection).save_test(TestRecord {
        session_id: payload.session_id().clone(),
        created_at: format_utc(*payload.completed_at()),
        mode_type: payload.mode_type().to_owned(),
        mode_config: payload.mode_config().clone(),
        language: payload.language().to_owned(),
        text_length: payload.text_length(),
        duration_ms: stats.duration_ms,
        wpm: stats.wpm,
        raw_wpm: stats.raw_wpm,
        accuracy: stats.accuracy,
        raw_accuracy: stats.raw_accuracy,
        consistency: stats.consistency,
        correct_chars: stats.correct_chars,
        incorrect_chars: stats.incorrect_chars,
        backspaces: stats.backspaces,
        char_stats: stats.char_stats.clone(),
        heatmap_data: stats.heatmap.clone(),
        graph_data: stats.graph_data.clone(),
        is_pb: false,
        tags: String::new(),
    })?;
    #[cfg(feature = "crash-test-support")]
    crash_at(
        crash_checkpoint,
        FinalizerCrashCheckpoint::AfterTestInsertion,
    );
    inject(failure_point, FailurePoint::AfterTestInsertion)?;

    let replay = replay_projection(intent, test_id)?;
    SqliteReplayRepository::new(connection).save_replay(test_id, &replay)?;
    #[cfg(feature = "crash-test-support")]
    crash_at(
        crash_checkpoint,
        FinalizerCrashCheckpoint::AfterReplayInsertion,
    );
    inject(failure_point, FailurePoint::AfterReplayInsertion)?;

    let mode_config = serde_json::to_string(payload.mode_config())
        .map_err(|_| DbError::Integrity("intent mode configuration is not serializable".into()))?;
    let completion_timestamp = format_utc(*payload.completed_at());
    let updates = SqlitePersonalBestsRepository::new(connection).check_and_update_at(
        payload.mode_type(),
        &mode_config,
        stats.wpm,
        stats.accuracy,
        test_id,
        &completion_timestamp,
    )?;
    if !updates.is_empty() {
        SqliteTestRepository::new(connection).mark_as_pb(test_id)?;
    }
    #[cfg(feature = "crash-test-support")]
    crash_at(
        crash_checkpoint,
        FinalizerCrashCheckpoint::AfterPersonalBestUpdate,
    );
    inject(failure_point, FailurePoint::AfterPersonalBestUpdate)?;

    let date = payload
        .completed_at()
        .with_timezone(&chrono::Local)
        .format("%Y-%m-%d")
        .to_string();
    let duration = i64::try_from(stats.duration_ms)
        .map_err(|_| DbError::Integrity("duration exceeds i64".into()))?;
    let characters = stats
        .correct_chars
        .checked_add(stats.incorrect_chars)
        .and_then(|value| i64::try_from(value).ok())
        .ok_or_else(|| DbError::Integrity("character count exceeds i64".into()))?;
    let daily = SqliteDailyStatsRepository::new(connection);
    daily.update_after_test(&date, duration, characters, stats.wpm, stats.accuracy)?;
    #[cfg(feature = "crash-test-support")]
    crash_at(
        crash_checkpoint,
        FinalizerCrashCheckpoint::AfterDailyStatisticsUpdate,
    );
    update_daily_streak(connection, &date)?;
    #[cfg(feature = "crash-test-support")]
    crash_at(
        crash_checkpoint,
        FinalizerCrashCheckpoint::AfterStreakUpdate,
    );

    if let Some(lesson_id) = payload.lesson_id() {
        let passed = SqliteLessonRepository::new(connection).complete_lesson_at(
            lesson_id,
            stats.wpm,
            stats.accuracy,
            &completion_timestamp,
        )?;
        // Счётчик пройденных уроков растёт только когда попытка прошла гейт
        // (accuracy ≥ 90% first-attempt И WPM ≥ 20), синхронно со статусом.
        if passed {
            daily.increment_lessons_completed(&date)?;
        }
        #[cfg(feature = "crash-test-support")]
        crash_at(
            crash_checkpoint,
            FinalizerCrashCheckpoint::AfterLessonUpdate,
        );
    }
    let day = daily
        .get_day(&date)?
        .ok_or_else(|| DbError::Integrity("daily statistics missing after update".into()))?;
    let goal_met = match payload.completion_policy().daily_goal() {
        DailyGoalPolicy::Time { target_minutes } => {
            time_goal_is_met(*target_minutes, day.total_time_ms)
        }
        DailyGoalPolicy::Wpm { target_wpm } => *target_wpm > 0.0 && day.best_wpm >= *target_wpm,
        DailyGoalPolicy::Accuracy { target_accuracy } => {
            *target_accuracy > 0.0 && day.avg_accuracy >= *target_accuracy
        }
    };
    if goal_met {
        daily.set_daily_goal_met(&date, true)?;
    }
    #[cfg(feature = "crash-test-support")]
    crash_at(
        crash_checkpoint,
        FinalizerCrashCheckpoint::AfterDailyGoalUpdate,
    );
    inject(failure_point, FailurePoint::AfterDailyStatisticsUpdate)?;
    Ok(())
}

#[cfg(feature = "crash-test-support")]
fn crash_at(configured: Option<&CrashCheckpointControl>, current: FinalizerCrashCheckpoint) {
    let Some(configured) = configured else {
        return;
    };
    if configured.checkpoint != current {
        return;
    }

    // The marker is a bounded test synchronization signal, never a durable
    // application record. A failed marker write still aborts; the parent then
    // fails the campaign because it cannot prove the selected boundary ran.
    if let Ok(mut marker) = std::fs::File::create(&configured.marker_path) {
        let _ = marker.write_all(current.storage_name().as_bytes());
        let _ = marker.sync_all();
    }
    std::process::abort();
}

fn inject(configured: Option<FailurePoint>, current: FailurePoint) -> Result<(), DbError> {
    if configured == Some(current) {
        return Err(DbError::Integrity("injected finalization failure".into()));
    }
    Ok(())
}

/// Uses elapsed minutes rather than truncating the configured minute target.
/// The comparison preserves the established zero-target behavior (`0 >= 0`),
/// retains negative finite targets if one is present in legacy data, and keeps
/// fractional targets precise without overflowing a millisecond conversion.
fn time_goal_is_met(target_minutes: f64, total_time_ms: i64) -> bool {
    (total_time_ms as f64 / 60_000.0) >= target_minutes
}

fn update_daily_streak(connection: &Connection, today: &str) -> Result<(), DbError> {
    let repository = SqliteStreakRepository::new(connection);
    let existing = repository.get("daily_test")?;
    let (current, longest, last_date, started_date) = existing
        .map(|record| {
            (
                record.current_streak,
                record.longest_streak,
                record.last_date,
                record.started_date,
            )
        })
        .unwrap_or((0, 0, None, None));
    let starts_new = last_date
        .as_deref()
        .is_none_or(|last| StreakEngine::days_between(last, today) > 1);
    let (current, longest, _) =
        StreakEngine::compute_streak(current, longest, last_date.as_deref(), today);
    repository.upsert(&StreakRecord {
        streak_type: "daily_test".to_owned(),
        current_streak: current,
        longest_streak: longest,
        last_date: Some(today.to_owned()),
        started_date: Some(if starts_new {
            today.to_owned()
        } else {
            started_date.unwrap_or_else(|| today.to_owned())
        }),
    })
}

enum StoredIntent {
    Found(Box<CompletionIntent>),
    UnsupportedCanonicalization,
    UnsupportedVersion,
    Corrupt,
}

fn load_intent(
    connection: &Connection,
    session_id: &SessionId,
) -> Result<Option<StoredIntent>, DbError> {
    let row = connection
        .query_row(
            "SELECT canonicalization_version, payload_version, fingerprint, canonical_payload, payload_byte_length
             FROM session_completion_intents WHERE session_id = ?1",
            params![session_id.as_str()],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, String>(2)?, row.get::<_, Vec<u8>>(3)?, row.get::<_, i64>(4)?)),
        )
        .optional()
        .map_err(query_error)?;
    let Some((canonicalization_version, payload_version, fingerprint, payload, recorded_length)) =
        row
    else {
        return Ok(None);
    };
    if canonicalization_version < 0
        || payload_version < 0
        || recorded_length < 0
        || usize::try_from(recorded_length).ok() != Some(payload.len())
    {
        return Ok(Some(StoredIntent::Corrupt));
    }
    let fingerprint = match CompletionIntentFingerprint::try_from_hex(&fingerprint) {
        Ok(value) if value.as_str() == fingerprint => value,
        _ => return Ok(Some(StoredIntent::Corrupt)),
    };
    match CompletionIntent::from_stored_payload(&payload, &fingerprint) {
        Ok(intent)
            if intent.payload().session_id() == session_id
                && u64::from(intent.canonicalization_version().as_u16())
                    == u64::try_from(canonicalization_version).unwrap_or(u64::MAX)
                && u64::from(intent.payload_version().as_u16())
                    == u64::try_from(payload_version).unwrap_or(u64::MAX) =>
        {
            Ok(Some(StoredIntent::Found(Box::new(intent))))
        }
        Ok(_)
        | Err(CompletionIntentLoadError::CorruptCanonicalPayload)
        | Err(CompletionIntentLoadError::PayloadTooLarge)
        | Err(CompletionIntentLoadError::FingerprintMismatch) => Ok(Some(StoredIntent::Corrupt)),
        Err(CompletionIntentLoadError::UnsupportedCanonicalizationVersion(_)) => {
            Ok(Some(StoredIntent::UnsupportedCanonicalization))
        }
        Err(CompletionIntentLoadError::UnsupportedVersion(_)) => {
            Ok(Some(StoredIntent::UnsupportedVersion))
        }
    }
}

enum StoredFinalization {
    Missing,
    Found(FinalizationRecord),
    Corrupt,
}

fn load_finalization(
    connection: &Connection,
    session_id: &SessionId,
) -> Result<StoredFinalization, DbError> {
    let row = connection
        .query_row(
            "SELECT fingerprint, state, claimed_at, committed_at, quarantine_reason
             FROM session_finalizations WHERE session_id = ?1",
            params![session_id.as_str()],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                ))
            },
        )
        .optional()
        .map_err(query_error)?;
    let Some((fingerprint, state, claimed_at, committed_at, reason)) = row else {
        return Ok(StoredFinalization::Missing);
    };
    let fingerprint = match CompletionIntentFingerprint::try_from_hex(&fingerprint) {
        Ok(value) if value.as_str() == fingerprint => value,
        _ => return Ok(StoredFinalization::Corrupt),
    };
    let Some(state) = FinalizationLedgerState::from_storage_name(&state) else {
        return Ok(StoredFinalization::Corrupt);
    };
    let claimed_at = match chrono::DateTime::parse_from_rfc3339(&claimed_at) {
        Ok(value) if claimed_at.ends_with('Z') => value.with_timezone(&Utc),
        _ => return Ok(StoredFinalization::Corrupt),
    };
    let committed_at = match committed_at {
        None => None,
        Some(value) => match chrono::DateTime::parse_from_rfc3339(&value) {
            Ok(parsed) if value.ends_with('Z') => Some(parsed.with_timezone(&Utc)),
            _ => return Ok(StoredFinalization::Corrupt),
        },
    };
    let reason = match reason {
        None => None,
        Some(value) => {
            match racoon_application::FinalizationQuarantineReason::from_storage_name(&value) {
                Some(value) => Some(value),
                None => return Ok(StoredFinalization::Corrupt),
            }
        }
    };
    match FinalizationRecord::new(
        session_id.clone(),
        fingerprint,
        state,
        claimed_at,
        committed_at,
        reason,
    ) {
        Ok(record) => Ok(StoredFinalization::Found(record)),
        Err(FinalizationRecordError::InconsistentStateMetadata) => Ok(StoredFinalization::Corrupt),
    }
}

fn load_session_state(
    connection: &Connection,
    session_id: &SessionId,
) -> Result<Option<DurableSessionState>, DbError> {
    let stored: Option<String> = connection
        .query_row(
            "SELECT state FROM session_ledger WHERE session_id = ?1",
            params![session_id.as_str()],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(query_error)?;
    stored
        .map(|value| serde_json::from_value(serde_json::Value::String(value)))
        .transpose()
        .map_err(|_| DbError::Integrity("invalid session state".into()))
}

fn test_id_for_session(
    connection: &Connection,
    session_id: &SessionId,
) -> Result<Option<i64>, DbError> {
    connection
        .query_row(
            "SELECT id FROM tests WHERE session_id = ?1",
            params![session_id.as_str()],
            |row| row.get(0),
        )
        .optional()
        .map_err(query_error)
}

type StoredTestComparison = (
    i64,
    String,
    String,
    String,
    String,
    i64,
    i64,
    f64,
    f64,
    f64,
    f64,
    Option<f64>,
    i64,
    i64,
    i64,
    String,
    String,
    Option<String>,
    i64,
    String,
);

/// The terminal V006/V008 markers are necessary but not sufficient evidence
/// for an idempotent retry. This checks every immutable field written to the
/// primary result and every projected replay frame before reporting success.
/// `is_pb` is intentionally not compared to V007: it is a historical outcome
/// of the personal-best comparison, which depends on other sessions and is not
/// an intent input. Its SQLite boolean shape is still validated. `tags` is
/// deterministic in both completion paths and therefore must remain empty.
fn test_matches_intent(
    connection: &Connection,
    intent: &CompletionIntent,
) -> Result<Option<bool>, DbError> {
    let row: Option<StoredTestComparison> = connection
        .query_row(
            "SELECT id, created_at, mode_type, mode_config, language, text_length, duration_ms,
                    wpm, raw_wpm, accuracy, raw_accuracy, consistency,
                    correct_chars, incorrect_chars, backspaces, char_stats, heatmap_data,
                    graph_data, is_pb, tags
             FROM tests WHERE session_id = ?1",
            params![intent.payload().session_id().as_str()],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get(8)?,
                    row.get(9)?,
                    row.get(10)?,
                    row.get(11)?,
                    row.get(12)?,
                    row.get(13)?,
                    row.get(14)?,
                    row.get(15)?,
                    row.get(16)?,
                    row.get(17)?,
                    row.get(18)?,
                    row.get(19)?,
                ))
            },
        )
        .optional()
        .map_err(query_error)?;
    let Some((
        test_id,
        created_at,
        mode_type,
        mode_config,
        language,
        text_length,
        duration_ms,
        wpm,
        raw_wpm,
        accuracy,
        raw_accuracy,
        consistency,
        correct_chars,
        incorrect_chars,
        backspaces,
        char_stats,
        heatmap_data,
        graph_data,
        is_pb,
        tags,
    )) = row
    else {
        return Ok(None);
    };
    let payload = intent.payload();
    let stats = payload.final_stats();
    let primary_matches = created_at == format_utc(*payload.completed_at())
        && mode_type == payload.mode_type()
        && json_text_matches(&mode_config, payload.mode_config())
        && language == payload.language()
        && text_length == i64::try_from(payload.text_length()).unwrap_or(i64::MIN)
        && duration_ms == i64::try_from(stats.duration_ms).unwrap_or(i64::MIN)
        && wpm.to_bits() == stats.wpm.to_bits()
        && raw_wpm.to_bits() == stats.raw_wpm.to_bits()
        && accuracy.to_bits() == stats.accuracy.to_bits()
        && raw_accuracy.to_bits() == stats.raw_accuracy.to_bits()
        && same_optional_f64(consistency, stats.consistency)
        && correct_chars == i64::try_from(stats.correct_chars).unwrap_or(i64::MIN)
        && incorrect_chars == i64::try_from(stats.incorrect_chars).unwrap_or(i64::MIN)
        && backspaces == i64::try_from(stats.backspaces).unwrap_or(i64::MIN)
        && json_text_matches(&char_stats, &stats.char_stats)
        && json_text_matches(&heatmap_data, &stats.heatmap)
        && optional_json_text_matches(graph_data.as_deref(), stats.graph_data.as_ref())
        && matches!(is_pb, 0 | 1)
        && tags.is_empty();
    Ok(Some(
        primary_matches && replay_matches_intent(connection, intent, test_id)?,
    ))
}

fn json_text_matches(stored: &str, expected: &serde_json::Value) -> bool {
    serde_json::from_str::<serde_json::Value>(stored).is_ok_and(|value| value == *expected)
}

fn optional_json_text_matches(stored: Option<&str>, expected: Option<&serde_json::Value>) -> bool {
    match (stored, expected) {
        (None, None) => true,
        (Some(stored), Some(expected)) => json_text_matches(stored, expected),
        _ => false,
    }
}

fn replay_projection(intent: &CompletionIntent, test_id: i64) -> Result<Vec<ReplayFrame>, DbError> {
    intent
        .payload()
        .replay_frames()
        .iter()
        .enumerate()
        .map(|(index, frame)| {
            Ok(ReplayFrame {
                id: 0,
                test_id,
                frame_index: i64::try_from(index)
                    .map_err(|_| DbError::Integrity("replay index exceeds i64".into()))?,
                timestamp_ms: i64::try_from(frame.timestamp_ms)
                    .map_err(|_| DbError::Integrity("replay timestamp exceeds i64".into()))?,
                position: i64::try_from(frame.caret_pos)
                    .map_err(|_| DbError::Integrity("replay position exceeds i64".into()))?,
                expected_char: frame.expected_char.to_string(),
                typed_char: Some(
                    frame
                        .typed_char
                        .map_or_else(|| frame.key.clone(), |character| character.to_string()),
                ),
                correct: frame.char_status == CharStatus::Correct,
            })
        })
        .collect()
}

fn replay_matches_intent(
    connection: &Connection,
    intent: &CompletionIntent,
    test_id: i64,
) -> Result<bool, DbError> {
    let expected = replay_projection(intent, test_id)?;
    let actual = SqliteReplayRepository::new(connection).load_replay(test_id)?;
    Ok(actual.len() == expected.len()
        && actual
            .iter()
            .zip(expected.iter())
            .all(|(actual, expected)| {
                actual.test_id == test_id
                    && actual.frame_index == expected.frame_index
                    && actual.timestamp_ms == expected.timestamp_ms
                    && actual.position == expected.position
                    && actual.expected_char == expected.expected_char
                    && actual.typed_char == expected.typed_char
                    && actual.correct == expected.correct
            }))
}

fn same_optional_f64(left: Option<f64>, right: Option<f64>) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(left), Some(right)) => left.to_bits() == right.to_bits(),
        _ => false,
    }
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

fn format_utc(value: chrono::DateTime<Utc>) -> String {
    value.to_rfc3339_opts(chrono::SecondsFormat::Micros, true)
}
fn invalid_contract() -> RecoveryPortFailure {
    RecoveryPortFailure::PermanentFailure(RecoveryPermanentFailure::InvalidContract)
}
fn query_error(error: rusqlite::Error) -> DbError {
    DbError::from_sqlite("session finalizer query", error)
}
fn write_error(error: rusqlite::Error) -> DbError {
    DbError::from_sqlite("session finalizer write", error)
}
fn port_failure(error: DbError) -> RecoveryPortFailure {
    match error {
        DbError::Sqlite {
            code: ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked,
            ..
        }
        | DbError::LockPoisoned => RecoveryPortFailure::RetryableFailure,
        DbError::Migration(_) => {
            RecoveryPortFailure::PermanentFailure(RecoveryPermanentFailure::UnsupportedSchema)
        }
        _ => RecoveryPortFailure::PermanentFailure(RecoveryPermanentFailure::IntegrityFailure),
    }
}
