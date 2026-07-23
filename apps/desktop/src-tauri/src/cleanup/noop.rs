// cleanup/noop.rs — The default "no cleanup" backend.
//
// Returns the raw transcript unchanged.
// This is the default when the user has not configured a cleanup backend.
// It has zero network calls, zero latency overhead.

use super::{CleanupBackend, CleanupError};

pub struct NoOpCleanup;

#[async_trait::async_trait]
impl CleanupBackend for NoOpCleanup {
    async fn refine(&self, raw: &str) -> Result<String, CleanupError> {
        // Pass-through — no modification, no network call.
        Ok(raw.to_owned())
    }
}
