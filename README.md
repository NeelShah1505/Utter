# Dictate

> Type with your voice. Everywhere. Offline.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Dictate is an open-source, cross-platform voice dictation app. Press a hotkey, speak, and your words appear in any app — offline, with no account and no telemetry.

---

## How it works

1. Install the app for your platform.
2. Grant microphone (and, on macOS, Accessibility) permission once.
3. Press `Cmd+Shift+D` (macOS) or `Ctrl+Shift+D` (Windows) anywhere.
4. Speak. Text appears in the focused application.

No network connection required. No account. No cloud.

---

## Download

Visit the [releases page](https://github.com/NeelShah1505/Utter/releases/latest) to download the latest build for your platform:

| Platform              | File                        |
|-----------------------|-----------------------------|
| macOS (Apple Silicon) | `app-macos-arm64.dmg`       |
| macOS (Intel)         | `app-macos-x64.dmg`         |
| Windows 10/11 x64     | `app-windows-x64.msi`       |
| Windows ARM64         | `app-windows-arm64.msi`     |

<!-- TODO: Replace NeelShah1505/Utter with real values once Q-001 is resolved (see docs/CONTEXT.md) -->

---

## Project status

**Phase 0 — scaffold + marketing site.** The desktop app does not exist yet. See [docs/CONTEXT.md](docs/CONTEXT.md) for the current phase and roadmap.

| Phase | What                              | Status           |
|-------|-----------------------------------|------------------|
| 0     | Scaffold + marketing site         | ✅ In progress   |
| 1     | Tauri shell + hotkey + mic        | ⏳ Not started   |
| 2     | Apple Silicon ASR engine          | ⏳ Not started   |
| 3     | whisper.cpp (Intel + Windows)     | ⏳ Not started   |
| 4     | Optional AI cleanup layer         | ⏳ Not started   |
| 5     | Polish, signing, 1.0 release      | ⏳ Not started   |

---

## Tech stack

- **Desktop shell:** [Tauri](https://tauri.app/) (Rust core + native webview)
- **ASR on Apple Silicon:** FluidAudio + Parakeet TDT v3 (CoreML/ANE)
- **ASR on Intel Mac / Windows:** [whisper.cpp](https://github.com/ggerganov/whisper.cpp)
- **Optional cleanup:** Local Ollama / Remote Ollama / OpenAI-compatible API (OFF by default)
- **Marketing site:** Vanilla HTML/CSS/JS, no framework, no build step

---

## Documentation

| File | Contents |
|------|----------|
| [docs/CONTEXT.md](docs/CONTEXT.md) | Running session log — read this first every session |
| [docs/DESIGN.md](docs/DESIGN.md) | Product & UX decisions and rationale |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Technical design: IPC contract, engine trait, CI pipeline |
| [docs/SECURITY.md](docs/SECURITY.md) | Threat model, privacy guarantees, credential storage |
| [docs/TESTING.md](docs/TESTING.md) | Test plan, UA matrix, Lighthouse targets, QA checklist |
| [docs/MEMORY.md](docs/MEMORY.md) | Hard invariants, canonical strings, anti-hallucination reference |

---

## Contributing

<!-- TODO: Add CONTRIBUTING.md once the app reaches Phase 1 and has something to contribute to -->

Issues and discussions are welcome. For security vulnerabilities, see [docs/SECURITY.md](docs/SECURITY.md) §9.

---

## License

MIT — see [LICENSE](LICENSE).
