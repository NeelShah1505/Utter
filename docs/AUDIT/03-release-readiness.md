# Release readiness

**No-go.** Do not post this on Hacker News yet.

Host: macOS 26.5.1 (25F80), arm64. rustc 1.88.0. Swift 6.1. Debug `cargo test`, 2026-10-29.

## Proven

- The project builds clean from this machine: `cargo build` on `apps/desktop/src-tauri` finished with **0 warnings**; the binary is `Mach-O 64-bit executable arm64`.
- A full `cargo tauri build` (release) produced `Utter.app` and `Utter_0.1.0_aarch64.dmg` (4.8 MB, arm64). The release workflow renames that dmg to `app-macos-arm64.dmg` and publishes its `.sha256`, which is the exact filename the site links.
- The release binary boots without panic: log shows `Utter v0.1.0 starting`, `hotkey: registered`, `setup complete`. RSS ~84 MB and CPU 0.3% just after boot.
- With the app idle and running, `lsof -nP -i -p <pid>` shows **no open network sockets**. This is telemetry-at-rest evidence only; it is not a dictation-time packet capture.
- `setRequiresOnDeviceRecognition:` and `supportsOnDeviceRecognition` are in the compiled Swift object `utter_transcribe.o`. `strings` on that object also shows `setAddsPunctuation:`.
- Lint gate is clean: `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` both pass (after F-026 fixes).
- `cargo audit` reports **0 vulnerabilities** (after F-028); 13 remaining items are allowed unmaintained/unsound warnings in Tauri's Linux-only transitive deps.
- `cargo test --bin utter`: 9 passed, 0 failed.
- `node --test` on the site suites: 29 passed, 0 failed.
- The test binary linked, which means `IsSecureEventInputEnabled` in Carbon resolved. It was not called against a password field.
- CI no longer has jobs that write the string `placeholder - not a real installer` into a `.dmg` or `.msi`, and now runs fmt, clippy, tests, and audit.

## Not proven

| Check | What is needed |
|-------|----------------|
| A dictation produces text | Run the app, grant Microphone, Speech Recognition, and Accessibility, speak, see text in Notes. |
| No packets during dictation | Packet capture with cleanup off and Wi-Fi on, then again with Wi-Fi off. Little Snitch, `tcpdump`, or `nettop`. |
| On-device language missing | A Mac without the English (US) dictation asset. Expect a refusal, not a network fallback. |
| Password field | Focus a password box, dictate, confirm nothing is typed. |
| Notarization | Apple Developer ID, `notarytool`, staple. No certificate is in this repo and none was invented. |
| Clean install | A fresh user account, the signed dmg, first launch without right-click. Today the binary is ad-hoc signed, so Gatekeeper will block a normal double-click. |
| Hotkey latency, idle CPU, 15-minute session | Not measured. No number in the README is a measurement from this session. |
| Windows or Intel | Not built. |

## Go / no-go from the prompt

| Item | State |
|------|--------|
| Zero open S0 | No. F-014 (notarization) is open. F-002 is fixed in code and unverified on the wire. |
| Every claim true | README, homepage, and the current-fact banners in `docs/ARCHITECTURE.md`, `DESIGN.md`, `MEMORY.md`, `TESTING.md`, and `CONTEXT.md` match the code. Older phase tables remain as history under those banners. |
| Nothing downloadable that cannot run | CI and the homepage agree. A tag has not been pushed, so GitHub Releases was not checked live. |
| Panic hook, local only | `main.rs` writes `~/Library/Logs/Utter/crash.log`. No "copy diagnostics" button. |
| Signed and notarized | No. |
| Chaos matrix | Not run. |
| First run under 90 seconds | Not timed. |
