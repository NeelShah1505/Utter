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
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Opaque session handle. u64 is sufficient for a per-process monotonic counter.
pub type SessionId = u64;

/// Which speech engine to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum EngineKind {
    /// Apple SFSpeechRecognizer — zero download, macOS only, en-US.
    #[default]
    Apple,
    /// whisper.cpp — needs a model file, higher accuracy, cross-platform.
    Whisper,
}

/// Configuration passed to `AsrEngine::init`.
#[derive(Debug, Clone, Default)]
pub struct EngineConfig {
    /// Path to the model file. Empty = use platform default.
    pub model_path: String,
    /// Recognition bias (whisper `initial_prompt`): custom vocabulary so proper
    /// nouns and code terms are recognized during decoding, not patched after.
    pub prompt_hint: String,
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
            EngineError::ModelLoad(m) => AppError::ModelLoad(m),
            EngineError::ModelNotFound(m) => AppError::ModelNotFound(m),
            EngineError::NotImplemented(m) => AppError::EngineUnavailable(m),
            other => AppError::Internal(other.to_string()),
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
    /// `on_partial` is called asynchronously with (transcript, is_final) whenever a new result is available.
    fn start_session(
        &self,
        sample_rate: u32,
        on_partial: Box<dyn Fn(String, bool) + Send + 'static>,
    ) -> Result<SessionId, EngineError>;

    /// Feed a chunk of PCM audio (f32 format).
    fn feed_audio(&self, session: SessionId, samples: &[f32]) -> Result<(), EngineError>;

    /// End the session and flush the final transcript.
    fn end_session(&self, session: SessionId) -> Result<String, EngineError>;
}

// ---------------------------------------------------------------------------
// Runtime engine selection
// ---------------------------------------------------------------------------

/// Build the engine the user selected. `init` is not object-safe (it returns
/// `Self`), so we call it on the concrete type here and hand back a trait object
/// the rest of the app stores behind `Arc<dyn AsrEngine>`.
pub fn create_engine(
    kind: EngineKind,
    config: &EngineConfig,
) -> Result<Arc<dyn AsrEngine>, EngineError> {
    match kind {
        #[cfg(target_os = "macos")]
        EngineKind::Apple => Ok(Arc::new(macos_speech::MacOsSpeechEngine::init(config)?)),
        #[cfg(not(target_os = "macos"))]
        EngineKind::Apple => Err(EngineError::NotImplemented(
            "Apple speech recognition is only available on macOS.".into(),
        )),
        EngineKind::Whisper => Ok(Arc::new(whisper_cpp::WhisperCppEngine::init(config)?)),
    }
}
