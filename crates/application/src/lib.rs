//! Infrastructure-free application kernel contracts.
//!
//! The kernel depends on the domain and typing engine only. Tauri, SQLite,
//! filesystems, and embedded resources belong to adapters outside this crate.
//! Phase 3B.2 adds runtime provider seams while keeping production providers
//! in adapters. Recovery, replay, restart safety, and persistence policy are
//! intentionally outside this crate's scope.

pub mod kernel;
pub mod ports;
pub mod recovery;
pub mod reporting;
pub mod session;
pub mod startup_recovery;

pub use kernel::{
    SessionAbortError, SessionKernel, SessionLifecycleError, SessionProcessError, SessionStartError,
};
pub use ports::{
    AnalyticsReportingPort, FinalizationLedger, HistoryReportingPort, PersonalBestReportingPort,
    ProgressReportingPort, SessionClock, SessionCompletionStore, SessionFinalizer, SessionIdSource,
    SessionModeFactory, SessionRandomSource, SessionRecoveryLedger, SessionWallClock,
};
pub use recovery::{
    classify_finalization_claim, classify_recovery_candidate, compare_completion_intents,
    validate_durable_state_transition, validate_finalization_intent,
    validate_recovery_candidate_intent, validate_sanitized_session_descriptor,
    CanonicalizationVersion, CompletionIntent, CompletionIntentComparison, CompletionIntentError,
    CompletionIntentFingerprint, CompletionIntentLoadError, CompletionIntentLoadOutcome,
    CompletionIntentMetadata, CompletionIntentPayload, CompletionIntentVersion,
    CompletionPolicySnapshot, DailyGoalPolicy, DurableSessionState, DurableStateTransitionOutcome,
    FinalizationClaimOutcome, FinalizationCommitOutcome, FinalizationConflict,
    FinalizationIntentValidation, FinalizationLedgerClaimOutcome, FinalizationLedgerState,
    FinalizationLoadOutcome, FinalizationOutcome, FinalizationQuarantineReason, FinalizationRecord,
    FinalizationRecordError, InterruptionReason, LedgerConflict, LedgerMutationOutcome,
    QuarantineReason, RecoveryCandidate, RecoveryDecision, RecoveryIntentValidation,
    RecoveryPermanentFailure, RecoveryPortFailure, RecoveryReadiness, RecoveryReadinessEvent,
    RecoveryReadinessTransitionError, StartedSession, StartedSessionError,
    StoredCompletionIntentHeader, StoredHeaderValue, UnsupportedCanonicalizationVersion,
    UnsupportedIntentVersion, CURRENT_CANONICALIZATION_VERSION, CURRENT_COMPLETION_INTENT_VERSION,
    INTERRUPTED_SESSION_RETENTION_DAYS, MAX_COMPLETION_INTENT_PAYLOAD_BYTES,
    MAX_SESSION_DESCRIPTOR_BYTES, MAX_SESSION_DESCRIPTOR_DEPTH, MAX_SESSION_DESCRIPTOR_KEY_BYTES,
    MAX_SESSION_DESCRIPTOR_STRING_BYTES,
};
pub use reporting::{
    AchievementInputQuery, AchievementInputs, AnalyticsSnapshot, BuildTestHistoryExport,
    BuildTestHistoryExportRequest, DailyStatisticsPoint, DailyStatisticsRange, ExportDataset,
    ExportDatasetSource, ExportQuery, ExportRow, GetAnalyticsSnapshot, GetReportingSummary,
    GetStreakReport, GetTestDetails, GetTestReplayPage, HistoryFilter, HistoryItem, HistoryPage,
    HistoryPageSource, HistoryQuery, InclusiveDateRange, InsightInputQuery, InsightInputs,
    ListDailyStatistics, ListPersonalBests, ListTestHistory, ListTestHistoryRequest,
    OffsetPagination, PersonalBestConfigurationKey, PersonalBestDimension, PersonalBestEntry,
    RelativeReportingPeriod, ReplayFrame, ReplayPage, ReplayPageSource, ReplayQuery, ReportingDay,
    ReportingError, ReportingLanguage, ReportingLessonId, ReportingMetricSample, ReportingMode,
    ReportingModeFilter, ReportingSummary, StreakReport, TestDetails, ACHIEVEMENT_HISTORY_LIMIT,
    ANALYTICS_HISTORY_LIMIT, DASHBOARD_ACTIVITY_HISTORY_LIMIT, DEFAULT_EXPORT_PAGE_LIMIT,
    DEFAULT_HISTORY_PAGE_LIMIT, MAX_REPORTING_PAGE_LIMIT, MAX_REPORTING_PAGE_OFFSET,
};
pub use session::{SessionCompletion, SessionPersistenceReceipt, SessionStartRequest};
pub use startup_recovery::{
    StartupRecoveryBlockReason, StartupRecoveryCandidateAction, StartupRecoveryCandidateResult,
    StartupRecoveryCoordinator, StartupRecoveryGate, StartupRecoveryGateError,
    StartupRecoveryReport, StartupRecoveryRetryPolicy, StartupRecoveryRunOutcome,
    StartupRecoverySleeper, MAX_STARTUP_RECOVERY_CANDIDATE_RESULTS,
};
