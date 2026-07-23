// engine/macos_speech.rs — macOS Speech Recognition engine (Phase 2).
//
// Uses the macOS SFSpeechRecognizer (CoreML/ANE) via a compiled Swift helper binary
// bundled inside the app: Contents/MacOS/utter-transcribe.
//
// WHY a subprocess and not direct FFI:
//   1. SFSpeechRecognizer is an Objective-C async framework. Direct FFI from Rust
//      requires manual Dispatch queue management, callback bridging, and ARC memory
//      management. The Swift helper is ~80 LOC and compiles to a native binary with
//      zero runtime overhead beyond process spawn (~5ms).
//   2. The helper has a clean stdin/stdout protocol: send raw f32 PCM → receive text.
//      This is easy to test independently (no Rust toolchain needed).
//   3. The same pattern used by Whisper.app, MacWhisper, and other macOS voice apps.
//
// On-device: SFSpeechRecognizer with requiresOnDeviceRecognition = true.
//   - Completely offline. No Siri calls. No network.
//   - CoreML model is shipped with macOS. Nothing to download.
//   - Apple Neural Engine (ANE) acceleration on Apple Silicon.
//   - Automatic punctuation included (macOS 13+).
//
// Audio protocol:
//   stdin: raw f32 samples (little-endian), 16 kHz, mono
//   stdout: transcript text with trailing newline
//   exit 0: success (transcript may be empty)
//   exit 1: fatal error (printed to stderr)

use super::{AsrEngine, EngineConfig, EngineError, SessionId};
use std::collections::HashMap;
use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

// --------------------------------------------------------------------------
// Engine struct
// --------------------------------------------------------------------------

pub struct MacOsSpeechEngine {
    /// Path to the compiled utter-transcribe binary.
    helper_path: String,
    /// Per-session audio buffers. Cleared when the session ends.
    sessions: Mutex<HashMap<SessionId, Vec<f32>>>,
    /// Monotonic session ID counter.
    next_id: AtomicU64,
}

impl MacOsSpeechEngine {
    /// Resolve the path to the utter-transcribe helper binary.
    /// During dev (`cargo tauri dev`): the binary is in helpers/.
    /// In a release bundle: it is at Contents/MacOS/utter-transcribe.
    fn resolve_helper_path(_config: &EngineConfig) -> Result<String, EngineError> {
        // 1. Check alongside the main executable (release bundle)
        if let Ok(exe) = std::env::current_exe() {
            let bundled = exe.parent().unwrap_or(&exe).join("utter-transcribe");
            if bundled.exists() {
                return Ok(bundled.to_string_lossy().into_owned());
            }
        }

        // 2. Check in the helpers/ directory relative to src-tauri/ (dev mode)
        let candidates = [
            // Running from src-tauri/target/debug/
            "../../helpers/utter-transcribe",
            // Running from project root
            "apps/desktop/src-tauri/helpers/utter-transcribe",
        ];
        for c in &candidates {
            let p = std::path::Path::new(c);
            if p.exists() {
                return Ok(p.to_string_lossy().into_owned());
            }
        }

        Err(EngineError::ModelNotFound(
            "utter-transcribe helper not found. \
             In dev mode, run: swiftc -O -target arm64-apple-macos13.0 \
             -framework Speech -framework AVFoundation -framework Foundation \
             apps/desktop/src-tauri/helpers/utter-transcribe.swift \
             -o apps/desktop/src-tauri/helpers/utter-transcribe"
                .into(),
        ))
    }
}

// --------------------------------------------------------------------------
// AsrEngine implementation
// --------------------------------------------------------------------------

impl AsrEngine for MacOsSpeechEngine {
    fn init(config: &EngineConfig) -> Result<Self, EngineError> {
        let helper_path = Self::resolve_helper_path(config)?;
        log::info!("macos_speech: helper at {helper_path}");

        // Verify the binary exists and is executable
        let meta = std::fs::metadata(&helper_path)
            .map_err(|e| EngineError::ModelLoad(format!("helper not accessible: {e}")))?;

        if !meta.is_file() {
            return Err(EngineError::ModelLoad(format!(
                "helper path is not a file: {helper_path}"
            )));
        }

        Ok(Self {
            helper_path,
            sessions: Mutex::new(HashMap::new()),
            next_id: AtomicU64::new(1),
        })
    }

    fn start_session(&self) -> Result<SessionId, EngineError> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let mut sessions = self.sessions.lock().expect("session lock poisoned");
        sessions.insert(id, Vec::with_capacity(16_000 * 30)); // pre-alloc 30s
        log::debug!("macos_speech: session {id} started");
        Ok(id)
    }

    fn feed_audio(&self, session: SessionId, samples: &[f32]) -> Result<(), EngineError> {
        let mut sessions = self.sessions.lock().expect("session lock poisoned");
        let buf = sessions
            .get_mut(&session)
            .ok_or_else(|| EngineError::Session(format!("unknown session {session}")))?;
        buf.extend_from_slice(samples);
        Ok(())
    }

    fn poll_partial(&self, _session: SessionId) -> Result<Option<String>, EngineError> {
        // SFSpeechRecognizer processes the full buffer at end_session.
        // Streaming partials would require sending chunks incrementally —
        // implement in Phase 2.5 if needed.
        Ok(None)
    }

    fn end_session(&self, session: SessionId) -> Result<String, EngineError> {
        // Take the audio buffer out of the session map
        let samples = {
            let mut sessions = self.sessions.lock().expect("session lock poisoned");
            sessions
                .remove(&session)
                .ok_or_else(|| EngineError::Session(format!("unknown session {session}")))?
        };

        log::info!(
            "macos_speech: end_session {session} — {} samples ({:.1}s)",
            samples.len(),
            samples.len() as f32 / 16000.0
        );

        if samples.is_empty() {
            return Ok(String::new());
        }

        // Spawn the helper and pipe raw f32 PCM to its stdin
        let mut child = std::process::Command::new(&self.helper_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| EngineError::Session(format!("spawn helper: {e}")))?;

        // Write f32 samples as raw little-endian bytes
        if let Some(mut stdin) = child.stdin.take() {
            let bytes: Vec<u8> = samples
                .iter()
                .flat_map(|s| s.to_le_bytes())
                .collect();
            stdin
                .write_all(&bytes)
                .map_err(|e| EngineError::Session(format!("write audio: {e}")))?;
            // stdin drops here → EOF sent to helper
        }

        let output = child
            .wait_with_output()
            .map_err(|e| EngineError::Session(format!("wait helper: {e}")))?;

        if !output.stderr.is_empty() {
            let err_str = String::from_utf8_lossy(&output.stderr);
            for line in err_str.lines() {
                if line.starts_with("error:") {
                    log::error!("utter-transcribe: {line}");
                } else {
                    log::warn!("utter-transcribe: {line}");
                }
            }
        }

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(EngineError::Session(format!(
                "helper exited {}: {}",
                output.status,
                stderr.trim()
            )));
        }

        let transcript = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_owned();

        log::info!("macos_speech: transcript = {:?}", transcript);
        Ok(transcript)
    }
}
