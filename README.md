# Utter

Type with your voice on a Mac. Speech is transcribed on your machine; nothing leaves it unless you turn on optional AI cleanup.

Utter is a menu-bar dictation app for **macOS Apple Silicon**. Press a hotkey, speak English, press it again, and the text is typed into the focused app. MIT license. No account. No telemetry.

**Two on-device speech engines:**
- **Whisper (default)** — whisper.cpp with a downloadable model (`base.en` ≈142 MB recommended). Higher accuracy on numbers, names, and code terms, and your Dictionary biases recognition. Runs fully offline.
- **Apple on-device** — zero download, English (US). Used automatically as a fallback until you download a Whisper model, or selectable in Settings.

| Platform | Status |
|----------|--------|
| macOS Apple Silicon (macOS 13+) | Works. Whisper (download) or Apple on-device, English (United States). |
| macOS Intel | Not shipped. |
| Windows | Not shipped. |
| Linux | Not shipped. |

The build is ad-hoc signed. The first time you open it, right-click the app and choose Open.

## What you need

1. A Mac with Apple silicon, macOS 13 or later.
2. A speech model. Open Studio → Settings → Transcription engine and download a Whisper model (`base.en` is recommended). Until you do, Utter falls back to Apple on-device speech, which needs the English (United States) dictation language installed in System Settings.
3. Permissions, the first time you dictate: Microphone and Accessibility (and Speech Recognition if you use the Apple engine). Accessibility is what lets Utter type into other apps. Without it, the text is kept in History and is not typed.

Default hotkey: `Cmd+Shift+D`.

## What it does not do

- It does not run on Windows or Intel Macs. Those downloads are not published.
- It does not speak languages other than English (US).
- It does not type while you are still talking, unless you turn on **Live typing** in Settings. That mode is experimental. It sends backspaces when Apple revises a partial, and it can fight with your own typing.
- It does not type into password fields.
- It does not phone home. A crash writes a note to `~/Library/Logs/Utter/crash.log` and nowhere else.

Optional cleanup (Ollama or any OpenAI-compatible server) is off until you enable it. Keys go in the macOS Keychain under the service `com.utter.app`. The bundle id is `com.utter.studio`. Those two strings are different on purpose; do not "fix" one to match the other or existing keys become unreachable.

## Build from source

```bash
cd apps/desktop
npm install
npm run build
cargo install tauri-cli --version "^2"
cargo tauri dev
```

Rust tests, including the text-editing tests:

```bash
cargo test --bin utter --manifest-path apps/desktop/src-tauri/Cargo.toml
```

Site tests:

```bash
node --test site/assets/detect.test.js site/assets/releases.test.js
```

## Where things live

| Data | Path |
|------|------|
| Settings | `~/Library/Application Support/Utter/config.toml` |
| History | `~/Library/Application Support/Utter/history.json` (cleared from Studio) |
| Crash note | `~/Library/Logs/Utter/crash.log` |
| API keys | Keychain service `com.utter.app` |

## Layout

- `apps/desktop/` — Tauri 2 app. Rust core, a small Swift speech helper, HTML/CSS/JS UI.
- `site/` — static download page.
- `docs/` — design notes and the audit in `docs/AUDIT/`.

## License

MIT. See [LICENSE](LICENSE).
