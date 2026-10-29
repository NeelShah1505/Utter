# DESIGN — Product & UX Decisions

> Rationale for every significant design choice. Update whenever a decision changes.
> If a decision is contentious or revisited, log it in CONTEXT.md §Decision Log too.

---

## 1. Product Principles

1. **Offline first, online optional.** The app's core value — "type with your voice, anywhere" — must work with the network cable unplugged. Any feature that requires network is opt-in and clearly labeled.
2. **Zero configuration to first transcription.** Install → grant mic permission → press hotkey → talk. No model downloads at first launch if we can avoid it (bundle a small default model; let power users swap).
3. **Privacy is the default, not a setting.** No telemetry, no account, no cloud — these aren't toggles, they're absent.
4. **Speed is a feature.** Perceived latency from voice-end to text-inserted must be <300ms on a mid-range laptop. If we can't hit that, the feature isn't done.
5. **Honest marketing.** No fake testimonials, no inflated numbers, no screenshots of features that don't exist. If we don't have it, we don't show it.
6. **Open source means open process.** Decisions, bugs, and tradeoffs are documented publicly. ROADMAP lives in the repo.

---

## 2. Tech Stack Choices

### 2.1 Desktop Shell: Tauri (vs. Electron, vs. native)

| Option        | Pros                                          | Cons                                            | Verdict |
|---------------|-----------------------------------------------|-------------------------------------------------|---------|
| **Tauri**     | ~10MB install, Rust core, native webview, low RAM | Smaller plugin ecosystem than Electron; webview quirks per OS | **Chosen** (D-005) |
| Electron      | Mature, huge ecosystem, predictable webview   | ~100MB install, ~150MB RAM idle, Chromium baggage | Rejected — bloat violates "speed is a feature" |
| Native (Swift/C#/Qt) | Best perf, smallest binary               | Three codebases; whisper.cpp/CoreML bridge work duplicated per platform | Rejected — maintenance cost too high for solo/small team |

**Rationale:** Tauri gives us a single Rust codebase for the shell, native webviews (no Chromium bundle), and a path to ~10MB installers. The webview quirks are manageable because our UI is simple (transcript bubble + settings panel).

### 2.2 ASR engine that ships: Apple Speech, on device

The 2024 plan was FluidAudio + Parakeet on Apple Silicon and whisper.cpp everywhere else. Those engines were not built. `apple_silicon.rs` and `whisper_cpp.rs` return an error and are not used on macOS.

macOS uses `SFSpeechRecognizer` through `helpers/utter_transcribe.swift`. Recognition must stay on device. If English (US) on-device speech is unavailable, dictation does not start. There is no network fallback.

Live typing of partials is off by default. The finished sentence is typed once. Newlines are not typed, because Return sends a chat message and submits a shell line.

### 2.3 Other platforms

Not shipped. Do not describe Windows or Intel Mac builds as available.

### 2.4 Cleanup Layer: Pluggable, OFF by Default

**Why optional:** The product promise is "works offline, zero config." LLM cleanup breaks both. So: default OFF, user opts in, and the user picks the backend.

**Why pluggable backends (none / local Ollama / remote Ollama / OpenAI-compatible API key):**
- *None* — the default. App does ASR only.
- *Local Ollama* — privacy-preserving; requires user to install Ollama separately (we don't bundle it).
- *Remote Ollama* — for users who already run Ollama on a home server.
- *OpenAI-compatible API key* — for users who want best quality and accept sending text to a third party. Stored in OS keychain (see SECURITY.md).

We expose a single OpenAI-compatible HTTP interface regardless of backend, so adding a new backend = adding a new config endpoint.

---

## 3. Website Decisions

### 3.1 Site Framework: Vanilla HTML/CSS/JS (no build step) — D-001

**Rationale:**
- The site is 5–7 sections, 1 page. A framework is overhead.
- Lighthouse 95+ requires minimal JS. A build step adds risk of shipping unused code.
- No build step = no Node version requirement for site contributors. Lower friction.
- If the site ever grows complex (blog, docs subsite), revisit and migrate to Astro.

**Tradeoffs accepted:**
- Hand-written HTML is verbose. Acceptable for a single landing page.
- No component reuse via JSX. We use HTML `<template>` + small vanilla JS for the few repeating elements (download buttons).

### 3.2 Deploy Target: GitHub Pages — D-003

**Rationale:**
- Free, lives next to the source, no separate account.
- Custom domain supported via CNAME.
- Deploy via GitHub Action on push to `main`.
- No vendor lock-in: the site is static, can move to Netlify/Vercel in 5 minutes if needed.

**Tradeoffs:**
- GitHub Pages has a 1GB site size limit and 100GB/month bandwidth. Fine for a marketing site.
- No edge functions — but we don't need any. The GitHub Releases API is called client-side.

### 3.3 Visual Design Direction

- **Palette:** Neutral (warm off-white background `#FAF8F5`, near-black text `#1A1A1A`), one accent color (deep teal `#0E6B6B` — communicates calm, trust, focus; not the overused tech-blue).
- **Typography:** System font stack (`-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif`) for instant load and native feel. Mono accents for code/filenames (`ui-monospace, "SF Mono", "Cascadia Mono", monospace`).
- **Layout:** Single column, max-width 720px for prose, max-width 1200px for hero. Generous whitespace (8px base grid).
- **Iconography:** Inline SVG only, no icon font, no icon library. Each icon <1KB.
- **Motion:** Minimal. Hover states only. No scroll animations. No carousels. Respect `prefers-reduced-motion`.
- **Imagery:** None until we have real screenshots. Use type and color, not stock photos.

### 3.4 Tone of Voice

- Factual, not promotional. "Works offline" not "Blazingly fast offline magic."
- Short sentences. Active voice. No marketing weasel words ("seamless," "revolutionary," "game-changing").
- Technical accuracy over persuasion. If a feature has a caveat, state the caveat.
- Use "you" for the user, "we" for the project maintainers, never "I".
- No emoji in marketing copy. (Allowed in code comments and docs if it aids scanning.)

---

## 4. Website Structure

```
site/
├── index.html              # Single-page landing
├── assets/
│   ├── styles.css          # All styles, one file
│   ├── detect.js           # OS/arch detection (no deps)
│   ├── releases.js         # GitHub Releases API fetcher (no deps)
│   ├── ui.js               # Wire detection → DOM (no deps)
│   └── favicon.svg
├── robots.txt
└── CNAME                   # If using custom domain — otherwise omit
```

**Sections of `index.html`:**
1. Hero — headline, subhead, primary Download button (JS-populated label), secondary "Other platforms" disclosure.
2. Features — 5 cards: streaming transcription, works offline, no account, optional AI cleanup, open source.
3. How it works — 3 steps: install → grant mic → press hotkey. Plain language.
4. System requirements — table per platform.
5. Open source — link to repo, license badge, contributing link.
6. Footer — copyright, license, GitHub link, "No cookies. No tracking."

**JS-disabled fallback:** Hero shows a `<noscript>` block with all four download links as a static list. The dynamic button is hidden via `<noscript>` CSS. Repo link is always visible (never JS-dependent).

---

## 5. UX Principles for the Desktop App (Phase 1+)

1. **One global hotkey to start/stop dictation.** Default: `Cmd/Ctrl+Shift+D`. User-configurable.
2. **Visual feedback when listening.** A small floating indicator (not a full window) near the cursor or system tray.
3. **Text insertion = direct keyboard simulation.** Not clipboard paste (breaks user's clipboard). Use accessibility APIs per OS.
4. **Settings are minimal.** Five sections max: hotkey, model, cleanup backend, audio input device, about.
5. **Errors are actionable.** Never "Something went wrong." Always: what failed, why, what the user can do.
6. **No notifications for normal operation.** Notifications only for errors and permission requests.

---

## 6. Anti-Patterns Explicitly Rejected

- **Dark patterns:** No "download" buttons that are actually ads. No fake countdown timers. No "X people downloaded this today" widgets.
- **Newsletter popups.** If we add a newsletter, it's a footer link, not a modal.
- **Cookie banner.** We don't set cookies. State this in the footer with a one-line "No cookies. No tracking."
- **Animated stat counters.** If we cite a number (e.g., "10MB install size"), it's a static number, not a count-up animation.
- **Stock photos of people wearing headsets.** Never.
