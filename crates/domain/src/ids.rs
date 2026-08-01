//! Идентификаторы сущностей.

use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type TestId = i64;
pub type LessonId = String;
pub type ModuleId = String;
pub type QuoteId = i64;

/// Immutable identity for a typing session.
///
/// New sessions use UUIDv7: the canonical representation is globally unique,
/// naturally sortable by creation time, and interoperable with databases and
/// future synchronization services. Legacy rows are accepted when they carry
/// the deterministic `legacy-test-` prefix created by the migration.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[serde(transparent)]
pub struct SessionId(String);

impl SessionId {
    /// Creates a new backend-owned UUIDv7 session identity.
    pub fn new() -> Self {
        Self(Uuid::now_v7().to_string())
    }

    /// Parses an identity received over a trust boundary.
    pub fn parse(value: impl AsRef<str>) -> Result<Self, SessionIdError> {
        let value = value.as_ref();
        if let Ok(uuid) = Uuid::parse_str(value) {
            if uuid.get_version_num() == 7 {
                return Ok(Self(uuid.to_string()));
            }
            return Err(SessionIdError::UnsupportedVersion);
        }

        if value.starts_with("legacy-test-")
            && value.len() <= 128
            && value.len() > "legacy-test-".len()
            && value[12..]
                .chars()
                .all(|character| character.is_ascii_hexdigit() || character == '-')
        {
            return Ok(Self(value.to_string()));
        }

        Err(SessionIdError::InvalidFormat)
    }

    /// Loads a value from a repository row after migration has established the
    /// compatibility prefix. This is intentionally separate from IPC parsing.
    pub fn from_storage(value: String) -> Self {
        Self(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for SessionId {
    type Err = SessionIdError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl From<SessionId> for String {
    fn from(value: SessionId) -> Self {
        value.0
    }
}

/// Trusted internal callers (including core tests) may provide a readable
/// fixture value. IPC deserialization uses `parse` and therefore remains
/// restricted to UUIDv7 or migrated legacy identities.
impl From<&str> for SessionId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for SessionId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl PartialEq<&str> for SessionId {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl<'de> Deserialize<'de> for SessionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionIdError {
    InvalidFormat,
    UnsupportedVersion,
}

impl fmt::Display for SessionIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidFormat => "session id must be a UUIDv7 or migrated legacy identity",
            Self::UnsupportedVersion => "session id must use UUIDv7",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_id_is_unique_ordered_and_v7() {
        let a = SessionId::new();
        let b = SessionId::new();
        assert_ne!(a, b);
        assert!(a < b);
        assert_eq!(Uuid::parse_str(a.as_str()).unwrap().get_version_num(), 7);
        assert_eq!(SessionId::parse(a.as_str()).unwrap(), a);
    }

    #[test]
    fn legacy_identity_is_accepted_only_for_storage_compatibility() {
        assert!(SessionId::parse("legacy-test-0000000000000001").is_ok());
        assert!(SessionId::parse("timestamp-123").is_err());
    }
}
