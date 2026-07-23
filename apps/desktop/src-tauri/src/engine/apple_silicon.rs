// engine/apple_silicon.rs — Apple Silicon ASR engine stub.
//
// Full implementation: Phase 2.
// This engine uses FluidAudio + Parakeet TDT v3 via CoreML/ANE for streaming
// transcription on Apple Silicon (M1, M2, M3, M4).
//
// WHY FluidAudio + Parakeet TDT v3 (from DESIGN.md §2.2):
//   - FluidAudio is a streaming-first ASR framework optimised for Apple's ANE.
//     Real-time factor <0.3 on M1 — 1 second of audio transcribes in <0.3s.
//   - Parakeet TDT v3: best WER per FLOP in its size class, MIT/Apache licensed,
//     CoreML representation available for native ANE acceleration.
//   - Whisper on Apple Silicon is batch-oriented; streaming requires windowing hacks
//     that hurt latency. Parakeet TDT is token-and-duration transducer, natively streaming.
//
// TODO: Implement in Phase 2 once FluidAudio Rust bindings are available.
//       Track in CONTEXT.md §Session Log when work begins.

use super::{AsrEngine, EngineConfig, EngineError, SessionId};

pub struct AppleSiliconEngine {
    // TODO: FluidAudio session handle will go here in Phase 2.
    _model_path: String,
}

impl AsrEngine for AppleSiliconEngine {
    fn init(_config: &EngineConfig) -> Result<Self, EngineError> {
        // TODO (Phase 2): Load Parakeet TDT v3 CoreML model via FluidAudio.
        // The model path comes from config.model_path or a bundled default.
        // Example pseudo-code:
        //   let session = FluidAudio::load_model(&config.model_path)?;
        //   Ok(Self { session })
        log::error!(
            "AppleSiliconEngine::init called but FluidAudio + Parakeet TDT \
             CoreML binding is not yet implemented. Phase 2 required."
        );
        panic!(
            "not implemented: FluidAudio + Parakeet TDT CoreML binding \
             (Phase 2). See ARCHITECTURE.md §4 and CONTEXT.md."
        );
        #[allow(unreachable_code)]
        Ok(Self { _model_path: _config.model_path.clone() })
    }

    fn start_session(&self) -> Result<SessionId, EngineError> {
        Err(EngineError::NotImplemented(
            "AppleSiliconEngine::start_session — Phase 2".into(),
        ))
    }

    fn feed_audio(&self, _session: SessionId, _samples: &[f32]) -> Result<(), EngineError> {
        Err(EngineError::NotImplemented(
            "AppleSiliconEngine::feed_audio — Phase 2".into(),
        ))
    }

    fn poll_partial(&self, _session: SessionId) -> Result<Option<String>, EngineError> {
        Err(EngineError::NotImplemented(
            "AppleSiliconEngine::poll_partial — Phase 2".into(),
        ))
    }

    fn end_session(&self, _session: SessionId) -> Result<String, EngineError> {
        Err(EngineError::NotImplemented(
            "AppleSiliconEngine::end_session — Phase 2".into(),
        ))
    }
}
