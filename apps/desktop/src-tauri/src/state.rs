// state.rs — Application state machine.
//
// The state machine has four states:
//   Idle          — app is running, not dictating
//   Listening     — mic is open, ASR session active, audio flowing
//   Transcribing  — audio ended, waiting for final transcript flush
//   Error(String) — something went wrong; message is user-facing
//
// State is stored in Arc<Mutex<AppState>> so it can be shared across
// async Tauri commands and the hotkey callback without data races.
//
// WHY Arc<Mutex<>> and not Arc<RwLock<>>:
//   Writes (state transitions) are infrequent; lock contention is negligible.
//   Mutex is simpler and its poisoning behaviour is easier to reason about.

use crate::engine::SessionId;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// The four states of the dictation state machine.
/// Serialisable so it can be sent to the webview via Tauri events.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status", content = "message")]
pub enum Status {
    Idle,
    Listening,
    Transcribing,
    Error(String),
}

/// Full runtime state, held behind Arc<Mutex<>>.
pub struct AppState {
    pub status:     Status,
    /// Active ASR session ID, if any.
    pub session_id: Option<SessionId>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            status:     Status::Idle,
            session_id: None,
        }
    }
}

/// Convenience type alias — this is what gets stored in Tauri's managed state.
pub type SharedState = Arc<Mutex<AppState>>;

pub fn new_shared_state() -> SharedState {
    Arc::new(Mutex::new(AppState::new()))
}
