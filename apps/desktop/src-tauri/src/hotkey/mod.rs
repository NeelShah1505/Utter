// hotkey/mod.rs — Global hotkey registration.
//
// Registers Cmd+Shift+D (macOS) / Ctrl+Shift+D (Windows) as a system-wide hotkey.
// When the hotkey fires, the handler:
//   1. Reads the current state from SharedState.
//   2. If Idle  → calls start_dictation_impl().
//   3. If Listening → calls stop_dictation_impl().
//   4. If Transcribing → ignores (already in-flight).
//   5. Emits a 'state_change' event to the webview.
//
// Uses tauri-plugin-global-shortcut which wraps the OS hotkey APIs:
//   - macOS: CGEventTap / Carbon RegisterEventHotKey
//   - Windows: RegisterHotKey

use crate::state::SharedState;
use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

/// The global hotkey accelerator string. CmdOrCtrl maps to Cmd on macOS / Ctrl on Windows.
pub const HOTKEY: &str = "CmdOrCtrl+Shift+D";

/// Register the global hotkey. Call once at app startup after the app is built.
/// The hotkey fires regardless of which window/app has focus.
pub fn register(app: &AppHandle, state: SharedState) -> anyhow::Result<()> {
    let shortcut: Shortcut = HOTKEY.parse().map_err(|e| anyhow::anyhow!("{e}"))?;

    app.global_shortcut().on_shortcut(shortcut, {
        let state  = state.clone();
        let app    = app.clone();
        move |_app, shortcut, event| {
            // Only act on key-down, not key-up
            if event.state() != ShortcutState::Pressed {
                return;
            }
            log::debug!("hotkey fired: {shortcut}");
            handle_hotkey(&app, &state);
        }
    })?;

    log::info!("hotkey: registered '{HOTKEY}'");
    Ok(())
}

/// Toggle dictation: Idle→Listening or Listening→Transcribing.
fn handle_hotkey(app: &AppHandle, state: &SharedState) {
    use crate::state::Status;

    let current_status = {
        let locked = state.lock().expect("state poisoned");
        locked.status.clone()
    };

    match current_status {
        Status::Idle => {
            log::info!("hotkey: starting dictation");
            // TODO (Phase 2): call start_dictation_impl(app, state)
            // For now, just emit a state_change event so the UI can show something.
            let _ = app.emit(
                "state_change",
                serde_json::json!({ "status": "Listening" }),
            );
            let mut locked = state.lock().expect("state poisoned");
            locked.status = Status::Listening;
        }
        Status::Listening => {
            log::info!("hotkey: stopping dictation");
            // TODO (Phase 2): call stop_dictation_impl(app, state)
            let _ = app.emit(
                "state_change",
                serde_json::json!({ "status": "Idle" }),
            );
            let mut locked = state.lock().expect("state poisoned");
            locked.status = Status::Idle;
            locked.session_id = None;
        }
        Status::Transcribing => {
            log::debug!("hotkey: ignoring — transcription already in-flight");
        }
        Status::Error(_) => {
            // Allow hotkey to clear error state and restart
            log::info!("hotkey: clearing error, returning to Idle");
            let _ = app.emit(
                "state_change",
                serde_json::json!({ "status": "Idle" }),
            );
            let mut locked = state.lock().expect("state poisoned");
            locked.status = Status::Idle;
        }
    }
}
