//! Production runtime-value adapter used by embedded resource selection.

use racoon_application::SessionRandomSource;

/// Supplies the existing system-time-based runtime values used by resource
/// selection when no application-owned source is supplied.
pub struct SystemRandomSource;

impl SessionRandomSource for SystemRandomSource {
    fn next_u64(&mut self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};

        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }
}
