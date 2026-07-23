# Phase 1 starts here

This directory will contain the Tauri desktop application — the voice dictation shell.

**Do not add any code here during Phase 0.** The desktop app is out of scope until Phase 0 is reviewed and approved.

## What goes here (Phase 1+)

```
apps/desktop/
├── src-tauri/              # Rust core (Tauri shell)
│   ├── src/
│   │   ├── main.rs
│   │   ├── engine/         # ASR engine trait + implementations
│   │   │   ├── mod.rs
│   │   │   ├── apple_silicon.rs
│   │   │   └── whisper_cpp.rs
│   │   ├── cleanup/        # Optional cleanup layer backends
│   │   ├── hotkey/         # Global hotkey listener (per-OS)
│   │   ├── mic/            # Audio capture
│   │   ├── insert/         # Text insertion per OS
│   │   └── settings/       # Config + keychain
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                    # Webview UI (HTML/CSS/JS)
│   ├── index.html
│   ├── settings.html
│   └── ...
└── package.json
```

## References

- [docs/ARCHITECTURE.md](../../docs/ARCHITECTURE.md) — full technical spec, IPC contract, engine trait
- [docs/DESIGN.md](../../docs/DESIGN.md) — UX principles for the desktop app
- [docs/CONTEXT.md](../../docs/CONTEXT.md) — current phase and what's next
