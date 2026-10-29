# Findings ledger

Audited 2026-10-29 on macOS 26.5.1 (build 25F80), arm64, rustc 1.88.0, Swift 6.1.

Severity: S0 ship-blocker, S1 major, S2 moderate, S3 polish.

| ID | Sev | Hat | Area | Evidence | Status |
|----|-----|-----|------|----------|--------|
| F-001 | S0 | Release | CI | `.github/workflows/build-release.yml` wrote `echo "placeholder - not a real installer"` into `app-macos-x64.dmg`, `app-windows-x64.msi`, and `app-windows-arm64.msi`. | Fixed. Those jobs are gone. Release uploads `app-macos-arm64.dmg` and its `.sha256` only. |
| F-002 | S0 | Security | Speech | `helpers/utter_transcribe.swift` did not set `requiresOnDeviceRecognition`. Apple can use the network when that flag is false. | Fixed in source. Compiled object contains `supportsOnDeviceRecognition` and `setRequiresOnDeviceRecognition:` (see 03-release-readiness.md). Packet capture not run. |
| F-003 | S0 | Security | History | `history.rs` writes `history.json` on every final transcript. `docs/SECURITY.md` said transcripts are not saved unless the user opts in. | Docs now match the code. Writes are atomic and mode 0600. Clear History deletes the file. Default remains on, and the Studio copy says so. |
| F-004 | S0 | Platform | Swift build | `build.rs` hardcoded `-target arm64-apple-macos13.0`. | Fixed: target follows `TARGET` (`arm64` or `x86_64`, macOS 13). Intel job is not in CI. An Intel link was not run. |
| F-005 | S0 | Product | Insertion | `commands.rs` called `insert_delta` on every partial. `SFSpeechRecognizer` revises partials, so the app sent backspaces into the focused app. | Fixed. `live_typing` defaults to false. The final string is typed once. |
| F-006 | S0 | Security | Clipboard | `insert/mod.rs` copied the transcript to the clipboard when Accessibility was off, and did not restore the previous clipboard. | Fixed. The clipboard is not touched. The bubble shows the Accessibility error. The text stays in History. |
| F-007 | S1 | Platform | Keychain | Service is `com.utter.app` (`cleanup/mod.rs`). Bundle id is `com.utter.studio` (`tauri.conf.json`). | Left as-is and documented. Changing either string orphans saved keys. A Developer ID build will not read Keychain items created by an ad-hoc build. FROM MEMORY — CHECK on a signed build. |
| F-008 | S0 | Security | Cleanup | The raw transcript was the user message, with no delimiter. A spoken "ignore previous instructions" was instructions to the model. | Fixed: `<transcript>` wrapping, a system rule that overrides the user prompt, and rejection of replies that start like chatbot chatter. This is not a proof against a jailbroken model. |
| F-009 | S1 | Security | Cleanup | HTTP was allowed to any host. Redirects followed. No response cap. Ollama posted to `/v1/chat/completions` and parsed the native `{message.content}` shape, so a normal Ollama reply failed the parse and was silently dropped. | Fixed: https, or http on localhost only; no userinfo in the URL; redirects refused; 32KB cap; both JSON shapes accepted. |
| F-010 | S0 | Rust | Engines | `whisper_cpp.rs` and `apple_silicon.rs` `panic!` in `init`. Non-macOS builds select the whisper engine. | Fixed: `init` returns `EngineError::NotImplemented`, mapped to `E_ENGINE_UNAVAILABLE`. Those builds are not shipped. |
| F-011 | S1 | Rust | Audio | The cpal callback locked a buffer and called into Swift. | Fixed: the callback only `try_send`s. A thread named `utter-audio-feed` calls `feed_audio`. The callback still allocates a `Vec` per chunk. |
| F-012 | S2 | QA | Logs | `get_logs` returned `vec![]`. | Settings has Copy diagnostics. The report is version, status, cleanup type, startup warning, and `crash.log`. No transcripts and no API keys. |
| F-013 | S1 | Platform | Locale | `utter_transcribe.swift` uses `Locale(identifier: "en-US")` only. | Open. Documented. Not a silent fallback to another language. |
| F-014 | S0 | Release | Signing | `tauri.conf.json` has `"signingIdentity": null`. No notarization, no stapler, no updater keys. | Open. Needs an Apple Developer ID. Not invented here. |
| F-015 | S2 | Hygiene | Repo | Unused React files in `components/ui/` and `apps/desktop/src/components/ui/`. | Deleted. |
| F-016 | S0 | Product | Site | Homepage said "Everywhere" and "macOS and Windows", and the engine column said FluidAudio and whisper.cpp. | Copy now says macOS Apple Silicon, Apple on-device English. Unsupported rows say "Not shipped" and are not links. |
| F-017 | S2 | UX | Hotkey | `main.rs` logged a failed hotkey registration and continued. | Studio shows a banner from `get_startup_warning`. Saving a hotkey that fails returns that error to the save button. |
| F-018 | S2 | Product | Dictionary | Default dictionary rewrote "neel" to "NEEL" and "utter" to "Utter Flow". | Default is empty. Existing `config.toml` files are left alone. |
| F-019 | S1 | Security | Logs | `stop_dictation` logged the raw and final transcript with `{:?}`. | Logs the character count only. |
| F-020 | S1 | UX | Newlines | A cleanup model can return newlines. Typing Return in Slack or Terminal can send or run a command. | `sanitize_insertion` turns newlines, tabs, and ESC sequences into spaces or nothing. Tested. |
| F-021 | S1 | Platform | Secure input | No check before synthetic typing. | `IsSecureEventInputEnabled` (Carbon). The unit-test binary linked, so the symbol resolves. Not exercised against a password field in this session. |
| F-022 | S2 | UX | Snippets | Studio showed "my email → name@example.com" as if the feature worked. The button had no handler. | Copy says the cards do nothing. The button is removed. |
| F-023 | S2 | UX | Smart routing | The checkbox was not read or saved. Rust always classified the front app. | Saved as `smart_routing` (default true). It only changes the cleanup prompt. |
| F-024 | S2 | Security | Plugins | `tauri-plugin-shell`, dialog, and notification were registered and unused. | Removed from `Cargo.toml` and `main.rs`. |
| F-025 | S1 | QA | Runtime | No dictation was performed. No packet capture. No clean-account install. | Partially closed 2026-10-29: clean `cargo tauri build` produces `Utter.app` + `Utter_0.1.0_aarch64.dmg` (arm64), and the release binary boots to the tray without panic (`hotkey: registered`, `setup complete`; RSS ~84 MB, CPU 0.3% just after boot). A spoken dictation, packet capture, and clean-account install still need a human at the machine. |
| F-026 | S1 | Rust | Lint | `cargo clippy --all-targets -- -D warnings` failed: 6 errors in `build.rs` (`needless_borrow`, `uninlined_format_args`) and 11 in the crate (`duplicated_attributes` on an `extern` block in `insert/mod.rs`, collapsible `if`, `match`-as-equality, a manual loop, etc.). The ledger implied lint was clean; it was never run with `-D warnings`. | Fixed 2026-10-29. `clippy --all-targets -- -D warnings` and `cargo fmt --check` both pass. The duplicated `#[link]` was split into one `extern` block per framework. |
| F-027 | S1 | Release | CI | `.github/workflows/ci.yml` ran only `cargo test --bin utter`. No clippy, no fmt, no dependency audit — yet the Definition of Done requires "CI green on lint, tests, audit." | Fixed 2026-10-29. CI now runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and a separate `cargo audit` job. |
| F-028 | S1 | Security | Deps | `cargo audit` reported 11 vulnerabilities in the live TLS/HTTP stack (`rustls` 0.23.42, `rustls-webpki`, `webpki-roots`) plus stale `quinn-proto` lock entries not in the build graph. The rustls advisories (TLS 1.3 cross-level acceptance, ServerHello downgrade, distrusted intermediates) bear on the cleanup-endpoint threat model (F-009). | Fixed 2026-10-29 by a targeted `cargo update -p rustls -p rustls-webpki -p webpki-roots -p quinn-proto` (rustls → 0.23.46). `tauri` untouched, so the crate still builds on rustc 1.88. `cargo audit` now reports 0 vulnerabilities; the 13 remaining are `unmaintained`/`unsound` warnings in Tauri's Linux-only GTK/D-Bus transitive deps, which do not fail the gate and are not compiled on macOS. |

## Tests that exist now

```
cargo test --bin utter --manifest-path apps/desktop/src-tauri/Cargo.toml -- logic:: cleanup::policy::
# 9 passed, 0 failed, 2026-10-29, debug profile, macOS 26.5.1 arm64

node --test site/assets/detect.test.js site/assets/releases.test.js
# 29 passed, 0 failed
```

There is no test that speaks into the microphone.
