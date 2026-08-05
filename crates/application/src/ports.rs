//! Business-oriented ports for the session vertical slice.
//!
//! These traits deliberately describe application capabilities rather than
//! SQLite queries or Tauri state. Implementations belong to adapters and may
//! be introduced incrementally as individual use cases move out of `racoon-app`.

use chrono::{DateTime, Utc};
use racoon_core::TestMode;
use racoon_domain::SessionId;

use crate::recovery::{
    CompletionIntent, CompletionIntentFingerprint, CompletionIntentLoadOutcome,
    FinalizationClaimOutcome, FinalizationCommitOutcome, FinalizationLedgerClaimOutcome,
    FinalizationLoadOutcome, FinalizationOutcome, FinalizationQuarantineReason,
    LedgerMutationOutcome, QuarantineReason, RecoveryCandidate, RecoveryPortFailure,
    StartedSession,
};
use crate::reporting::{
    AchievementInputQuery, AchievementInputs, ExportDatasetSource, ExportQuery, HistoryPageSource,
    HistoryQuery, InclusiveDateRange, InsightInputQuery, InsightInputs, PersonalBestEntry,
    ReplayPageSource, ReplayQuery, ReportingDay, ReportingError, ReportingModeFilter, TestDetails,
};
use crate::session::{SessionCompletion, SessionStartRequest};

/// Supplies a backend-owned identity for an accepted session.
///
/// The kernel provides no implementation. Production and deterministic test
/// sources can be supplied without changing the use-case contract.
pub trait SessionIdSource {
    fn next_session_id(&mut self) -> SessionId;
}

/// Supplies elapsed timestamps used by session input processing.
///
/// The production monotonic clock remains in an adapter; tests can provide a
/// fixed or scripted value source.
pub trait SessionClock {
    fn monotonic_timestamp_ms(&self) -> u64;
}

/// Supplies the wall-clock value captured when a session completes.
///
/// The kernel only requests a value at the completion boundary. Production
/// adapters may use the system UTC clock, while tests can provide a fixed
/// timestamp without changing the completion contract.
pub trait SessionWallClock {
    fn utc_now(&self) -> DateTime<Utc>;
}

/// Supplies runtime-generated values used by resource selection.
///
/// The port deliberately exposes a value source rather than a resource or
/// selection API. Resource adapters retain the selection policy and tests can
/// control the sequence without relying on system time.
pub trait SessionRandomSource {
    fn next_u64(&mut self) -> u64;
}

/// Selects and constructs a validated typing mode for a session request.
///
/// Resource loading and validation are adapter concerns. Returning a core
/// `TestMode` keeps this port focused on the business capability rather than
/// exposing a resource catalog or persistence API.
pub trait SessionModeFactory {
    type Error;

    fn build_mode(
        &self,
        request: &SessionStartRequest,
        language: &str,
        random_source: &mut dyn SessionRandomSource,
    ) -> Result<Box<dyn TestMode>, Self::Error>;
}

/// Persists one immutable completion snapshot as one business operation.
///
/// The port intentionally does not expose a connection, repository, or SQL
/// transaction. The adapter owns transaction boundaries and returns the
/// resulting local test identity only after the commit succeeds.
pub trait SessionCompletionStore {
    type Error;

    fn persist_completion(
        &self,
        completion: &SessionCompletion,
    ) -> Result<crate::session::SessionPersistenceReceipt, Self::Error>;
}

/// Application-facing capabilities for durable session lifecycle records.
///
/// Implementations own storage details and transaction boundaries. The
/// application receives business outcomes only; it never receives SQL,
/// connections, transactions, row identifiers, or table names.
pub trait SessionRecoveryLedger {
    fn record_started(
        &self,
        session: &StartedSession,
    ) -> Result<LedgerMutationOutcome, RecoveryPortFailure>;

    fn record_completion_intent(
        &self,
        intent: &CompletionIntent,
    ) -> Result<LedgerMutationOutcome, RecoveryPortFailure>;

    /// Claims a valid completion intent for future finalization.
    ///
    /// The business transition is exactly
    /// `AwaitingPersistence → FinalizationPending`. `Claimed` is the first
    /// successful transition; `AlreadyPending` is the identical retry. A
    /// differing fingerprint is a `Conflict`, a missing session is
    /// `NotFound`, and invalid/unsupported intent data is `Quarantined`.
    /// `Finalized` is an idempotent `AlreadyFinalized` no-op; other terminal
    /// states are returned as `RejectedTerminal` and must not be reopened.
    /// Implementations report infrastructure-independent retryable or
    /// permanent failures through `RecoveryPortFailure`.
    fn claim_completion_for_finalization(
        &self,
        session_id: &SessionId,
        expected_fingerprint: &CompletionIntentFingerprint,
    ) -> Result<FinalizationClaimOutcome, RecoveryPortFailure>;

    fn list_recovery_candidates(&self) -> Result<Vec<RecoveryCandidate>, RecoveryPortFailure>;

    fn load_completion_intent(
        &self,
        session_id: &SessionId,
    ) -> Result<CompletionIntentLoadOutcome, RecoveryPortFailure>;

    fn mark_interrupted(
        &self,
        session_id: &SessionId,
        reason: crate::recovery::InterruptionReason,
    ) -> Result<LedgerMutationOutcome, RecoveryPortFailure>;

    fn mark_aborted(
        &self,
        session_id: &SessionId,
    ) -> Result<LedgerMutationOutcome, RecoveryPortFailure>;

    fn quarantine(
        &self,
        session_id: &SessionId,
        reason: QuarantineReason,
    ) -> Result<LedgerMutationOutcome, RecoveryPortFailure>;
}

/// Durable business ledger for future restart-safe finalization.
///
/// This port owns only the claim and committed marker. It deliberately does
/// not apply test, replay, statistics, lesson, or other completion effects;
/// that atomic orchestration remains a later milestone.
pub trait FinalizationLedger {
    fn claim_finalization(
        &self,
        session_id: &SessionId,
        expected_fingerprint: &CompletionIntentFingerprint,
        claimed_at: DateTime<Utc>,
    ) -> Result<FinalizationLedgerClaimOutcome, RecoveryPortFailure>;

    fn mark_finalization_committed(
        &self,
        session_id: &SessionId,
        expected_fingerprint: &CompletionIntentFingerprint,
        committed_at: DateTime<Utc>,
    ) -> Result<FinalizationCommitOutcome, RecoveryPortFailure>;

    fn load_finalization(
        &self,
        session_id: &SessionId,
    ) -> Result<FinalizationLoadOutcome, RecoveryPortFailure>;

    fn quarantine_finalization(
        &self,
        session_id: &SessionId,
        expected_fingerprint: &CompletionIntentFingerprint,
        reason: FinalizationQuarantineReason,
    ) -> Result<FinalizationCommitOutcome, RecoveryPortFailure>;
}

/// Application-facing completion finalization capability.
///
/// The adapter will later own the atomic completion transaction. It must load
/// and validate the immutable stored completion intent for `session_id`, then
/// require its fingerprint to equal `expected_fingerprint` before applying any
/// effect. This milestone defines only the result vocabulary and does not
/// execute effects.
pub trait SessionFinalizer {
    fn finalize_completion(
        &self,
        session_id: &SessionId,
        expected_fingerprint: &CompletionIntentFingerprint,
    ) -> Result<FinalizationOutcome, RecoveryPortFailure>;
}

/// Read capabilities for history, test details, replay pages, and typed export
/// rows. Implementations guarantee stable storage ordering; the application
/// validates that guarantee before exposing a result.
///
/// All requests and results use application/domain values. In particular,
/// durable [`SessionId`] values are the only test identities crossing this
/// boundary.
pub trait HistoryReportingPort {
    fn list_history(&self, query: &HistoryQuery) -> Result<HistoryPageSource, ReportingError>;

    /// `Ok(None)` means no test record exists for this durable session.
    fn find_test_details(
        &self,
        session_id: &SessionId,
    ) -> Result<Option<TestDetails>, ReportingError>;

    /// `Ok(None)` means a test has no optional replay relation.
    ///
    /// A returned page must be strictly ascending by `frame_index`.
    fn list_replay_frames(
        &self,
        query: &ReplayQuery,
    ) -> Result<Option<ReplayPageSource>, ReportingError>;

    fn list_export_rows(&self, query: &ExportQuery) -> Result<ExportDatasetSource, ReportingError>;
}

/// Read capabilities for persisted daily aggregates and activity-derived
/// reporting policy. Daily rows are intentionally sparse; adapters must not
/// synthesize zero-value rows.
pub trait ProgressReportingPort {
    fn count_tests(&self) -> Result<u64, ReportingError>;

    /// Returns sparse daily rows in strictly ascending UTC calendar-day order.
    fn load_daily_statistics(
        &self,
        range: InclusiveDateRange,
    ) -> Result<Vec<crate::reporting::DailyStatisticsPoint>, ReportingError>;

    /// Returns the complete maintained streak projection relative to `as_of`.
    ///
    /// This must not be derived from a bounded test-history page: long-lived
    /// profiles can have streaks older than any UI or analytics sample window.
    fn load_streak_report(
        &self,
        as_of: ReportingDay,
    ) -> Result<crate::reporting::StreakReport, ReportingError>;
}

/// Read capability for the accepted persisted personal-best projections.
pub trait PersonalBestReportingPort {
    /// Returns entries ordered by update time descending and then by stable
    /// opaque dimension key ascending.
    fn list_personal_bests(
        &self,
        mode: Option<ReportingModeFilter>,
    ) -> Result<Vec<PersonalBestEntry>, ReportingError>;
}

/// Bounded inputs for the established achievements, insights, and consistency
/// calculations. The application selects limits, date ranges, and lesson
/// language policy; adapters execute the corresponding reads.
pub trait AnalyticsReportingPort {
    fn load_achievement_inputs(
        &self,
        query: &AchievementInputQuery,
    ) -> Result<AchievementInputs, ReportingError>;

    fn load_insight_inputs(
        &self,
        query: &InsightInputQuery,
    ) -> Result<InsightInputs, ReportingError>;
}
