# CONTEXT — Running Project Memory

> **READ THIS FILE FIRST at the start of every session.**
> **APPEND TO IT before ending every session.**
> Most recent entry on top. Never delete prior entries — strike through if obsolete.

---

## Project Identity

- **Name (working): Utter (rename pending — see Decision D-002)
- **Tagline:** "Type with your voice. Everywhere. Offline."
- **License:** MIT
- **Repo (intended):** `github.com/NeelShah1505/Utter` (org TBD — see Open Question Q-001)
- **Status:** Phase 0 — scaffold + marketing site (this session's scope)

---

## Current Phase

### Phase 0 (CURRENT — this session)
- [x] Repo scaffold created
- [x] `docs/` populated (this commit)
- [x] `apps/desktop/` empty Tauri scaffold placeholder
- [x] `site/` static marketing site built
- [x] OS/architecture detection implemented and tested
- [x] GitHub Releases API integration for live version/asset links
- [x] `.github/workflows/build-release.yml` skeleton with correct artifact names
- [x] `.github/workflows/deploy-site.yml` for static deploy
- [ ] Lighthouse run, scores recorded in TESTING.md
- [x] JS-disabled fallback verified (noscript block in index.html)

### Phase 1 (NEXT — do NOT start until Phase 0 is reviewed)
- Tauri shell scaffold (Rust core + web UI shell)
- IPC contract stub between shell and engine
- Global hotkey listener (per-OS)
- Microphone permission flow on macOS + Windows

### Phase 2
- macOS Apple Silicon engine: FluidAudio + Parakeet TDT v3 via CoreML/ANE
- Streaming transcription pipeline
- UI: floating transcript bubble, settings panel

### Phase 3
- whisper.cpp integration for macOS Intel + Windows x64 + Windows ARM64
- CUDA runtime detection on Windows

### Phase 4
- Cleanup layer: local Ollama, remote Ollama, OpenAI-compatible API key
- Settings UI for cleanup backend selection
- OS keychain storage for API keys

### Phase 5
- Polish, packaging, notarization (macOS), code signing (Windows)
- Public 1.0 release

---

## Session Log

### Session 1 — 2024-07-23
**Done:**
- Created repo scaffold per `ARCHITECTURE.md` directory layout
- Authored `CONTEXT.md`, `DESIGN.md`, `SECURITY.md`, `ARCHITECTURE.md`, `TESTING.md`, `MEMORY.md`
- `apps/desktop/README.md` placeholder noting "Phase 1 starts here"
- `README.md` at repo root
- `LICENSE` — MIT
- `site/index.html` with all 6 sections: Hero, Features, How It Works, System Requirements, Open Source, Footer
- `site/assets/styles.css` — responsive, dark mode, system fonts, warm off-white + deep teal palette
- `site/assets/detect.js` — OS/arch detection: userAgentData → UA parse → WebGL heuristic
- `site/assets/detect.test.js` — 12-row UA matrix test coverage (Node built-in runner)
- `site/assets/releases.js` — GitHub Releases API fetch, 5-min sessionStorage cache, graceful fallback
- `site/assets/releases.test.js` — mock-based tests for fetch and fallback behavior
- `site/assets/ui.js` — wires detect + releases to DOM, updates download button and "Other platforms" list
- `site/assets/favicon.svg` — microphone SVG icon (<1KB)
- `site/robots.txt`
- `.github/workflows/build-release.yml` — 4 build jobs + release job, exact canonical filenames, functional artifact upload
- `.github/workflows/deploy-site.yml` — triggers on site/** push to main, deploys to GitHub Pages

**Decisions made:**
- D-001: Static site framework = vanilla HTML/CSS/JS (see DESIGN.md §3.1 for rationale)
- D-002: Working product name "Utter" — placeholder, final name TBD before public launch
- D-003: Deploy target = GitHub Pages (zero-cost, lives next to repo, no vendor lock-in)
- D-004: No build step for the site — ships as raw static files (see DESIGN.md)

**Currently broken / incomplete:**
- `NeelShah1505/Utter` placeholders throughout — blocked on Q-001 (no GitHub org chosen yet)
- Lighthouse scores not yet recorded — need to serve site locally and run CLI
- No actual GitHub Release exists — the site's fallback to `/releases/latest` is active

**Next session should:**
1. Replace all `NeelShah1505/Utter` placeholders once Q-001 is resolved
2. Run Lighthouse against locally-served site and record scores in TESTING.md §4
3. Verify actual download URLs once first GitHub Release is created
4. Begin Phase 1 (Tauri shell scaffold) only after Phase 0 review sign-off

---

## Decision Log

| ID  | Date       | Decision                                                            | Rationale                                          | Revisit?          |
|-----|------------|---------------------------------------------------------------------|----------------------------------------------------|-------------------|
| D-001 | 2024-07-23 | Vanilla HTML/CSS/JS for site, no framework                          | Lighthouse 95+, instant load, no framework needed | Only if site grows complex |
| D-002 | 2024-07-23 | Working name "Utter"                                              | Placeholder; avoids bikeshedding now               | Yes, before 1.0   |
| D-003 | 2024-07-23 | GitHub Pages for site deploy                                        | Free, lives with repo, no vendor lock-in           | No                |
| D-004 | 2024-07-23 | No build step for the marketing site                                | Maximizes load speed; HTML/CSS/JS hand-written     | Only if forced    |
| D-005 | 2024-07-23 | Tauri over Electron for the desktop shell                           | ~10MB vs ~100MB install, native perf, Rust core    | No                |
| D-006 | 2024-07-23 | FluidAudio + Parakeet TDT v3 on Apple Silicon                       | ANE acceleration → real-time on M1+; see DESIGN.md | Only if model license changes |
| D-007 | 2024-07-23 | whisper.cpp on Intel Mac + Windows                                  | Best CPU STT; CUDA path on Windows if NVIDIA detected | No             |
| D-008 | 2024-07-23 | Cleanup layer OFF by default, pluggable backend                     | Core promise = works offline, zero config          | No                |

---

## Open Questions (UNRESOLVED — ask before proceeding if relevant)

- **Q-001:** What is the final GitHub org name and repo name? Currently using `NeelShah1505/Utter` placeholder. Site, CI, README all reference this — must be resolved before first deploy.
- **Q-002:** Final product name? "Utter" is a placeholder; may collide with existing trademarks. Needs a search before 1.0.
- **Q-003:** Code signing budget for Windows (EV cert ~$300/yr)? Without it, users see SmartScreen warnings. Deferred to Phase 5.
- **Q-004:** Notarization for macOS — Apple Developer ID required ($99/yr). Deferred to Phase 5.
- **Q-005:** Should we ship a universal macOS .dmg combining arm64+x64, or keep two separate builds? Currently scoped as two separate. Revisit if user feedback says otherwise.

---

## Hard Constraints (NEVER violate — see also MEMORY.md)

1. Core dictation MUST work with zero network calls, zero accounts, zero API keys.
2. No telemetry, no analytics in the desktop app. Period.
3. Site analytics (if any) MUST be self-hosted, anonymous, cookieless, GDPR-friendly — and disclosed in SECURITY.md.
4. Exact build target filenames MUST match MEMORY.md §Canonical Filenames — the site and CI must agree byte-for-byte.
5. No fake content shipped as real. No placeholder testimonials, no fake screenshots, no inflated user counts. Mark placeholders with visible `TODO:` comments.
6. MIT license only. No GPL/AGPL dependencies snuck in via transitive deps — verify with `cargo license` and `npm ls` before each release.
7. API keys MUST be stored in OS keychain (Keychain on macOS, Credential Manager on Windows). Never plaintext. Never in `localStorage`. Never in `appsettings.json`.
8. Every `try/catch`, every `Result<T,E>`, every `Promise.catch` MUST handle failure explicitly. Empty catches are a bug.

---

## Glossary

- **Shell** — the Tauri wrapper (Rust core + webview UI). Owns hotkey, mic permission, tray, settings.
- **Engine** — the ASR backend (FluidAudio/Parakeet on Apple Silicon, whisper.cpp elsewhere). Pluggable.
- **Cleanup layer** — optional post-ASR text refinement via LLM (local Ollama / remote Ollama / OpenAI-compatible API). OFF by default.
- **Build target** — a specific OS+arch combination producing one installer file. Four targets this project (see MEMORY.md).
- **Asset** — a downloadable file attached to a GitHub Release.
