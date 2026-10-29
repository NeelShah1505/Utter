// engine/whisper_cpp.rs — whisper.cpp engine stub.
//
// Full implementation: Phase 3.
// This engine wraps whisper.cpp via Rust FFI for macOS Intel and Windows.
// On Windows with an NVIDIA GPU, CUDA acceleration is used if detected.
//
// WHY whisper.cpp (from DESIGN.md §2.3):
//   - Best-maintained CPU STT in open source. SIMD-optimised (AVX2/NEON).
//   - CUDA path for NVIDIA GPUs on Windows.
//   - One codebase handles x86_64 macOS, x86_64 Windows, aarch64 Windows.
//   - Faster-whisper requires a Python runtime — we are not shipping Python.
//
// TODO: Implement in Phase 3 via the `whisper-rs` crate (safe Rust bindings to whisper.cpp).
//       Track in CONTEXT.md §Session Log when work begins.

use super::{AsrEngine, EngineConfig, EngineError, SessionId};

pub struct WhisperCppEngine {
    // TODO: whisper_rs::WhisperContext will go here in Phase 3.
    _model_path: String,
}

impl AsrEngine for WhisperCppEngine {
    fn init(_config: &EngineConfig) -> Result<Self, EngineError> {
        // TODO (Phase 3): Load whisper.cpp model via whisper-rs crate.
        // Example pseudo-code:
        //   let ctx = WhisperContext::new_with_params(
        //       &config.model_path,
        //       WhisperContextParameters::default(),
        //   ).map_err(|e| EngineError::ModelLoad(e.to_string()))?;
        //   Ok(Self { ctx })
        Err(EngineError::NotImplemented(
            "This build only dictates on macOS, using Apple's on-device speech recognizer. \
             Windows and Intel Mac installers are not shipped."
                .into(),
        ))
    }

    fn start_session(
        &self, 
        _sample_rate: u32,
        _on_partial: Box<dyn Fn(String, bool) + Send + 'static>
    ) -> Result<SessionId, EngineError> {
        Ok(1)
    }

    fn feed_audio(&self, _session: SessionId, _samples: &[f32]) -> Result<(), EngineError> {
        Err(EngineError::NotImplemented(
            "WhisperCppEngine::feed_audio — Phase 3".into(),
        ))
    }

    fn end_session(&self, _session: SessionId) -> Result<String, EngineError> {
        Err(EngineError::NotImplemented(
            "WhisperCppEngine::end_session — Phase 3".into(),
        ))
    }
}
