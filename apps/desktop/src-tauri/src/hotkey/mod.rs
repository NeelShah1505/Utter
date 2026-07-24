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

/// The global hotkey accelerator string. CmdOrCtrl maps to Cmd on macOS / Ctrl on Windows.
pub const HOTKEY: &str = "CmdOrCtrl+Shift+D";

/// Register the global hotkey. Call once at app startup after the app is built.
pub fn register(app: &AppHandle, state: SharedState) -> anyhow::Result<()> {
    let shortcut: Shortcut = HOTKEY.parse().map_err(|e| anyhow::anyhow!("{e}"))?;

    app.global_shortcut().on_shortcut(shortcut, {
        let state = state.clone();
        let app   = app.clone();
        move |_app, shortcut, event| {
            if event.state() != ShortcutState::Pressed {
                return;
            }
            log::debug!("hotkey fired: {shortcut}");
            handle_hotkey(app.clone(), state.clone());
        }
    })?;

    log::info!("hotkey: registered '{HOTKEY}'");
    Ok(())
}

/// Toggle dictation on each key-press.
fn handle_hotkey(app: AppHandle, state: SharedState) {
    let current_status = {
        let locked = state.lock().expect("state poisoned");
        locked.status.clone()
    };

    match current_status {
        // ── Idle → start recording ───────────────────────────────────────
        Status::Idle => {
            log::info!("hotkey: starting dictation");
            tokio::spawn(async move {
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
            tokio::spawn(async move {
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

        // ── Error → clear and return to Idle ─────────────────────────────
        Status::Error(_) => {
            log::info!("hotkey: clearing error, returning to Idle");
            let _ = app.emit("state_change", serde_json::json!({ "status": "Idle" }));
            let mut locked = state.lock().expect("state poisoned");
            locked.status = Status::Idle;
        }
    }
}
