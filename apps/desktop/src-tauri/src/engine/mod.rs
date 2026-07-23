// engine/mod.rs — ASR Engine trait and runtime selector.
//
// Platform dispatch (compile-time):
//   aarch64-apple-darwin  → MacOsSpeechEngine (SFSpeechRecognizer via CoreML/ANE)
//   x86_64-apple-darwin   → MacOsSpeechEngine (SFSpeechRecognizer, no ANE but offline)
//   Windows / Linux       → WhisperCppEngine  (whisper.cpp — Phase 3)
//
// The macOS engines share the same Swift helper (utter-transcribe).
// On Apple Silicon the helper uses the ANE automatically.

pub mod apple_silicon;
pub mod macos_speech;
pub mod whisper_cpp;

use crate::error::AppError;

/// Opaque session handle. u64 is sufficient for a per-process monotonic counter.
pub type SessionId = u64;

/// Configuration passed to `AsrEngine::init`.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Path to the model file. Empty = use platform default.
    pub model_path: String,
}

/// Errors specific to the ASR engine layer.
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("Model load failed: {0}")]
    ModelLoad(String),

    #[error("Model not found at path: {0}")]
    ModelNotFound(String),

    #[error("Session error: {0}")]
    Session(String),

    #[error("Audio feed error: {0}")]
    AudioFeed(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

impl From<EngineError> for AppError {
    fn from(e: EngineError) -> Self {
        match e {
            EngineError::ModelLoad(m)     => AppError::ModelLoad(m),
            EngineError::ModelNotFound(m) => AppError::ModelNotFound(m),
            EngineError::NotImplemented(m)=> AppError::Internal(format!("not implemented: {m}")),
            other                         => AppError::Internal(other.to_string()),
        }
    }
}

/// The core ASR engine contract.
/// All implementations must be Send + Sync — they are held behind Arc<dyn AsrEngine>.
pub trait AsrEngine: Send + Sync {
    /// Called once at startup. Loads the model into memory (or verifies helper exists).
    fn init(config: &EngineConfig) -> Result<Self, EngineError>
    where
        Self: Sized;

    /// Begin a new transcription session. Returns an opaque session handle.
    fn start_session(&self) -> Result<SessionId, EngineError>;

    /// Feed a chunk of PCM audio (16 kHz, mono, f32).
    fn feed_audio(&self, session: SessionId, samples: &[f32]) -> Result<(), EngineError>;

    /// Poll for a new partial transcript. Non-blocking. Returns None if not ready.
    fn poll_partial(&self, session: SessionId) -> Result<Option<String>, EngineError>;

    /// End the session and flush the final transcript.
    fn end_session(&self, session: SessionId) -> Result<String, EngineError>;
}

// ---------------------------------------------------------------------------
// Compile-time platform selection
// ---------------------------------------------------------------------------

/// The concrete engine used on macOS (both arm64 and x86_64).
#[cfg(target_os = "macos")]
pub type PlatformEngine = macos_speech::MacOsSpeechEngine;

/// The concrete engine used on Windows/Linux (Phase 3 — currently stubs).
#[cfg(not(target_os = "macos"))]
pub type PlatformEngine = whisper_cpp::WhisperCppEngine;
