// commands.rs — All Tauri IPC commands.
//
// Full pipeline (Phase 2):
//   start_dictation → open mic → feed audio → buffer samples
//   stop_dictation  → drop mic → engine.end_session() → cleanup → insert_text → emit event
//
// The engine is initialised lazily on first start_dictation call.

use crate::{
    cleanup,
    engine::{AsrEngine, EngineConfig, PlatformEngine},
    error::{AppError, Result},
    insert,
    mic,
    settings::{self, Settings},
    state::{SharedState, Status},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

// ---------------------------------------------------------------------------
// Dictation control
// ---------------------------------------------------------------------------

/// Start dictation. Transitions: Idle → Listening.
/// Opens the microphone and begins accumulating audio.
#[tauri::command]
pub async fn start_dictation(
    app: AppHandle,
    state: State<'_, SharedState>,
) -> Result<()> {
    // --- Read current status and settings (brief lock) ---
    let (current_status, engine, settings) = {
        let locked = state.lock().expect("state poisoned");
        (
            locked.status.clone(),
            locked.engine.clone(),
            locked.settings.clone(),
        )
    };

    match current_status {
        Status::Listening | Status::Transcribing => return Ok(()), // idempotent
        _ => {}
    }

    // --- Initialise engine lazily on first use ---
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

    // --- Start ASR session ---
    let session_id = engine.start_session().map_err(AppError::from)?;

    // --- Clear audio buffer ---
    let audio_buffer = {
        let locked = state.lock().expect("state poisoned");
        locked.audio_buffer.lock().expect("audio lock poisoned").clear();
        locked.audio_buffer.clone()
    };

    // --- Open microphone ---
    let device = mic::find_device(&settings.audio_device)?;
    let mic_config = mic::preferred_config(&device)?;

    let capture = mic::CaptureHandle::start(&device, mic_config, move |chunk| {
        let mut buf = audio_buffer.lock().expect("audio lock poisoned");
        buf.extend_from_slice(chunk);
    })?;

    // --- Commit to Listening state ---
    {
        let mut locked = state.lock().expect("state poisoned");
        locked.status         = Status::Listening;
        locked.session_id     = Some(session_id);
        locked.capture_handle = Some(capture);
    }

    let _ = app.emit("state_change", serde_json::json!({ "status": "Listening" }));
    log::info!("start_dictation: mic open, session {session_id}");
    Ok(())
}

/// Stop dictation. Transitions: Listening → Transcribing → Idle.
/// Stops the mic, runs ASR + cleanup, inserts text, emits transcript_final.
#[tauri::command]
pub async fn stop_dictation(
    app: AppHandle,
    state: State<'_, SharedState>,
) -> Result<()> {
    // --- Grab what we need and stop the mic (brief lock) ---
    let (engine, session_id, audio_buffer, cleanup_cfg) = {
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

    engine.feed_audio(session_id, &samples).map_err(|e| {
        let err = AppError::from(e);
        let mut locked = state.lock().expect("state poisoned");
        locked.status = Status::Error(err.to_string());
        err
    })?;

    let raw = engine.end_session(session_id).map_err(|e| {
        let err = AppError::from(e);
        let mut locked = state.lock().expect("state poisoned");
        locked.status = Status::Error(err.to_string());
        err
    })?;

    log::info!("stop_dictation: raw = {:?}", raw);

    // --- Optional cleanup pass ---
    let backend = cleanup::build_backend(&cleanup_cfg);
    let final_text = if raw.is_empty() {
        String::new()
    } else {
        backend.refine(&raw).await.map_err(|e| {
            let err = AppError::from(e);
            log::warn!("cleanup failed: {err} — using raw transcript");
            // Don't fail — use raw transcript instead
            err
        })
        .unwrap_or(raw.clone())
    };

    log::info!("stop_dictation: final = {:?}", final_text);

    // --- Insert text into focused app ---
    if !final_text.is_empty() {
        if let Err(e) = insert::insert_text(&final_text) {
            log::warn!("insert_text failed: {e} — transcript available in event");
            // Don't fail — the user still gets the transcript via the event
        }
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
    }
    let _ = app.emit("state_change", serde_json::json!({ "status": "Idle" }));

    Ok(())
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
    state: State<'_, SharedState>,
    new_settings: Settings,
) -> Result<()> {
    settings::save(&new_settings)?;
    let mut locked = state.lock().expect("state poisoned");
    locked.settings = new_settings;
    // Reset engine so it re-initialises with the new model path on next dictation
    locked.engine = None;
    log::info!("settings updated via IPC");
    Ok(())
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
