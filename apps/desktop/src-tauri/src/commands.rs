// commands.rs — All Tauri IPC commands.
//
// These are the 9 commands defined in ARCHITECTURE.md §3.1.
// Each is a Tauri #[command] function that the webview can invoke via
// window.__TAURI__.core.invoke('command_name', payload).
//
// Command → Return type mapping (ARCHITECTURE.md §3.2):
//   start_dictation  → Ok(())   | AppError
//   stop_dictation   → Ok(())   | AppError
//   get_status       → Status
//   get_settings     → Settings
//   set_settings     → Ok(())   | AppError
//   list_audio_devices → Vec<String>
//   list_models      → Vec<ModelInfo>
//   test_cleanup     → CleanupTestResult | AppError
//   get_logs         → Vec<LogEntry>

use crate::{
    cleanup,
    error::{AppError, Result},
    mic,
    settings::{self, Settings},
    state::{SharedState, Status},
};
use serde::{Deserialize, Serialize};
use tauri::State;

// ---------------------------------------------------------------------------
// Dictation control
// ---------------------------------------------------------------------------

/// Start dictation. Transitions: Idle → Listening.
/// Returns immediately; audio capture begins asynchronously.
#[tauri::command]
pub async fn start_dictation(state: State<'_, SharedState>) -> Result<()> {
    let current = {
        let locked = state.lock().expect("state poisoned");
        locked.status.clone()
    };

    match current {
        Status::Listening | Status::Transcribing => {
            // Already dictating — idempotent, not an error.
            return Ok(());
        }
        Status::Error(_) | Status::Idle => {}
    }

    // TODO (Phase 2): Start mic capture + ASR engine session.
    // For now, transition state and return.
    {
        let mut locked = state.lock().expect("state poisoned");
        locked.status = Status::Listening;
    }
    log::info!("start_dictation: state → Listening");
    Ok(())
}

/// Stop dictation. Transitions: Listening → Transcribing → Idle.
/// Returns immediately; final transcript is emitted as a 'transcript_final' event.
#[tauri::command]
pub async fn stop_dictation(state: State<'_, SharedState>) -> Result<()> {
    let current = {
        let locked = state.lock().expect("state poisoned");
        locked.status.clone()
    };

    if current != Status::Listening {
        return Ok(()); // Idempotent
    }

    // TODO (Phase 2): Stop mic capture, call engine.end_session(), run cleanup.
    {
        let mut locked = state.lock().expect("state poisoned");
        locked.status = Status::Idle;
        locked.session_id = None;
    }
    log::info!("stop_dictation: state → Idle");
    Ok(())
}

// ---------------------------------------------------------------------------
// Status
// ---------------------------------------------------------------------------

/// Get the current dictation status.
#[tauri::command]
pub fn get_status(state: State<'_, SharedState>) -> Status {
    let locked = state.lock().expect("state poisoned");
    locked.status.clone()
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

/// Get the current settings from disk.
#[tauri::command]
pub fn get_settings() -> Settings {
    settings::load()
}

/// Validate and save new settings to disk.
#[tauri::command]
pub fn set_settings(new_settings: Settings) -> Result<()> {
    settings::save(&new_settings)?;
    log::info!("settings updated via IPC");
    Ok(())
}

// ---------------------------------------------------------------------------
// Audio devices
// ---------------------------------------------------------------------------

/// List all available audio input device names.
#[tauri::command]
pub fn list_audio_devices() -> Result<Vec<String>> {
    mic::list_input_devices()
}

// ---------------------------------------------------------------------------
// Models
// ---------------------------------------------------------------------------

/// Information about a discoverable model file.
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
}

/// List model files found in the default model search path.
/// Returns an empty list if no models are found (not an error).
#[tauri::command]
pub fn list_models() -> Vec<ModelInfo> {
    // Default search path: ~/Library/Application Support/Utter/models (macOS)
    // or %APPDATA%\Utter\models (Windows)
    let base = match dirs::config_dir() {
        Some(d) => d.join("Utter").join("models"),
        None    => return vec![],
    };

    if !base.exists() {
        return vec![];
    }

    std::fs::read_dir(&base)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .filter(|e| {
            let name = e.file_name();
            let n = name.to_string_lossy();
            n.ends_with(".bin") || n.ends_with(".gguf") || n.ends_with(".mlmodelc")
        })
        .map(|e| {
            let size = e.metadata().map(|m| m.len()).unwrap_or(0);
            ModelInfo {
                name:       e.file_name().to_string_lossy().into_owned(),
                path:       e.path().to_string_lossy().into_owned(),
                size_bytes: size,
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Cleanup test
// ---------------------------------------------------------------------------

/// Result of a test cleanup round-trip.
#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupTestResult {
    pub input:        String,
    pub output:       String,
    pub latency_ms:   u128,
}

/// Run a test cleanup request with a fixed input string.
/// Used by the settings UI to verify connectivity before saving.
#[tauri::command]
pub async fn test_cleanup(settings: Settings) -> Result<CleanupTestResult> {
    let backend = cleanup::build_backend(&settings.cleanup);
    const TEST_INPUT: &str = "um hello uh this is a test you know";

    let start = std::time::Instant::now();
    let output = backend
        .refine(TEST_INPUT)
        .await
        .map_err(AppError::from)?;
    let latency_ms = start.elapsed().as_millis();

    Ok(CleanupTestResult {
        input:      TEST_INPUT.into(),
        output,
        latency_ms,
    })
}

// ---------------------------------------------------------------------------
// Logs
// ---------------------------------------------------------------------------

/// A single structured log entry.
#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level:     String,
    pub message:   String,
}

/// Return the last N log entries from the in-memory log buffer.
/// Used by the settings UI "About / Logs" section.
/// (Full on-disk log rotation is Phase 5.)
#[tauri::command]
pub fn get_logs() -> Vec<LogEntry> {
    // TODO (Phase 5): Implement ring-buffer log storage.
    // For now, return an empty list (the settings UI handles this gracefully).
    vec![]
}
