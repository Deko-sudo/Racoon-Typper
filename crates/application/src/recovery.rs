//! Pure application contracts for durable session recovery.
//!
//! This module deliberately contains no persistence implementation, startup
//! wiring, or runtime recovery. It defines the vocabulary and deterministic
//! policy that later ledger and recovery adapters must implement.

use std::{fmt, str::FromStr};

use chrono::{DateTime, Utc};
use racoon_core::ReplayFrame;
use racoon_domain::{FinalStats, SessionId};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use crate::session::SessionCompletion;

/// Current version of the immutable completion-intent payload.
pub const CURRENT_COMPLETION_INTENT_VERSION: u16 = 1;

/// Current version of the canonical JSON representation.
///
/// This version is independent from the completion-intent schema version. A
/// future canonicalization change can therefore be introduced without
/// pretending that the business payload itself changed.
pub const CURRENT_CANONICALIZATION_VERSION: u16 = 1;

/// Maximum canonical completion-intent envelope size.
pub const MAX_COMPLETION_INTENT_PAYLOAD_BYTES: usize = 8 * 1024 * 1024;

/// Maximum sanitized session descriptor size.
pub const MAX_SESSION_DESCRIPTOR_BYTES: usize = 16 * 1024;

/// Recommended retention period for interrupted session records.
pub const INTERRUPTED_SESSION_RETENTION_DAYS: u64 = 90;

/// Durable lifecycle state. This is deliberately separate from the
/// in-memory [`racoon_domain::SessionState`] because restart semantics differ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DurableSessionState {
    Running,
    AwaitingPersistence,
    FinalizationPending,
    Finalized,
    Aborted,
    Interrupted,
    Quarantined,
}

impl DurableSessionState {
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Finalized | Self::Aborted | Self::Interrupted | Self::Quarantined
        )
    }
}

/// Result of validating a durable state transition.
///
/// The transition policy is deliberately exhaustive over the durable state
/// enum. Adding a new state requires updating the policy before the
/// application can compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurableStateTransitionOutcome {
    Valid,
    Idempotent,
    Invalid {
        from: Option<DurableSessionState>,
        to: DurableSessionState,
    },
    ForbiddenFromTerminal {
        from: DurableSessionState,
        to: DurableSessionState,
    },
}

/// Validates creation or movement between durable session states.
///
/// `None` represents creation of a new durable session. Creation is only
/// valid when the initial state is `Running`. Terminal states are immutable:
/// repeating the same terminal state is idempotent, while every other target
/// is explicitly forbidden.
pub const fn validate_durable_state_transition(
    from: Option<DurableSessionState>,
    to: DurableSessionState,
) -> DurableStateTransitionOutcome {
    match from {
        None => match to {
            DurableSessionState::Running => DurableStateTransitionOutcome::Valid,
            DurableSessionState::AwaitingPersistence
            | DurableSessionState::FinalizationPending
            | DurableSessionState::Finalized
            | DurableSessionState::Aborted
            | DurableSessionState::Interrupted
            | DurableSessionState::Quarantined => {
                DurableStateTransitionOutcome::Invalid { from: None, to }
            }
        },
        Some(DurableSessionState::Running) => match to {
            DurableSessionState::Running => DurableStateTransitionOutcome::Idempotent,
            DurableSessionState::AwaitingPersistence
            | DurableSessionState::Aborted
            | DurableSessionState::Interrupted
            | DurableSessionState::Quarantined => DurableStateTransitionOutcome::Valid,
            DurableSessionState::FinalizationPending | DurableSessionState::Finalized => {
                DurableStateTransitionOutcome::Invalid {
                    from: Some(DurableSessionState::Running),
                    to,
                }
            }
        },
        Some(DurableSessionState::AwaitingPersistence) => match to {
            DurableSessionState::AwaitingPersistence => DurableStateTransitionOutcome::Idempotent,
            DurableSessionState::FinalizationPending
            | DurableSessionState::Aborted
            | DurableSessionState::Quarantined => DurableStateTransitionOutcome::Valid,
            DurableSessionState::Running
            | DurableSessionState::Finalized
            | DurableSessionState::Interrupted => DurableStateTransitionOutcome::Invalid {
                from: Some(DurableSessionState::AwaitingPersistence),
                to,
            },
        },
        Some(DurableSessionState::FinalizationPending) => match to {
            DurableSessionState::FinalizationPending => DurableStateTransitionOutcome::Idempotent,
            DurableSessionState::Finalized
            | DurableSessionState::Aborted
            | DurableSessionState::Quarantined => DurableStateTransitionOutcome::Valid,
            DurableSessionState::Running
            | DurableSessionState::AwaitingPersistence
            | DurableSessionState::Interrupted => DurableStateTransitionOutcome::Invalid {
                from: Some(DurableSessionState::FinalizationPending),
                to,
            },
        },
        Some(DurableSessionState::Finalized) => match to {
            DurableSessionState::Finalized => DurableStateTransitionOutcome::Idempotent,
            DurableSessionState::Running
            | DurableSessionState::AwaitingPersistence
            | DurableSessionState::FinalizationPending
            | DurableSessionState::Aborted
            | DurableSessionState::Interrupted
            | DurableSessionState::Quarantined => {
                DurableStateTransitionOutcome::ForbiddenFromTerminal {
                    from: DurableSessionState::Finalized,
                    to,
                }
            }
        },
        Some(DurableSessionState::Aborted) => match to {
            DurableSessionState::Aborted => DurableStateTransitionOutcome::Idempotent,
            DurableSessionState::Running
            | DurableSessionState::AwaitingPersistence
            | DurableSessionState::FinalizationPending
            | DurableSessionState::Finalized
            | DurableSessionState::Interrupted
            | DurableSessionState::Quarantined => {
                DurableStateTransitionOutcome::ForbiddenFromTerminal {
                    from: DurableSessionState::Aborted,
                    to,
                }
            }
        },
        Some(DurableSessionState::Interrupted) => match to {
            DurableSessionState::Interrupted => DurableStateTransitionOutcome::Idempotent,
            DurableSessionState::Running
            | DurableSessionState::AwaitingPersistence
            | DurableSessionState::FinalizationPending
            | DurableSessionState::Finalized
            | DurableSessionState::Aborted
            | DurableSessionState::Quarantined => {
                DurableStateTransitionOutcome::ForbiddenFromTerminal {
                    from: DurableSessionState::Interrupted,
                    to,
                }
            }
        },
        Some(DurableSessionState::Quarantined) => match to {
            DurableSessionState::Quarantined => DurableStateTransitionOutcome::Idempotent,
            DurableSessionState::Running
            | DurableSessionState::AwaitingPersistence
            | DurableSessionState::FinalizationPending
            | DurableSessionState::Finalized
            | DurableSessionState::Aborted
            | DurableSessionState::Interrupted => {
                DurableStateTransitionOutcome::ForbiddenFromTerminal {
                    from: DurableSessionState::Quarantined,
                    to,
                }
            }
        },
    }
}

/// Version marker included in every canonical completion-intent envelope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CompletionIntentVersion(u16);

impl CompletionIntentVersion {
    pub const fn current() -> Self {
        Self(CURRENT_COMPLETION_INTENT_VERSION)
    }

    pub const fn as_u16(self) -> u16 {
        self.0
    }

    fn try_from_raw(version: u64) -> Result<Self, UnsupportedIntentVersion> {
        if version == u64::from(CURRENT_COMPLETION_INTENT_VERSION) {
            Ok(Self(version as u16))
        } else {
            Err(UnsupportedIntentVersion { version })
        }
    }
}

/// Version marker for the canonical JSON algorithm used by an intent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CanonicalizationVersion(u16);

impl CanonicalizationVersion {
    pub const fn current() -> Self {
        Self(CURRENT_CANONICALIZATION_VERSION)
    }

    pub const fn as_u16(self) -> u16 {
        self.0
    }

    fn try_from_raw(version: u64) -> Result<Self, UnsupportedCanonicalizationVersion> {
        if version == u64::from(CURRENT_CANONICALIZATION_VERSION) {
            Ok(Self(version as u16))
        } else {
            Err(UnsupportedCanonicalizationVersion { version })
        }
    }
}

/// Values from preferences that affect completion-side effects.
///
/// These are intentionally application-owned rather than a data-layer
/// settings type. A future recovery implementation must use this snapshot,
/// not mutable current settings, when deciding daily-goal effects.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompletionPolicySnapshot {
    daily_goal: DailyGoalPolicy,
}

impl CompletionPolicySnapshot {
    pub const fn time(target_minutes: f64) -> Self {
        Self {
            daily_goal: DailyGoalPolicy::Time { target_minutes },
        }
    }

    pub const fn wpm(target_wpm: f64) -> Self {
        Self {
            daily_goal: DailyGoalPolicy::Wpm { target_wpm },
        }
    }

    pub const fn accuracy(target_accuracy: f64) -> Self {
        Self {
            daily_goal: DailyGoalPolicy::Accuracy { target_accuracy },
        }
    }

    pub const fn daily_goal(&self) -> &DailyGoalPolicy {
        &self.daily_goal
    }
}

/// The completion-affecting daily-goal setting captured at completion time.
///
/// The existing settings model uses one legacy numeric field for both the
/// time target and the WPM target. This tagged representation preserves the
/// current behavior while making the unit of every target explicit in the
/// immutable intent.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", deny_unknown_fields)]
pub enum DailyGoalPolicy {
    Time { target_minutes: f64 },
    Wpm { target_wpm: f64 },
    Accuracy { target_accuracy: f64 },
}

/// Bounded metadata recorded when the application accepts a session.
///
/// The descriptor is configuration metadata, not a typed-input journal.
/// Construction validates a bounded, recursively sanitized metadata shape;
/// raw custom text, typed input, replay data, and equivalent nested fields
/// are rejected before a durable ledger adapter can receive this value.
#[derive(Clone, PartialEq, Serialize)]
pub struct StartedSession {
    session_id: SessionId,
    mode_type: String,
    mode_descriptor: Value,
    language: String,
    started_at: DateTime<Utc>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredStartedSession {
    session_id: SessionId,
    mode_type: String,
    mode_descriptor: Value,
    language: String,
    started_at: DateTime<Utc>,
}

impl<'de> Deserialize<'de> for StartedSession {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let stored = StoredStartedSession::deserialize(deserializer)?;
        Self::new(
            stored.session_id,
            stored.mode_type,
            stored.mode_descriptor,
            stored.language,
            stored.started_at,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl StartedSession {
    pub fn new(
        session_id: SessionId,
        mode_type: impl Into<String>,
        mode_descriptor: Value,
        language: impl Into<String>,
        started_at: DateTime<Utc>,
    ) -> Result<Self, StartedSessionError> {
        validate_sanitized_session_descriptor(&mode_descriptor)?;

        Ok(Self {
            session_id,
            mode_type: mode_type.into(),
            mode_descriptor,
            language: language.into(),
            started_at,
        })
    }

    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    pub fn mode_type(&self) -> &str {
        &self.mode_type
    }

    pub fn mode_descriptor(&self) -> &Value {
        &self.mode_descriptor
    }

    pub fn language(&self) -> &str {
        &self.language
    }

    pub const fn started_at(&self) -> &DateTime<Utc> {
        &self.started_at
    }
}

/// Maximum permitted nesting depth for a durable session descriptor.
pub const MAX_SESSION_DESCRIPTOR_DEPTH: usize = 8;
/// Maximum byte length of one descriptor object key.
pub const MAX_SESSION_DESCRIPTOR_KEY_BYTES: usize = 64;
/// Maximum byte length of a scalar descriptor string.
pub const MAX_SESSION_DESCRIPTOR_STRING_BYTES: usize = 128;

/// Validates metadata that is safe to store in the durable session ledger.
///
/// The root must be an object. Objects and arrays may nest to a maximum depth
/// of eight. Keys are limited to 64 bytes and normalized by lowercasing and
/// removing non-alphanumeric separators before policy comparison. Known
/// content-bearing fields are forbidden at every depth. Strings are limited
/// to 128 bytes and may occur only at explicitly metadata-only identifier
/// keys; numbers, booleans, and null are allowed. The whole canonical JSON
/// representation remains bounded by [`MAX_SESSION_DESCRIPTOR_BYTES`].
///
/// This policy intentionally has no payload-bearing error details: callers
/// receive only a classification, never rejected descriptor content.
pub fn validate_sanitized_session_descriptor(
    descriptor: &Value,
) -> Result<(), StartedSessionError> {
    let descriptor_size = canonical_json_bytes(descriptor)
        .map_err(|_| StartedSessionError::InvalidDescriptor)?
        .len();
    if descriptor_size > MAX_SESSION_DESCRIPTOR_BYTES {
        return Err(StartedSessionError::DescriptorTooLarge {
            actual: descriptor_size,
            maximum: MAX_SESSION_DESCRIPTOR_BYTES,
        });
    }
    if !matches!(descriptor, Value::Object(_)) {
        return Err(StartedSessionError::InvalidDescriptor);
    }
    validate_descriptor_value(descriptor, 0, None)
}

fn validate_descriptor_value(
    value: &Value,
    depth: usize,
    parent_key: Option<&str>,
) -> Result<(), StartedSessionError> {
    if depth > MAX_SESSION_DESCRIPTOR_DEPTH {
        return Err(StartedSessionError::DescriptorTooDeep {
            maximum: MAX_SESSION_DESCRIPTOR_DEPTH,
        });
    }
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) => Ok(()),
        Value::String(value) => {
            if value.len() > MAX_SESSION_DESCRIPTOR_STRING_BYTES {
                return Err(StartedSessionError::DescriptorStringTooLong {
                    maximum: MAX_SESSION_DESCRIPTOR_STRING_BYTES,
                });
            }
            if parent_key.is_some_and(is_safe_descriptor_string_key) {
                Ok(())
            } else {
                Err(StartedSessionError::UnsafeDescriptorValue)
            }
        }
        Value::Array(values) => {
            for value in values {
                validate_descriptor_value(value, depth + 1, None)?;
            }
            Ok(())
        }
        Value::Object(values) => {
            for (key, value) in values {
                if key.len() > MAX_SESSION_DESCRIPTOR_KEY_BYTES {
                    return Err(StartedSessionError::DescriptorKeyTooLong {
                        maximum: MAX_SESSION_DESCRIPTOR_KEY_BYTES,
                    });
                }
                let normalized_key = normalize_descriptor_key(key);
                if normalized_key.is_empty() {
                    return Err(StartedSessionError::InvalidDescriptor);
                }
                if is_forbidden_descriptor_key(&normalized_key) {
                    return Err(StartedSessionError::SensitiveDescriptorField);
                }
                validate_descriptor_value(value, depth + 1, Some(&normalized_key))?;
            }
            Ok(())
        }
    }
}

fn normalize_descriptor_key(key: &str) -> String {
    key.chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .map(|character| character.to_ascii_lowercase())
        .collect()
}

fn is_forbidden_descriptor_key(key: &str) -> bool {
    matches!(
        key,
        "text"
            | "customtext"
            | "content"
            | "typedtext"
            | "typedchars"
            | "input"
            | "replay"
            | "replayframes"
            | "frames"
            | "keys"
            | "keystrokes"
            | "expectedtext"
            | "quotetext"
            | "lessontext"
    )
}

fn is_safe_descriptor_string_key(key: &str) -> bool {
    matches!(
        key,
        "kind"
            | "language"
            | "mode"
            | "modetype"
            | "sourceid"
            | "quoteid"
            | "lessonid"
            | "moduleid"
            | "configversion"
            | "difficulty"
    )
}

impl fmt::Debug for StartedSession {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("StartedSession")
            .field("session_id", &self.session_id)
            .field("mode_type", &self.mode_type)
            .field("mode_descriptor", &"<redacted>")
            .field("language", &self.language)
            .field("started_at", &self.started_at)
            .finish()
    }
}

/// Immutable, versioned completion payload used by future finalization.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompletionIntentPayload {
    session_id: SessionId,
    completed_at: DateTime<Utc>,
    final_stats: FinalStats,
    mode_type: String,
    mode_config: Value,
    language: String,
    text_length: usize,
    replay_frames: Vec<ReplayFrame>,
    lesson_id: Option<String>,
    completion_policy: CompletionPolicySnapshot,
}

impl CompletionIntentPayload {
    pub fn from_completion(
        completion: &SessionCompletion,
        completion_policy: CompletionPolicySnapshot,
    ) -> Self {
        Self {
            session_id: completion.session_id.clone(),
            completed_at: completion.completed_at,
            final_stats: completion.final_stats.clone(),
            mode_type: completion.mode_type.clone(),
            mode_config: completion.mode_config.clone(),
            language: completion.language.clone(),
            text_length: completion.text_length,
            replay_frames: completion.replay_frames.clone(),
            lesson_id: completion.lesson_id.clone(),
            completion_policy,
        }
    }

    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    pub const fn completed_at(&self) -> &DateTime<Utc> {
        &self.completed_at
    }

    pub const fn final_stats(&self) -> &FinalStats {
        &self.final_stats
    }

    pub fn mode_type(&self) -> &str {
        &self.mode_type
    }

    pub fn mode_config(&self) -> &Value {
        &self.mode_config
    }

    pub fn language(&self) -> &str {
        &self.language
    }

    pub const fn text_length(&self) -> usize {
        self.text_length
    }

    pub fn replay_frames(&self) -> &[ReplayFrame] {
        &self.replay_frames
    }

    pub fn lesson_id(&self) -> Option<&str> {
        self.lesson_id.as_deref()
    }

    pub const fn completion_policy(&self) -> &CompletionPolicySnapshot {
        &self.completion_policy
    }

    fn validate_consistency(&self) -> Result<(), CompletionIntentError> {
        let mode_config = self.mode_config.as_object();
        let configured_language = mode_config
            .and_then(|config| config.get("language"))
            .map(|value| {
                value
                    .as_str()
                    .ok_or(CompletionIntentError::InconsistentMetadata)
            })
            .transpose()?;
        if configured_language.is_some_and(|language| language != self.language) {
            return Err(CompletionIntentError::InconsistentMetadata);
        }

        let configured_lesson_id = mode_config
            .and_then(|config| config.get("lesson_id"))
            .map(|value| {
                value
                    .as_str()
                    .ok_or(CompletionIntentError::InconsistentMetadata)
            })
            .transpose()?;

        match self.mode_type.as_str() {
            "lesson" => {
                let lesson_id = self
                    .lesson_id
                    .as_deref()
                    .filter(|lesson_id| !lesson_id.is_empty())
                    .ok_or(CompletionIntentError::InconsistentMetadata)?;
                if configured_lesson_id != Some(lesson_id) {
                    return Err(CompletionIntentError::InconsistentMetadata);
                }
            }
            _ => {
                if self.lesson_id.is_some() || configured_lesson_id.is_some() {
                    return Err(CompletionIntentError::InconsistentMetadata);
                }
            }
        }

        Ok(())
    }
}

impl fmt::Debug for CompletionIntentPayload {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CompletionIntentPayload")
            .field("session_id", &self.session_id)
            .field("completed_at", &self.completed_at)
            .field("mode_type", &self.mode_type)
            .field("mode_config", &"<redacted>")
            .field("language", &self.language)
            .field("text_length", &self.text_length)
            .field("replay_frame_count", &self.replay_frames.len())
            .field("lesson_id_present", &self.lesson_id.is_some())
            .field("completion_policy", &self.completion_policy)
            .finish()
    }
}

/// Stable lowercase hexadecimal SHA-256 identity for a canonical intent.
#[derive(Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct CompletionIntentFingerprint(String);

impl CompletionIntentFingerprint {
    pub fn from_canonical_bytes(bytes: &[u8]) -> Self {
        let digest = Sha256::digest(bytes);
        let mut hex = String::with_capacity(digest.len() * 2);
        for byte in digest {
            hex.push(HEX_DIGITS[(byte >> 4) as usize] as char);
            hex.push(HEX_DIGITS[(byte & 0x0f) as usize] as char);
        }
        Self(hex)
    }

    pub fn try_from_hex(value: impl AsRef<str>) -> Result<Self, InvalidFingerprint> {
        let value = value.as_ref();
        if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(InvalidFingerprint);
        }
        Ok(Self(value.to_ascii_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for CompletionIntentFingerprint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("CompletionIntentFingerprint")
            .field(&self.0)
            .finish()
    }
}

impl fmt::Display for CompletionIntentFingerprint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for CompletionIntentFingerprint {
    type Err = InvalidFingerprint;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_from_hex(value)
    }
}

impl<'de> Deserialize<'de> for CompletionIntentFingerprint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::try_from_hex(value).map_err(|_| serde::de::Error::custom("invalid fingerprint"))
    }
}

/// Immutable completion intent, including its canonical bytes and identity.
#[derive(Clone)]
pub struct CompletionIntent {
    canonicalization_version: CanonicalizationVersion,
    payload_version: CompletionIntentVersion,
    payload: CompletionIntentPayload,
    canonical_payload: Vec<u8>,
    fingerprint: CompletionIntentFingerprint,
}

impl PartialEq for CompletionIntent {
    fn eq(&self, other: &Self) -> bool {
        self.canonicalization_version == other.canonicalization_version
            && self.payload_version == other.payload_version
            && self.canonical_payload == other.canonical_payload
            && self.fingerprint == other.fingerprint
    }
}

impl CompletionIntent {
    pub fn from_completion(
        completion: &SessionCompletion,
        completion_policy: CompletionPolicySnapshot,
    ) -> Result<Self, CompletionIntentError> {
        Self::from_payload(CompletionIntentPayload::from_completion(
            completion,
            completion_policy,
        ))
    }

    pub fn from_payload(payload: CompletionIntentPayload) -> Result<Self, CompletionIntentError> {
        payload.validate_consistency()?;
        if completion_payload_has_non_finite_number(&payload) {
            return Err(CompletionIntentError::NonFiniteNumber);
        }
        let canonicalization_version = CanonicalizationVersion::current();
        let payload_version = CompletionIntentVersion::current();
        let payload_value =
            serde_json::to_value(&payload).map_err(|_| CompletionIntentError::Serialization)?;
        let canonical_payload = canonical_envelope_bytes(
            canonicalization_version.as_u16(),
            payload_version.as_u16(),
            payload_value,
        )
        .map_err(|error| match error {
            CanonicalJsonError::Serialization => CompletionIntentError::Serialization,
            CanonicalJsonError::NonFiniteNumber => CompletionIntentError::NonFiniteNumber,
        })?;
        if canonical_payload.len() > MAX_COMPLETION_INTENT_PAYLOAD_BYTES {
            return Err(CompletionIntentError::PayloadTooLarge {
                actual: canonical_payload.len(),
                maximum: MAX_COMPLETION_INTENT_PAYLOAD_BYTES,
            });
        }
        let fingerprint = CompletionIntentFingerprint::from_canonical_bytes(&canonical_payload);
        Ok(Self {
            canonicalization_version,
            payload_version,
            payload,
            canonical_payload,
            fingerprint,
        })
    }

    /// Reconstructs and validates a stored canonical envelope without exposing
    /// raw payload details in the error vocabulary.
    pub fn from_stored_payload(
        canonical_payload: &[u8],
        expected_fingerprint: &CompletionIntentFingerprint,
    ) -> Result<Self, CompletionIntentLoadError> {
        if canonical_payload.len() > MAX_COMPLETION_INTENT_PAYLOAD_BYTES {
            return Err(CompletionIntentLoadError::PayloadTooLarge);
        }

        let envelope: Value = serde_json::from_slice(canonical_payload)
            .map_err(|_| CompletionIntentLoadError::CorruptCanonicalPayload)?;
        let object = envelope
            .as_object()
            .ok_or(CompletionIntentLoadError::CorruptCanonicalPayload)?;
        let raw_canonicalization_version = object
            .get("canonicalization_version")
            .and_then(Value::as_u64)
            .ok_or(CompletionIntentLoadError::CorruptCanonicalPayload)?;
        let canonicalization_version =
            CanonicalizationVersion::try_from_raw(raw_canonicalization_version)
                .map_err(CompletionIntentLoadError::UnsupportedCanonicalizationVersion)?;
        let raw_version = object
            .get("payload_version")
            .and_then(Value::as_u64)
            .ok_or(CompletionIntentLoadError::CorruptCanonicalPayload)?;
        let payload_version = CompletionIntentVersion::try_from_raw(raw_version)
            .map_err(CompletionIntentLoadError::UnsupportedVersion)?;
        let payload_value = object
            .get("payload")
            .cloned()
            .ok_or(CompletionIntentLoadError::CorruptCanonicalPayload)?;
        let canonical = canonical_envelope_bytes(
            canonicalization_version.as_u16(),
            payload_version.as_u16(),
            payload_value.clone(),
        )
        .map_err(|_| CompletionIntentLoadError::CorruptCanonicalPayload)?;
        if canonical != canonical_payload {
            return Err(CompletionIntentLoadError::CorruptCanonicalPayload);
        }
        let payload: CompletionIntentPayload = serde_json::from_value(payload_value)
            .map_err(|_| CompletionIntentLoadError::CorruptCanonicalPayload)?;
        payload
            .validate_consistency()
            .map_err(|_| CompletionIntentLoadError::CorruptCanonicalPayload)?;
        let typed_payload = serde_json::to_value(&payload)
            .map_err(|_| CompletionIntentLoadError::CorruptCanonicalPayload)?;
        let typed_canonical = canonical_envelope_bytes(
            canonicalization_version.as_u16(),
            payload_version.as_u16(),
            typed_payload,
        )
        .map_err(|_| CompletionIntentLoadError::CorruptCanonicalPayload)?;
        if typed_canonical != canonical {
            return Err(CompletionIntentLoadError::CorruptCanonicalPayload);
        }
        let fingerprint = CompletionIntentFingerprint::from_canonical_bytes(&canonical);
        if &fingerprint != expected_fingerprint {
            return Err(CompletionIntentLoadError::FingerprintMismatch);
        }
        Ok(Self {
            canonicalization_version,
            payload_version,
            payload,
            canonical_payload: canonical,
            fingerprint,
        })
    }

    pub const fn payload_version(&self) -> CompletionIntentVersion {
        self.payload_version
    }

    pub const fn canonicalization_version(&self) -> CanonicalizationVersion {
        self.canonicalization_version
    }

    pub const fn payload(&self) -> &CompletionIntentPayload {
        &self.payload
    }

    pub fn canonical_payload(&self) -> &[u8] {
        &self.canonical_payload
    }

    pub const fn fingerprint(&self) -> &CompletionIntentFingerprint {
        &self.fingerprint
    }
}

fn completion_payload_has_non_finite_number(payload: &CompletionIntentPayload) -> bool {
    !payload.final_stats.wpm.is_finite()
        || !payload.final_stats.raw_wpm.is_finite()
        || !payload.final_stats.accuracy.is_finite()
        || !payload.final_stats.raw_accuracy.is_finite()
        || payload
            .final_stats
            .consistency
            .is_some_and(|value| !value.is_finite())
        || match payload.completion_policy.daily_goal() {
            DailyGoalPolicy::Time { target_minutes } => !target_minutes.is_finite(),
            DailyGoalPolicy::Wpm { target_wpm } => !target_wpm.is_finite(),
            DailyGoalPolicy::Accuracy { target_accuracy } => !target_accuracy.is_finite(),
        }
}

impl fmt::Debug for CompletionIntent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CompletionIntent")
            .field("session_id", self.payload.session_id())
            .field("canonicalization_version", &self.canonicalization_version)
            .field("payload_version", &self.payload_version)
            .field("payload_bytes", &self.canonical_payload.len())
            .field("fingerprint", &self.fingerprint)
            .finish()
    }
}

/// How an incoming intent relates to an existing intent for a session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionIntentComparison {
    IdempotentMatch,
    Conflict(CompletionIntentConflict),
}

/// Redacted conflict summary suitable for diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionIntentConflict {
    pub existing_session_id: SessionId,
    pub incoming_session_id: SessionId,
    pub existing_fingerprint: CompletionIntentFingerprint,
    pub incoming_fingerprint: CompletionIntentFingerprint,
}

pub fn compare_completion_intents(
    existing: &CompletionIntent,
    incoming: &CompletionIntent,
) -> CompletionIntentComparison {
    if existing.payload.session_id() == incoming.payload.session_id()
        && existing.fingerprint() == incoming.fingerprint()
    {
        CompletionIntentComparison::IdempotentMatch
    } else {
        CompletionIntentComparison::Conflict(CompletionIntentConflict {
            existing_session_id: existing.payload.session_id().clone(),
            incoming_session_id: incoming.payload.session_id().clone(),
            existing_fingerprint: existing.fingerprint().clone(),
            incoming_fingerprint: incoming.fingerprint().clone(),
        })
    }
}

/// Result of checking the immutable intent supplied to finalization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FinalizationIntentValidation {
    Match,
    Conflict(FinalizationConflict),
}

/// Redacted finalization conflict summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinalizationConflict {
    pub session_id: SessionId,
    pub expected_fingerprint: CompletionIntentFingerprint,
    pub stored_fingerprint: CompletionIntentFingerprint,
}

/// Validates the finalizer invariant that only the stored immutable intent
/// for the requested session and its expected fingerprint may be finalized.
pub fn validate_finalization_intent(
    session_id: &SessionId,
    expected_fingerprint: &CompletionIntentFingerprint,
    stored_intent: &CompletionIntent,
) -> FinalizationIntentValidation {
    if stored_intent.payload.session_id() == session_id
        && stored_intent.fingerprint() == expected_fingerprint
    {
        FinalizationIntentValidation::Match
    } else {
        FinalizationIntentValidation::Conflict(FinalizationConflict {
            session_id: session_id.clone(),
            expected_fingerprint: expected_fingerprint.clone(),
            stored_fingerprint: stored_intent.fingerprint().clone(),
        })
    }
}

/// Metadata returned while listing candidates for recovery classification.
///
/// This type intentionally contains no completion payload, replay frames, or
/// mode configuration. A full validated intent is loaded separately through
/// `SessionRecoveryLedger::load_completion_intent` when finalization requires
/// it.
#[derive(Clone, PartialEq)]
pub enum CompletionIntentMetadata {
    Missing,
    Present {
        session_id: SessionId,
        canonicalization_version: CanonicalizationVersion,
        payload_version: CompletionIntentVersion,
        fingerprint: CompletionIntentFingerprint,
    },
    UnsupportedCanonicalizationVersion {
        version: u64,
    },
    UnsupportedVersion {
        version: u64,
    },
    Corrupt,
}

/// A raw field read from untrusted durable intent-header storage.
///
/// `Missing` represents a SQL `NULL` or absent required column. `Invalid`
/// represents a value whose storage representation cannot be read as the
/// expected application value (for example, a text value where an integer is
/// required). Storage adapters describe the raw condition with this type; the
/// application owns the resulting corruption classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoredHeaderValue<T> {
    Missing,
    Value(T),
    Invalid,
}

/// Header-only data for one optional stored completion intent.
///
/// This infrastructure-neutral input deliberately excludes canonical payload
/// bytes, replay frames, and mode configuration. It lets a future ledger
/// adapter classify a candidate from indexed header columns without claiming
/// that the full payload is valid. Full payload validation remains the job of
/// [`CompletionIntent::from_stored_payload`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoredCompletionIntentHeader<'a> {
    /// No completion-intent row exists for the session.
    Missing,
    /// A completion-intent row exists. Every required field must be present
    /// and valid before it can produce supported metadata.
    Present {
        session_id: StoredHeaderValue<&'a str>,
        canonicalization_version: StoredHeaderValue<i64>,
        payload_version: StoredHeaderValue<i64>,
        fingerprint: StoredHeaderValue<&'a str>,
    },
}

impl CompletionIntentMetadata {
    pub fn present(intent: &CompletionIntent) -> Self {
        Self::Present {
            session_id: intent.payload.session_id().clone(),
            canonicalization_version: intent.canonicalization_version,
            payload_version: intent.payload_version,
            fingerprint: intent.fingerprint.clone(),
        }
    }

    /// Classifies an untrusted stored intent header without loading its
    /// canonical payload.
    ///
    /// The deterministic precedence is: malformed header, unsupported
    /// canonicalization version, unsupported payload version, then supported
    /// metadata. A syntactically supported header proves only that its header
    /// values are usable for candidate classification; callers must still use
    /// [`CompletionIntent::from_stored_payload`] before any completion effect.
    pub fn from_stored_header(header: StoredCompletionIntentHeader<'_>) -> Self {
        let StoredCompletionIntentHeader::Present {
            session_id,
            canonicalization_version,
            payload_version,
            fingerprint,
        } = header
        else {
            return Self::Missing;
        };

        let (
            StoredHeaderValue::Value(raw_session_id),
            StoredHeaderValue::Value(raw_canonicalization_version),
            StoredHeaderValue::Value(raw_payload_version),
            StoredHeaderValue::Value(raw_fingerprint),
        ) = (
            session_id,
            canonicalization_version,
            payload_version,
            fingerprint,
        )
        else {
            return Self::Corrupt;
        };

        let Ok(session_id) = SessionId::parse(raw_session_id) else {
            return Self::Corrupt;
        };
        let Ok(raw_canonicalization_version) = u64::try_from(raw_canonicalization_version) else {
            return Self::Corrupt;
        };
        let Ok(raw_payload_version) = u64::try_from(raw_payload_version) else {
            return Self::Corrupt;
        };

        // `try_from_hex` normalizes normal caller input. Persisted canonical
        // headers are stricter: the stored form itself must already be the
        // lowercase digest, so an uppercase variant is corrupt rather than
        // silently normalized.
        let Ok(fingerprint) = CompletionIntentFingerprint::try_from_hex(raw_fingerprint) else {
            return Self::Corrupt;
        };
        if fingerprint.as_str() != raw_fingerprint {
            return Self::Corrupt;
        }

        let canonicalization_version =
            match CanonicalizationVersion::try_from_raw(raw_canonicalization_version) {
                Ok(version) => version,
                Err(_) => {
                    return Self::UnsupportedCanonicalizationVersion {
                        version: raw_canonicalization_version,
                    }
                }
            };
        let payload_version = match CompletionIntentVersion::try_from_raw(raw_payload_version) {
            Ok(version) => version,
            Err(_) => {
                return Self::UnsupportedVersion {
                    version: raw_payload_version,
                }
            }
        };

        Self::Present {
            session_id,
            canonicalization_version,
            payload_version,
            fingerprint,
        }
    }

    pub fn session_id(&self) -> Option<&SessionId> {
        match self {
            Self::Present { session_id, .. } => Some(session_id),
            Self::Missing
            | Self::UnsupportedCanonicalizationVersion { .. }
            | Self::UnsupportedVersion { .. }
            | Self::Corrupt => None,
        }
    }

    pub const fn payload_version(&self) -> Option<CompletionIntentVersion> {
        match self {
            Self::Present {
                payload_version, ..
            } => Some(*payload_version),
            Self::Missing
            | Self::UnsupportedCanonicalizationVersion { .. }
            | Self::UnsupportedVersion { .. }
            | Self::Corrupt => None,
        }
    }

    pub fn fingerprint(&self) -> Option<&CompletionIntentFingerprint> {
        match self {
            Self::Present { fingerprint, .. } => Some(fingerprint),
            Self::Missing
            | Self::UnsupportedCanonicalizationVersion { .. }
            | Self::UnsupportedVersion { .. }
            | Self::Corrupt => None,
        }
    }
}

impl fmt::Debug for CompletionIntentMetadata {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => formatter.write_str("Missing"),
            Self::Present {
                session_id,
                canonicalization_version,
                payload_version,
                fingerprint,
            } => formatter
                .debug_struct("Present")
                .field("session_id", session_id)
                .field("canonicalization_version", canonicalization_version)
                .field("payload_version", payload_version)
                .field("fingerprint", fingerprint)
                .finish(),
            Self::UnsupportedCanonicalizationVersion { version } => formatter
                .debug_struct("UnsupportedCanonicalizationVersion")
                .field("version", version)
                .finish(),
            Self::UnsupportedVersion { version } => formatter
                .debug_struct("UnsupportedVersion")
                .field("version", version)
                .finish(),
            Self::Corrupt => formatter.write_str("Corrupt"),
        }
    }
}

/// Candidate returned by a future ledger adapter.
#[derive(Debug, Clone, PartialEq)]
pub struct RecoveryCandidate {
    session_id: SessionId,
    state: DurableSessionState,
    intent_metadata: CompletionIntentMetadata,
}

impl RecoveryCandidate {
    pub fn new(
        session_id: SessionId,
        state: DurableSessionState,
        intent_metadata: CompletionIntentMetadata,
    ) -> Self {
        Self {
            session_id,
            state,
            intent_metadata,
        }
    }

    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    pub const fn state(&self) -> DurableSessionState {
        self.state
    }

    pub const fn intent_metadata(&self) -> &CompletionIntentMetadata {
        &self.intent_metadata
    }
}

/// Pure decision returned by recovery classification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryDecision {
    MarkInterrupted {
        session_id: SessionId,
        reason: InterruptionReason,
    },
    EligibleForFinalization {
        session_id: SessionId,
        fingerprint: CompletionIntentFingerprint,
    },
    NoOp {
        session_id: SessionId,
        state: DurableSessionState,
    },
    Quarantine {
        session_id: SessionId,
        reason: QuarantineReason,
    },
}

/// Result of validating a separately loaded immutable intent against a
/// metadata-only recovery candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryIntentValidation {
    Valid,
    Quarantine(QuarantineReason),
}

/// Validates the full intent loaded after candidate listing.
pub fn validate_recovery_candidate_intent(
    candidate: &RecoveryCandidate,
    intent: &CompletionIntent,
) -> RecoveryIntentValidation {
    match candidate.intent_metadata() {
        CompletionIntentMetadata::Missing => {
            RecoveryIntentValidation::Quarantine(QuarantineReason::MissingCompletionIntent)
        }
        CompletionIntentMetadata::UnsupportedCanonicalizationVersion { .. } => {
            RecoveryIntentValidation::Quarantine(
                QuarantineReason::UnsupportedCanonicalizationVersion,
            )
        }
        CompletionIntentMetadata::UnsupportedVersion { .. } => {
            RecoveryIntentValidation::Quarantine(QuarantineReason::UnsupportedIntentVersion)
        }
        CompletionIntentMetadata::Corrupt => {
            RecoveryIntentValidation::Quarantine(QuarantineReason::CorruptCompletionPayload)
        }
        CompletionIntentMetadata::Present {
            session_id,
            canonicalization_version,
            payload_version,
            fingerprint,
        } => {
            if session_id != candidate.session_id()
                || intent.payload.session_id() != candidate.session_id()
                || intent.canonicalization_version() != *canonicalization_version
                || intent.payload_version() != *payload_version
            {
                RecoveryIntentValidation::Quarantine(QuarantineReason::InconsistentDurableMetadata)
            } else if intent.fingerprint() != fingerprint {
                RecoveryIntentValidation::Quarantine(QuarantineReason::FingerprintMismatch)
            } else {
                RecoveryIntentValidation::Valid
            }
        }
    }
}

pub fn classify_recovery_candidate(candidate: &RecoveryCandidate) -> RecoveryDecision {
    let session_id = candidate.session_id.clone();
    match candidate.state {
        DurableSessionState::Running => match candidate.intent_metadata() {
            CompletionIntentMetadata::Missing => RecoveryDecision::MarkInterrupted {
                session_id,
                reason: InterruptionReason::ProcessRestart,
            },
            CompletionIntentMetadata::Present { .. }
            | CompletionIntentMetadata::UnsupportedCanonicalizationVersion { .. }
            | CompletionIntentMetadata::UnsupportedVersion { .. }
            | CompletionIntentMetadata::Corrupt => RecoveryDecision::Quarantine {
                session_id,
                reason: QuarantineReason::InvalidStateRecord,
            },
        },
        DurableSessionState::AwaitingPersistence | DurableSessionState::FinalizationPending => {
            match candidate.intent_metadata() {
                CompletionIntentMetadata::Present {
                    session_id: intent_session_id,
                    fingerprint,
                    ..
                } if intent_session_id == candidate.session_id() => {
                    RecoveryDecision::EligibleForFinalization {
                        session_id,
                        fingerprint: fingerprint.clone(),
                    }
                }
                CompletionIntentMetadata::Present { .. } => RecoveryDecision::Quarantine {
                    session_id,
                    reason: QuarantineReason::InconsistentDurableMetadata,
                },
                CompletionIntentMetadata::UnsupportedCanonicalizationVersion { .. } => {
                    RecoveryDecision::Quarantine {
                        session_id,
                        reason: QuarantineReason::UnsupportedCanonicalizationVersion,
                    }
                }
                CompletionIntentMetadata::Missing => RecoveryDecision::Quarantine {
                    session_id,
                    reason: QuarantineReason::MissingCompletionIntent,
                },
                CompletionIntentMetadata::UnsupportedVersion { .. } => {
                    RecoveryDecision::Quarantine {
                        session_id,
                        reason: QuarantineReason::UnsupportedIntentVersion,
                    }
                }
                CompletionIntentMetadata::Corrupt => RecoveryDecision::Quarantine {
                    session_id,
                    reason: QuarantineReason::CorruptCompletionPayload,
                },
            }
        }
        state => RecoveryDecision::NoOp { session_id, state },
    }
}

/// Reasons for classifying an active session as interrupted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterruptionReason {
    ProcessRestart,
}

/// Reasons that a record must not be finalized automatically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuarantineReason {
    UnsupportedCanonicalizationVersion,
    UnsupportedIntentVersion,
    CorruptCompletionPayload,
    MissingCompletionIntent,
    ConflictingCompletionIntent,
    InvalidStateRecord,
    FingerprintMismatch,
    InconsistentDurableMetadata,
}

impl fmt::Display for QuarantineReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::UnsupportedCanonicalizationVersion => "unsupported canonicalization version",
            Self::UnsupportedIntentVersion => "unsupported completion intent version",
            Self::CorruptCompletionPayload => "corrupt completion payload",
            Self::MissingCompletionIntent => "missing completion intent",
            Self::ConflictingCompletionIntent => "conflicting completion intent",
            Self::InvalidStateRecord => "invalid durable session state",
            Self::FingerprintMismatch => "completion fingerprint mismatch",
            Self::InconsistentDurableMetadata => "inconsistent durable session metadata",
        })
    }
}

/// Readiness state for a future startup recovery gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryReadiness {
    NotStarted,
    Recovering,
    Ready,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryReadinessEvent {
    Begin,
    Complete,
    Block,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryReadinessTransitionError {
    InvalidTransition {
        state: RecoveryReadiness,
        event: RecoveryReadinessEvent,
    },
}

impl RecoveryReadiness {
    pub const fn transition(
        self,
        event: RecoveryReadinessEvent,
    ) -> Result<Self, RecoveryReadinessTransitionError> {
        match (self, event) {
            (Self::NotStarted, RecoveryReadinessEvent::Begin) => Ok(Self::Recovering),
            (Self::NotStarted, RecoveryReadinessEvent::Block)
            | (Self::Recovering, RecoveryReadinessEvent::Block) => Ok(Self::Blocked),
            (Self::Recovering, RecoveryReadinessEvent::Complete) => Ok(Self::Ready),
            _ => Err(RecoveryReadinessTransitionError::InvalidTransition { state: self, event }),
        }
    }
}

/// Business outcomes shared by ledger mutations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LedgerMutationOutcome {
    Created,
    AlreadyExistsIdentical,
    Conflicting(LedgerConflict),
    NotFound,
    Quarantined(QuarantineReason),
}

/// Business outcomes for the explicit `awaiting_persistence` to
/// `finalization_pending` claim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FinalizationClaimOutcome {
    /// The valid intent was claimed and the durable state must now be
    /// `FinalizationPending`.
    Claimed,
    /// A retry found the same valid intent already in `FinalizationPending`.
    AlreadyPending,
    /// The durable session is already finalized; no work is required.
    AlreadyFinalized,
    NotFound,
    Conflict(FinalizationConflict),
    Quarantined(QuarantineReason),
    RejectedTerminal {
        state: DurableSessionState,
    },
}

/// Durable state of a finalization ledger record.
///
/// This is intentionally narrower than [`DurableSessionState`]. It records
/// only the future finalizer's durable claim and commit marker; it does not
/// model individual completion effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinalizationLedgerState {
    Pending,
    Committed,
    Quarantined,
}

impl FinalizationLedgerState {
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Committed | Self::Quarantined)
    }

    /// Parses the stable storage spelling without accepting aliases.
    pub fn from_storage_name(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "committed" => Some(Self::Committed),
            "quarantined" => Some(Self::Quarantined),
            _ => None,
        }
    }

    pub const fn storage_name(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Committed => "committed",
            Self::Quarantined => "quarantined",
        }
    }
}

/// Bounded reasons for an otherwise valid finalization record to stop.
///
/// These are durable classifications, never raw storage or payload errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinalizationQuarantineReason {
    MissingCompletionIntent,
    CorruptDurableMetadata,
    FingerprintMismatch,
    InvalidFinalizationState,
}

impl FinalizationQuarantineReason {
    pub fn from_storage_name(value: &str) -> Option<Self> {
        match value {
            "missing_completion_intent" => Some(Self::MissingCompletionIntent),
            "corrupt_durable_metadata" => Some(Self::CorruptDurableMetadata),
            "fingerprint_mismatch" => Some(Self::FingerprintMismatch),
            "invalid_finalization_state" => Some(Self::InvalidFinalizationState),
            _ => None,
        }
    }

    pub const fn storage_name(self) -> &'static str {
        match self {
            Self::MissingCompletionIntent => "missing_completion_intent",
            Self::CorruptDurableMetadata => "corrupt_durable_metadata",
            Self::FingerprintMismatch => "fingerprint_mismatch",
            Self::InvalidFinalizationState => "invalid_finalization_state",
        }
    }
}

/// Metadata-only durable finalization record for diagnostics and future
/// recovery. It deliberately excludes completion payload and effect details.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinalizationRecord {
    session_id: SessionId,
    fingerprint: CompletionIntentFingerprint,
    state: FinalizationLedgerState,
    claimed_at: DateTime<Utc>,
    committed_at: Option<DateTime<Utc>>,
    quarantine_reason: Option<FinalizationQuarantineReason>,
}

impl FinalizationRecord {
    pub fn new(
        session_id: SessionId,
        fingerprint: CompletionIntentFingerprint,
        state: FinalizationLedgerState,
        claimed_at: DateTime<Utc>,
        committed_at: Option<DateTime<Utc>>,
        quarantine_reason: Option<FinalizationQuarantineReason>,
    ) -> Result<Self, FinalizationRecordError> {
        let valid_shape = match state {
            FinalizationLedgerState::Pending => {
                committed_at.is_none() && quarantine_reason.is_none()
            }
            FinalizationLedgerState::Committed => {
                committed_at.is_some() && quarantine_reason.is_none()
            }
            FinalizationLedgerState::Quarantined => {
                committed_at.is_none() && quarantine_reason.is_some()
            }
        };
        if !valid_shape {
            return Err(FinalizationRecordError::InconsistentStateMetadata);
        }
        Ok(Self {
            session_id,
            fingerprint,
            state,
            claimed_at,
            committed_at,
            quarantine_reason,
        })
    }

    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    pub fn fingerprint(&self) -> &CompletionIntentFingerprint {
        &self.fingerprint
    }

    pub const fn state(&self) -> FinalizationLedgerState {
        self.state
    }

    pub const fn claimed_at(&self) -> DateTime<Utc> {
        self.claimed_at
    }

    pub const fn committed_at(&self) -> Option<DateTime<Utc>> {
        self.committed_at
    }

    pub const fn quarantine_reason(&self) -> Option<FinalizationQuarantineReason> {
        self.quarantine_reason
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinalizationRecordError {
    InconsistentStateMetadata,
}

impl fmt::Display for FinalizationRecordError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("finalization record has inconsistent state metadata")
    }
}

impl std::error::Error for FinalizationRecordError {}

/// Business outcomes for creating or retrying a durable finalization claim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FinalizationLedgerClaimOutcome {
    Claimed,
    AlreadyPending,
    AlreadyCommitted,
    NotFound,
    Conflict(FinalizationConflict),
    MissingCompletionIntent,
    Quarantined(FinalizationQuarantineReason),
    Corrupt,
}

/// Business outcomes for marking the durable finalization marker committed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FinalizationCommitOutcome {
    Committed,
    AlreadyCommitted,
    NotFound,
    NotPending { state: FinalizationLedgerState },
    Conflict(FinalizationConflict),
    Quarantined(FinalizationQuarantineReason),
    Corrupt,
}

/// Metadata-only result of loading the finalization ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FinalizationLoadOutcome {
    Found(FinalizationRecord),
    NotFound,
    Corrupt,
}

/// Pure policy for the durable finalization claim.
///
/// A missing candidate is the business-level `NotFound` case. The only valid
/// source state is `AwaitingPersistence`; `FinalizationPending` with the same
/// fingerprint is an idempotent retry. The finalizer can safely no-op a
/// `Finalized` record, while other terminal states reject the claim without
/// reopening the session.
pub fn classify_finalization_claim(
    candidate: Option<&RecoveryCandidate>,
    expected_fingerprint: &CompletionIntentFingerprint,
) -> FinalizationClaimOutcome {
    let Some(candidate) = candidate else {
        return FinalizationClaimOutcome::NotFound;
    };

    if candidate.state == DurableSessionState::Finalized {
        return FinalizationClaimOutcome::AlreadyFinalized;
    }
    if candidate.state.is_terminal() {
        return FinalizationClaimOutcome::RejectedTerminal {
            state: candidate.state,
        };
    }
    if candidate.state == DurableSessionState::Running {
        return FinalizationClaimOutcome::Quarantined(QuarantineReason::InvalidStateRecord);
    }

    let CompletionIntentMetadata::Present {
        session_id,
        fingerprint,
        ..
    } = candidate.intent_metadata()
    else {
        return FinalizationClaimOutcome::Quarantined(match candidate.intent_metadata() {
            CompletionIntentMetadata::Missing => QuarantineReason::MissingCompletionIntent,
            CompletionIntentMetadata::UnsupportedCanonicalizationVersion { .. } => {
                QuarantineReason::UnsupportedCanonicalizationVersion
            }
            CompletionIntentMetadata::UnsupportedVersion { .. } => {
                QuarantineReason::UnsupportedIntentVersion
            }
            CompletionIntentMetadata::Corrupt => QuarantineReason::CorruptCompletionPayload,
            CompletionIntentMetadata::Present { .. } => {
                QuarantineReason::InconsistentDurableMetadata
            }
        });
    };

    if session_id != candidate.session_id() {
        return FinalizationClaimOutcome::Quarantined(
            QuarantineReason::InconsistentDurableMetadata,
        );
    }
    if fingerprint != expected_fingerprint {
        return FinalizationClaimOutcome::Conflict(FinalizationConflict {
            session_id: candidate.session_id().clone(),
            expected_fingerprint: expected_fingerprint.clone(),
            stored_fingerprint: fingerprint.clone(),
        });
    }

    match candidate.state {
        DurableSessionState::AwaitingPersistence => FinalizationClaimOutcome::Claimed,
        DurableSessionState::FinalizationPending => FinalizationClaimOutcome::AlreadyPending,
        DurableSessionState::Running
        | DurableSessionState::Finalized
        | DurableSessionState::Aborted
        | DurableSessionState::Interrupted
        | DurableSessionState::Quarantined => {
            FinalizationClaimOutcome::Quarantined(QuarantineReason::InvalidStateRecord)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LedgerConflict {
    SessionStart(SessionId),
    CompletionIntent(CompletionIntentConflict),
}

/// Business outcomes for loading a stored intent.
#[derive(Debug, Clone, PartialEq)]
pub enum CompletionIntentLoadOutcome {
    Found(Box<CompletionIntent>),
    NotFound,
    UnsupportedCanonicalizationVersion { version: u64 },
    UnsupportedVersion { version: u64 },
    Corrupt,
    Quarantined(QuarantineReason),
}

/// Port-level failure classes. They intentionally contain no storage details.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryPortFailure {
    RetryableFailure,
    PermanentFailure(RecoveryPermanentFailure),
}

/// Business result vocabulary for the future completion finalizer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FinalizationOutcome {
    NewlyFinalized,
    AlreadyFinalized,
    NotFound,
    Conflict(FinalizationConflict),
    Quarantined(QuarantineReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryPermanentFailure {
    UnsupportedSchema,
    IntegrityFailure,
    InvalidContract,
}

/// Failure while constructing an immutable completion intent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionIntentError {
    Serialization,
    NonFiniteNumber,
    InconsistentMetadata,
    PayloadTooLarge { actual: usize, maximum: usize },
}

impl fmt::Display for CompletionIntentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serialization => {
                formatter.write_str("completion intent payload is not serializable")
            }
            Self::NonFiniteNumber => {
                formatter.write_str("completion intent contains a non-finite number")
            }
            Self::InconsistentMetadata => {
                formatter.write_str("completion intent contains inconsistent metadata")
            }
            Self::PayloadTooLarge { actual, maximum } => write!(
                formatter,
                "completion intent payload is {actual} bytes; maximum is {maximum} bytes"
            ),
        }
    }
}

impl std::error::Error for CompletionIntentError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionIntentLoadError {
    CorruptCanonicalPayload,
    UnsupportedCanonicalizationVersion(UnsupportedCanonicalizationVersion),
    UnsupportedVersion(UnsupportedIntentVersion),
    PayloadTooLarge,
    FingerprintMismatch,
}

impl fmt::Display for CompletionIntentLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::CorruptCanonicalPayload => "completion intent payload is corrupt",
            Self::UnsupportedCanonicalizationVersion(_) => {
                "completion intent canonicalization version is unsupported"
            }
            Self::UnsupportedVersion(_) => "completion intent version is unsupported",
            Self::PayloadTooLarge => "completion intent payload exceeds the size limit",
            Self::FingerprintMismatch => "completion intent fingerprint does not match",
        })
    }
}

impl std::error::Error for CompletionIntentLoadError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnsupportedIntentVersion {
    pub version: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnsupportedCanonicalizationVersion {
    pub version: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidFingerprint;

impl fmt::Display for InvalidFingerprint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("fingerprint must be 64 hexadecimal characters")
    }
}

impl std::error::Error for InvalidFingerprint {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartedSessionError {
    InvalidDescriptor,
    DescriptorTooLarge { actual: usize, maximum: usize },
    DescriptorTooDeep { maximum: usize },
    DescriptorKeyTooLong { maximum: usize },
    DescriptorStringTooLong { maximum: usize },
    SensitiveDescriptorField,
    UnsafeDescriptorValue,
}

impl fmt::Display for StartedSessionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDescriptor => formatter.write_str("session descriptor is invalid"),
            Self::DescriptorTooLarge { actual, maximum } => write!(
                formatter,
                "session descriptor is {actual} bytes; maximum is {maximum} bytes"
            ),
            Self::DescriptorTooDeep { maximum } => {
                write!(
                    formatter,
                    "session descriptor exceeds maximum depth {maximum}"
                )
            }
            Self::DescriptorKeyTooLong { maximum } => {
                write!(formatter, "session descriptor key exceeds {maximum} bytes")
            }
            Self::DescriptorStringTooLong { maximum } => {
                write!(
                    formatter,
                    "session descriptor string exceeds {maximum} bytes"
                )
            }
            Self::SensitiveDescriptorField => {
                formatter.write_str("session descriptor contains prohibited metadata")
            }
            Self::UnsafeDescriptorValue => {
                formatter.write_str("session descriptor contains unsupported metadata")
            }
        }
    }
}

impl std::error::Error for StartedSessionError {}

const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

fn canonical_envelope_bytes(
    canonicalization_version: u16,
    payload_version: u16,
    payload: Value,
) -> Result<Vec<u8>, CanonicalJsonError> {
    let mut envelope = Map::new();
    envelope.insert(
        "canonicalization_version".to_string(),
        Value::from(canonicalization_version),
    );
    envelope.insert("payload".to_string(), payload);
    envelope.insert("payload_version".to_string(), Value::from(payload_version));
    canonical_json_bytes(&Value::Object(envelope))
}

fn canonical_json_bytes(value: &Value) -> Result<Vec<u8>, CanonicalJsonError> {
    let mut output = Vec::new();
    write_canonical_json(value, &mut output)?;
    Ok(output)
}

fn write_canonical_json(value: &Value, output: &mut Vec<u8>) -> Result<(), CanonicalJsonError> {
    match value {
        Value::Null => output.extend_from_slice(b"null"),
        Value::Bool(value) => output.extend_from_slice(if *value { b"true" } else { b"false" }),
        Value::Number(value) => {
            write_canonical_number(value, output)?;
        }
        Value::String(value) => {
            let encoded =
                serde_json::to_string(value).map_err(|_| CanonicalJsonError::Serialization)?;
            output.extend_from_slice(encoded.as_bytes());
        }
        Value::Array(values) => {
            output.push(b'[');
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    output.push(b',');
                }
                write_canonical_json(value, output)?;
            }
            output.push(b']');
        }
        Value::Object(values) => {
            let mut entries = values.iter().collect::<Vec<_>>();
            entries.sort_unstable_by(|left, right| left.0.cmp(right.0));
            output.push(b'{');
            for (index, (key, value)) in entries.into_iter().enumerate() {
                if index > 0 {
                    output.push(b',');
                }
                let encoded_key =
                    serde_json::to_string(key).map_err(|_| CanonicalJsonError::Serialization)?;
                output.extend_from_slice(encoded_key.as_bytes());
                output.push(b':');
                write_canonical_json(value, output)?;
            }
            output.push(b'}');
        }
    }
    Ok(())
}

/// Writes the version-1 internal numeric canonicalization.
///
/// Integers use base-10 decimal notation without a sign on zero. Finite
/// floating-point values use the locale-independent Rust decimal formatter;
/// integer-valued floats are emitted as integers, so `1`, `1.0`, and `1e0`
/// agree, as do `-0` and `-0.0`. Fractional values use the shortest
/// round-trippable decimal representation of the in-memory `f64`. This is a
/// deliberately versioned application rule rather than an appeal to a map's
/// or serializer's incidental formatting.
fn write_canonical_number(
    value: &serde_json::Number,
    output: &mut Vec<u8>,
) -> Result<(), CanonicalJsonError> {
    if let Some(number) = value.as_i64() {
        output.extend_from_slice(number.to_string().as_bytes());
        return Ok(());
    }
    if let Some(number) = value.as_u64() {
        output.extend_from_slice(number.to_string().as_bytes());
        return Ok(());
    }

    let number = value.as_f64().ok_or(CanonicalJsonError::NonFiniteNumber)?;
    if !number.is_finite() {
        return Err(CanonicalJsonError::NonFiniteNumber);
    }
    if number == 0.0 {
        output.extend_from_slice(b"0");
    } else if number.fract() == 0.0 {
        output.extend_from_slice(format!("{number:.0}").as_bytes());
    } else {
        output.extend_from_slice(format!("{number}").as_bytes());
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CanonicalJsonError {
    Serialization,
    NonFiniteNumber,
}

#[cfg(test)]
mod tests {
    use super::*;
    use racoon_domain::CharStatus;

    fn timestamp(seconds: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(seconds, 0).expect("test timestamp should be valid")
    }

    fn completion(mode_config: Value) -> SessionCompletion {
        let lesson_id = mode_config
            .get("lesson_id")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);
        let language = mode_config
            .get("language")
            .and_then(Value::as_str)
            .unwrap_or("en")
            .to_string();
        SessionCompletion {
            session_id: SessionId::from("018f0c2e-7b8d-7abc-8def-0123456789ab"),
            completed_at: timestamp(1_752_969_600),
            final_stats: FinalStats {
                wpm: 60.0,
                raw_wpm: 62.0,
                accuracy: 0.98,
                raw_accuracy: 0.99,
                consistency: Some(0.9),
                correct_chars: 5,
                incorrect_chars: 0,
                backspaces: 0,
                char_stats: serde_json::json!({"a": {"correct": 1, "incorrect": 0}}),
                heatmap: serde_json::json!({"a": {"count": 1}}),
                graph_data: Some(serde_json::json!([60.0])),
                duration_ms: 1_000,
            },
            mode_type: if lesson_id.is_some() {
                "lesson".to_string()
            } else {
                "custom".to_string()
            },
            mode_config,
            language,
            text_length: 5,
            replay_frames: vec![ReplayFrame {
                timestamp_ms: 10,
                key: "a".to_string(),
                caret_pos: 1,
                char_status: CharStatus::Correct,
                expected_char: 'a',
                typed_char: Some('a'),
            }],
            lesson_id,
        }
    }

    fn policy() -> CompletionPolicySnapshot {
        CompletionPolicySnapshot::time(15.0)
    }

    fn intent(mode_config: Value) -> CompletionIntent {
        CompletionIntent::from_completion(&completion(mode_config), policy())
            .expect("fixture intent should be valid")
    }

    fn canonical_stored_payload(payload: Value) -> (Vec<u8>, CompletionIntentFingerprint) {
        let bytes = canonical_envelope_bytes(
            CURRENT_CANONICALIZATION_VERSION,
            CURRENT_COMPLETION_INTENT_VERSION,
            payload,
        )
        .expect("test payload should be canonicalizable");
        let fingerprint = CompletionIntentFingerprint::from_canonical_bytes(&bytes);
        (bytes, fingerprint)
    }

    fn candidate(
        state: DurableSessionState,
        metadata: CompletionIntentMetadata,
    ) -> RecoveryCandidate {
        RecoveryCandidate::new(
            SessionId::from("018f0c2e-7b8d-7abc-8def-0123456789ab"),
            state,
            metadata,
        )
    }

    fn stored_header(intent: &CompletionIntent) -> StoredCompletionIntentHeader<'_> {
        StoredCompletionIntentHeader::Present {
            session_id: StoredHeaderValue::Value(intent.payload().session_id().as_str()),
            canonicalization_version: StoredHeaderValue::Value(i64::from(
                intent.canonicalization_version().as_u16(),
            )),
            payload_version: StoredHeaderValue::Value(i64::from(intent.payload_version().as_u16())),
            fingerprint: StoredHeaderValue::Value(intent.fingerprint().as_str()),
        }
    }

    #[test]
    fn stored_header_classification_constructs_supported_metadata_without_payload_bytes() {
        let intent = intent(serde_json::json!({
            "language": "en",
            "secret": "payload-content-must-not-enter-metadata"
        }));
        let metadata = CompletionIntentMetadata::from_stored_header(stored_header(&intent));

        assert_eq!(metadata, CompletionIntentMetadata::present(&intent));
        let debug = format!("{metadata:?}");
        assert!(!debug.contains("payload-content-must-not-enter-metadata"));
        assert!(!debug.contains("replay_frames"));
    }

    #[test]
    fn stored_header_classification_distinguishes_missing_intent_rows() {
        assert_eq!(
            CompletionIntentMetadata::from_stored_header(StoredCompletionIntentHeader::Missing),
            CompletionIntentMetadata::Missing
        );
    }

    #[test]
    fn stored_header_classification_has_deterministic_version_precedence() {
        let intent = intent(serde_json::json!({"language": "en"}));
        let session_id = intent.payload().session_id().as_str();
        let fingerprint = intent.fingerprint().as_str();

        assert_eq!(
            CompletionIntentMetadata::from_stored_header(StoredCompletionIntentHeader::Present {
                session_id: StoredHeaderValue::Value(session_id),
                canonicalization_version: StoredHeaderValue::Value(2),
                payload_version: StoredHeaderValue::Value(1),
                fingerprint: StoredHeaderValue::Value(fingerprint),
            }),
            CompletionIntentMetadata::UnsupportedCanonicalizationVersion { version: 2 }
        );
        assert_eq!(
            CompletionIntentMetadata::from_stored_header(StoredCompletionIntentHeader::Present {
                session_id: StoredHeaderValue::Value(session_id),
                canonicalization_version: StoredHeaderValue::Value(1),
                payload_version: StoredHeaderValue::Value(2),
                fingerprint: StoredHeaderValue::Value(fingerprint),
            }),
            CompletionIntentMetadata::UnsupportedVersion { version: 2 }
        );
        assert_eq!(
            CompletionIntentMetadata::from_stored_header(StoredCompletionIntentHeader::Present {
                session_id: StoredHeaderValue::Value(session_id),
                canonicalization_version: StoredHeaderValue::Value(2),
                payload_version: StoredHeaderValue::Value(2),
                fingerprint: StoredHeaderValue::Value(fingerprint),
            }),
            CompletionIntentMetadata::UnsupportedCanonicalizationVersion { version: 2 }
        );
    }

    #[test]
    fn stored_header_classification_rejects_invalid_versions_and_partial_headers() {
        let intent = intent(serde_json::json!({"language": "en"}));
        let session_id = intent.payload().session_id().as_str();
        let fingerprint = intent.fingerprint().as_str();
        let corrupt_headers = [
            StoredCompletionIntentHeader::Present {
                session_id: StoredHeaderValue::Value(session_id),
                canonicalization_version: StoredHeaderValue::Value(-1),
                payload_version: StoredHeaderValue::Value(1),
                fingerprint: StoredHeaderValue::Value(fingerprint),
            },
            StoredCompletionIntentHeader::Present {
                session_id: StoredHeaderValue::Value(session_id),
                canonicalization_version: StoredHeaderValue::Value(1),
                payload_version: StoredHeaderValue::Value(-1),
                fingerprint: StoredHeaderValue::Value(fingerprint),
            },
            StoredCompletionIntentHeader::Present {
                session_id: StoredHeaderValue::Value(session_id),
                canonicalization_version: StoredHeaderValue::Invalid,
                payload_version: StoredHeaderValue::Value(1),
                fingerprint: StoredHeaderValue::Value(fingerprint),
            },
            StoredCompletionIntentHeader::Present {
                session_id: StoredHeaderValue::Missing,
                canonicalization_version: StoredHeaderValue::Value(1),
                payload_version: StoredHeaderValue::Value(1),
                fingerprint: StoredHeaderValue::Value(fingerprint),
            },
            StoredCompletionIntentHeader::Present {
                session_id: StoredHeaderValue::Value(session_id),
                canonicalization_version: StoredHeaderValue::Missing,
                payload_version: StoredHeaderValue::Value(1),
                fingerprint: StoredHeaderValue::Value(fingerprint),
            },
            StoredCompletionIntentHeader::Present {
                session_id: StoredHeaderValue::Value(session_id),
                canonicalization_version: StoredHeaderValue::Value(1),
                payload_version: StoredHeaderValue::Missing,
                fingerprint: StoredHeaderValue::Value(fingerprint),
            },
            StoredCompletionIntentHeader::Present {
                session_id: StoredHeaderValue::Value(session_id),
                canonicalization_version: StoredHeaderValue::Value(1),
                payload_version: StoredHeaderValue::Value(1),
                fingerprint: StoredHeaderValue::Invalid,
            },
        ];

        for header in corrupt_headers {
            assert_eq!(
                CompletionIntentMetadata::from_stored_header(header),
                CompletionIntentMetadata::Corrupt
            );
        }

        for (canonicalization_version, payload_version, expected) in [
            (
                i64::MAX,
                1,
                CompletionIntentMetadata::UnsupportedCanonicalizationVersion {
                    version: i64::MAX as u64,
                },
            ),
            (
                1,
                i64::MAX,
                CompletionIntentMetadata::UnsupportedVersion {
                    version: i64::MAX as u64,
                },
            ),
        ] {
            assert_eq!(
                CompletionIntentMetadata::from_stored_header(
                    StoredCompletionIntentHeader::Present {
                        session_id: StoredHeaderValue::Value(session_id),
                        canonicalization_version: StoredHeaderValue::Value(
                            canonicalization_version
                        ),
                        payload_version: StoredHeaderValue::Value(payload_version),
                        fingerprint: StoredHeaderValue::Value(fingerprint),
                    }
                ),
                expected
            );
        }
    }

    #[test]
    fn stored_header_classification_requires_a_strict_session_id_and_lowercase_fingerprint() {
        let intent = intent(serde_json::json!({"language": "en"}));
        let session_id = intent.payload().session_id().as_str();
        let fingerprint = intent.fingerprint().as_str();
        let short_fingerprint = "a".repeat(63);
        let long_fingerprint = "a".repeat(65);
        let uppercase_fingerprint = fingerprint.to_ascii_uppercase();
        let non_hex_fingerprint = "g".repeat(64);
        let whitespace_fingerprint = format!(" {fingerprint}");
        let corrupt_headers = [
            StoredCompletionIntentHeader::Present {
                session_id: StoredHeaderValue::Value(""),
                canonicalization_version: StoredHeaderValue::Value(1),
                payload_version: StoredHeaderValue::Value(1),
                fingerprint: StoredHeaderValue::Value(fingerprint),
            },
            StoredCompletionIntentHeader::Present {
                session_id: StoredHeaderValue::Value("not-a-session-id"),
                canonicalization_version: StoredHeaderValue::Value(1),
                payload_version: StoredHeaderValue::Value(1),
                fingerprint: StoredHeaderValue::Value(fingerprint),
            },
            StoredCompletionIntentHeader::Present {
                session_id: StoredHeaderValue::Value(session_id),
                canonicalization_version: StoredHeaderValue::Value(1),
                payload_version: StoredHeaderValue::Value(1),
                fingerprint: StoredHeaderValue::Missing,
            },
            StoredCompletionIntentHeader::Present {
                session_id: StoredHeaderValue::Value(session_id),
                canonicalization_version: StoredHeaderValue::Value(1),
                payload_version: StoredHeaderValue::Value(1),
                fingerprint: StoredHeaderValue::Value(""),
            },
            StoredCompletionIntentHeader::Present {
                session_id: StoredHeaderValue::Value(session_id),
                canonicalization_version: StoredHeaderValue::Value(1),
                payload_version: StoredHeaderValue::Value(1),
                fingerprint: StoredHeaderValue::Value(&short_fingerprint),
            },
            StoredCompletionIntentHeader::Present {
                session_id: StoredHeaderValue::Value(session_id),
                canonicalization_version: StoredHeaderValue::Value(1),
                payload_version: StoredHeaderValue::Value(1),
                fingerprint: StoredHeaderValue::Value(&long_fingerprint),
            },
            StoredCompletionIntentHeader::Present {
                session_id: StoredHeaderValue::Value(session_id),
                canonicalization_version: StoredHeaderValue::Value(1),
                payload_version: StoredHeaderValue::Value(1),
                fingerprint: StoredHeaderValue::Value(&uppercase_fingerprint),
            },
            StoredCompletionIntentHeader::Present {
                session_id: StoredHeaderValue::Value(session_id),
                canonicalization_version: StoredHeaderValue::Value(1),
                payload_version: StoredHeaderValue::Value(1),
                fingerprint: StoredHeaderValue::Value(&non_hex_fingerprint),
            },
            StoredCompletionIntentHeader::Present {
                session_id: StoredHeaderValue::Value(session_id),
                canonicalization_version: StoredHeaderValue::Value(1),
                payload_version: StoredHeaderValue::Value(1),
                fingerprint: StoredHeaderValue::Value(&whitespace_fingerprint),
            },
        ];

        for header in corrupt_headers {
            assert_eq!(
                CompletionIntentMetadata::from_stored_header(header),
                CompletionIntentMetadata::Corrupt
            );
        }
    }

    #[test]
    fn supported_header_does_not_bypass_full_payload_validation() {
        let intent = intent(serde_json::json!({"language": "en"}));
        assert!(matches!(
            CompletionIntentMetadata::from_stored_header(stored_header(&intent)),
            CompletionIntentMetadata::Present { .. }
        ));
        assert_eq!(
            CompletionIntent::from_stored_payload(b"not-json", intent.fingerprint()),
            Err(CompletionIntentLoadError::CorruptCanonicalPayload)
        );
    }

    #[test]
    fn durable_states_distinguish_terminal_values() {
        assert!(!DurableSessionState::Running.is_terminal());
        assert!(!DurableSessionState::AwaitingPersistence.is_terminal());
        assert!(!DurableSessionState::FinalizationPending.is_terminal());
        for state in [
            DurableSessionState::Finalized,
            DurableSessionState::Aborted,
            DurableSessionState::Interrupted,
            DurableSessionState::Quarantined,
        ] {
            assert!(state.is_terminal());
        }
    }

    #[test]
    fn durable_state_transition_matrix_is_explicit_and_exhaustive() {
        let states = [
            DurableSessionState::Running,
            DurableSessionState::AwaitingPersistence,
            DurableSessionState::FinalizationPending,
            DurableSessionState::Finalized,
            DurableSessionState::Aborted,
            DurableSessionState::Interrupted,
            DurableSessionState::Quarantined,
        ];

        assert_eq!(
            validate_durable_state_transition(None, DurableSessionState::Running),
            DurableStateTransitionOutcome::Valid
        );
        for state in states {
            assert_eq!(
                validate_durable_state_transition(Some(state), state),
                DurableStateTransitionOutcome::Idempotent
            );
        }

        for (from, to) in [
            (
                DurableSessionState::Running,
                DurableSessionState::AwaitingPersistence,
            ),
            (DurableSessionState::Running, DurableSessionState::Aborted),
            (
                DurableSessionState::Running,
                DurableSessionState::Interrupted,
            ),
            (
                DurableSessionState::Running,
                DurableSessionState::Quarantined,
            ),
            (
                DurableSessionState::AwaitingPersistence,
                DurableSessionState::FinalizationPending,
            ),
            (
                DurableSessionState::AwaitingPersistence,
                DurableSessionState::Aborted,
            ),
            (
                DurableSessionState::AwaitingPersistence,
                DurableSessionState::Quarantined,
            ),
            (
                DurableSessionState::FinalizationPending,
                DurableSessionState::Finalized,
            ),
            (
                DurableSessionState::FinalizationPending,
                DurableSessionState::Aborted,
            ),
            (
                DurableSessionState::FinalizationPending,
                DurableSessionState::Quarantined,
            ),
        ] {
            assert_eq!(
                validate_durable_state_transition(Some(from), to),
                DurableStateTransitionOutcome::Valid
            );
        }

        for terminal in [
            DurableSessionState::Finalized,
            DurableSessionState::Aborted,
            DurableSessionState::Interrupted,
            DurableSessionState::Quarantined,
        ] {
            for target in states {
                if target != terminal {
                    assert_eq!(
                        validate_durable_state_transition(Some(terminal), target),
                        DurableStateTransitionOutcome::ForbiddenFromTerminal {
                            from: terminal,
                            to: target,
                        }
                    );
                }
            }
        }

        for target in [
            DurableSessionState::AwaitingPersistence,
            DurableSessionState::FinalizationPending,
            DurableSessionState::Finalized,
            DurableSessionState::Aborted,
            DurableSessionState::Interrupted,
            DurableSessionState::Quarantined,
        ] {
            assert_eq!(
                validate_durable_state_transition(None, target),
                DurableStateTransitionOutcome::Invalid {
                    from: None,
                    to: target
                }
            );
        }

        for (from, to) in [
            (
                DurableSessionState::Running,
                DurableSessionState::FinalizationPending,
            ),
            (DurableSessionState::Running, DurableSessionState::Finalized),
            (
                DurableSessionState::AwaitingPersistence,
                DurableSessionState::Running,
            ),
            (
                DurableSessionState::AwaitingPersistence,
                DurableSessionState::Finalized,
            ),
            (
                DurableSessionState::AwaitingPersistence,
                DurableSessionState::Interrupted,
            ),
            (
                DurableSessionState::FinalizationPending,
                DurableSessionState::Running,
            ),
            (
                DurableSessionState::FinalizationPending,
                DurableSessionState::AwaitingPersistence,
            ),
            (
                DurableSessionState::FinalizationPending,
                DurableSessionState::Interrupted,
            ),
        ] {
            assert_eq!(
                validate_durable_state_transition(Some(from), to),
                DurableStateTransitionOutcome::Invalid {
                    from: Some(from),
                    to,
                }
            );
        }
    }

    #[test]
    fn readiness_transitions_are_closed_and_deterministic() {
        assert_eq!(
            RecoveryReadiness::NotStarted
                .transition(RecoveryReadinessEvent::Begin)
                .unwrap(),
            RecoveryReadiness::Recovering
        );
        assert_eq!(
            RecoveryReadiness::Recovering
                .transition(RecoveryReadinessEvent::Complete)
                .unwrap(),
            RecoveryReadiness::Ready
        );
        assert_eq!(
            RecoveryReadiness::NotStarted
                .transition(RecoveryReadinessEvent::Block)
                .unwrap(),
            RecoveryReadiness::Blocked
        );
        assert_eq!(
            RecoveryReadiness::Recovering
                .transition(RecoveryReadinessEvent::Block)
                .unwrap(),
            RecoveryReadiness::Blocked
        );
        assert!(RecoveryReadiness::Ready
            .transition(RecoveryReadinessEvent::Begin)
            .is_err());
        assert!(RecoveryReadiness::Blocked
            .transition(RecoveryReadinessEvent::Complete)
            .is_err());
    }

    #[test]
    fn running_sessions_are_interrupted_and_never_resumed() {
        let decision = classify_recovery_candidate(&candidate(
            DurableSessionState::Running,
            CompletionIntentMetadata::Missing,
        ));
        assert!(matches!(
            decision,
            RecoveryDecision::MarkInterrupted {
                reason: InterruptionReason::ProcessRestart,
                ..
            }
        ));
    }

    #[test]
    fn pending_states_with_valid_intents_are_eligible() {
        let valid = CompletionIntentMetadata::present(&intent(serde_json::json!({
            "language": "en"
        })));
        for state in [
            DurableSessionState::AwaitingPersistence,
            DurableSessionState::FinalizationPending,
        ] {
            let decision = classify_recovery_candidate(&candidate(state, valid.clone()));
            assert!(matches!(
                decision,
                RecoveryDecision::EligibleForFinalization { .. }
            ));
        }
    }

    #[test]
    fn finalization_claim_policy_covers_every_business_outcome() {
        let full_intent = intent(serde_json::json!({"language": "en"}));
        let fingerprint = full_intent.fingerprint();

        assert_eq!(
            classify_finalization_claim(None, fingerprint),
            FinalizationClaimOutcome::NotFound
        );

        let awaiting = candidate(
            DurableSessionState::AwaitingPersistence,
            CompletionIntentMetadata::present(&full_intent),
        );
        assert_eq!(
            classify_finalization_claim(Some(&awaiting), fingerprint),
            FinalizationClaimOutcome::Claimed
        );

        let pending = candidate(
            DurableSessionState::FinalizationPending,
            CompletionIntentMetadata::present(&full_intent),
        );
        assert_eq!(
            classify_finalization_claim(Some(&pending), fingerprint),
            FinalizationClaimOutcome::AlreadyPending
        );

        let finalized = candidate(
            DurableSessionState::Finalized,
            CompletionIntentMetadata::Missing,
        );
        assert_eq!(
            classify_finalization_claim(Some(&finalized), fingerprint),
            FinalizationClaimOutcome::AlreadyFinalized
        );

        for state in [
            DurableSessionState::Aborted,
            DurableSessionState::Interrupted,
            DurableSessionState::Quarantined,
        ] {
            let terminal = candidate(state, CompletionIntentMetadata::Missing);
            assert_eq!(
                classify_finalization_claim(Some(&terminal), fingerprint),
                FinalizationClaimOutcome::RejectedTerminal { state }
            );
        }

        let running = candidate(
            DurableSessionState::Running,
            CompletionIntentMetadata::Missing,
        );
        assert_eq!(
            classify_finalization_claim(Some(&running), fingerprint),
            FinalizationClaimOutcome::Quarantined(QuarantineReason::InvalidStateRecord)
        );

        let missing = candidate(
            DurableSessionState::AwaitingPersistence,
            CompletionIntentMetadata::Missing,
        );
        assert_eq!(
            classify_finalization_claim(Some(&missing), fingerprint),
            FinalizationClaimOutcome::Quarantined(QuarantineReason::MissingCompletionIntent)
        );

        let unsupported = candidate(
            DurableSessionState::AwaitingPersistence,
            CompletionIntentMetadata::UnsupportedVersion { version: 2 },
        );
        assert_eq!(
            classify_finalization_claim(Some(&unsupported), fingerprint),
            FinalizationClaimOutcome::Quarantined(QuarantineReason::UnsupportedIntentVersion)
        );

        let conflicting = intent(serde_json::json!({"language": "fr"}));
        let conflict_candidate = candidate(
            DurableSessionState::AwaitingPersistence,
            CompletionIntentMetadata::present(&conflicting),
        );
        assert!(matches!(
            classify_finalization_claim(Some(&conflict_candidate), fingerprint),
            FinalizationClaimOutcome::Conflict(FinalizationConflict { .. })
        ));

        let inconsistent = RecoveryCandidate::new(
            full_intent.payload().session_id().clone(),
            DurableSessionState::AwaitingPersistence,
            CompletionIntentMetadata::Present {
                session_id: SessionId::from("018f0c2e-7b8d-7abc-8def-0123456789ac"),
                canonicalization_version: full_intent.canonicalization_version(),
                payload_version: full_intent.payload_version(),
                fingerprint: full_intent.fingerprint().clone(),
            },
        );
        assert_eq!(
            classify_finalization_claim(Some(&inconsistent), fingerprint),
            FinalizationClaimOutcome::Quarantined(QuarantineReason::InconsistentDurableMetadata)
        );
    }

    #[test]
    fn finalization_requires_the_expected_immutable_intent_fingerprint() {
        let original = intent(serde_json::json!({"language": "en"}));
        let session_id = original.payload().session_id().clone();
        assert_eq!(
            validate_finalization_intent(&session_id, original.fingerprint(), &original),
            FinalizationIntentValidation::Match
        );

        let conflicting = intent(serde_json::json!({"language": "fr"}));
        let validation =
            validate_finalization_intent(&session_id, original.fingerprint(), &conflicting);
        let conflict = match validation {
            FinalizationIntentValidation::Conflict(conflict) => conflict,
            FinalizationIntentValidation::Match => {
                panic!("mismatched intent must not validate for finalization")
            }
        };
        let outcome = FinalizationOutcome::Conflict(conflict);
        assert!(matches!(outcome, FinalizationOutcome::Conflict(_)));

        let mut other_session_payload = original.payload().clone();
        other_session_payload.session_id = SessionId::from("018f0c2e-7b8d-7abc-8def-0123456789ac");
        let other_session = CompletionIntent::from_payload(other_session_payload).unwrap();
        assert!(matches!(
            validate_finalization_intent(&session_id, original.fingerprint(), &other_session),
            FinalizationIntentValidation::Conflict(_)
        ));
    }

    #[test]
    fn finalizer_outcomes_are_business_only_and_failure_classes_are_explicit() {
        let outcomes = [
            FinalizationOutcome::NewlyFinalized,
            FinalizationOutcome::AlreadyFinalized,
            FinalizationOutcome::NotFound,
            FinalizationOutcome::Quarantined(QuarantineReason::InvalidStateRecord),
        ];
        assert!(matches!(outcomes[0], FinalizationOutcome::NewlyFinalized));
        assert!(matches!(outcomes[1], FinalizationOutcome::AlreadyFinalized));
        assert!(matches!(outcomes[2], FinalizationOutcome::NotFound));
        assert!(matches!(outcomes[3], FinalizationOutcome::Quarantined(_)));

        assert_eq!(
            RecoveryPortFailure::RetryableFailure,
            RecoveryPortFailure::RetryableFailure
        );
        assert_eq!(
            RecoveryPortFailure::PermanentFailure(RecoveryPermanentFailure::InvalidContract),
            RecoveryPortFailure::PermanentFailure(RecoveryPermanentFailure::InvalidContract)
        );
    }

    #[test]
    fn metadata_only_candidates_require_separately_loaded_intents() {
        let full_intent = intent(serde_json::json!({"secret": "typed-secret-content"}));
        let candidate = candidate(
            DurableSessionState::FinalizationPending,
            CompletionIntentMetadata::present(&full_intent),
        );
        let debug = format!("{candidate:?}");
        assert!(!debug.contains("typed-secret-content"));
        assert!(!debug.contains("replay_frames"));
        assert!(matches!(
            classify_recovery_candidate(&candidate),
            RecoveryDecision::EligibleForFinalization { .. }
        ));
        assert_eq!(
            validate_recovery_candidate_intent(&candidate, &full_intent),
            RecoveryIntentValidation::Valid
        );

        let conflicting = intent(serde_json::json!({"language": "fr"}));
        assert_eq!(
            validate_recovery_candidate_intent(&candidate, &conflicting),
            RecoveryIntentValidation::Quarantine(QuarantineReason::FingerprintMismatch)
        );
    }

    #[test]
    fn mismatched_candidate_metadata_is_quarantined() {
        let full_intent = intent(serde_json::json!({"language": "en"}));
        let candidate = RecoveryCandidate::new(
            SessionId::from("018f0c2e-7b8d-7abc-8def-0123456789ab"),
            DurableSessionState::FinalizationPending,
            CompletionIntentMetadata::Present {
                session_id: SessionId::from("018f0c2e-7b8d-7abc-8def-0123456789ac"),
                canonicalization_version: full_intent.canonicalization_version(),
                payload_version: full_intent.payload_version(),
                fingerprint: full_intent.fingerprint().clone(),
            },
        );
        assert!(matches!(
            classify_recovery_candidate(&candidate),
            RecoveryDecision::Quarantine {
                reason: QuarantineReason::InconsistentDurableMetadata,
                ..
            }
        ));
    }

    #[test]
    fn terminal_states_are_no_ops() {
        for state in [
            DurableSessionState::Finalized,
            DurableSessionState::Aborted,
            DurableSessionState::Interrupted,
            DurableSessionState::Quarantined,
        ] {
            let decision =
                classify_recovery_candidate(&candidate(state, CompletionIntentMetadata::Missing));
            assert!(
                matches!(decision, RecoveryDecision::NoOp { state: actual, .. } if actual == state)
            );
        }
    }

    #[test]
    fn pending_invalid_intents_are_quarantined() {
        let cases = [
            (
                CompletionIntentMetadata::Missing,
                QuarantineReason::MissingCompletionIntent,
            ),
            (
                CompletionIntentMetadata::UnsupportedCanonicalizationVersion { version: 2 },
                QuarantineReason::UnsupportedCanonicalizationVersion,
            ),
            (
                CompletionIntentMetadata::UnsupportedVersion { version: 2 },
                QuarantineReason::UnsupportedIntentVersion,
            ),
            (
                CompletionIntentMetadata::Corrupt,
                QuarantineReason::CorruptCompletionPayload,
            ),
        ];
        for (record, reason) in cases {
            let decision = classify_recovery_candidate(&candidate(
                DurableSessionState::FinalizationPending,
                record,
            ));
            assert!(
                matches!(decision, RecoveryDecision::Quarantine { reason: actual, .. } if actual == reason)
            );
        }
    }

    #[test]
    fn inconsistent_running_record_is_quarantined_without_resume() {
        let decision = classify_recovery_candidate(&candidate(
            DurableSessionState::Running,
            CompletionIntentMetadata::present(&intent(serde_json::json!({
                "language": "en"
            }))),
        ));
        assert!(matches!(
            decision,
            RecoveryDecision::Quarantine {
                reason: QuarantineReason::InvalidStateRecord,
                ..
            }
        ));
    }

    #[test]
    fn equivalent_object_order_has_the_same_fingerprint() {
        let first = intent(serde_json::json!({"b": 2, "a": 1}));
        let second = intent(serde_json::from_str(r#"{"a":1,"b":2}"#).unwrap());
        assert_eq!(first.fingerprint(), second.fingerprint());
        assert_eq!(first.canonical_payload(), second.canonical_payload());
    }

    #[test]
    fn completion_relevant_changes_change_the_fingerprint() {
        let base = intent(serde_json::json!({"language": "en"}));

        let mut changed_timestamp = base.payload().clone();
        changed_timestamp.completed_at = timestamp(1_752_969_601);
        assert_ne!(
            base.fingerprint(),
            CompletionIntent::from_payload(changed_timestamp)
                .unwrap()
                .fingerprint()
        );

        let mut changed_stats = base.payload().clone();
        changed_stats.final_stats.accuracy = 0.97;
        assert_ne!(
            base.fingerprint(),
            CompletionIntent::from_payload(changed_stats)
                .unwrap()
                .fingerprint()
        );

        let mut changed_policy = base.payload().clone();
        changed_policy.completion_policy = CompletionPolicySnapshot::accuracy(0.90);
        assert_ne!(
            base.fingerprint(),
            CompletionIntent::from_payload(changed_policy)
                .unwrap()
                .fingerprint()
        );

        let mut changed_replay = base.payload().clone();
        changed_replay.replay_frames[0].timestamp_ms = 11;
        assert_ne!(
            base.fingerprint(),
            CompletionIntent::from_payload(changed_replay)
                .unwrap()
                .fingerprint()
        );
    }

    #[test]
    fn every_completion_policy_variant_changes_the_fingerprint() {
        let base = intent(serde_json::json!({"language": "en"}));
        for policy in [
            CompletionPolicySnapshot::time(16.0),
            CompletionPolicySnapshot::wpm(65.0),
            CompletionPolicySnapshot::accuracy(0.95),
        ] {
            let mut payload = base.payload().clone();
            payload.completion_policy = policy;
            assert_ne!(
                base.fingerprint(),
                CompletionIntent::from_payload(payload)
                    .unwrap()
                    .fingerprint()
            );
        }

        let mut changed_time = base.payload().clone();
        changed_time.completion_policy = CompletionPolicySnapshot::time(16.0);
        assert_ne!(
            base.fingerprint(),
            CompletionIntent::from_payload(changed_time)
                .unwrap()
                .fingerprint()
        );
    }

    #[test]
    fn redundant_completion_metadata_must_agree() {
        let valid_lesson = intent(serde_json::json!({
            "language": "en",
            "lesson_id": "en_m1_l1",
            "module_id": "m1"
        }));
        assert_eq!(valid_lesson.payload().mode_type(), "lesson");
        assert_eq!(valid_lesson.payload().lesson_id(), Some("en_m1_l1"));

        let mut conflicting_language = intent(serde_json::json!({"language": "en"}))
            .payload()
            .clone();
        conflicting_language.language = "fr".to_string();
        assert_eq!(
            CompletionIntent::from_payload(conflicting_language),
            Err(CompletionIntentError::InconsistentMetadata)
        );

        let mut conflicting_lesson = valid_lesson.payload().clone();
        conflicting_lesson.lesson_id = Some("en_m1_l2".to_string());
        assert_eq!(
            CompletionIntent::from_payload(conflicting_lesson),
            Err(CompletionIntentError::InconsistentMetadata)
        );

        let mut non_lesson_with_lesson = intent(serde_json::json!({"language": "en"}))
            .payload()
            .clone();
        non_lesson_with_lesson.lesson_id = Some("en_m1_l1".to_string());
        assert_eq!(
            CompletionIntent::from_payload(non_lesson_with_lesson),
            Err(CompletionIntentError::InconsistentMetadata)
        );

        let mut mode_config_lesson_conflict = valid_lesson.payload().clone();
        mode_config_lesson_conflict.mode_config = serde_json::json!({
            "language": "en",
            "lesson_id": "en_m1_l2"
        });
        assert_eq!(
            CompletionIntent::from_payload(mode_config_lesson_conflict),
            Err(CompletionIntentError::InconsistentMetadata)
        );

        let mut stored_conflict =
            serde_json::to_value(intent(serde_json::json!({"language": "en"})).payload()).unwrap();
        stored_conflict["mode_config"]["language"] = Value::String("fr".to_string());
        let (stored_bytes, stored_fingerprint) = canonical_stored_payload(stored_conflict);
        assert_eq!(
            CompletionIntent::from_stored_payload(&stored_bytes, &stored_fingerprint),
            Err(CompletionIntentLoadError::CorruptCanonicalPayload)
        );
    }

    #[test]
    fn signed_zero_is_normalized_before_fingerprinting() {
        let base = intent(serde_json::json!({"language": "en"}));
        let mut negative_zero = base.payload().clone();
        negative_zero.completion_policy = CompletionPolicySnapshot::time(-0.0);
        let mut positive_zero = base.payload().clone();
        positive_zero.completion_policy = CompletionPolicySnapshot::time(0.0);

        let negative = CompletionIntent::from_payload(negative_zero).unwrap();
        let positive = CompletionIntent::from_payload(positive_zero).unwrap();
        assert_eq!(negative.fingerprint(), positive.fingerprint());
        assert_eq!(negative.canonical_payload(), positive.canonical_payload());
    }

    #[test]
    fn canonical_numbers_use_fixed_nested_vectors() {
        let vectors = [
            (
                serde_json::json!({"value": 1}),
                br#"{"value":1}"#.as_slice(),
            ),
            (
                serde_json::json!({"value": 1.0}),
                br#"{"value":1}"#.as_slice(),
            ),
            (
                serde_json::json!({"value": 1e0}),
                br#"{"value":1}"#.as_slice(),
            ),
            (
                serde_json::json!({"value": -0.0}),
                br#"{"value":0}"#.as_slice(),
            ),
            (
                serde_json::json!({"nested": [1.25, 0.0000001, 1e20]}),
                br#"{"nested":[1.25,0.0000001,100000000000000000000]}"#.as_slice(),
            ),
        ];
        for (value, expected) in vectors {
            assert_eq!(canonical_json_bytes(&value).unwrap(), expected);
        }
    }

    #[test]
    fn canonicalization_version_is_independent_and_fingerprinted() {
        let payload = serde_json::json!({"nested": {"b": 2, "a": 1}});
        let current = canonical_envelope_bytes(1, 1, payload.clone()).unwrap();
        let future = canonical_envelope_bytes(2, 1, payload).unwrap();
        assert_ne!(current, future);
        assert_ne!(
            CompletionIntentFingerprint::from_canonical_bytes(&current),
            CompletionIntentFingerprint::from_canonical_bytes(&future)
        );
        assert_eq!(
            String::from_utf8(current).unwrap(),
            r#"{"canonicalization_version":1,"payload":{"nested":{"a":1,"b":2}},"payload_version":1}"#
        );
    }

    #[test]
    fn non_finite_completion_values_are_rejected_before_fingerprinting() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut payload = intent(serde_json::json!({"language": "en"}))
                .payload()
                .clone();
            payload.completion_policy = CompletionPolicySnapshot::wpm(value);
            assert_eq!(
                CompletionIntent::from_payload(payload),
                Err(CompletionIntentError::NonFiniteNumber)
            );
        }
    }

    #[test]
    fn schema_version_participates_in_the_fingerprint() {
        let base = intent(serde_json::json!({"language": "en"}));
        let payload_value = serde_json::to_value(base.payload()).unwrap();
        let version_one = canonical_envelope_bytes(1, 1, payload_value.clone()).unwrap();
        let version_two = canonical_envelope_bytes(1, 2, payload_value).unwrap();
        assert_ne!(
            CompletionIntentFingerprint::from_canonical_bytes(&version_one),
            CompletionIntentFingerprint::from_canonical_bytes(&version_two)
        );
    }

    #[test]
    fn fingerprint_output_is_stable_lowercase_sha256() {
        let intent = intent(serde_json::json!({"language": "en"}));
        let fingerprint = intent.fingerprint();
        assert_eq!(fingerprint.as_str().len(), 64);
        assert!(fingerprint
            .as_str()
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit()));
        assert_eq!(
            fingerprint.as_str(),
            "374aa80791330a457b758e6ab0abb4dfdea2ee8969b03cb00b6e6a568cc12b43"
        );
    }

    #[test]
    fn stored_payload_validation_rejects_corruption_and_unknown_versions() {
        let original = intent(serde_json::json!({"language": "en"}));
        let corrupt = CompletionIntent::from_stored_payload(b"not-json", original.fingerprint());
        assert_eq!(
            corrupt,
            Err(CompletionIntentLoadError::CorruptCanonicalPayload)
        );

        let mut unsupported: Value = serde_json::from_slice(original.canonical_payload()).unwrap();
        unsupported["payload_version"] = Value::from(2_u16);
        let unsupported_bytes = serde_json::to_vec(&unsupported).unwrap();
        assert_eq!(
            CompletionIntent::from_stored_payload(&unsupported_bytes, original.fingerprint()),
            Err(CompletionIntentLoadError::UnsupportedVersion(
                UnsupportedIntentVersion { version: 2 }
            ))
        );

        let mut unsupported_canonicalization: Value =
            serde_json::from_slice(original.canonical_payload()).unwrap();
        unsupported_canonicalization["canonicalization_version"] = Value::from(2_u16);
        let unsupported_canonicalization_bytes =
            serde_json::to_vec(&unsupported_canonicalization).unwrap();
        assert_eq!(
            CompletionIntent::from_stored_payload(
                &unsupported_canonicalization_bytes,
                original.fingerprint()
            ),
            Err(
                CompletionIntentLoadError::UnsupportedCanonicalizationVersion(
                    UnsupportedCanonicalizationVersion { version: 2 }
                )
            )
        );
    }

    #[test]
    fn canonical_stored_payload_round_trips_without_changing_the_intent() {
        let original = intent(serde_json::json!({"language": "en"}));
        let restored = CompletionIntent::from_stored_payload(
            original.canonical_payload(),
            original.fingerprint(),
        )
        .expect("canonical intent should round-trip");
        assert_eq!(restored, original);
        assert_eq!(restored.canonical_payload(), original.canonical_payload());
        assert_eq!(restored.fingerprint(), original.fingerprint());
    }

    #[test]
    fn stored_payload_fingerprint_mismatches_are_typed_and_redacted() {
        let original = intent(serde_json::json!({"language": "en"}));

        let mut changed_payload = original.payload().clone();
        changed_payload.language = "fr".to_string();
        changed_payload.mode_config = serde_json::json!({"language": "fr"});
        let changed_intent = CompletionIntent::from_payload(changed_payload).unwrap();
        assert_eq!(
            CompletionIntent::from_stored_payload(
                changed_intent.canonical_payload(),
                original.fingerprint()
            ),
            Err(CompletionIntentLoadError::FingerprintMismatch)
        );

        let another_fingerprint = intent(serde_json::json!({"language": "fr"}));
        assert_eq!(
            CompletionIntent::from_stored_payload(
                original.canonical_payload(),
                another_fingerprint.fingerprint()
            ),
            Err(CompletionIntentLoadError::FingerprintMismatch)
        );

        let mut other_session_payload = original.payload().clone();
        other_session_payload.session_id = SessionId::from("018f0c2e-7b8d-7abc-8def-0123456789ac");
        let other_session = CompletionIntent::from_payload(other_session_payload).unwrap();
        assert_eq!(
            CompletionIntent::from_stored_payload(
                other_session.canonical_payload(),
                original.fingerprint()
            ),
            Err(CompletionIntentLoadError::FingerprintMismatch)
        );
    }

    #[test]
    fn stored_payload_requires_exact_canonical_bytes_and_known_schema_fields() {
        let original = intent(serde_json::json!({"language": "en"}));
        let payload_value = serde_json::to_value(original.payload()).unwrap();
        let payload_json = serde_json::to_string(&payload_value).unwrap();

        let reordered = format!(r#"{{"payload_version":1,"payload":{payload_json}}}"#);
        assert_eq!(
            CompletionIntent::from_stored_payload(&reordered.into_bytes(), original.fingerprint()),
            Err(CompletionIntentLoadError::CorruptCanonicalPayload)
        );

        let duplicate_field =
            format!(r#"{{"payload":{payload_json},"payload_version":1,"payload_version":1}}"#);
        assert_eq!(
            CompletionIntent::from_stored_payload(
                duplicate_field.as_bytes(),
                original.fingerprint()
            ),
            Err(CompletionIntentLoadError::CorruptCanonicalPayload)
        );

        let mut unknown_envelope: Value = serde_json::from_slice(original.canonical_payload())
            .expect("fixture envelope should be valid JSON");
        unknown_envelope["unknown_envelope_field"] = Value::Bool(true);
        let unknown_envelope_bytes = serde_json::to_vec(&unknown_envelope).unwrap();
        assert_eq!(
            CompletionIntent::from_stored_payload(&unknown_envelope_bytes, original.fingerprint()),
            Err(CompletionIntentLoadError::CorruptCanonicalPayload)
        );

        let mut unknown_payload = payload_value.as_object().unwrap().clone();
        unknown_payload.insert("unknown_field".to_string(), Value::Bool(true));
        let unknown_bytes = canonical_envelope_bytes(1, 1, Value::Object(unknown_payload)).unwrap();
        let unknown_fingerprint = CompletionIntentFingerprint::from_canonical_bytes(&unknown_bytes);
        assert_eq!(
            CompletionIntent::from_stored_payload(&unknown_bytes, &unknown_fingerprint),
            Err(CompletionIntentLoadError::CorruptCanonicalPayload)
        );

        let mut missing_payload = payload_value.as_object().unwrap().clone();
        missing_payload.remove("language");
        let missing_bytes = canonical_envelope_bytes(1, 1, Value::Object(missing_payload)).unwrap();
        let missing_fingerprint = CompletionIntentFingerprint::from_canonical_bytes(&missing_bytes);
        assert_eq!(
            CompletionIntent::from_stored_payload(&missing_bytes, &missing_fingerprint),
            Err(CompletionIntentLoadError::CorruptCanonicalPayload)
        );
    }

    #[test]
    fn stored_payload_rejects_unknown_nested_fields_without_silent_loss() {
        let original = intent(serde_json::json!({"language": "en"}));

        let mut final_stats = serde_json::to_value(original.payload()).unwrap();
        final_stats["final_stats"]["unknown_stat"] = Value::Bool(true);
        let (final_stats_bytes, final_stats_fingerprint) = canonical_stored_payload(final_stats);
        assert_eq!(
            CompletionIntent::from_stored_payload(&final_stats_bytes, &final_stats_fingerprint),
            Err(CompletionIntentLoadError::CorruptCanonicalPayload)
        );

        let mut replay_frame = serde_json::to_value(original.payload()).unwrap();
        replay_frame["replay_frames"][0]["unknown_frame_field"] = Value::String("x".into());
        let (replay_bytes, replay_fingerprint) = canonical_stored_payload(replay_frame);
        assert_eq!(
            CompletionIntent::from_stored_payload(&replay_bytes, &replay_fingerprint),
            Err(CompletionIntentLoadError::CorruptCanonicalPayload)
        );

        let mut policy = serde_json::to_value(original.payload()).unwrap();
        policy["completion_policy"]["unknown_policy_field"] = Value::Bool(true);
        let (policy_bytes, policy_fingerprint) = canonical_stored_payload(policy);
        assert_eq!(
            CompletionIntent::from_stored_payload(&policy_bytes, &policy_fingerprint),
            Err(CompletionIntentLoadError::CorruptCanonicalPayload)
        );
    }

    #[test]
    fn opaque_mode_configuration_is_preserved_without_an_application_schema() {
        let original = intent(serde_json::json!({
            "language": "en",
            "future_mode": {"unknown_nested_setting": true}
        }));
        let restored = CompletionIntent::from_stored_payload(
            original.canonical_payload(),
            original.fingerprint(),
        )
        .expect("opaque mode configuration should round-trip");
        assert_eq!(
            restored.payload().mode_config(),
            original.payload().mode_config()
        );
    }

    #[test]
    fn stored_payload_size_is_checked_before_parsing() {
        let original = intent(serde_json::json!({"language": "en"}));
        let oversized = vec![b'x'; MAX_COMPLETION_INTENT_PAYLOAD_BYTES + 1];
        assert_eq!(
            CompletionIntent::from_stored_payload(&oversized, original.fingerprint()),
            Err(CompletionIntentLoadError::PayloadTooLarge)
        );
    }

    #[test]
    fn identical_and_conflicting_intents_have_distinct_outcomes() {
        let original = intent(serde_json::json!({"language": "en"}));
        let identical = original.clone();
        assert_eq!(
            compare_completion_intents(&original, &identical),
            CompletionIntentComparison::IdempotentMatch
        );

        let conflicting = intent(serde_json::json!({"language": "fr"}));
        assert!(matches!(
            compare_completion_intents(&original, &conflicting),
            CompletionIntentComparison::Conflict(_)
        ));
    }

    #[test]
    fn payload_size_is_bounded() {
        let oversized = intent(serde_json::json!({"data": "x"}));
        let mut payload = oversized.payload().clone();
        payload.mode_config = Value::String("x".repeat(MAX_COMPLETION_INTENT_PAYLOAD_BYTES));
        assert!(matches!(
            CompletionIntent::from_payload(payload),
            Err(CompletionIntentError::PayloadTooLarge { .. })
        ));
    }

    #[test]
    fn payload_bound_accepts_exact_size_and_rejects_one_byte_over() {
        let empty_config = intent(Value::String(String::new()));
        let empty_size = empty_config.canonical_payload().len();
        let exact_chars = MAX_COMPLETION_INTENT_PAYLOAD_BYTES - empty_size;

        let mut exact_payload = empty_config.payload().clone();
        exact_payload.mode_config = Value::String("x".repeat(exact_chars));
        let exact = CompletionIntent::from_payload(exact_payload).expect("exact bound is valid");
        assert_eq!(
            exact.canonical_payload().len(),
            MAX_COMPLETION_INTENT_PAYLOAD_BYTES
        );
        assert_eq!(
            CompletionIntent::from_stored_payload(exact.canonical_payload(), exact.fingerprint())
                .expect("exact canonical envelope should load")
                .fingerprint(),
            exact.fingerprint()
        );

        let mut oversized_payload = empty_config.payload().clone();
        oversized_payload.mode_config = Value::String("x".repeat(exact_chars + 1));
        assert_eq!(
            CompletionIntent::from_payload(oversized_payload),
            Err(CompletionIntentError::PayloadTooLarge {
                actual: MAX_COMPLETION_INTENT_PAYLOAD_BYTES + 1,
                maximum: MAX_COMPLETION_INTENT_PAYLOAD_BYTES
            })
        );

        let mut stored_one_byte_over = exact.canonical_payload().to_vec();
        stored_one_byte_over.push(b' ');
        assert_eq!(
            CompletionIntent::from_stored_payload(&stored_one_byte_over, exact.fingerprint()),
            Err(CompletionIntentLoadError::PayloadTooLarge)
        );
    }

    #[test]
    fn debug_and_error_output_redact_sensitive_payload_content() {
        let secret = "typed-secret-content";
        let intent = intent(serde_json::json!({"secret": secret}));
        let intent_debug = format!("{intent:?}");
        let payload_debug = format!("{:?}", intent.payload());
        let error = CompletionIntent::from_stored_payload(b"not-json", intent.fingerprint())
            .expect_err("corrupt payload should fail");
        let error_display = error.to_string();
        assert!(!intent_debug.contains(secret));
        assert!(!payload_debug.contains(secret));
        assert!(!error_display.contains(secret));
    }

    #[test]
    fn started_session_descriptor_is_bounded() {
        let result = StartedSession::new(
            SessionId::from("018f0c2e-7b8d-7abc-8def-0123456789ab"),
            "custom",
            Value::String("x".repeat(MAX_SESSION_DESCRIPTOR_BYTES)),
            "en",
            timestamp(1_752_969_600),
        );
        assert!(matches!(
            result,
            Err(StartedSessionError::DescriptorTooLarge { .. })
        ));
    }

    #[test]
    fn started_session_descriptor_allows_only_bounded_sanitized_metadata() {
        let allowed = serde_json::json!({
            "kind": "generated",
            "language": "en",
            "selection": {
                "source_id": "words-basic",
                "word_count": 25,
                "punctuation": false,
                "weights": [1, 2, null]
            }
        });
        assert!(StartedSession::new(
            SessionId::from("018f0c2e-7b8d-7abc-8def-0123456789ab"),
            "custom",
            allowed,
            "en",
            timestamp(1_752_969_600),
        )
        .is_ok());
    }

    #[test]
    fn started_session_descriptor_rejects_sensitive_content_recursively_and_redacts_diagnostics() {
        let secret = "descriptor-secret-content";
        let cases = [
            serde_json::json!({"text": secret}),
            serde_json::json!({"config": {"custom_text": secret}}),
            serde_json::json!({"items": [{"typedChars": secret}]}),
            serde_json::json!({"replay_frames": [{"key": secret}]}),
            serde_json::json!({"mode": {"expected-text": secret}}),
            serde_json::json!({"mode": {"Typed Text": secret}}),
        ];

        for descriptor in cases {
            let error = StartedSession::new(
                SessionId::from("018f0c2e-7b8d-7abc-8def-0123456789ab"),
                "custom",
                descriptor,
                "en",
                timestamp(1_752_969_600),
            )
            .expect_err("sensitive descriptor must be rejected");
            assert!(matches!(
                error,
                StartedSessionError::SensitiveDescriptorField
            ));
            assert!(!error.to_string().contains(secret));
            assert!(!format!("{error:?}").contains(secret));
        }

        let serialized = serde_json::json!({
            "session_id": "018f0c2e-7b8d-7abc-8def-0123456789ab",
            "mode_type": "custom",
            "mode_descriptor": {"text": secret},
            "language": "en",
            "started_at": "2025-07-16T12:00:00Z"
        });
        let deserialize_error = serde_json::from_value::<StartedSession>(serialized)
            .expect_err("deserialization must use the same privacy validation");
        assert!(!deserialize_error.to_string().contains(secret));
    }

    #[test]
    fn started_session_descriptor_rejects_depth_and_scalar_bounds() {
        let mut deep = serde_json::json!({"kind": "generated"});
        for _ in 0..=MAX_SESSION_DESCRIPTOR_DEPTH {
            deep = serde_json::json!({"config": deep});
        }
        assert!(matches!(
            validate_sanitized_session_descriptor(&deep),
            Err(StartedSessionError::DescriptorTooDeep { .. })
        ));

        let oversized_key = "k".repeat(MAX_SESSION_DESCRIPTOR_KEY_BYTES + 1);
        let mut descriptor = serde_json::Map::new();
        descriptor.insert(oversized_key, Value::Bool(true));
        assert!(matches!(
            validate_sanitized_session_descriptor(&Value::Object(descriptor)),
            Err(StartedSessionError::DescriptorKeyTooLong { .. })
        ));

        assert!(matches!(
            validate_sanitized_session_descriptor(&serde_json::json!({
                "kind": "x".repeat(MAX_SESSION_DESCRIPTOR_STRING_BYTES + 1)
            })),
            Err(StartedSessionError::DescriptorStringTooLong { .. })
        ));
        assert!(matches!(
            validate_sanitized_session_descriptor(&serde_json::json!({"label": "safe"})),
            Err(StartedSessionError::UnsafeDescriptorValue)
        ));
    }

    #[test]
    fn finalization_ledger_record_shapes_are_closed_and_metadata_only() {
        let session_id = SessionId::parse("018f0c2e-7b8d-7abc-8def-0123456789ab")
            .expect("fixture session identity");
        let fingerprint =
            CompletionIntentFingerprint::try_from_hex("a".repeat(64)).expect("fixture fingerprint");
        let claimed_at = timestamp(1_752_969_600);
        assert!(FinalizationRecord::new(
            session_id.clone(),
            fingerprint.clone(),
            FinalizationLedgerState::Pending,
            claimed_at,
            None,
            None,
        )
        .is_ok());
        assert!(FinalizationRecord::new(
            session_id,
            fingerprint,
            FinalizationLedgerState::Committed,
            claimed_at,
            None,
            None,
        )
        .is_err());
        assert_eq!(
            FinalizationLedgerState::from_storage_name("pending"),
            Some(FinalizationLedgerState::Pending)
        );
        assert_eq!(FinalizationLedgerState::from_storage_name("PENDING"), None);
    }
}
