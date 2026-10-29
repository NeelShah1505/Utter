# ARCHITECTURE — Technical Design

> Current as of 2026-10-29. The speech engine is Apple `SFSpeechRecognizer` on macOS Apple Silicon, forced on-device, English (US). Intel Mac and Windows are not built. `apple_silicon.rs` (Parakeet) and `whisper_cpp.rs` return `EngineUnavailable` and are not selected on macOS.
>
> If this file disagrees with `apps/desktop/src-tauri`, the code wins until this file is updated. See `docs/AUDIT/`.

---

## 1. High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Tauri Shell (Rust)                      │
│                                                              │
│  ┌─────────────┐  ┌─────────────┐  ┌────────────────────┐   │
│  │  Hotkey     │  │  Mic        │  │  Tray + Settings   │   │
│  │  Listener   │  │  Capture    │  │  Window (webview)  │   │
│  └──────┬──────┘  └──────┬──────┘  └─────────┬──────────┘   │
│         │                │                    │              │
│         ▼                ▼                    │              │
│  ┌─────────────────────────────┐              │              │
│  │      Engine Interface       │◄─────────────┘              │
│  │  (trait: start/stop/feed)   │              IPC            │
│  └────────────┬────────────────┘                             │
│               │                                              │
│       ┌───────┴────────┐                                     │
│       │                │                                     │
│       ▼                ▼                                     │
│  ┌─────────┐     ┌──────────┐                                │
│  │ Apple   │     │ whisper  │                                │
│  │ Silicon │     │ .cpp     │                                │
│  │ engine  │     │ engine   │                                │
│  │ (Para-  │     │          │                                │
│  │  keet)  │     │          │                                │
│  └─────────┘     └──────────┘                                │
│                                                              │
│  ┌─────────────────────────────────────────────┐            │
│  │  Optional Cleanup Layer (OFF by default)    │            │
│  │  ┌─────┐  ┌──────────┐  ┌──────────────┐    │            │
│  │  │None │  │  Ollama  │  │  OpenAI-compat│    │            │
│  │  │     │  │ (local/  │  │  (API key)    │    │            │
│  │  │     │  │  remote) │  │               │    │            │
│  │  └─────┘  └──────────┘  └──────────────┘    │            │
│  └─────────────────────────────────────────────┘            │
│                                                              │
│  ┌─────────────────────────────────────────────┐            │
│  │  Text Insertion (accessibility / SendInput) │            │
│  └─────────────────────────────────────────────┘            │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. Process Model

Single process (Tauri default). The webview UI runs in the same process as the Rust core. The ASR engine runs in-process (loaded as a Rust crate or via FFI).

**Rationale for single process:** Simpler IPC, lower latency (no cross-process audio buffering), smaller memory footprint. Tradeoff: a crash in the ASR engine crashes the whole app — acceptable for v1; revisit if stability is an issue.

**Future option (Phase 5 hardening):** Run the ASR engine in a sandboxed child process with minimal permissions. This is documented in SECURITY.md as a known future item.

---

## 3. IPC Contract (Shell ↔ Webview UI)

The webview UI communicates with the Rust shell via Tauri's `invoke` mechanism. Every command is documented here; the webview may only call these.

### 3.1 Commands (webview → shell)

| Command              | Args                                | Returns                          | Notes                                                  |
|----------------------|-------------------------------------|----------------------------------|--------------------------------------------------------|
| `start_dictation`    | —                                   | `Result<(), Error>`              | Begins mic capture + ASR.                              |
| `stop_dictation`     | —                                   | `Result<(), Error>`              | Stops capture; flushes pending audio.                  |
| `get_status`         | —                                   | `Status`                         | `Idle` / `Listening` / `Transcribing` / `Error(String)`|
| `get_settings`       | —                                   | `Settings`                       | See Settings struct below.                             |
| `set_settings`       | `Settings`                          | `Result<(), Error>`              | Persists to OS-appropriate config location.            |
| `list_audio_devices` | —                                   | `Vec<AudioDevice>`               | For settings UI.                                       |
| `list_models`        | —                                   | `Vec<ModelInfo>`                 | For settings UI.                                       |
| `test_cleanup`       | `CleanupConfig`                     | `Result<String, Error>`          | Sends "Hello, world!" through the configured backend.  |
| `get_logs`           | `lines: u32`                        | `String`                         | For the About/Debug panel.                             |

### 3.2 Events (shell → webview)

| Event                | Payload                             | When emitted                                                |
|----------------------|-------------------------------------|-------------------------------------------------------------|
| `transcript_partial` | `{ text: String, ts: u64 }`         | Streaming ASR produces interim text.                        |
| `transcript_final`   | `{ text: String, ts: u64 }`         | ASR finalizes a segment.                                    |
| `cleanup_applied`    | `{ before: String, after: String }` | Cleanup backend transformed text.                           |
| `error`              | `{ code: String, message: String }` | Any error — see Error Codes.                                |
| `state_change`       | `Status`                            | State machine transitioned.                                 |

### 3.3 Error Codes

| Code                   | Meaning                                                | User action                                  |
|------------------------|--------------------------------------------------------|----------------------------------------------|
| `E_MIC_PERMISSION`    | OS denied microphone access                            | Grant in System Settings / Windows Settings  |
| `E_MIC_BUSY`           | Another app holds the mic                              | Close other app                              |
| `E_ACCESSIBILITY`     | macOS accessibility not granted                        | Grant in System Settings → Privacy           |
| `E_MODEL_LOAD`         | ASR model failed to load                               | Reinstall app; check disk space              |
| `E_MODEL_NOT_FOUND`    | Configured model file missing                          | Re-download from settings                    |
| `E_CLEANUP_UNREACHABLE`| Cleanup endpoint not reachable                         | Check endpoint URL / network                 |
| `E_CLEANUP_AUTH`       | Cleanup endpoint returned 401/403                      | Check API key                                |
| `E_CLEANUP_RATE`       | Cleanup endpoint rate-limited                          | Wait; or switch to local Ollama              |
| `E_INSERT_FAILED`     | Couldn't insert text into focused app                  | Make sure target app is focused              |
| `E_INTERNAL`           | Anything else — full stack in log file                 | Check logs; file bug report                  |

---

## 4. ASR Engine Interface (Rust trait)

```rust
// apps/desktop/src-tauri/src/engine/mod.rs (will be created in Phase 2)

pub trait AsrEngine: Send + Sync {
    /// Called once at startup. Load model into memory.
    fn init(config: &EngineConfig) -> Result<Self, EngineError> where Self: Sized;

    /// Begin a transcription session. Returns a session handle.
    fn start_session(&self) -> Result<SessionId, EngineError>;

    /// Feed PCM audio (16kHz, mono, f32). Called ~10x/sec.
    fn feed_audio(&self, session: SessionId, samples: &[f32]) -> Result<(), EngineError>;

    /// Poll for partial transcript. Non-blocking.
    fn poll_partial(&self, session: SessionId) -> Result<Option<String>, EngineError>;

    /// End session, get final transcript.
    fn end_session(&self, session: SessionId) -> Result<String, EngineError>;
}
```

**What actually runs:**
- macOS, both architectures that compile the Swift helper: `MacOsSpeechEngine` in `engine/macos_speech.rs`, linked to `helpers/utter_transcribe.swift`. `requiresOnDeviceRecognition` is set. Locale is `en-US`.
- Any other target: `WhisperCppEngine::init` returns `EngineUnavailable`. It does not panic. That target is not released.

`AppleSiliconEngine` (Parakeet / FluidAudio) is a stub and is not the type selected on macOS.

---

## 5. Cleanup Backend Interface

```rust
pub trait CleanupBackend: Send + Sync {
    async fn refine(&self, raw: &str) -> Result<String, CleanupError>;
}

pub struct NoOpCleanup;       // Default — returns raw unchanged
pub struct LocalOllama { /* http://localhost:11434 */ }
pub struct RemoteOllama { /* user URL + optional bearer token */ }
pub struct OpenAiCompat { /* user URL + API key, OpenAI chat-completions format */ }
```

All implementations speak the same `refine(raw) -> clean` contract. The shell doesn't know or care which is active.

---

## 6. Build Target Matrix

| Target | OS | Arch | Engine | Installer | Shipped? |
|--------|----|------|--------|-----------|----------|
| macOS Apple Silicon | macOS 13+ | arm64 | Apple Speech, on device, en-US | `app-macos-arm64.dmg` | Yes. This is the only release asset. |
| macOS Intel | — | x86_64 | Not built | — | No |
| Windows | — | — | Not built | — | No |

`build.rs` can compile the Swift helper for `x86_64-apple-macos13.0` when `TARGET` says so. That build is not in CI and has not been linked.

---

## 7. CI/CD Pipeline

### 7.1 `build-release.yml` (skeleton — stubbed this session)

Triggers: push of tag `v*.*.*` on `main`.

Jobs:
- `build-macos-arm64` runs `cargo tauri build` and uploads `app-macos-arm64.dmg`.
- `release` publishes that file and `app-macos-arm64.dmg.sha256`.

`.github/workflows/ci.yml` runs the site tests on Ubuntu and `cargo test --bin utter` on macos-14. Intel and Windows jobs were removed because they uploaded placeholder files.

### 7.2 `deploy-site.yml`

Triggers: push to `main` affecting `site/**`.

Steps:
1. Checkout
2. Upload `site/` to GitHub Pages using `actions/deploy-pages@v4`
3. (No build step — see DESIGN.md §3.1)

---

## 8. UA Detection Spec (client-side, on the marketing site)

The site must detect, from the visitor's browser:

| Detected               | Maps to asset                          |
|------------------------|----------------------------------------|
| macOS + Apple Silicon  | `app-macos-arm64.dmg`                  |
| macOS + Intel          | `app-macos-x64.dmg`                    |
| Windows + x64          | `app-windows-x64.msi`                  |
| Windows + ARM64        | `app-windows-arm64.msi` (experimental) |
| Linux / iOS / Android  | Show all options; highlight none       |
| Unknown                | Show all options; highlight none       |

### 8.1 Detection algorithm

```
1. Try navigator.userAgentData (Chromium 90+):
   - platform: 'macOS' | 'Windows' | 'Linux' | 'Android' | 'iOS'?
   - getHighEntropyValues(['architecture', 'bitness'])
     → architecture: 'arm' | 'x86', bitness: '64' | '32'
2. Fallback: parse navigator.userAgent
   - Mac: /Mac/i.test(userAgent)
   - Windows: /Windows/i.test(userAgent)
   - iOS: /iPhone|iPad/i.test(userAgent)
   - Android: /Android/i.test(userAgent)
   - Linux: /Linux/i.test(userAgent)
3. Fallback for arch on Mac (no UA hint):
   - Use WebGL renderer string:
     canvas.toDataURL → check for 'Apple M' / 'Apple GPU' → arm64
     Otherwise default to x64 (Intel Mac)
   - Document this fallback is heuristic; "Other platforms" link always visible
4. Fallback for arch on Windows:
   - navigator.userAgentData architecture if available
   - Else: assume x64 (vast majority)
   - Show ARM64 link prominently in "Other platforms" for Surface users
5. If anything inconclusive → show all four links, no auto-selected primary.
```

### 8.2 Why not use `navigator.platform`?

Deprecated. Still works in current browsers but will be removed. We use it only as a last-resort tiebreaker.

### 8.3 Why not server-side detection?

The site is static on GitHub Pages. No server. Client-side is the only option. Also: client-side keeps the UA string on the user's device — privacy-positive.

---

## 9. GitHub Releases API Integration

```js
// site/assets/releases.js
const REPO = 'NeelShah1505/Utter'; // placeholder — replace when Q-001 resolved
const API = `https://api.github.com/repos/${REPO}/releases/latest`;

export async function fetchLatestRelease() {
  const res = await fetch(API, { headers: { 'Accept': 'application/vnd.github+json' } });
  if (!res.ok) throw new Error(`GitHub API ${res.status}`);
  const data = await res.json();
  return {
    version: data.tag_name,           // e.g. "v1.0.0"
    publishedAt: data.published_at,
    assets: data.assets.map(a => ({
      name: a.name,                   // e.g. "app-macos-arm64.dmg"
      url: a.browser_download_url,
      size: a.size,
      downloadCount: a.download_count,
    })),
    releaseNotes: data.body_html,
  };
}
```

**Caching:** GitHub's API allows 60 req/hr per IP unauthenticated. For a marketing site this is fine — but cache the response in `sessionStorage` for 5 minutes to be polite.

**Fallback:** If the API returns 404 (no releases) or any error, fall back to a static link to the releases page. Never show a broken download button.

---

## 10. Directory Layout

```
/
├── apps/
│   └── desktop/           # Tauri app — Phase 1+
│       ├── src-tauri/
│       │   ├── src/
│       │   │   ├── main.rs
│       │   │   ├── engine/
│       │   │   ├── cleanup/
│       │   │   ├── hotkey/
│       │   │   ├── mic/
│       │   │   ├── insert/
│       │   │   └── settings/
│       │   ├── Cargo.toml
│       │   └── tauri.conf.json
│       ├── src/
│       └── package.json
├── site/
│   ├── index.html
│   ├── assets/
│   │   ├── styles.css
│   │   ├── detect.js
│   │   ├── releases.js
│   │   ├── ui.js
│   │   └── favicon.svg
│   └── robots.txt
├── .github/
│   └── workflows/
│       ├── build-release.yml
│       └── deploy-site.yml
├── docs/
│   ├── CONTEXT.md
│   ├── DESIGN.md
│   ├── SECURITY.md
│   ├── ARCHITECTURE.md
│   ├── TESTING.md
│   └── MEMORY.md
├── README.md
└── LICENSE
```

---

## 11. Configuration File Locations (per OS)

| Setting              | macOS                                              | Windows                                  |
|----------------------|----------------------------------------------------|------------------------------------------|
| App config (non-secret) | `~/Library/Application Support/Utter/config.toml` | `%APPDATA%\Utter\config.toml`        |
| Logs                 | `~/Library/Logs/Utter/`                          | `%APPDATA%\Utter\logs\`                |
| Bundled models       | `~/Library/Application Support/Utter/models/`    | `%APPDATA%\Utter\models\`              |
| User-added models    | Same dir, user can drop files                      | Same dir                                 |
| API keys (secret)    | Keychain, service `com.utter.app`                | Credential Manager, target `Utter::Cleanup` |

**Config file format:** TOML. Human-readable, human-editable, no schema validation surprise.

---

## 12. Performance Budget

| Metric                                       | Budget       |
|----------------------------------------------|--------------|
| Marketing site Lighthouse performance        | ≥ 95         |
| Marketing site LCP                            | < 1.5s on 4G |
| Marketing site total JS (gzipped)            | < 8KB        |
| Marketing site total CSS (gzipped)           | < 10KB       |
| App cold-start to ready                      | < 800ms      |
| Hotkey press → mic capture begins             | < 50ms       |
| Voice end → transcript inserted (no cleanup) | < 300ms      |
| Cleanup round-trip (local Ollama)            | < 1500ms     |
| App idle RAM                                  | < 80MB       |

If a budget is exceeded, the PR must justify it or fix it.
