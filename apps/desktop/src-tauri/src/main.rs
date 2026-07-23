// main.rs — Utter desktop app entry point.
//
// Responsibilities:
//   1. Initialise the logger.
//   2. Load settings from disk.
//   3. Build Tauri app with all plugins and managed state.
//   4. Register all IPC commands.
//   5. Register the global hotkey.
//   6. Run the event loop.
//
// WHY no #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]:
//   Tauri 2.x handles this via the tauri.conf.json build settings.
//   We leave main.rs clean.

// Suppress dead_code warnings on Phase 2/3/4/5 stub items.
// These are forward declarations that will be used once each phase is implemented.
#![allow(dead_code)]

mod cleanup;
mod commands;
mod engine;
mod error;
mod hotkey;
mod insert;
mod mic;
mod settings;
mod state;

use state::new_shared_state;


fn main() {
    // Initialise env_logger.
    // In release builds, default to 'info'. In debug builds, default to 'debug'.
    #[cfg(debug_assertions)]
    let default_level = "utter=debug,info";
    #[cfg(not(debug_assertions))]
    let default_level = "utter=info,warn";

    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(default_level),
    )
    .init();

    log::info!("Utter v{} starting", env!("CARGO_PKG_VERSION"));

    let shared_state = new_shared_state();

    tauri::Builder::default()
        // --- Plugins ---
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        // --- Managed state ---
        .manage(shared_state.clone())
        // --- IPC commands ---
        .invoke_handler(tauri::generate_handler![
            commands::start_dictation,
            commands::stop_dictation,
            commands::get_status,
            commands::get_settings,
            commands::set_settings,
            commands::list_audio_devices,
            commands::list_models,
            commands::test_cleanup,
            commands::get_logs,
        ])
        // --- Setup callback: runs once after the window is created ---
        .setup(move |app| {
            let handle = app.handle();

            // Register the global hotkey
            if let Err(e) = hotkey::register(handle, shared_state.clone()) {
                log::error!("hotkey registration failed: {e}");
                // Not fatal — app still works, just no hotkey.
            }

            // macOS: hide the Dock icon (Utter lives in the menu bar)
            #[cfg(target_os = "macos")]
            {
                use tauri::ActivationPolicy;
                handle
                    .set_activation_policy(ActivationPolicy::Accessory)
                    .unwrap_or_else(|e| log::warn!("set_activation_policy: {e}"));
            }

            log::info!("setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Tauri app error");
}
