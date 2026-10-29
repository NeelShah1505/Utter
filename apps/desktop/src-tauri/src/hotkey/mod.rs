// hotkey/mod.rs — Global hotkey registration.
//
// Registers Cmd+Shift+D (macOS) / Ctrl+Shift+D (Windows) as a system-wide hotkey.
// When the hotkey fires, the handler:
//   1. Reads the current state from SharedState.
//   2. If Idle       → runs start_dictation (opens mic, starts ASR session).
//   3. If Listening  → runs stop_dictation (stops mic, runs ASR, inserts text).
//   4. If Transcribing → ignores (already in-flight, wait for it to finish).
//   5. If Error      → clears error, returns to Idle.
//
// WHY tokio::spawn for the async commands:
//   The hotkey callback runs on the OS hotkey thread (not a tokio thread).
//   start_dictation and stop_dictation are async (stop_dictation calls the
//   speech helper subprocess and awaits it). We spawn them on the tokio runtime
//   so the hotkey callback returns immediately.

use crate::commands;
use crate::state::{SharedState, Status};
use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

// Removed hardcoded HOTKEY const; the hotkey is now dynamically loaded from settings.

/// Register the global hotkey dynamically.
pub fn register(app: &AppHandle, state: SharedState, hotkey_str: &str) -> anyhow::Result<()> {
    let shortcut: Shortcut = hotkey_str.parse().map_err(|e| anyhow::anyhow!("{e}"))?;

    app.global_shortcut().on_shortcut(shortcut, {
        let state = state.clone();
        let app   = app.clone();
        move |_app, shortcut, event| {
            if event.state() != ShortcutState::Pressed {
                return;
            }
            log::debug!("hotkey fired: {shortcut}");
            
            let state_clone = state.clone();
            let app_clone = app.clone();
            
            let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                toggle_dictation(app_clone, state_clone);
            }));
            
            if let Err(e) = res {
                log::error!("PANIC IN HOTKEY HANDLER: {:?}", e);
            }
        }
    })?;

    log::info!("hotkey: registered '{hotkey_str}'");
    Ok(())
}

/// Unregister all global hotkeys (useful when updating the hotkey setting).
pub fn unregister_all(app: &AppHandle) -> anyhow::Result<()> {
    app.global_shortcut().unregister_all()?;
    Ok(())
}

/// Toggle dictation on each key-press or tray menu click.
pub fn toggle_dictation(app: AppHandle, state: SharedState) {
    let current_status = {
        let locked = state.lock().expect("state poisoned");
        locked.status.clone()
    };

    match current_status {
        // Error is recoverable: the next press starts a new dictation.
        Status::Idle | Status::Error(_) => {
            log::info!("hotkey: starting dictation");
            tauri::async_runtime::spawn(async move {
                if let Err(e) = commands::start_dictation_impl(&app, &state).await {
                    log::error!("start_dictation failed: {e}");
                    let _ = app.emit(
                        "state_change",
                        serde_json::json!({ "status": "Error", "message": e.to_string() }),
                    );
                    let mut locked = state.lock().expect("state poisoned");
                    locked.status = Status::Error(e.to_string());
                }
            });
        }

        // ── Listening → stop recording, run ASR, insert text ─────────────
        Status::Listening => {
            log::info!("hotkey: stopping dictation");
            tauri::async_runtime::spawn(async move {
                if let Err(e) = commands::stop_dictation_impl(&app, &state).await {
                    log::error!("stop_dictation failed: {e}");
                    let _ = app.emit(
                        "state_change",
                        serde_json::json!({ "status": "Error", "message": e.to_string() }),
                    );
                    let mut locked = state.lock().expect("state poisoned");
                    locked.status = Status::Error(e.to_string());
                }
            });
        }

        // ── Transcribing → ignore (ASR already running) ──────────────────
        Status::Transcribing => {
            log::debug!("hotkey: ignoring — transcription already in-flight");
        }

    }
}
