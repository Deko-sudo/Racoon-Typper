//! Application-owned startup recovery orchestration.
//!
//! The coordinator composes only business ports. It never reads a database,
//! canonical payload, replay, or concrete adapter directly: candidate listing
//! is metadata-only, and the full immutable intent is loaded only for a
//! candidate that is eligible for finalization.

use std::num::NonZeroUsize;
use std::sync::Mutex;
use std::time::Duration;

use racoon_domain::SessionId;

use crate::ports::{FinalizationLedger, SessionFinalizer, SessionRecoveryLedger, SessionWallClock};
use crate::recovery::{
    classify_recovery_candidate, validate_recovery_candidate_intent, CompletionIntentLoadOutcome,
    DurableSessionState, FinalizationClaimOutcome, FinalizationLedgerClaimOutcome,
    FinalizationOutcome, FinalizationQuarantineReason, QuarantineReason, RecoveryCandidate,
    RecoveryDecision, RecoveryIntentValidation, RecoveryPermanentFailure, RecoveryPortFailure,
    RecoveryReadiness, RecoveryReadinessEvent,
};

/// Upper bound on per-candidate detail retained in one process-local report.
/// Aggregate counters always cover the complete scan.
pub const MAX_STARTUP_RECOVERY_CANDIDATE_RESULTS: usize = 256;

/// Bounded retry policy for temporary recovery-port failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartupRecoveryRetryPolicy {
    max_attempts: NonZeroUsize,
    delay: Duration,
}

impl StartupRecoveryRetryPolicy {
    pub const fn new(max_attempts: NonZeroUsize, delay: Duration) -> Self {
        Self {
            max_attempts,
            delay,
        }
    }

    pub const fn max_attempts(self) -> NonZeroUsize {
        self.max_attempts
    }

    pub const fn delay(self) -> Duration {
        self.delay
    }
}

impl Default for StartupRecoveryRetryPolicy {
    fn default() -> Self {
        Self::new(
            NonZeroUsize::new(3).expect("literal retry count is nonzero"),
            Duration::from_millis(25),
        )
    }
}

/// Application port used to delay retry attempts. Production chooses how to
/// wait; deterministic tests use a no-op implementation.
pub trait StartupRecoverySleeper {
    fn sleep(&self, delay: Duration);
}

/// Bounded reason why startup cannot safely enable normal mutation commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupRecoveryBlockReason {
    CandidateScanRetryExhausted,
    CandidateScanPermanentFailure(RecoveryPermanentFailure),
    CandidateRetryExhausted,
    CandidatePermanentFailure(RecoveryPermanentFailure),
    CandidateConflict,
    StateUnavailable,
}

/// Per-candidate business action retained in a startup recovery report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupRecoveryCandidateAction {
    Interrupted,
    Finalized,
    AlreadyFinalized,
    Quarantined(QuarantineReason),
    SkippedTerminal(DurableSessionState),
    Conflict,
    RetryExhausted,
    PermanentFailure(RecoveryPermanentFailure),
}

/// Bounded metadata about one processed recovery candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartupRecoveryCandidateResult {
    session_id: SessionId,
    original_state: DurableSessionState,
    action: StartupRecoveryCandidateAction,
    attempts: NonZeroUsize,
}

impl StartupRecoveryCandidateResult {
    fn new(
        session_id: SessionId,
        original_state: DurableSessionState,
        action: StartupRecoveryCandidateAction,
        attempts: NonZeroUsize,
    ) -> Self {
        Self {
            session_id,
            original_state,
            action,
            attempts,
        }
    }

    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    pub const fn original_state(&self) -> DurableSessionState {
        self.original_state
    }

    pub const fn action(&self) -> StartupRecoveryCandidateAction {
        self.action
    }

    pub const fn attempts(&self) -> NonZeroUsize {
        self.attempts
    }
}

/// Deterministic, process-local summary of one startup recovery run.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StartupRecoveryReport {
    scanned: usize,
    finalized: usize,
    already_finalized: usize,
    quarantined: usize,
    interrupted: usize,
    skipped_terminal: usize,
    retryable_failures: usize,
    permanent_failures: usize,
    conflicts: usize,
    candidate_results: Vec<StartupRecoveryCandidateResult>,
    candidate_results_truncated: bool,
}

impl StartupRecoveryReport {
    pub const fn scanned(&self) -> usize {
        self.scanned
    }

    pub const fn finalized(&self) -> usize {
        self.finalized
    }

    pub const fn already_finalized(&self) -> usize {
        self.already_finalized
    }

    pub const fn quarantined(&self) -> usize {
        self.quarantined
    }

    pub const fn interrupted(&self) -> usize {
        self.interrupted
    }

    pub const fn skipped_terminal(&self) -> usize {
        self.skipped_terminal
    }

    pub const fn retryable_failures(&self) -> usize {
        self.retryable_failures
    }

    pub const fn permanent_failures(&self) -> usize {
        self.permanent_failures
    }

    pub const fn conflicts(&self) -> usize {
        self.conflicts
    }

    pub fn candidate_results(&self) -> &[StartupRecoveryCandidateResult] {
        &self.candidate_results
    }

    pub const fn candidate_results_truncated(&self) -> bool {
        self.candidate_results_truncated
    }

    fn push(&mut self, result: StartupRecoveryCandidateResult) {
        self.scanned += 1;
        match result.action {
            StartupRecoveryCandidateAction::Interrupted => self.interrupted += 1,
            StartupRecoveryCandidateAction::Finalized => self.finalized += 1,
            StartupRecoveryCandidateAction::AlreadyFinalized => self.already_finalized += 1,
            StartupRecoveryCandidateAction::Quarantined(_) => self.quarantined += 1,
            StartupRecoveryCandidateAction::SkippedTerminal(_) => self.skipped_terminal += 1,
            StartupRecoveryCandidateAction::Conflict => self.conflicts += 1,
            StartupRecoveryCandidateAction::RetryExhausted => self.retryable_failures += 1,
            StartupRecoveryCandidateAction::PermanentFailure(_) => self.permanent_failures += 1,
        }
        if self.candidate_results.len() < MAX_STARTUP_RECOVERY_CANDIDATE_RESULTS {
            self.candidate_results.push(result);
        } else {
            self.candidate_results_truncated = true;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StartupRecoveryGateInner {
    state: RecoveryReadiness,
    report: Option<StartupRecoveryReport>,
    block_reason: Option<StartupRecoveryBlockReason>,
}

/// Thread-safe process-local readiness gate shared by startup wiring and
/// command adapters. The state transition is atomic under one mutex, so two
/// callers cannot begin recovery concurrently.
pub struct StartupRecoveryGate {
    inner: Mutex<StartupRecoveryGateInner>,
}

impl Default for StartupRecoveryGate {
    fn default() -> Self {
        Self::new()
    }
}

impl StartupRecoveryGate {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(StartupRecoveryGateInner {
                state: RecoveryReadiness::NotStarted,
                report: None,
                block_reason: None,
            }),
        }
    }

    pub fn state(&self) -> Result<RecoveryReadiness, StartupRecoveryGateError> {
        Ok(self.lock()?.state)
    }

    pub fn report(&self) -> Result<Option<StartupRecoveryReport>, StartupRecoveryGateError> {
        Ok(self.lock()?.report.clone())
    }

    pub fn block_reason(
        &self,
    ) -> Result<Option<StartupRecoveryBlockReason>, StartupRecoveryGateError> {
        Ok(self.lock()?.block_reason)
    }

    fn begin(&self) -> Result<StartupRecoveryBegin, StartupRecoveryGateError> {
        let mut inner = self.lock()?;
        match inner.state {
            RecoveryReadiness::NotStarted => {
                inner.state = inner
                    .state
                    .transition(RecoveryReadinessEvent::Begin)
                    .expect("accepted readiness transition");
                Ok(StartupRecoveryBegin::Started)
            }
            RecoveryReadiness::Recovering => Ok(StartupRecoveryBegin::InProgress),
            RecoveryReadiness::Ready => Ok(StartupRecoveryBegin::AlreadyCompleted(
                inner
                    .report
                    .clone()
                    .expect("ready state stores recovery report"),
            )),
            RecoveryReadiness::Blocked => Ok(StartupRecoveryBegin::AlreadyBlocked(
                inner.block_reason.expect("blocked state stores reason"),
            )),
        }
    }

    fn complete(&self, report: StartupRecoveryReport) -> Result<(), StartupRecoveryGateError> {
        let mut inner = self.lock()?;
        inner.state = inner
            .state
            .transition(RecoveryReadinessEvent::Complete)
            .map_err(|_| StartupRecoveryGateError::InvalidTransition)?;
        inner.report = Some(report);
        Ok(())
    }

    fn block(
        &self,
        report: StartupRecoveryReport,
        reason: StartupRecoveryBlockReason,
    ) -> Result<(), StartupRecoveryGateError> {
        let mut inner = self.lock()?;
        inner.state = inner
            .state
            .transition(RecoveryReadinessEvent::Block)
            .map_err(|_| StartupRecoveryGateError::InvalidTransition)?;
        inner.report = Some(report);
        inner.block_reason = Some(reason);
        Ok(())
    }

    fn lock(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, StartupRecoveryGateInner>, StartupRecoveryGateError> {
        self.inner
            .lock()
            .map_err(|_| StartupRecoveryGateError::StateUnavailable)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupRecoveryGateError {
    StateUnavailable,
    InvalidTransition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartupRecoveryRunOutcome {
    Completed(StartupRecoveryReport),
    AlreadyCompleted(StartupRecoveryReport),
    InProgress,
    Blocked(StartupRecoveryBlockReason),
}

enum StartupRecoveryBegin {
    Started,
    InProgress,
    AlreadyCompleted(StartupRecoveryReport),
    AlreadyBlocked(StartupRecoveryBlockReason),
}

/// Application-owned coordinator for one process startup recovery pass.
pub struct StartupRecoveryCoordinator<'a, L, F, C, W, S> {
    recovery_ledger: &'a L,
    finalization_ledger: &'a F,
    session_finalizer: &'a C,
    wall_clock: &'a W,
    sleeper: &'a S,
    retry_policy: StartupRecoveryRetryPolicy,
}

impl<'a, L, F, C, W, S> StartupRecoveryCoordinator<'a, L, F, C, W, S>
where
    L: SessionRecoveryLedger,
    F: FinalizationLedger,
    C: SessionFinalizer,
    W: SessionWallClock,
    S: StartupRecoverySleeper,
{
    pub fn new(
        recovery_ledger: &'a L,
        finalization_ledger: &'a F,
        session_finalizer: &'a C,
        wall_clock: &'a W,
        sleeper: &'a S,
        retry_policy: StartupRecoveryRetryPolicy,
    ) -> Self {
        Self {
            recovery_ledger,
            finalization_ledger,
            session_finalizer,
            wall_clock,
            sleeper,
            retry_policy,
        }
    }

    /// Runs recovery at most once for a gate. Candidate order is the accepted
    /// recovery-ledger order, which is `(created_at, session_id)` in SQLite.
    pub fn run(
        &self,
        gate: &StartupRecoveryGate,
    ) -> Result<StartupRecoveryRunOutcome, StartupRecoveryGateError> {
        match gate.begin()? {
            StartupRecoveryBegin::InProgress => return Ok(StartupRecoveryRunOutcome::InProgress),
            StartupRecoveryBegin::AlreadyCompleted(report) => {
                return Ok(StartupRecoveryRunOutcome::AlreadyCompleted(report));
            }
            StartupRecoveryBegin::AlreadyBlocked(reason) => {
                return Ok(StartupRecoveryRunOutcome::Blocked(reason));
            }
            StartupRecoveryBegin::Started => {}
        }

        let (candidates, _) = match self.retry(|| self.recovery_ledger.list_recovery_candidates()) {
            Ok(result) => result,
            Err(RecoveryPortFailure::RetryableFailure) => {
                let report = StartupRecoveryReport::default();
                let reason = StartupRecoveryBlockReason::CandidateScanRetryExhausted;
                gate.block(report, reason)?;
                return Ok(StartupRecoveryRunOutcome::Blocked(reason));
            }
            Err(RecoveryPortFailure::PermanentFailure(reason)) => {
                let report = StartupRecoveryReport::default();
                let block_reason =
                    StartupRecoveryBlockReason::CandidateScanPermanentFailure(reason);
                gate.block(report, block_reason)?;
                return Ok(StartupRecoveryRunOutcome::Blocked(block_reason));
            }
        };

        let mut report = StartupRecoveryReport::default();
        let mut blocked = None;
        for candidate in candidates {
            let result = self.process_candidate(&candidate);
            if let Some(reason) = block_reason_for(result.action) {
                blocked.get_or_insert(reason);
            }
            report.push(result);
        }

        if let Some(reason) = blocked {
            gate.block(report, reason)?;
            Ok(StartupRecoveryRunOutcome::Blocked(reason))
        } else {
            gate.complete(report.clone())?;
            Ok(StartupRecoveryRunOutcome::Completed(report))
        }
    }

    fn process_candidate(&self, candidate: &RecoveryCandidate) -> StartupRecoveryCandidateResult {
        let original_state = candidate.state();
        let session_id = candidate.session_id().clone();
        let (action, attempts) = match classify_recovery_candidate(candidate) {
            RecoveryDecision::MarkInterrupted { reason, .. } => {
                self.mutation_action(|| self.recovery_ledger.mark_interrupted(&session_id, reason))
            }
            RecoveryDecision::Quarantine { reason, .. } => {
                self.quarantine_action(&session_id, reason)
            }
            RecoveryDecision::NoOp { state, .. } => (
                match state {
                    DurableSessionState::Quarantined => {
                        StartupRecoveryCandidateAction::Quarantined(
                            QuarantineReason::InvalidStateRecord,
                        )
                    }
                    DurableSessionState::Interrupted => StartupRecoveryCandidateAction::Interrupted,
                    _ => StartupRecoveryCandidateAction::SkippedTerminal(state),
                },
                NonZeroUsize::MIN,
            ),
            RecoveryDecision::EligibleForFinalization { fingerprint, .. } => {
                self.finalize_candidate(candidate, &fingerprint)
            }
        };
        StartupRecoveryCandidateResult::new(session_id, original_state, action, attempts)
    }

    fn finalize_candidate(
        &self,
        candidate: &RecoveryCandidate,
        metadata_fingerprint: &crate::recovery::CompletionIntentFingerprint,
    ) -> (StartupRecoveryCandidateAction, NonZeroUsize) {
        let session_id = candidate.session_id();
        let (loaded, attempts) =
            match self.retry(|| self.recovery_ledger.load_completion_intent(session_id)) {
                Ok(value) => value,
                Err(error) => {
                    return failure_action(
                        error,
                        NonZeroUsize::new(self.retry_policy.max_attempts.get()).expect("nonzero"),
                    )
                }
            };
        let intent = match loaded {
            CompletionIntentLoadOutcome::Found(intent) => intent,
            CompletionIntentLoadOutcome::NotFound => {
                return self
                    .quarantine_action(session_id, QuarantineReason::MissingCompletionIntent);
            }
            CompletionIntentLoadOutcome::UnsupportedCanonicalizationVersion { .. } => {
                return self.quarantine_action(
                    session_id,
                    QuarantineReason::UnsupportedCanonicalizationVersion,
                );
            }
            CompletionIntentLoadOutcome::UnsupportedVersion { .. } => {
                return self
                    .quarantine_action(session_id, QuarantineReason::UnsupportedIntentVersion);
            }
            CompletionIntentLoadOutcome::Corrupt => {
                return self
                    .quarantine_action(session_id, QuarantineReason::CorruptCompletionPayload);
            }
            CompletionIntentLoadOutcome::Quarantined(reason) => {
                return self.quarantine_action(session_id, reason);
            }
        };

        match validate_recovery_candidate_intent(candidate, &intent) {
            RecoveryIntentValidation::Valid => {}
            RecoveryIntentValidation::Quarantine(reason) => {
                return self.quarantine_action(session_id, reason)
            }
        }
        if intent.fingerprint() != metadata_fingerprint {
            return self.quarantine_action(session_id, QuarantineReason::FingerprintMismatch);
        }

        if candidate.state() == DurableSessionState::AwaitingPersistence {
            let (claim, claim_attempts) = match self.retry(|| {
                self.recovery_ledger
                    .claim_completion_for_finalization(session_id, intent.fingerprint())
            }) {
                Ok(value) => value,
                Err(error) => return failure_action(error, attempts),
            };
            match claim {
                FinalizationClaimOutcome::Claimed | FinalizationClaimOutcome::AlreadyPending => {}
                FinalizationClaimOutcome::AlreadyFinalized => {
                    return (
                        StartupRecoveryCandidateAction::AlreadyFinalized,
                        claim_attempts,
                    );
                }
                FinalizationClaimOutcome::NotFound => {
                    return (
                        StartupRecoveryCandidateAction::PermanentFailure(
                            RecoveryPermanentFailure::IntegrityFailure,
                        ),
                        claim_attempts,
                    );
                }
                FinalizationClaimOutcome::Conflict(_) => {
                    return (StartupRecoveryCandidateAction::Conflict, claim_attempts);
                }
                FinalizationClaimOutcome::Quarantined(reason) => {
                    return (
                        StartupRecoveryCandidateAction::Quarantined(reason),
                        claim_attempts,
                    );
                }
                FinalizationClaimOutcome::RejectedTerminal { state } => {
                    return (
                        StartupRecoveryCandidateAction::SkippedTerminal(state),
                        claim_attempts,
                    );
                }
            }
        }

        let (claim, claim_attempts) = match self.retry(|| {
            self.finalization_ledger.claim_finalization(
                session_id,
                intent.fingerprint(),
                self.wall_clock.utc_now(),
            )
        }) {
            Ok(value) => value,
            Err(error) => return failure_action(error, attempts),
        };
        match claim {
            FinalizationLedgerClaimOutcome::Claimed
            | FinalizationLedgerClaimOutcome::AlreadyPending => {}
            FinalizationLedgerClaimOutcome::AlreadyCommitted => {
                // The metadata-only candidate may be stale: another process
                // can commit V008 and finalize V006 after the scan but before
                // this claim. Let the authoritative finalizer inspect the
                // current terminal evidence and converge to AlreadyFinalized
                // (or report durable corruption) instead of quarantining a
                // successfully completed session from stale scan metadata.
            }
            FinalizationLedgerClaimOutcome::NotFound
            | FinalizationLedgerClaimOutcome::MissingCompletionIntent => {
                return self
                    .quarantine_action(session_id, QuarantineReason::MissingCompletionIntent);
            }
            FinalizationLedgerClaimOutcome::Conflict(_) => {
                return (StartupRecoveryCandidateAction::Conflict, claim_attempts);
            }
            FinalizationLedgerClaimOutcome::Quarantined(
                FinalizationQuarantineReason::InvalidFinalizationState,
            ) => {
                // The scan can be stale when another coordinator has already
                // moved V006 to a terminal state. The finalizer owns the
                // authoritative terminal-proof check and can return
                // AlreadyFinalized safely.
            }
            FinalizationLedgerClaimOutcome::Quarantined(reason) => {
                return self
                    .quarantine_action(session_id, recovery_reason_from_finalization(reason));
            }
            FinalizationLedgerClaimOutcome::Corrupt => {
                return self
                    .quarantine_action(session_id, QuarantineReason::InconsistentDurableMetadata);
            }
        }

        match self.retry(|| {
            self.session_finalizer
                .finalize_completion(session_id, intent.fingerprint())
        }) {
            Ok((FinalizationOutcome::NewlyFinalized, finalizer_attempts)) => (
                StartupRecoveryCandidateAction::Finalized,
                finalizer_attempts,
            ),
            Ok((FinalizationOutcome::AlreadyFinalized, finalizer_attempts)) => (
                StartupRecoveryCandidateAction::AlreadyFinalized,
                finalizer_attempts,
            ),
            Ok((FinalizationOutcome::Conflict(_), finalizer_attempts)) => {
                (StartupRecoveryCandidateAction::Conflict, finalizer_attempts)
            }
            Ok((FinalizationOutcome::Quarantined(reason), _)) => {
                self.quarantine_action(session_id, reason)
            }
            Ok((FinalizationOutcome::NotFound, finalizer_attempts)) => (
                StartupRecoveryCandidateAction::PermanentFailure(
                    RecoveryPermanentFailure::IntegrityFailure,
                ),
                finalizer_attempts,
            ),
            Err(error) => failure_action(error, claim_attempts),
        }
    }

    fn mutation_action(
        &self,
        operation: impl FnMut() -> Result<crate::recovery::LedgerMutationOutcome, RecoveryPortFailure>,
    ) -> (StartupRecoveryCandidateAction, NonZeroUsize) {
        match self.retry(operation) {
            Ok((crate::recovery::LedgerMutationOutcome::Created, attempts))
            | Ok((crate::recovery::LedgerMutationOutcome::AlreadyExistsIdentical, attempts)) => {
                (StartupRecoveryCandidateAction::Interrupted, attempts)
            }
            Ok((crate::recovery::LedgerMutationOutcome::Quarantined(reason), attempts)) => (
                StartupRecoveryCandidateAction::Quarantined(reason),
                attempts,
            ),
            Ok((crate::recovery::LedgerMutationOutcome::Conflicting(_), attempts)) => {
                (StartupRecoveryCandidateAction::Conflict, attempts)
            }
            Ok((crate::recovery::LedgerMutationOutcome::NotFound, attempts)) => (
                StartupRecoveryCandidateAction::PermanentFailure(
                    RecoveryPermanentFailure::IntegrityFailure,
                ),
                attempts,
            ),
            Err(error) => failure_action(error, self.retry_policy.max_attempts),
        }
    }

    fn quarantine_action(
        &self,
        session_id: &SessionId,
        reason: QuarantineReason,
    ) -> (StartupRecoveryCandidateAction, NonZeroUsize) {
        match self.retry(|| self.recovery_ledger.quarantine(session_id, reason)) {
            Ok((crate::recovery::LedgerMutationOutcome::Created, attempts))
            | Ok((crate::recovery::LedgerMutationOutcome::AlreadyExistsIdentical, attempts))
            | Ok((crate::recovery::LedgerMutationOutcome::Quarantined(_), attempts)) => (
                StartupRecoveryCandidateAction::Quarantined(reason),
                attempts,
            ),
            Ok((crate::recovery::LedgerMutationOutcome::Conflicting(_), attempts)) => {
                (StartupRecoveryCandidateAction::Conflict, attempts)
            }
            Ok((crate::recovery::LedgerMutationOutcome::NotFound, attempts)) => (
                StartupRecoveryCandidateAction::PermanentFailure(
                    RecoveryPermanentFailure::IntegrityFailure,
                ),
                attempts,
            ),
            Err(error) => failure_action(error, self.retry_policy.max_attempts),
        }
    }

    fn retry<T>(
        &self,
        mut operation: impl FnMut() -> Result<T, RecoveryPortFailure>,
    ) -> Result<(T, NonZeroUsize), RecoveryPortFailure> {
        let mut attempts = NonZeroUsize::MIN;
        loop {
            match operation() {
                Ok(value) => return Ok((value, attempts)),
                Err(RecoveryPortFailure::RetryableFailure)
                    if attempts < self.retry_policy.max_attempts =>
                {
                    self.sleeper.sleep(self.retry_policy.delay);
                    attempts = NonZeroUsize::new(attempts.get() + 1).expect("incremented nonzero");
                }
                Err(error) => return Err(error),
            }
        }
    }
}

fn recovery_reason_from_finalization(reason: FinalizationQuarantineReason) -> QuarantineReason {
    match reason {
        FinalizationQuarantineReason::MissingCompletionIntent => {
            QuarantineReason::MissingCompletionIntent
        }
        FinalizationQuarantineReason::CorruptDurableMetadata => {
            QuarantineReason::InconsistentDurableMetadata
        }
        FinalizationQuarantineReason::FingerprintMismatch => QuarantineReason::FingerprintMismatch,
        FinalizationQuarantineReason::InvalidFinalizationState => {
            QuarantineReason::InvalidStateRecord
        }
    }
}

fn failure_action(
    failure: RecoveryPortFailure,
    attempts: NonZeroUsize,
) -> (StartupRecoveryCandidateAction, NonZeroUsize) {
    match failure {
        RecoveryPortFailure::RetryableFailure => {
            (StartupRecoveryCandidateAction::RetryExhausted, attempts)
        }
        RecoveryPortFailure::PermanentFailure(reason) => (
            StartupRecoveryCandidateAction::PermanentFailure(reason),
            attempts,
        ),
    }
}

fn block_reason_for(action: StartupRecoveryCandidateAction) -> Option<StartupRecoveryBlockReason> {
    match action {
        StartupRecoveryCandidateAction::RetryExhausted => {
            Some(StartupRecoveryBlockReason::CandidateRetryExhausted)
        }
        StartupRecoveryCandidateAction::PermanentFailure(reason) => Some(
            StartupRecoveryBlockReason::CandidatePermanentFailure(reason),
        ),
        StartupRecoveryCandidateAction::Conflict => {
            Some(StartupRecoveryBlockReason::CandidateConflict)
        }
        StartupRecoveryCandidateAction::Interrupted
        | StartupRecoveryCandidateAction::Finalized
        | StartupRecoveryCandidateAction::AlreadyFinalized
        | StartupRecoveryCandidateAction::Quarantined(_)
        | StartupRecoveryCandidateAction::SkippedTerminal(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use chrono::{DateTime, Utc};
    use racoon_core::ReplayFrame;
    use racoon_domain::{CharStatus, FinalStats};
    use serde_json::json;

    use super::*;
    use crate::recovery::{
        CompletionIntent, CompletionIntentFingerprint, CompletionPolicySnapshot,
        FinalizationCommitOutcome, FinalizationLoadOutcome, InterruptionReason,
        LedgerMutationOutcome,
    };
    use crate::session::SessionCompletion;

    const SESSION_A: &str = "018f0c2e-7b8d-7abc-8def-0123456789aa";

    fn session_id() -> SessionId {
        SessionId::parse(SESSION_A).expect("fixture UUIDv7")
    }

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-07-16T12:00:00Z")
            .expect("fixture timestamp")
            .with_timezone(&Utc)
    }

    fn intent() -> CompletionIntent {
        CompletionIntent::from_completion(
            &SessionCompletion {
                session_id: session_id(),
                completed_at: now(),
                final_stats: FinalStats {
                    wpm: 60.0,
                    raw_wpm: 60.0,
                    accuracy: 1.0,
                    raw_accuracy: 1.0,
                    consistency: None,
                    correct_chars: 1,
                    incorrect_chars: 0,
                    backspaces: 0,
                    char_stats: json!({}),
                    heatmap: json!({}),
                    graph_data: None,
                    duration_ms: 1,
                },
                mode_type: "custom".to_string(),
                mode_config: json!({"language": "en"}),
                language: "en".to_string(),
                text_length: 1,
                replay_frames: vec![ReplayFrame {
                    timestamp_ms: 1,
                    key: "a".to_string(),
                    caret_pos: 1,
                    char_status: CharStatus::Correct,
                    expected_char: 'a',
                    typed_char: Some('a'),
                }],
                lesson_id: None,
            },
            CompletionPolicySnapshot::time(1.0),
        )
        .expect("fixture intent")
    }

    struct FixedClock;

    impl SessionWallClock for FixedClock {
        fn utc_now(&self) -> DateTime<Utc> {
            now()
        }
    }

    #[derive(Default)]
    struct NoopSleeper;

    impl StartupRecoverySleeper for NoopSleeper {
        fn sleep(&self, _: Duration) {}
    }

    struct FakeRecoveryLedger {
        candidates: Vec<RecoveryCandidate>,
        intent: Option<CompletionIntent>,
        list_failures: Mutex<Vec<RecoveryPortFailure>>,
        interruptions: Mutex<Vec<SessionId>>,
        quarantines: Mutex<Vec<(SessionId, QuarantineReason)>>,
        claims: Mutex<Vec<SessionId>>,
    }

    impl FakeRecoveryLedger {
        fn empty() -> Self {
            Self {
                candidates: vec![],
                intent: None,
                list_failures: Mutex::new(vec![]),
                interruptions: Mutex::new(vec![]),
                quarantines: Mutex::new(vec![]),
                claims: Mutex::new(vec![]),
            }
        }
    }

    impl SessionRecoveryLedger for FakeRecoveryLedger {
        fn record_started(
            &self,
            _: &crate::recovery::StartedSession,
        ) -> Result<LedgerMutationOutcome, RecoveryPortFailure> {
            unreachable!("not used by startup recovery")
        }

        fn record_completion_intent(
            &self,
            _: &CompletionIntent,
        ) -> Result<LedgerMutationOutcome, RecoveryPortFailure> {
            unreachable!("not used by startup recovery")
        }

        fn claim_completion_for_finalization(
            &self,
            session_id: &SessionId,
            _: &CompletionIntentFingerprint,
        ) -> Result<FinalizationClaimOutcome, RecoveryPortFailure> {
            self.claims
                .lock()
                .expect("fixture lock")
                .push(session_id.clone());
            Ok(FinalizationClaimOutcome::Claimed)
        }

        fn list_recovery_candidates(&self) -> Result<Vec<RecoveryCandidate>, RecoveryPortFailure> {
            let mut failures = self.list_failures.lock().expect("fixture lock");
            if !failures.is_empty() {
                return Err(failures.remove(0));
            }
            Ok(self.candidates.clone())
        }

        fn load_completion_intent(
            &self,
            _: &SessionId,
        ) -> Result<CompletionIntentLoadOutcome, RecoveryPortFailure> {
            Ok(self
                .intent
                .clone()
                .map(|intent| CompletionIntentLoadOutcome::Found(Box::new(intent)))
                .unwrap_or(CompletionIntentLoadOutcome::NotFound))
        }

        fn mark_interrupted(
            &self,
            session_id: &SessionId,
            _: InterruptionReason,
        ) -> Result<LedgerMutationOutcome, RecoveryPortFailure> {
            self.interruptions
                .lock()
                .expect("fixture lock")
                .push(session_id.clone());
            Ok(LedgerMutationOutcome::Created)
        }

        fn mark_aborted(
            &self,
            _: &SessionId,
        ) -> Result<LedgerMutationOutcome, RecoveryPortFailure> {
            unreachable!("not used by startup recovery")
        }

        fn quarantine(
            &self,
            session_id: &SessionId,
            reason: QuarantineReason,
        ) -> Result<LedgerMutationOutcome, RecoveryPortFailure> {
            self.quarantines
                .lock()
                .expect("fixture lock")
                .push((session_id.clone(), reason));
            Ok(LedgerMutationOutcome::Quarantined(reason))
        }
    }

    struct FakeFinalizationLedger;

    impl FinalizationLedger for FakeFinalizationLedger {
        fn claim_finalization(
            &self,
            _: &SessionId,
            _: &CompletionIntentFingerprint,
            _: DateTime<Utc>,
        ) -> Result<FinalizationLedgerClaimOutcome, RecoveryPortFailure> {
            Ok(FinalizationLedgerClaimOutcome::Claimed)
        }

        fn mark_finalization_committed(
            &self,
            _: &SessionId,
            _: &CompletionIntentFingerprint,
            _: DateTime<Utc>,
        ) -> Result<FinalizationCommitOutcome, RecoveryPortFailure> {
            unreachable!("not used by startup recovery")
        }

        fn load_finalization(
            &self,
            _: &SessionId,
        ) -> Result<FinalizationLoadOutcome, RecoveryPortFailure> {
            unreachable!("not used by startup recovery")
        }

        fn quarantine_finalization(
            &self,
            _: &SessionId,
            _: &CompletionIntentFingerprint,
            _: FinalizationQuarantineReason,
        ) -> Result<FinalizationCommitOutcome, RecoveryPortFailure> {
            unreachable!("not used by startup recovery")
        }
    }

    struct FakeFinalizer {
        outcome: FinalizationOutcome,
        calls: Mutex<usize>,
    }

    impl FakeFinalizer {
        fn new(outcome: FinalizationOutcome) -> Self {
            Self {
                outcome,
                calls: Mutex::new(0),
            }
        }
    }

    impl SessionFinalizer for FakeFinalizer {
        fn finalize_completion(
            &self,
            _: &SessionId,
            _: &CompletionIntentFingerprint,
        ) -> Result<FinalizationOutcome, RecoveryPortFailure> {
            *self.calls.lock().expect("fixture lock") += 1;
            Ok(self.outcome.clone())
        }
    }

    fn coordinator<'a>(
        recovery: &'a FakeRecoveryLedger,
        finalizations: &'a FakeFinalizationLedger,
        finalizer: &'a FakeFinalizer,
        clock: &'a FixedClock,
        sleeper: &'a NoopSleeper,
    ) -> StartupRecoveryCoordinator<
        'a,
        FakeRecoveryLedger,
        FakeFinalizationLedger,
        FakeFinalizer,
        FixedClock,
        NoopSleeper,
    > {
        StartupRecoveryCoordinator::new(
            recovery,
            finalizations,
            finalizer,
            clock,
            sleeper,
            StartupRecoveryRetryPolicy::new(NonZeroUsize::new(2).expect("nonzero"), Duration::ZERO),
        )
    }

    #[test]
    fn empty_recovery_becomes_ready_and_repeated_invocation_is_idempotent() {
        let recovery = FakeRecoveryLedger::empty();
        let finalizations = FakeFinalizationLedger;
        let finalizer = FakeFinalizer::new(FinalizationOutcome::NewlyFinalized);
        let clock = FixedClock;
        let sleeper = NoopSleeper;
        let coordinator = coordinator(&recovery, &finalizations, &finalizer, &clock, &sleeper);
        let gate = StartupRecoveryGate::new();

        let report = match coordinator.run(&gate).expect("run") {
            StartupRecoveryRunOutcome::Completed(report) => report,
            outcome => panic!("unexpected outcome: {outcome:?}"),
        };
        assert_eq!(report.scanned(), 0);
        assert_eq!(gate.state().expect("state"), RecoveryReadiness::Ready);
        assert!(matches!(
            coordinator.run(&gate).expect("repeat"),
            StartupRecoveryRunOutcome::AlreadyCompleted(_)
        ));
    }

    #[test]
    fn running_candidate_is_interrupted_without_loading_an_intent() {
        let recovery = FakeRecoveryLedger {
            candidates: vec![RecoveryCandidate::new(
                session_id(),
                DurableSessionState::Running,
                crate::recovery::CompletionIntentMetadata::Missing,
            )],
            ..FakeRecoveryLedger::empty()
        };
        let finalizations = FakeFinalizationLedger;
        let finalizer = FakeFinalizer::new(FinalizationOutcome::NewlyFinalized);
        let clock = FixedClock;
        let sleeper = NoopSleeper;
        let gate = StartupRecoveryGate::new();

        let outcome = coordinator(&recovery, &finalizations, &finalizer, &clock, &sleeper)
            .run(&gate)
            .expect("run");
        let StartupRecoveryRunOutcome::Completed(report) = outcome else {
            panic!("running record is recoverable")
        };
        assert_eq!(report.interrupted(), 1);
        assert_eq!(
            recovery.interruptions.lock().expect("fixture lock").len(),
            1
        );
        assert_eq!(*finalizer.calls.lock().expect("fixture lock"), 0);
    }

    #[test]
    fn finalization_pending_loads_full_intent_and_invokes_finalizer_once() {
        let intent = intent();
        let metadata = crate::recovery::CompletionIntentMetadata::present(&intent);
        let recovery = FakeRecoveryLedger {
            candidates: vec![RecoveryCandidate::new(
                session_id(),
                DurableSessionState::FinalizationPending,
                metadata,
            )],
            intent: Some(intent),
            ..FakeRecoveryLedger::empty()
        };
        let finalizations = FakeFinalizationLedger;
        let finalizer = FakeFinalizer::new(FinalizationOutcome::NewlyFinalized);
        let clock = FixedClock;
        let sleeper = NoopSleeper;
        let gate = StartupRecoveryGate::new();

        let outcome = coordinator(&recovery, &finalizations, &finalizer, &clock, &sleeper)
            .run(&gate)
            .expect("run");
        let StartupRecoveryRunOutcome::Completed(report) = outcome else {
            panic!("finalization should complete")
        };
        assert_eq!(report.finalized(), 1);
        assert_eq!(*finalizer.calls.lock().expect("fixture lock"), 1);
        assert_eq!(gate.state().expect("state"), RecoveryReadiness::Ready);
    }

    #[test]
    fn corrupt_metadata_is_quarantined_without_full_intent_or_finalizer() {
        let recovery = FakeRecoveryLedger {
            candidates: vec![RecoveryCandidate::new(
                session_id(),
                DurableSessionState::AwaitingPersistence,
                crate::recovery::CompletionIntentMetadata::Corrupt,
            )],
            ..FakeRecoveryLedger::empty()
        };
        let finalizations = FakeFinalizationLedger;
        let finalizer = FakeFinalizer::new(FinalizationOutcome::NewlyFinalized);
        let clock = FixedClock;
        let sleeper = NoopSleeper;
        let gate = StartupRecoveryGate::new();

        let outcome = coordinator(&recovery, &finalizations, &finalizer, &clock, &sleeper)
            .run(&gate)
            .expect("run");
        let StartupRecoveryRunOutcome::Completed(report) = outcome else {
            panic!("row-local corruption must not block startup")
        };
        assert_eq!(report.quarantined(), 1);
        assert_eq!(*finalizer.calls.lock().expect("fixture lock"), 0);
        assert_eq!(
            recovery.quarantines.lock().expect("fixture lock")[0].1,
            QuarantineReason::CorruptCompletionPayload
        );
    }

    #[test]
    fn finalizer_quarantine_is_durably_reflected_in_the_session_ledger() {
        let intent = intent();
        let recovery = FakeRecoveryLedger {
            candidates: vec![RecoveryCandidate::new(
                session_id(),
                DurableSessionState::FinalizationPending,
                crate::recovery::CompletionIntentMetadata::present(&intent),
            )],
            intent: Some(intent),
            ..FakeRecoveryLedger::empty()
        };
        let finalizations = FakeFinalizationLedger;
        let finalizer = FakeFinalizer::new(FinalizationOutcome::Quarantined(
            QuarantineReason::InconsistentDurableMetadata,
        ));
        let clock = FixedClock;
        let sleeper = NoopSleeper;
        let gate = StartupRecoveryGate::new();

        let outcome = coordinator(&recovery, &finalizations, &finalizer, &clock, &sleeper)
            .run(&gate)
            .expect("run");
        assert!(matches!(outcome, StartupRecoveryRunOutcome::Completed(_)));
        assert_eq!(
            recovery.quarantines.lock().expect("fixture lock")[0].1,
            QuarantineReason::InconsistentDurableMetadata
        );
        assert_eq!(gate.state().expect("state"), RecoveryReadiness::Ready);
    }

    #[test]
    fn retryable_scan_retries_then_reaches_ready_but_permanent_scan_blocks() {
        let recovery = FakeRecoveryLedger::empty();
        recovery
            .list_failures
            .lock()
            .expect("fixture lock")
            .push(RecoveryPortFailure::RetryableFailure);
        let finalizations = FakeFinalizationLedger;
        let finalizer = FakeFinalizer::new(FinalizationOutcome::NewlyFinalized);
        let clock = FixedClock;
        let sleeper = NoopSleeper;
        let gate = StartupRecoveryGate::new();
        assert!(matches!(
            coordinator(&recovery, &finalizations, &finalizer, &clock, &sleeper)
                .run(&gate)
                .expect("retry run"),
            StartupRecoveryRunOutcome::Completed(_)
        ));

        let permanent = FakeRecoveryLedger::empty();
        permanent.list_failures.lock().expect("fixture lock").push(
            RecoveryPortFailure::PermanentFailure(RecoveryPermanentFailure::UnsupportedSchema),
        );
        let gate = StartupRecoveryGate::new();
        assert_eq!(
            coordinator(&permanent, &finalizations, &finalizer, &clock, &sleeper)
                .run(&gate)
                .expect("permanent run"),
            StartupRecoveryRunOutcome::Blocked(
                StartupRecoveryBlockReason::CandidateScanPermanentFailure(
                    RecoveryPermanentFailure::UnsupportedSchema
                )
            )
        );
        assert_eq!(gate.state().expect("state"), RecoveryReadiness::Blocked);
    }

    #[test]
    fn gate_rejects_concurrent_start_and_keeps_terminal_startup_state() {
        let gate = StartupRecoveryGate::new();
        assert!(matches!(
            gate.begin().expect("begin"),
            StartupRecoveryBegin::Started
        ));
        assert_eq!(gate.state().expect("state"), RecoveryReadiness::Recovering);
        assert!(matches!(
            gate.begin().expect("concurrent begin"),
            StartupRecoveryBegin::InProgress
        ));
        let report = StartupRecoveryReport::default();
        gate.block(report, StartupRecoveryBlockReason::CandidateRetryExhausted)
            .expect("block");
        assert_eq!(gate.state().expect("state"), RecoveryReadiness::Blocked);
        assert!(matches!(
            gate.begin().expect("terminal begin"),
            StartupRecoveryBegin::AlreadyBlocked(
                StartupRecoveryBlockReason::CandidateRetryExhausted
            )
        ));
    }
}
