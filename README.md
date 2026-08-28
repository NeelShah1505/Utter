# Utter

> Type with your voice. Everywhere. Offline.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Utter is a beautiful, open-source, cross-platform voice dictation app. Press a hotkey, speak, and your words appear in any app which is offline, with no account and no telemetry. It features a stunning glassmorphic UI built natively for macOS.

---

## How it works

1. Install the `.dmg` from the Releases page.
2. Grant Microphone and Accessibility permissions.
3. Press `Cmd+Shift+D` anywhere.
4. Speak. Text appears in your active application, with a beautiful real-time transcript.

No network connection required. No account. No cloud.

---

## Download

Visit the [releases page](https://github.com/NeelShah1505/Utter/releases/latest) to download the latest build for your platform:

| Platform              | File                        |
|-----------------------|-----------------------------|
| macOS (Apple Silicon) | `app-macos-arm64.dmg`       |

*(Note: Windows and Intel Mac versions are coming soon!)*

> **macOS Security Note**: Utter is currently an ad-hoc signed open-source project. When you first open the app, macOS might block it. To open it, go to your Applications folder, **Right-Click** the Utter icon, and select **Open**.

---

## Features

- **Blazing Fast**: Uses the native macOS ASR engine (Speech framework).
- **Beautiful UI**: Heavy glassmorphism and smooth animations.
- **Privacy First**: Everything happens on your device.
- **Global Hotkey**: Instantly dictate anywhere.

---

## Tech stack

- **Desktop shell:** [Tauri v2](https://v2.tauri.app/) (Rust core + native webview)
- **Frontend:** Vanilla JS / CSS with ESBuild
- **ASR:** macOS Native Speech framework

---

## License

MIT — see [LICENSE](LICENSE).
