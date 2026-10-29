# Inventory (2026-10-29, before the fixes in this branch)

Counted with `git ls-files` and a Python line count. Binary assets inflate the total. Source that matters:

| Area | Lines |
|------|------:|
| Rust (`apps/desktop/src-tauri/src`) | 2,524 |
| Swift helper | 259 |
| Desktop JS/CSS/HTML | in the `apps` total; the running UI is `settings.html`, `index.html`, and the esbuild bundles |
| Site | 1,974 |
| Docs | 1,268 |
| Unused TSX (`components/`, `apps/desktop/src/components/`) | about 1,098 |

Those TSX files were not imported by the Tauri app. They were deleted on this branch.

## Entry points

- `apps/desktop/src-tauri/src/main.rs` — process start, tray, hotkey, panic hook.
- `apps/desktop/src-tauri/helpers/utter_transcribe.swift` — `SFSpeechRecognizer`, C ABI, frontmost app.
- `apps/desktop/src-tauri/build.rs` — `swiftc` then `ar`, then `tauri_build`.
- `site/index.html` — download page.

## FFI and threads

- C ABI: `speech_init_stream`, `speech_append_audio`, `speech_end_stream`, `speech_free_stream`, `get_frontmost_app_info`, `activate_app_to_front`.
- `unsafe` in `macos_speech.rs` (callback context), `insert/mod.rs` (Accessibility and secure-input), `commands.rs` (frontmost app buffers), `main.rs` (activate).
- Mic callback on the CoreAudio thread. As of this branch it only enqueues samples. `utter-audio-feed` calls Swift.
- Hotkey callback spawns `tauri::async_runtime`. `end_session` runs on `spawn_blocking`.

## Dependencies that dictate

Tauri 2, cpal 0.15, enigo 0.3, reqwest 0.12 (rustls), keyring 3. Licenses were not re-audited with `cargo deny` in this session. That tool is not installed here.
