// state.rs — Application state machine.
//
// State machine (ARCHITECTURE.md §2):
//   Idle         → not recording
//   Listening    → mic open, audio flowing into buffer
//   Transcribing → mic closed, engine running inference
//   Error        → something went wrong; message is user-facing
//
// SharedState = Arc<Mutex<AppState>> — shared between Tauri commands and hotkey handler.

use crate::engine::{AsrEngine, SessionId};
use crate::mic::CaptureHandle;
use crate::settings::{self, Settings};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// The four states.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status", content = "message")]
pub enum Status {
    Idle,
    Listening,
    Transcribing,
    Error(String),
}

/// Full runtime state.
pub struct AppState {
    pub status: Status,

    /// Active ASR session ID (set by start_dictation, cleared by stop_dictation).
    pub session_id: Option<SessionId>,

    /// Raw f32 PCM accumulates here while the mic is open.
    /// Shared with the cpal audio callback via Arc<Mutex<>>.
    pub audio_buffer: Arc<Mutex<Vec<f32>>>,

    /// Tracks live transcribed text inserted at the cursor for real-time diff calculations.
    pub last_streamed: Arc<Mutex<String>>,

    /// Owns the cpal stream. Dropping it stops the microphone.
    pub capture_handle: Option<CaptureHandle>,

    /// Initialised lazily on first dictation. Trait object so the engine can be
    /// chosen at runtime (Apple or whisper.cpp).
    pub engine: Option<Arc<dyn AsrEngine>>,

    /// Thread that forwards mic chunks to the speech engine.
    /// Joined on stop, after the capture handle (and its channel sender) is dropped.
    pub audio_feeder: Option<std::thread::JoinHandle<()>>,

    /// Current user settings (loaded from disk at startup, updated by set_settings).
    pub settings: Settings,

    /// Shown in Studio when the hotkey could not be registered. Not a crash.
    pub startup_warning: Option<String>,
}

pub fn unpoison<T>(result: std::sync::LockResult<T>) -> T {
    result.unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl AppState {
    pub fn new() -> Self {
        Self {
            status: Status::Idle,
            session_id: None,
            audio_buffer: Arc::new(Mutex::new(Vec::with_capacity(16_000 * 60))),
            last_streamed: Arc::new(Mutex::new(String::new())),
            capture_handle: None,
            engine: None,
            audio_feeder: None,
            settings: settings::load(),
            startup_warning: None,
        }
    }
}

/// The type stored in Tauri's managed state.
pub type SharedState = Arc<Mutex<AppState>>;

pub fn new_shared_state() -> SharedState {
    Arc::new(Mutex::new(AppState::new()))
}
