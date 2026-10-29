# Utter

Type with your voice on a Mac. Speech stays on the machine unless you turn on optional AI cleanup.

Utter is a menu-bar dictation app for **macOS Apple Silicon**. Press a hotkey, speak English, press it again, and the text is typed into the focused app. MIT license. No account. No telemetry.

| Platform | Status |
|----------|--------|
| macOS Apple Silicon (macOS 13+) | Works. Apple on-device speech, English (United States). |
| macOS Intel | Not shipped. |
| Windows | Not shipped. |
| Linux | Not shipped. |

The build is ad-hoc signed. The first time you open it, right-click the app and choose Open.

## What you need

1. A Mac with Apple silicon, macOS 13 or later.
2. English (United States) dictation language data downloaded. If it is missing, Utter stops instead of sending audio to Apple.
3. Three permissions, the first time you dictate: Microphone, Speech Recognition, and Accessibility. Accessibility is what lets Utter type into other apps. Without it, the text is kept in History and is not typed.

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
