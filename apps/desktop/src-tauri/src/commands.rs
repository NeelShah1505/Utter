// commands.rs — All Tauri IPC commands.
//
// Full pipeline (Phase 2):
//   start_dictation → open mic → feed audio → buffer samples
//   stop_dictation  → drop mic → engine.end_session() → cleanup → insert_text → emit event
//
// The engine is initialised lazily on first start_dictation call.

use crate::{
    cleanup,
    engine::{AsrEngine, EngineConfig, EngineError, PlatformEngine},
    error::{AppError, Result},
    insert,
    mic,
    settings::{self, Settings},
    state::{SharedState, Status},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};

// ---------------------------------------------------------------------------
// Dictation control
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Core dictation logic (public so hotkey/mod.rs can call directly)
// ---------------------------------------------------------------------------

/// Core logic for starting dictation.
/// Called by both the IPC command and the hotkey handler.
pub async fn start_dictation_impl(app: &AppHandle, state: &SharedState) -> Result<()> {
    let (current_status, engine, settings) = {
        let locked = state.lock().expect("state poisoned");
        (locked.status.clone(), locked.engine.clone(), locked.settings.clone())
    };

    match current_status {
        Status::Listening | Status::Transcribing => return Ok(()),
        _ => {}
    }

    // Initialise engine lazily on first use
    let engine: Arc<PlatformEngine> = match engine {
        Some(e) => e,
        None => {
            let config = EngineConfig { model_path: settings.model_path.clone() };
            let eng = PlatformEngine::init(&config).map_err(AppError::from)?;
            let arc = Arc::new(eng);
            state.lock().expect("state poisoned").engine = Some(arc.clone());
            arc
        }
    };

    log::info!("Finding audio device...");
    let device = mic::find_device(&settings.audio_device)?;
    log::info!("Found device. Getting preferred config...");
    let mic_config = mic::preferred_config(&device)?;
    let sample_rate = mic_config.sample_rate().0;
    log::info!("Mic config: {} Hz", sample_rate);

    let last_streamed_arc = {
        let locked = state.lock().expect("state poisoned");
        locked.last_streamed.lock().expect("last_streamed poisoned").clear();
        locked.last_streamed.clone()
    };

    let app_handle = app.clone();
    let stream_buffer = last_streamed_arc.clone();
    let on_partial = Box::new(move |transcript: String, is_final: bool| {
        log::info!("Live partial: {}", transcript);
        let _ = app_handle.emit("transcript_partial", serde_json::json!({ "text": transcript, "is_final": is_final }));
        
        // Real-time live cursor insertion!
        let mut old = stream_buffer.lock().expect("stream buffer poisoned");
        if !transcript.is_empty() && &*old != &transcript {
            let _ = crate::insert::insert_delta(&old, &transcript);
            *old = transcript;
        }
    });

    log::info!("Starting ASR session...");
    let session_id = engine.start_session(sample_rate, on_partial).map_err(AppError::from)?;
    log::info!("ASR session started! ID: {session_id}");

    let audio_buffer = {
        let locked = state.lock().expect("state poisoned");
        locked.audio_buffer.lock().expect("audio lock poisoned").clear();
        locked.audio_buffer.clone()
    };

    let engine_for_capture = engine.clone();
    let mut engine_buf = Vec::with_capacity(8192);
    let mut received_first_chunk = false;
    let capture = mic::CaptureHandle::start(&device, mic_config, move |chunk| {
        if !received_first_chunk {
            log::info!("Microphone successfully sent first audio chunk! (len: {})", chunk.len());
            received_first_chunk = true;
        }
        let mut buf = audio_buffer.lock().expect("audio lock poisoned");
        buf.extend_from_slice(chunk);
        
        // Accumulate into ~185ms chunks before feeding to Swift to force partials
        engine_buf.extend_from_slice(chunk);
        if engine_buf.len() >= 8192 {
            let _ = engine_for_capture.feed_audio(session_id, &engine_buf);
            engine_buf.clear();
        }
    })?;

    {
        let mut locked = state.lock().expect("state poisoned");
        locked.status         = Status::Listening;
        locked.session_id     = Some(session_id);
        locked.capture_handle = Some(capture);
    }

    log::info!("Emitting state_change 'Listening' to frontend...");
    let _ = app.emit("state_change", serde_json::json!({ "status": "Listening" }));
    
    // Show the main bubble window without stealing focus from user's active editor!
    if let Some(window) = app.get_webview_window("main") {
        log::info!("Showing bubble window...");
        let _ = window.show();
        // Explicitly do not call window.set_focus() so cursor remains in Word/Browser/IDE
    } else {
        log::error!("CRITICAL: 'main' window not found!");
    }
    
    log::info!("start_dictation_impl completed successfully.");

    log::info!("start_dictation: mic open, session {session_id}");
    Ok(())
}

// ---------------------------------------------------------------------------
// IPC command wrappers (thin — just unwrap Tauri State and delegate)
// ---------------------------------------------------------------------------

/// Start dictation (IPC). Transitions: Idle → Listening.
#[tauri::command]
pub async fn start_dictation(
    app: AppHandle,
    state: State<'_, SharedState>,
) -> Result<()> {
    start_dictation_impl(&app, &state).await
}


/// Core logic for stopping dictation.
/// Called by both the IPC command and the hotkey handler.
pub async fn stop_dictation_impl(app: &AppHandle, state: &SharedState) -> Result<()> {
    // --- Grab what we need and stop the mic (brief lock) ---
    let (engine, session_id, audio_buffer, cleanup_cfg, dictionary, last_streamed_arc) = {
        let mut locked = state.lock().expect("state poisoned");

        if locked.status != Status::Listening {
            return Ok(()); // idempotent
        }

        // Drop the capture handle → cpal stream stops → mic closes
        let _ = locked.capture_handle.take();
        locked.status = Status::Transcribing;

        (
            locked.engine.clone(),
            locked.session_id.take(),
            locked.audio_buffer.clone(),
            locked.settings.cleanup.clone(),
            locked.settings.dictionary.clone(),
            locked.last_streamed.clone(),
        )
    };

    let _ = app.emit("state_change", serde_json::json!({ "status": "Transcribing" }));

    // --- Validate engine and session ---
    let engine = match engine {
        Some(e) => e,
        None => {
            let msg = "Engine not initialised".to_string();
            let _ = app.emit("state_change", serde_json::json!({ "status": "Error", "message": msg }));
            let mut locked = state.lock().expect("state poisoned");
            locked.status = Status::Error(msg.clone());
            return Err(AppError::Internal(msg));
        }
    };

    let session_id = match session_id {
        Some(id) => id,
        None => {
            let mut locked = state.lock().expect("state poisoned");
            locked.status = Status::Idle;
            return Ok(());
        }
    };

    // --- Feed buffered audio to engine, then finalise ---
    let samples: Vec<f32> = {
        let buf = audio_buffer.lock().expect("audio lock poisoned");
        buf.clone()
    };

    log::info!("stop_dictation: {:.1}s of audio, running ASR…", samples.len() as f32 / 16000.0);

    let engine_clone = engine.clone();
    let raw = tokio::task::spawn_blocking(move || {
        engine_clone.end_session(session_id)
    }).await.unwrap_or_else(|e| Err(EngineError::Session(format!("spawn_blocking failed: {}", e)))).map_err(|e| {
        let err = AppError::from(e);
        let mut locked = state.lock().expect("state poisoned");
        locked.status = Status::Error(err.to_string());
        err
    })?;

    log::info!("stop_dictation: raw = {:?}", raw);

    // Snapshot what was actually typed to the cursor during real-time streaming
    let streamed_str = last_streamed_arc.lock().expect("stream poisoned").clone();

    // --- Optional cleanup pass ---
    let backend = cleanup::build_backend(&cleanup_cfg);
    let mut final_text = if raw.is_empty() {
        String::new()
    } else {
        backend.refine(&raw).await.map_err(|e| {
            let err = AppError::from(e);
            log::warn!("cleanup failed: {err} — using raw transcript");
            err
        })
        .unwrap_or(raw.clone())
    };

    // --- Apply custom dictionary rules ---
    for item in &dictionary {
        if let Some(ref rep) = item.replacement {
            if !rep.is_empty() && !item.word.is_empty() {
                final_text = final_text.replace(&item.word, rep);
                final_text = final_text.replace(&item.word.to_lowercase(), rep);
                let mut chars = item.word.chars();
                if let Some(first) = chars.next() {
                    let cap_word = format!("{}{}", first.to_uppercase(), chars.as_str());
                    final_text = final_text.replace(&cap_word, rep);
                }
            }
        }
    }

    log::info!("stop_dictation: final = {:?}", final_text);

    // --- Reconcile streamed text with final cleaned text ---
    if !final_text.is_empty() {
        if let Err(e) = insert::insert_delta(&streamed_str, &final_text) {
            log::warn!("insert_delta failed: {e} — falling back to standard insert");
            if streamed_str.is_empty() {
                let _ = insert::insert_text(&final_text);
            }
        }
    } else if !streamed_str.is_empty() {
        let _ = insert::insert_delta(&streamed_str, "");
    }

    // --- Emit final transcript event (UI shows it even if insert fails) ---
    let _ = app.emit(
        "transcript_final",
        serde_json::json!({ "text": final_text }),
    );

    // --- Return to Idle ---
    {
        let mut locked = state.lock().expect("state poisoned");
        locked.status = Status::Idle;
        locked.last_streamed.lock().expect("stream poisoned").clear();
    }
    let _ = app.emit("state_change", serde_json::json!({ "status": "Idle" }));

    // Hide the window after a short delay so the user can see the final text
    // The front-end handles its own delay for the CSS transition, but we also hide the Tauri window.
    if let Some(window) = app.get_webview_window("main") {
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
            let _ = window.hide();
        });
    }

    Ok(())
}

/// Stop dictation (IPC). Transitions: Listening → Transcribing → Idle.
#[tauri::command]
pub async fn stop_dictation(
    app: AppHandle,
    state: State<'_, SharedState>,
) -> Result<()> {
    stop_dictation_impl(&app, &state).await
}

// ---------------------------------------------------------------------------
// Status
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_status(state: State<'_, SharedState>) -> Status {
    state.lock().expect("state poisoned").status.clone()
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_settings(state: State<'_, SharedState>) -> Settings {
    state.lock().expect("state poisoned").settings.clone()
}

#[tauri::command]
pub fn set_settings(
    app: tauri::AppHandle,
    state: State<'_, SharedState>,
    new_settings: Settings,
) -> Result<()> {
    settings::save(&new_settings)?;
    
    // Unregister old hotkeys and register the new one
    let _ = crate::hotkey::unregister_all(&app);
    if let Err(e) = crate::hotkey::register(&app, state.inner().clone(), &new_settings.hotkey) {
        log::warn!("Failed to re-register hotkey: {e}");
    }

    let mut locked = state.lock().expect("state poisoned");
    locked.settings = new_settings;
    // Reset engine so it re-initialises with the new model path on next dictation
    locked.engine = None;
    log::info!("settings updated via IPC (and hotkey re-registered)");
    Ok(())
}

#[tauri::command]
pub fn set_keychain_secret(account: String, secret: String) -> Result<()> {
    crate::cleanup::store_keychain_secret(&account, &secret)
        .map_err(|e| crate::error::AppError::Internal(format!("keychain write: {e}")))
}

#[tauri::command]
pub fn delete_keychain_secret(account: String) -> Result<()> {
    crate::cleanup::delete_keychain_secret(&account)
        .map_err(|e| crate::error::AppError::Internal(format!("keychain delete: {e}")))
}

// ---------------------------------------------------------------------------
// Audio devices
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn list_audio_devices() -> Result<Vec<String>> {
    mic::list_input_devices()
}

// ---------------------------------------------------------------------------
// Models
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name:       String,
    pub path:       String,
    pub size_bytes: u64,
}

#[tauri::command]
pub fn list_models() -> Vec<ModelInfo> {
    let base = match dirs::config_dir() {
        Some(d) => d.join("Utter").join("models"),
        None    => return vec![],
    };
    if !base.exists() { return vec![]; }

    std::fs::read_dir(&base)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let n = e.file_name();
            let n = n.to_string_lossy();
            n.ends_with(".bin") || n.ends_with(".gguf") || n.ends_with(".mlmodelc")
        })
        .map(|e| ModelInfo {
            name:       e.file_name().to_string_lossy().into_owned(),
            path:       e.path().to_string_lossy().into_owned(),
            size_bytes: e.metadata().map(|m| m.len()).unwrap_or(0),
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Cleanup test
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupTestResult {
    pub input:      String,
    pub output:     String,
    pub latency_ms: u128,
}

#[tauri::command]
pub async fn test_cleanup(settings: Settings) -> Result<CleanupTestResult> {
    let backend = cleanup::build_backend(&settings.cleanup);
    const TEST_INPUT: &str = "um hello uh this is a test you know";

    let start = std::time::Instant::now();
    let output = backend.refine(TEST_INPUT).await.map_err(AppError::from)?;
    let latency_ms = start.elapsed().as_millis();

    Ok(CleanupTestResult {
        input:  TEST_INPUT.into(),
        output,
        latency_ms,
    })
}

// ---------------------------------------------------------------------------
// Logs
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level:     String,
    pub message:   String,
}

#[tauri::command]
pub fn get_logs() -> Vec<LogEntry> {
    // TODO (Phase 5): ring-buffer log storage
    vec![]
}

#[tauri::command]
pub fn check_accessibility() -> bool {
    insert::is_trusted()
}

#[tauri::command]
pub fn request_accessibility() -> bool {
    insert::request_permission()
}
