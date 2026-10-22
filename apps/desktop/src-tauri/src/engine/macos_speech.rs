// engine/macos_speech.rs — macOS Speech Recognition engine (Phase 2).
//
// Uses the macOS SFSpeechRecognizer (CoreML/ANE) via a compiled Swift static library.
//
// The Swift code is linked statically into the Rust binary, allowing the Rust app to
// directly use SFSpeechRecognizer without TCC permission separation issues.

use super::{AsrEngine, EngineConfig, EngineError, SessionId};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::os::raw::c_char;
use std::ffi::CStr;
use std::sync::mpsc;

#[link(name = "utter_transcribe", kind = "static")]
extern "C" {
    fn speech_init_stream(
        sample_rate: f64,
        context_ptr: *mut std::ffi::c_void,
        callback: extern "C" fn(*mut std::ffi::c_void, *const c_char, bool),
    ) -> *mut std::ffi::c_void;

    fn speech_append_audio(stream_ptr: *mut std::ffi::c_void, samples: *const f32, count: usize);
    fn speech_end_stream(stream_ptr: *mut std::ffi::c_void);
    fn speech_free_stream(stream_ptr: *mut std::ffi::c_void);
}

// --------------------------------------------------------------------------
// Callback State & C FFI Wrapper
// --------------------------------------------------------------------------

type CallbackBox = Box<dyn Fn(String, bool) + Send + 'static>;

struct CallbackState {
    on_partial: CallbackBox,
    final_tx: mpsc::Sender<String>,
    latest_transcript: Mutex<String>,
}

extern "C" fn swift_callback(context: *mut std::ffi::c_void, text: *const c_char, is_final: bool) {
    if context.is_null() { return; }
    let state = unsafe { &*(context as *const CallbackState) };
    
    let transcript = if text.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(text).to_string_lossy().into_owned() }
    };
    
    if !transcript.is_empty() {
        *state.latest_transcript.lock().unwrap() = transcript.clone();
    }
    
    // Call the Rust closure
    (state.on_partial)(transcript.clone(), is_final);
    
    if is_final {
        let final_str = state.latest_transcript.lock().unwrap().clone();
        let _ = state.final_tx.send(final_str);
    }
}

// --------------------------------------------------------------------------
// Engine struct
// --------------------------------------------------------------------------

struct StreamingSession {
    swift_ptr: *mut std::ffi::c_void,
    state_ptr: *mut CallbackState,
    final_rx: mpsc::Receiver<String>,
}

unsafe impl Send for StreamingSession {}
unsafe impl Sync for StreamingSession {}

pub struct MacOsSpeechEngine {
    sessions: Mutex<HashMap<SessionId, StreamingSession>>,
    next_id: AtomicU64,
}

// --------------------------------------------------------------------------
// AsrEngine Implementation
// --------------------------------------------------------------------------

impl AsrEngine for MacOsSpeechEngine {
    fn init(_config: &EngineConfig) -> Result<Self, EngineError> {
        log::info!("macos_speech: initialized streaming Swift engine");
        Ok(Self {
            sessions: Mutex::new(HashMap::new()),
            next_id: AtomicU64::new(1),
        })
    }

    fn start_session(
        &self, 
        sample_rate: u32,
        on_partial: Box<dyn Fn(String, bool) + Send + 'static>
    ) -> Result<SessionId, EngineError> {
        let (final_tx, final_rx) = mpsc::channel();
        
        let state = Box::new(CallbackState {
            on_partial,
            final_tx,
            latest_transcript: Mutex::new(String::new()),
        });
        
        let state_ptr = Box::into_raw(state);
        
        let swift_ptr = unsafe {
            speech_init_stream(
                sample_rate as f64,
                state_ptr as *mut std::ffi::c_void,
                swift_callback
            )
        };
        
        if swift_ptr.is_null() {
            unsafe { let _ = Box::from_raw(state_ptr); } // free it
            return Err(EngineError::Session("Failed to init Swift StreamingRecognizer (check permissions)".to_string()));
        }

        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let mut sessions = self.sessions.lock().expect("session lock poisoned");
        sessions.insert(id, StreamingSession {
            swift_ptr,
            state_ptr,
            final_rx,
        });
        
        log::debug!("macos_speech: stream session {id} started ({} Hz)", sample_rate);
        Ok(id)
    }

    fn feed_audio(&self, session: SessionId, samples: &[f32]) -> Result<(), EngineError> {
        let sessions = self.sessions.lock().expect("session lock poisoned");
        let s = sessions.get(&session).ok_or_else(|| EngineError::Session(format!("unknown session {session}")))?;
        
        if !samples.is_empty() {
            unsafe {
                speech_append_audio(s.swift_ptr, samples.as_ptr(), samples.len());
            }
        }
        Ok(())
    }

    fn end_session(&self, session: SessionId) -> Result<String, EngineError> {
        let s = {
            let mut sessions = self.sessions.lock().expect("session lock poisoned");
            sessions.remove(&session).ok_or_else(|| EngineError::Session(format!("unknown session {session}")))?
        };

        log::debug!("macos_speech: ending stream session {session}");
        
        // Tell Swift to finish the audio buffer
        unsafe { speech_end_stream(s.swift_ptr); }
        
        // Wait for final callback with timeout to guarantee no indefinite hangs
        let final_transcript = s.final_rx
            .recv_timeout(std::time::Duration::from_millis(600))
            .unwrap_or_else(|_| {
                let state = unsafe { &*s.state_ptr };
                state.latest_transcript.lock().unwrap().clone()
            });
        
        // Cleanup memory
        unsafe {
            speech_free_stream(s.swift_ptr);
            let _ = Box::from_raw(s.state_ptr); // drops the state and the on_partial closure
        }
        
        Ok(final_transcript.trim().to_string())
    }
}
