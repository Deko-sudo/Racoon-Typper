//! Идентификаторы сущностей.

use serde::{Deserialize, Serialize};

pub type TestId = i64;
pub type LessonId = String;
pub type ModuleId = String;
pub type SessionId = String;
pub type QuoteId = i64;

/// Новый тип для session_id (UUID).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SessionIdNew(pub String);

impl SessionIdNew {
    pub fn new() -> Self {
        Self(uuid_like())
    }
}

impl Default for SessionIdNew {
    fn default() -> Self {
        Self::new()
    }
}

fn uuid_like() -> String {
    // Простой генератор без uuid crate — достаточно для MVP
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:016x}", ts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_id_is_unique() {
        let a = SessionIdNew::new();
        std::thread::sleep(std::time::Duration::from_micros(1));
        let b = SessionIdNew::new();
        assert_ne!(a.0, b.0);
    }
}
