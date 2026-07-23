// engine/mod.rs — ASR Engine trait and runtime selector.
//
// The AsrEngine trait is the contract every ASR backend must satisfy.
// It is defined exactly as in ARCHITECTURE.md §4.
//
// Compile-time engine selection:
//   - aarch64-apple-darwin → AppleSiliconEngine (FluidAudio + Parakeet TDT v3)
//   - everything else      → WhisperCppEngine
//
// WHY compile-time and not runtime selection:
//   The two engines have different native library dependencies. Shipping both
//   in one binary would double the install size. The target triple already encodes
//   the hardware, so compile-time selection is correct and produces the smallest binary.

pub mod apple_silicon;
pub mod whisper_cpp;

use crate::error::AppError;

/// Opaque session handle returned by `start_session`.
/// u64 is sufficient for a per-process monotonic counter.
pub type SessionId = u64;

/// Configuration passed to `AsrEngine::init`.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Path to the model file. If empty, the engine uses its bundled default.
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
            EngineError::ModelLoad(m)    => AppError::ModelLoad(m),
            EngineError::ModelNotFound(m)=> AppError::ModelNotFound(m),
            EngineError::NotImplemented(m)=>AppError::Internal(format!("not implemented: {m}")),
            other                        => AppError::Internal(other.to_string()),
        }
    }
}

/// The core ASR engine contract.
/// All implementations must be Send + Sync — they are held behind Arc<dyn AsrEngine>.
pub trait AsrEngine: Send + Sync {
    /// Called once at startup. Loads the model into memory.
    /// Must be called before any other method.
    fn init(config: &EngineConfig) -> Result<Self, EngineError>
    where
        Self: Sized;

    /// Begin a new transcription session. Returns an opaque session handle.
    fn start_session(&self) -> Result<SessionId, EngineError>;

    /// Feed a chunk of PCM audio (16 kHz, mono, f32).
    /// Called approximately 10 times per second by the mic capture loop.
    fn feed_audio(&self, session: SessionId, samples: &[f32]) -> Result<(), EngineError>;

    /// Poll for any new partial transcript text. Non-blocking.
    /// Returns None if no new text is ready.
    fn poll_partial(&self, session: SessionId) -> Result<Option<String>, EngineError>;

    /// End the session and flush the final transcript.
    /// Blocks until the engine has finished processing all buffered audio.
    fn end_session(&self, session: SessionId) -> Result<String, EngineError>;
}

/// The concrete engine type used on this platform (selected at compile time).
///
/// On aarch64-apple-darwin: AppleSiliconEngine
/// Everywhere else:         WhisperCppEngine
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub type PlatformEngine = apple_silicon::AppleSiliconEngine;

#[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
pub type PlatformEngine = whisper_cpp::WhisperCppEngine;
