# Release readiness

**No-go.** Do not post this on Hacker News yet.

Host: macOS 26.5.1 (25F80), arm64. rustc 1.88.0. Swift 6.1. Debug `cargo test`, 2026-10-29.

## Proven

- `setRequiresOnDeviceRecognition:` and `supportsOnDeviceRecognition` are in the compiled Swift object `utter_transcribe.o` from the debug build of this session. `strings` on that object also shows `setAddsPunctuation:`.
- `cargo test --bin utter` filter `logic::` and `cleanup::policy::`: 9 passed, 0 failed.
- `node --test` on the site suites: 29 passed, 0 failed.
- The test binary linked, which means `IsSecureEventInputEnabled` in Carbon resolved. It was not called against a password field.
- CI no longer has jobs that write the string `placeholder - not a real installer` into a `.dmg` or `.msi`.

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
| Every claim true | README and the homepage were rewritten. Older docs under `docs/` still describe FluidAudio and four platforms in places. |
| Nothing downloadable that cannot run | CI and the homepage agree. A tag has not been pushed, so GitHub Releases was not checked live. |
| Panic hook, local only | `main.rs` writes `~/Library/Logs/Utter/crash.log`. No "copy diagnostics" button. |
| Signed and notarized | No. |
| Chaos matrix | Not run. |
| First run under 90 seconds | Not timed. |
