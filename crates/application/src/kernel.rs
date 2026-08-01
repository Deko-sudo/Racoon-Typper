//! Transport-neutral orchestration for the session vertical slice.
//!
//! `SessionKernel` owns the part of the workflow that is independent of
//! Tauri, SQLite, files, and embedded resources: lifecycle checks, backend
//! session correlation, core input processing, completion snapshot creation,
//! and the existing two-phase persistence handoff. Concrete providers are
//! supplied by adapters through the ports in [`crate::ports`].

use std::sync::Mutex;

use racoon_core::{CoreEngine, KeyEvent, TestMode, TestSessionInfo};
use racoon_domain::{EngineOutput, FinalStats, SessionId, SessionState};

use crate::ports::{
    SessionClock, SessionCompletionStore, SessionIdSource, SessionModeFactory, SessionRandomSource,
    SessionWallClock,
};
use crate::session::{SessionCompletion, SessionStartRequest};

/// A session cannot be replaced while it is running or while its completion
/// remains pending persistence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionLifecycleError {
    /// A new start was requested while another session is running.
    AlreadyActive,
    /// A completion is being persisted or is available for retry.
    Finalizing,
    /// The core rejected a transition from a state the kernel considered
    /// startable. This indicates an internal invariant violation.
    InvalidTransition,
}

/// Failure to construct or start a session.
#[derive(Debug)]
pub enum SessionStartError<E> {
    /// The adapter-owned mode factory rejected the request.
    Mode(E),
    /// The core session lifecycle rejected the start.
    Lifecycle(SessionLifecycleError),
}

/// Failure while processing an input frame or handing a completion to its
/// persistence adapter.
#[derive(Debug)]
pub enum SessionProcessError<E> {
    /// The caller supplied a stale or forged session correlation token.
    SessionNotFound(SessionId),
    /// The engine mutex could not be acquired.
    StateUnavailable,
    /// Another request already claimed the completion for persistence.
    Finalizing,
    /// The adapter failed to persist the immutable completion snapshot.
    Persistence(E),
    /// The core reported completion without the metadata required to build a
    /// persistence snapshot.
    InvalidCompletion(&'static str),
}

/// Failure while aborting a session.
#[derive(Debug)]
pub enum SessionAbortError {
    /// The caller supplied a stale or forged session correlation token.
    SessionNotFound(SessionId),
    /// The session has already completed and cannot be discarded.
    Finalizing,
}

/// Stateless application kernel for the session use cases.
///
/// Providers are passed to each operation instead of being stored here. This
/// keeps the kernel independent from application startup, makes the seams
/// replaceable by future runtimes, and avoids introducing a service container
/// before there is a concrete lifetime/ownership problem to solve.
#[derive(Debug, Clone, Copy, Default)]
pub struct SessionKernel;

impl SessionKernel {
    pub const fn new() -> Self {
        Self
    }

    /// Returns whether a new session may replace the current core session.
    pub fn ensure_session_can_start(engine: &CoreEngine) -> Result<(), SessionLifecycleError> {
        match engine.session_state() {
            SessionState::Idle | SessionState::Persisted => Ok(()),
            SessionState::Running => Err(SessionLifecycleError::AlreadyActive),
            SessionState::AwaitingPersistence | SessionState::Persisting => {
                Err(SessionLifecycleError::Finalizing)
            }
        }
    }

    /// Builds a mode through the adapter port and starts it with a backend
    /// identity. The mode factory owns resource lookup and request validation.
    pub fn start_session<I, M, R>(
        &self,
        engine: &mut CoreEngine,
        request: &SessionStartRequest,
        language: &str,
        id_source: &mut I,
        random_source: &mut R,
        mode_factory: &M,
    ) -> Result<TestSessionInfo, SessionStartError<M::Error>>
    where
        I: SessionIdSource,
        M: SessionModeFactory,
        R: SessionRandomSource,
    {
        let mode = mode_factory
            .build_mode(request, language, random_source)
            .map_err(SessionStartError::Mode)?;
        // Keep request validation/resource selection ahead of the lifecycle
        // gate, matching the existing adapter behavior and its error ordering.
        Self::ensure_session_can_start(engine).map_err(SessionStartError::Lifecycle)?;

        self.start_mode(engine, mode, id_source)
            .map_err(SessionStartError::Lifecycle)
    }

    /// Starts an already-constructed mode with an adapter-provided identity.
    /// This is used by adapter-owned custom-text and lesson preparation while
    /// keeping the engine transition and identity allocation in one place.
    pub fn start_mode<I>(
        &self,
        engine: &mut CoreEngine,
        mode: Box<dyn TestMode>,
        id_source: &mut I,
    ) -> Result<TestSessionInfo, SessionLifecycleError>
    where
        I: SessionIdSource,
    {
        Self::ensure_session_can_start(engine)?;
        let session_id = id_source.next_session_id();
        engine
            .start_test_mode(session_id, mode)
            .map_err(Self::lifecycle_error_from_core)
    }

    /// Processes one backend-authoritative key event and hands a completed
    /// session to the business-oriented completion port.
    ///
    /// The engine lock is released before persistence starts. A failed store
    /// call returns the core to its existing retryable state, preserving the
    /// completion snapshot and its original completion timestamp.
    #[allow(clippy::too_many_arguments)]
    pub fn process_key<C, W, S>(
        &self,
        engine_state: &Mutex<CoreEngine>,
        clock: &C,
        wall_clock: &W,
        completion_store: &S,
        session_id: SessionId,
        key: String,
        code: String,
    ) -> Result<EngineOutput, SessionProcessError<S::Error>>
    where
        C: SessionClock + ?Sized,
        W: SessionWallClock + ?Sized,
        S: SessionCompletionStore + ?Sized,
    {
        let (mut output, completion) = {
            let mut engine = engine_state
                .lock()
                .map_err(|_| SessionProcessError::StateUnavailable)?;
            Self::ensure_session_matches(&engine, &session_id)?;

            let key_event = KeyEvent {
                key,
                code,
                timestamp: clock.monotonic_timestamp_ms(),
            };
            let output = engine.process_key(&key_event);
            if output.test_complete.is_some() && engine.completion_timestamp().is_none() {
                let _ = engine.set_completion_timestamp(wall_clock.utc_now());
            }
            let completion = match output.test_complete.as_ref() {
                Some(final_stats) => Some(
                    Self::completion_from_engine(&engine, final_stats)
                        .map_err(SessionProcessError::InvalidCompletion)?,
                ),
                None => None,
            };

            if completion.is_some() && !engine.begin_persistence() {
                return Err(SessionProcessError::Finalizing);
            }

            (output, completion)
        };

        if let Some(completion) = completion {
            if let Err(error) = completion_store.persist_completion(&completion) {
                let mut engine = engine_state
                    .lock()
                    .map_err(|_| SessionProcessError::StateUnavailable)?;
                let _ = engine.mark_persistence_failed();
                return Err(SessionProcessError::Persistence(error));
            }

            let mut engine = engine_state
                .lock()
                .map_err(|_| SessionProcessError::StateUnavailable)?;
            if !engine.mark_persisted() {
                return Err(SessionProcessError::Finalizing);
            }
            output.session_state = engine.session_state();
        }

        Ok(output)
    }

    /// Aborts the session identified by the backend-issued identity.
    pub fn abort_session(
        &self,
        engine: &mut CoreEngine,
        session_id: SessionId,
    ) -> Result<(), SessionAbortError> {
        if engine.current_session_id() != Some(&session_id) {
            return Err(SessionAbortError::SessionNotFound(session_id));
        }
        if !engine.abort() {
            return Err(SessionAbortError::Finalizing);
        }
        Ok(())
    }

    fn ensure_session_matches<E>(
        engine: &CoreEngine,
        requested: &SessionId,
    ) -> Result<(), SessionProcessError<E>> {
        if engine.current_session_id() == Some(requested) {
            Ok(())
        } else {
            Err(SessionProcessError::SessionNotFound(requested.clone()))
        }
    }

    fn completion_from_engine(
        engine: &CoreEngine,
        final_stats: &FinalStats,
    ) -> Result<SessionCompletion, &'static str> {
        let completed_at = engine
            .completion_timestamp()
            .ok_or("completed session is missing its completion timestamp")?;
        let session_id = engine
            .current_session_id()
            .cloned()
            .ok_or("completed session is missing its session identity")?;
        let mode_config = engine
            .current_mode_config()
            .unwrap_or_else(|| serde_json::json!({}));
        let lesson_id = mode_config
            .get("lesson_id")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned);
        let mode_type = if lesson_id.is_some() {
            "lesson".to_string()
        } else {
            engine
                .current_mode_type()
                .map(|mode| mode.to_string())
                .unwrap_or_else(|| "time".to_string())
        };

        Ok(SessionCompletion {
            session_id,
            completed_at,
            final_stats: final_stats.clone(),
            mode_type,
            mode_config,
            language: engine.current_language().unwrap_or("en").to_string(),
            text_length: engine.current_text().map_or(0, |text| text.chars().count()),
            replay_frames: engine.replay_frames().to_vec(),
            lesson_id,
        })
    }

    fn lifecycle_error_from_core(state: SessionState) -> SessionLifecycleError {
        match state {
            SessionState::Running => SessionLifecycleError::AlreadyActive,
            SessionState::AwaitingPersistence | SessionState::Persisting => {
                SessionLifecycleError::Finalizing
            }
            SessionState::Idle | SessionState::Persisted => {
                SessionLifecycleError::InvalidTransition
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use chrono::{DateTime, Utc};
    use racoon_core::{TestMode, TimeMode};
    use racoon_domain::SessionState;

    use crate::session::SessionPersistenceReceipt;

    use super::*;

    struct FixedSessionIdSource;

    impl SessionIdSource for FixedSessionIdSource {
        fn next_session_id(&mut self) -> SessionId {
            SessionId::from("fixture-session")
        }
    }

    #[derive(Default)]
    struct FixedRandomSource {
        calls: usize,
    }

    impl SessionRandomSource for FixedRandomSource {
        fn next_u64(&mut self) -> u64 {
            self.calls += 1;
            0
        }
    }

    struct FixtureModes;

    impl SessionModeFactory for FixtureModes {
        type Error = Infallible;

        fn build_mode(
            &self,
            _request: &SessionStartRequest,
            _language: &str,
            random_source: &mut dyn SessionRandomSource,
        ) -> Result<Box<dyn TestMode>, Self::Error> {
            let _ = random_source.next_u64();
            Ok(Box::new(TimeMode::new(
                "a".to_string(),
                "en".to_string(),
                30,
            )))
        }
    }

    struct FixedClock;

    impl SessionClock for FixedClock {
        fn monotonic_timestamp_ms(&self) -> u64 {
            10
        }
    }

    impl SessionWallClock for FixedClock {
        fn utc_now(&self) -> DateTime<Utc> {
            DateTime::parse_from_rfc3339("2026-07-13T00:00:00Z")
                .expect("fixed timestamp should parse")
                .with_timezone(&Utc)
        }
    }

    #[derive(Default)]
    struct RecordingCompletionStore {
        completions: Mutex<Vec<SessionCompletion>>,
    }

    impl SessionCompletionStore for RecordingCompletionStore {
        type Error = Infallible;

        fn persist_completion(
            &self,
            completion: &SessionCompletion,
        ) -> Result<SessionPersistenceReceipt, Self::Error> {
            self.completions
                .lock()
                .expect("fixture store mutex should not be poisoned")
                .push(completion.clone());
            Ok(SessionPersistenceReceipt { test_id: 1 })
        }
    }

    fn request() -> SessionStartRequest {
        SessionStartRequest {
            mode: "time".to_string(),
            text: None,
            duration: Some(30),
            word_count: None,
            quote_id: None,
            language: Some("en".to_string()),
        }
    }

    #[test]
    fn start_uses_the_id_and_mode_ports() {
        let kernel = SessionKernel::new();
        let mut engine = CoreEngine::new();
        let mut ids = FixedSessionIdSource;
        let mut random_source = FixedRandomSource::default();

        let info = kernel
            .start_session(
                &mut engine,
                &request(),
                "en",
                &mut ids,
                &mut random_source,
                &FixtureModes,
            )
            .expect("fixture session should start");

        assert_eq!(info.session_id, "fixture-session");
        assert_eq!(engine.session_state(), SessionState::Running);
        assert_eq!(random_source.calls, 1);
    }

    #[test]
    fn process_persists_an_authoritative_completion_and_marks_it_persisted() {
        let kernel = SessionKernel::new();
        let mut engine = CoreEngine::new();
        let mut ids = FixedSessionIdSource;
        let mut random_source = FixedRandomSource::default();
        kernel
            .start_session(
                &mut engine,
                &request(),
                "en",
                &mut ids,
                &mut random_source,
                &FixtureModes,
            )
            .expect("fixture session should start");

        let engine_state = Mutex::new(engine);
        let store = RecordingCompletionStore::default();
        let output = kernel
            .process_key(
                &engine_state,
                &FixedClock,
                &FixedClock,
                &store,
                SessionId::from("fixture-session"),
                "a".to_string(),
                "KeyA".to_string(),
            )
            .expect("completion should persist");

        assert_eq!(output.session_state, SessionState::Persisted);
        let completions = store
            .completions
            .lock()
            .expect("fixture store mutex should not be poisoned");
        assert_eq!(completions.len(), 1);
        assert_eq!(
            completions[0].session_id,
            SessionId::from("fixture-session")
        );
        assert_eq!(completions[0].text_length, 1);
        assert_eq!(
            completions[0].completed_at,
            DateTime::parse_from_rfc3339("2026-07-13T00:00:00Z")
                .expect("fixed timestamp should parse")
                .with_timezone(&Utc)
        );
    }

    #[test]
    fn process_rejects_a_stale_session_identity_before_touching_the_engine() {
        let kernel = SessionKernel::new();
        let mut engine = CoreEngine::new();
        let mut ids = FixedSessionIdSource;
        let mut random_source = FixedRandomSource::default();
        kernel
            .start_session(
                &mut engine,
                &request(),
                "en",
                &mut ids,
                &mut random_source,
                &FixtureModes,
            )
            .expect("fixture session should start");
        let engine_state = Mutex::new(engine);
        let store = RecordingCompletionStore::default();

        let result = kernel.process_key(
            &engine_state,
            &FixedClock,
            &FixedClock,
            &store,
            SessionId::from("wrong-session"),
            "a".to_string(),
            "KeyA".to_string(),
        );

        assert!(matches!(
            result,
            Err(SessionProcessError::SessionNotFound(id)) if id.as_str() == "wrong-session"
        ));
        assert!(store
            .completions
            .lock()
            .expect("fixture store mutex should not be poisoned")
            .is_empty());
    }
}
