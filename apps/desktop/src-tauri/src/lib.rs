// lib.rs — Library entry point for Tauri mobile builds.
//
// Tauri 2.x requires a lib crate alongside the binary for mobile (iOS/Android) support.
// All modules are declared in main.rs. This file just re-exports what mobile
// scaffolding needs.
//
// NOTE: Desktop builds use main.rs. This file is compiled for mobile targets only.

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Mirrors the setup in main.rs — kept in sync manually.
    // Full mobile setup comes in a later phase.
}
