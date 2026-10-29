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
    insert, mic,
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
        (
            locked.status.clone(),
            locked.engine.clone(),
            locked.settings.clone(),
        )
    };

    match current_status {
        Status::Listening | Status::Transcribing => return Ok(()),
        _ => {}
    }

    // Initialise engine lazily on first use
    let engine: Arc<PlatformEngine> = match engine {
        Some(e) => e,
        None => {
            let config = EngineConfig {
                model_path: settings.model_path.clone(),
            };
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
    log::info!("Mic config: {sample_rate} Hz");

    let last_streamed_arc = {
        let locked = state.lock().expect("state poisoned");
        locked
            .last_streamed
            .lock()
            .expect("last_streamed poisoned")
            .clear();
        locked.last_streamed.clone()
    };

    let live_typing = settings.live_typing;
    let app_handle = app.clone();
    let stream_buffer = last_streamed_arc.clone();
    let on_partial = Box::new(move |transcript: String, is_final: bool| {
        log::info!(
            "partial transcript, {} chars, final={is_final}",
            transcript.chars().count()
        );
        let _ = app_handle.emit(
            "transcript_partial",
            serde_json::json!({ "text": transcript, "is_final": is_final }),
        );

        if !live_typing || is_final {
            return;
        }
        let typed = crate::logic::sanitize_insertion(&transcript);
        let mut old = crate::state::unpoison(stream_buffer.lock());
        if !typed.is_empty() && *old != typed && crate::insert::insert_delta(&old, &typed).is_ok() {
            *old = typed;
        }
    });

    log::info!("Starting ASR session...");
    let session_id = engine
        .start_session(sample_rate, on_partial)
        .map_err(AppError::from)?;
    log::info!("ASR session started! ID: {session_id}");

    let audio_buffer = {
        let locked = state.lock().expect("state poisoned");
        locked
            .audio_buffer
            .lock()
            .expect("audio lock poisoned")
            .clear();
        locked.audio_buffer.clone()
    };

    let (tx, rx) = std::sync::mpsc::sync_channel::<Vec<f32>>(64);
    let engine_for_capture = engine.clone();
    let feeder = std::thread::Builder::new()
        .name("utter-audio-feed".into())
        .spawn(move || {
            let mut pending = Vec::<f32>::new();
            while let Ok(chunk) = rx.recv() {
                pending.extend_from_slice(&chunk);
                if pending.len() >= 8192 {
                    let _ = engine_for_capture.feed_audio(session_id, &pending);
                    pending.clear();
                }
            }
            if !pending.is_empty() {
                let _ = engine_for_capture.feed_audio(session_id, &pending);
            }
        })
        .map_err(|e| AppError::Internal(format!("Could not start the audio thread: {e}")))?;

    let mut announced_first_chunk = false;
    let capture = mic::CaptureHandle::start(&device, mic_config, move |chunk| {
        if !announced_first_chunk {
            log::info!("microphone delivered first chunk ({} samples)", chunk.len());
            announced_first_chunk = true;
        }
        if let Ok(mut buf) = audio_buffer.lock() {
            buf.extend_from_slice(chunk);
        }
        let _ = tx.try_send(chunk.to_vec());
    })?;

    {
        let mut locked = crate::state::unpoison(state.lock());
        locked.status = Status::Listening;
        locked.session_id = Some(session_id);
        locked.capture_handle = Some(capture);
        locked.audio_feeder = Some(feeder);
    }

    let active_app = detect_active_app();
    log::info!(
        "Smart Routing detected target app: {} (mode: {})",
        active_app.name,
        active_app.mode_label
    );
    let _ = app.emit("active_app_detected", &active_app);

    log::info!("Emitting state_change 'Listening' to frontend...");
    let _ = app.emit(
        "state_change",
        serde_json::json!({
            "status": "Listening",
            "app": active_app
        }),
    );

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
pub async fn start_dictation(app: AppHandle, state: State<'_, SharedState>) -> Result<()> {
    start_dictation_impl(&app, &state).await
}

/// Core logic for stopping dictation.
/// Called by both the IPC command and the hotkey handler.
pub async fn stop_dictation_impl(app: &AppHandle, state: &SharedState) -> Result<()> {
    // --- Grab what we need and stop the mic (brief lock) ---
    let (
        engine,
        session_id,
        audio_buffer,
        cleanup_cfg,
        dictionary,
        last_streamed_arc,
        smart_routing,
        capture,
        feeder,
    ) = {
        let mut locked = crate::state::unpoison(state.lock());

        if locked.status != Status::Listening {
            return Ok(()); // idempotent
        }

        let capture = locked.capture_handle.take();
        let feeder = locked.audio_feeder.take();
        locked.status = Status::Transcribing;

        (
            locked.engine.clone(),
            locked.session_id.take(),
            locked.audio_buffer.clone(),
            locked.settings.cleanup.clone(),
            locked.settings.dictionary.clone(),
            locked.last_streamed.clone(),
            locked.settings.smart_routing,
            capture,
            feeder,
        )
    };
    drop(capture);
    if let Some(handle) = feeder {
        let _ = handle.join();
    }

    let _ = app.emit(
        "state_change",
        serde_json::json!({ "status": "Transcribing" }),
    );

    // --- Validate engine and session ---
    let engine = match engine {
        Some(e) => e,
        None => {
            let msg = "Engine not initialised".to_string();
            let _ = app.emit(
                "state_change",
                serde_json::json!({ "status": "Error", "message": msg }),
            );
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

    log::info!(
        "stop_dictation: {:.1}s of audio, running ASR…",
        samples.len() as f32 / 16000.0
    );

    let engine_clone = engine.clone();
    let raw = tokio::task::spawn_blocking(move || engine_clone.end_session(session_id))
        .await
        .unwrap_or_else(|e| Err(EngineError::Session(format!("spawn_blocking failed: {e}"))))
        .map_err(|e| {
            let err = AppError::from(e);
            let mut locked = state.lock().expect("state poisoned");
            locked.status = Status::Error(err.to_string());
            err
        })?;

    log::info!(
        "stop_dictation: raw transcript is {} chars",
        raw.chars().count()
    );

    // Snapshot what was actually typed to the cursor during real-time streaming
    let streamed_str = last_streamed_arc.lock().expect("stream poisoned").clone();

    // --- Optional cleanup pass with Context-Aware Smart Routing ---
    let active_app = detect_active_app();
    let mut cleanup_cfg_effective = cleanup_cfg.clone();
    let context_directive = if !smart_routing {
        " Fix grammar, remove filler words (um, uh, like), and polish punctuation into crisp, natural sentences."
    } else {
        match active_app.mode.as_str() {
        "code" => " Format the text cleanly for code comments, technical markdown, or code identifiers. Suppress conversational filler and accurately preserve technical terms and variable casing.",
        "chat" => " Format the text as a natural, concise instant message with conversational punctuation.",
        "email" => " Format the text as a polished, professional email with well-structured sentences and a polite tone.",
        _ => " Fix grammar, remove filler words (um, uh, like), and polish punctuation into crisp, natural sentences.",
        }
    };

    // All three LLM backends share one default system prompt; fill it in only
    // when the user left the prompt blank.
    let prompt_slot = match cleanup_cfg_effective {
        settings::CleanupConfig::OpenAiCompat { ref mut prompt, .. }
        | settings::CleanupConfig::LocalOllama { ref mut prompt, .. }
        | settings::CleanupConfig::RemoteOllama { ref mut prompt, .. } => Some(prompt),
        settings::CleanupConfig::None => None,
    };
    if let Some(prompt) = prompt_slot {
        if prompt.is_none() || prompt.as_deref() == Some("") {
            *prompt = Some(format!("You are a transcript editor.{context_directive} Return ONLY the final corrected transcript — no explanations, no comments, no quotes."));
        }
    }

    let backend = cleanup::build_backend(&cleanup_cfg_effective);
    let mut final_text = if raw.is_empty() {
        String::new()
    } else {
        backend
            .refine(&raw)
            .await
            .map_err(|e| {
                let err = AppError::from(e);
                log::warn!("cleanup failed: {err} — using raw transcript");
                err
            })
            .unwrap_or(raw.clone())
    };

    let rules: Vec<(String, String)> = dictionary
        .iter()
        .filter_map(|item| {
            let repl = item.replacement.clone()?;
            if repl.is_empty() || item.word.is_empty() {
                None
            } else {
                Some((item.word.clone(), repl))
            }
        })
        .collect();
    final_text = crate::logic::apply_dictionary(&final_text, &rules);
    final_text = crate::logic::sanitize_insertion(&final_text);

    log::info!(
        "stop_dictation: final transcript is {} chars",
        final_text.chars().count()
    );

    let mut insert_error: Option<String> = None;
    if !final_text.is_empty() {
        if let Err(e) = insert::insert_delta(&streamed_str, &final_text) {
            log::warn!("text insertion failed: {e}");
            insert_error = Some(e.to_string());
        }
    } else if !streamed_str.is_empty() {
        if let Err(e) = insert::insert_delta(&streamed_str, "") {
            insert_error = Some(e.to_string());
        }
    }

    if !final_text.trim().is_empty() {
        let history_entry = crate::history::add_entry(&final_text).ok();
        let _ = app.emit(
            "transcript_final",
            serde_json::json!({
                "text": final_text,
                "entry": history_entry,
                "insert_error": insert_error,
            }),
        );
    } else {
        let _ = app.emit(
            "transcript_final",
            serde_json::json!({ "text": final_text, "insert_error": insert_error }),
        );
    }

    {
        let mut locked = crate::state::unpoison(state.lock());
        if let Some(msg) = insert_error.clone() {
            locked.status = Status::Error(msg);
        } else {
            locked.status = Status::Idle;
        }
        crate::state::unpoison(locked.last_streamed.lock()).clear();
    }
    if let Some(msg) = insert_error {
        let _ = app.emit(
            "state_change",
            serde_json::json!({ "status": "Error", "message": msg }),
        );
    } else {
        let _ = app.emit("state_change", serde_json::json!({ "status": "Idle" }));
        if let Some(window) = app.get_webview_window("main") {
            tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
                let _ = window.hide();
            });
        }
    }

    Ok(())
}

/// Stop dictation (IPC). Transitions: Listening → Transcribing → Idle.
#[tauri::command]
pub async fn stop_dictation(app: AppHandle, state: State<'_, SharedState>) -> Result<()> {
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
    let hotkey_result = crate::hotkey::register(&app, state.inner().clone(), &new_settings.hotkey);

    let mut locked = state.lock().expect("state poisoned");
    let hotkey_label = new_settings.hotkey.clone();
    locked.settings = new_settings;
    locked.engine = None;

    match hotkey_result {
        Ok(()) => {
            locked.startup_warning = None;
            log::info!("settings updated via IPC (and hotkey re-registered)");
            Ok(())
        }
        Err(e) => {
            let message = format!(
                "Settings were saved, but the hotkey {hotkey_label} could not be registered. Choose a different shortcut. {e}"
            );
            locked.startup_warning = Some(message.clone());
            Err(AppError::Internal(message))
        }
    }
}

#[tauri::command]
pub fn get_startup_warning(state: State<'_, SharedState>) -> Option<String> {
    state
        .lock()
        .expect("state poisoned")
        .startup_warning
        .clone()
}

/// Text the user can paste into a bug report. No transcripts, no API keys.
#[tauri::command]
pub fn diagnostics_report(state: State<'_, SharedState>) -> String {
    let locked = state.lock().expect("state poisoned");
    let cleanup = match &locked.settings.cleanup {
        crate::settings::CleanupConfig::None => "off",
        crate::settings::CleanupConfig::LocalOllama { .. } => "local Ollama",
        crate::settings::CleanupConfig::RemoteOllama { .. } => "remote Ollama",
        crate::settings::CleanupConfig::OpenAiCompat { .. } => "OpenAI-compatible",
    };
    let warning = locked
        .startup_warning
        .clone()
        .unwrap_or_else(|| "none".into());
    let status = format!("{:?}", locked.status);
    drop(locked);

    let crashes = get_logs()
        .into_iter()
        .map(|e| e.message)
        .collect::<Vec<_>>()
        .join("\n");
    let crashes = if crashes.is_empty() {
        "(no crash log)".into()
    } else {
        crashes
    };

    format!(
        "Utter {version}\nstatus: {status}\ncleanup: {cleanup}\nstartup warning: {warning}\n\ncrash log:\n{crashes}\n",
        version = env!("CARGO_PKG_VERSION"),
    )
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
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
}

#[tauri::command]
pub fn list_models() -> Vec<ModelInfo> {
    let base = match dirs::config_dir() {
        Some(d) => d.join("Utter").join("models"),
        None => return vec![],
    };
    if !base.exists() {
        return vec![];
    }

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
            name: e.file_name().to_string_lossy().into_owned(),
            path: e.path().to_string_lossy().into_owned(),
            size_bytes: e.metadata().map(|m| m.len()).unwrap_or(0),
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Cleanup test
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupTestResult {
    pub input: String,
    pub output: String,
    pub latency_ms: u128,
}

#[tauri::command]
pub async fn test_cleanup(settings: Settings) -> Result<CleanupTestResult> {
    if settings.cleanup == crate::settings::CleanupConfig::None {
        return Err(AppError::Internal(
            "Raw Transcription (None) is active. Select Local Ollama or Cloud AI to test connection.".into(),
        ));
    }

    let backend = cleanup::build_backend(&settings.cleanup);
    const TEST_INPUT: &str = "um hello uh this is a test you know";

    let start = std::time::Instant::now();
    let output = backend.refine(TEST_INPUT).await.map_err(AppError::from)?;
    let latency_ms = start.elapsed().as_millis();

    Ok(CleanupTestResult {
        input: TEST_INPUT.into(),
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
    pub level: String,
    pub message: String,
}

#[tauri::command]
pub fn get_logs() -> Vec<LogEntry> {
    let path = if cfg!(target_os = "macos") {
        dirs::home_dir().map(|h| h.join("Library/Logs/Utter/crash.log"))
    } else {
        dirs::config_dir().map(|d| d.join("Utter").join("logs").join("crash.log"))
    };
    let Some(path) = path else { return vec![] };
    let raw = std::fs::read_to_string(&path).unwrap_or_default();
    raw.lines()
        .rev()
        .take(200)
        .map(|line| LogEntry {
            timestamp: String::new(),
            level: "crash".into(),
            message: line.to_string(),
        })
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

#[tauri::command]
pub fn check_accessibility() -> bool {
    insert::is_trusted()
}

#[tauri::command]
pub fn request_accessibility() -> bool {
    insert::request_permission()
}

#[tauri::command]
pub fn get_history() -> Vec<crate::history::HistoryEntry> {
    crate::history::load()
}

#[tauri::command]
pub fn delete_history_item(id: String) -> Result<Vec<crate::history::HistoryEntry>> {
    crate::history::delete_entry(&id)
}

#[tauri::command]
pub fn update_history_item(
    id: String,
    new_text: String,
) -> Result<Vec<crate::history::HistoryEntry>> {
    crate::history::update_entry(&id, &new_text)
}

#[tauri::command]
pub fn clear_history() -> Result<()> {
    crate::history::clear_all()
}

#[tauri::command]
pub fn open_app_folder() -> Result<()> {
    if let Ok(path) = settings::config_path() {
        if let Some(parent) = path.parent() {
            let _ = std::process::Command::new("open").arg(parent).spawn();
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveAppInfo {
    pub name: String,
    pub bundle_id: String,
    pub mode: String,
    pub mode_label: String,
    pub icon: String,
}

#[cfg(target_os = "macos")]
extern "C" {
    fn get_frontmost_app_info(
        name_buf: *mut std::os::raw::c_char,
        name_max: usize,
        bundle_buf: *mut std::os::raw::c_char,
        bundle_max: usize,
    ) -> bool;
}

pub fn detect_active_app() -> ActiveAppInfo {
    #[cfg(target_os = "macos")]
    {
        let mut name_buf = [0 as std::os::raw::c_char; 256];
        let mut bundle_buf = [0 as std::os::raw::c_char; 256];
        let ok = unsafe {
            get_frontmost_app_info(name_buf.as_mut_ptr(), 256, bundle_buf.as_mut_ptr(), 256)
        };
        if ok {
            let name = unsafe { std::ffi::CStr::from_ptr(name_buf.as_ptr()) }
                .to_string_lossy()
                .into_owned();
            let bundle_id = unsafe { std::ffi::CStr::from_ptr(bundle_buf.as_ptr()) }
                .to_string_lossy()
                .into_owned();

            let kind = crate::logic::classify_app(&bundle_id, &name);
            return ActiveAppInfo {
                name,
                bundle_id,
                mode: kind.as_str().to_string(),
                mode_label: kind.label().to_string(),
                icon: kind.icon().to_string(),
            };
        }
    }

    ActiveAppInfo {
        name: "Active App".to_string(),
        bundle_id: "com.apple.generic".to_string(),
        mode: "pro".to_string(),
        mode_label: "Pro Document".to_string(),
        icon: "pro".to_string(),
    }
}

#[tauri::command]
pub fn get_active_app() -> ActiveAppInfo {
    detect_active_app()
}
