# MEMORY — Hard Invariants & Anti-Hallucination Reference

> **This file is the agent's long-term memory.**
> **Read at the start of EVERY session, before doing anything else.**
> **Update whenever a hard fact changes. Do not delete prior entries.**

This file exists to prevent the most common failure modes:
- Hallucinating filenames, API shapes, or feature decisions
- Forgetting what's done vs. planned
- Re-deciding already-decided things
- Drifting from the product principles

---

## 0. How To Use This File

1. **Start of session:** Read sections 1, 2, 3, 5 in full.
2. **Before writing any code:** Check section 2 (Canonical Strings) — does your code match?
3. **Before answering a product question:** Check section 3 (Decisions Already Made) — has this been decided?
4. **Before claiming "done":** Check section 6 (State Tracker) — is it actually in scope this phase?
5. **End of session:** Update section 6 (State Tracker) and append to CONTEXT.md.

---

## 1. Hard Invariants (NEVER violate)

These are non-negotiable. Violating any of them is a critical bug.

1. **Core dictation works with zero network calls, zero accounts, zero API keys.** If your code makes a network request during core dictation, it is wrong.
2. **No telemetry in the desktop app.** No analytics SDK. No crash reporter. No "anonymous" stats.
3. **Marketing site sets no cookies and runs no third-party scripts.** No Google Analytics, no fonts CDN, no CDN JS.
4. **Exact asset filenames must match section 2 below.** Site JS, CI workflow, and docs must all agree byte-for-byte.
5. **API keys stored in OS keychain only.** Never `localStorage`. Never a JSON file. Never env vars (except CI build secrets).
6. **MIT license only.** No GPL/AGPL transitive deps. Run `cargo license` / `npm ls --license` before releases.
7. **No fake content.** No invented testimonials, no fake user counts, no screenshots of unbuilt features. Use visible `TODO:` comments instead.
8. **Every error path handles failure explicitly.** No empty `catch {}`. No `Result::Err(_)` that's silently dropped. No `Promise.catch(() => {})`.
9. **No placeholder logic shipped as "done."** If a function isn't implemented, it must `panic!("not implemented: <reason>")` (Rust) or `throw new Error("not implemented: <reason>")` (JS), AND have a `TODO:` comment AND be tracked in CONTEXT.md.
10. **Phase scope is strict.** Do not start Phase 1+ work in a Phase 0 session. If asked, confirm scope with the human first.

---

## 2. Canonical Strings (must match exactly)

### 2.1 Build target filenames

```
app-macos-arm64.dmg       # macOS Apple Silicon
app-macos-x64.dmg         # macOS Intel
app-windows-x64.msi       # Windows 10/11 64-bit
app-windows-arm64.msi     # Windows on ARM (experimental)
```

**Why exact:** The site fetches release assets by name. The CI uploads by name. If they disagree, downloads 404.

### 2.2 GitHub repo placeholder

```
REPO = 'NeelShah1505/Utter'     # TO BE REPLACED — see Open Question Q-001 in CONTEXT.md
```

Until resolved, all references in code use `NeelShah1505/Utter` literally. CI workflows use `${{ github.repository }}` so they auto-resolve.

### 2.3 License string

```
MIT
```

Copyright line in LICENSE file: `Copyright (c) <year> <maintainer-name>` — name TBD (Q-001).

### 2.4 Working product name

```
Utter
```

This is a placeholder. Final name TBD before 1.0 (Decision D-002). Do not register domains, social handles, or package names with this name yet.

### 2.5 OS keychain identifiers

| OS       | Service / target                  |
|----------|-----------------------------------|
| macOS    | `com.utter.app`                 |
| Windows  | `Utter::Cleanup`                |

If you change these, existing users lose their saved keys. Don't change them post-release.

### 2.6 Config file paths

```
macOS:   ~/Library/Application Support/Utter/config.toml
Windows: %APPDATA%\Utter\config.toml
```

### 2.7 Default hotkey

```
macOS:   Cmd+Shift+D
Windows: Ctrl+Shift+D
```

User-configurable. These are defaults only.

### 2.8 Default cleanup endpoint (local Ollama)

```
http://localhost:11434
```

No auth expected. User can change in settings.

---

## 3. Decisions Already Made (do not re-decide)

These are settled. If you want to revisit one, raise it explicitly with the human — don't silently override.

| Topic                          | Decision                                 | Reference        |
|--------------------------------|------------------------------------------|------------------|
| Desktop shell                  | Tauri (Rust + native webview)            | DESIGN.md §2.1, D-005 |
| ASR on Apple Silicon           | FluidAudio + Parakeet TDT v3 (CoreML/ANE) | DESIGN.md §2.2, D-006 |
| ASR on Intel Mac + Windows     | whisper.cpp (CPU; CUDA on Windows/NVIDIA) | DESIGN.md §2.3, D-007 |
| Cleanup layer                  | OFF by default; pluggable (none/local Ollama/remote Ollama/OpenAI-compat) | DESIGN.md §2.4, D-008 |
| Site framework                 | Vanilla HTML/CSS/JS, no build step       | DESIGN.md §3.1, D-001 |
| Site deploy                    | GitHub Pages                              | DESIGN.md §3.2, D-003 |
| License                        | MIT                                       | Root LICENSE     |
| Telemetry (desktop)            | None                                      | SECURITY.md §6   |
| Analytics (site)               | None (would be self-hosted + disclosed if added) | SECURITY.md §6 |
| Repo org/name                  | TBD — see Q-001                           | CONTEXT.md       |

---

## 4. Phase Scope (DO NOT EXPAND)

### Phase 0 (current) — ONLY this is in scope
1. Repo scaffold (directories + README + LICENSE + empty `apps/desktop/`)
2. Marketing site (`site/index.html` + assets)
3. CI workflow stubs (`.github/workflows/build-release.yml` + `deploy-site.yml`)
4. All `docs/*.md` populated (this file included)
5. UA detection implemented + tested against the matrix in TESTING.md §2
6. GitHub Releases API integration (live fetch, graceful fallback)
7. Lighthouse run, scores recorded in TESTING.md §4
8. JS-disabled fallback verified

### NOT in Phase 0 (do not start)
- Tauri app code (Rust or webview UI)
- ASR engine code
- Cleanup layer code
- Hotkey implementation
- Mic capture
- Text insertion
- Settings UI
- Code signing / notarization
- Auto-updater

If asked to do any of the above, ask: "Phase 0 scope is X. Starting Y means expanding scope. Confirm?"

---

## 5. Common Hallucination Traps (specific to this project)

These are things the agent might be tempted to invent but must not:

### 5.1 Don't invent
- A GitHub org name. Use `NeelShah1505/Utter` literally.
- Release version numbers. Fetch from GitHub API at runtime.
- Asset file sizes. Fetch from GitHub API at runtime.
- Download counts. Fetch from GitHub API at runtime (or omit).
- User testimonials. None exist. Don't write any.
- Screenshots of the app. The app doesn't exist yet. Use illustrations or type, never fake screenshots.
- A "v1.0" or "v0.1" tag. We haven't released anything.
- Model file names or URLs. We haven't picked final models. Reference "Parakeet TDT v3" by name only; don't fabricate a HuggingFace URL.
- Tauri config values (`tauri.conf.json` schema fields). Don't write that file until Phase 1. If forced to scaffold, use Tauri's official generator output unchanged.
- Rust crate versions. Use `cargo add` and let the resolver pick, or check crates.io.

### 5.2 Don't conflate
- "FluidAudio" is a framework, not a model. Parakeet TDT v3 is the model. Don't write "FluidAudio model."
- "whisper.cpp" is the C++ implementation. "Whisper" is the model family. Don't write "Whisper.cpp model."
- "Ollama" is the server. The model is e.g. "llama3" or "qwen2.5". Don't write "Ollama model."
- "Apple Silicon" includes M1, M2, M3, M4. "Intel Mac" is explicitly not Apple Silicon. Don't write "Apple Silicon (including Intel Macs)."
- "OpenAI-compatible" means any endpoint speaking the OpenAI chat-completions API. It does not mean "uses OpenAI." Could be LM Studio, vLLM, local-Ollama with compat shim, etc.

### 5.3 Don't assume
- That `navigator.platform` is reliable. It's deprecated. Use `userAgentData` first.
- That WebGL renderer string reliably identifies Apple Silicon. It's a heuristic. Always offer "Other platforms" link.
- That GitHub Releases API will succeed. Handle 403 (rate limit), 404 (no releases), network failure.
- That the visitor has JS enabled. `<noscript>` fallback is mandatory.
- That dark mode is requested. Support it via `prefers-color-scheme` but don't assume.
- That the user wants analytics. They don't. Don't add any.

### 5.4 Don't over-engineer
- The site is one page. Don't add a router. Don't add a build step. Don't add TypeScript unless complexity demands it (it doesn't yet).
- The site has 4 download links. Don't add an "auto-updater web component." Don't add a "release notes RSS feed." Don't add a blog.
- The UA detection is ~50 lines of JS. Don't write a "platform detection library."
- If you find yourself adding a dependency, stop and ask. The budget is <8KB gzipped total JS.

---

## 6. State Tracker (update every session)

| Item                                              | Status         | Last updated     |
|---------------------------------------------------|----------------|------------------|
| Repo scaffold                                     | ✅ done        | Session 1        |
| `docs/*.md` populated                             | ✅ done        | Session 1        |
| `MEMORY.md` (this file)                           | ✅ done        | Session 1        |
| `site/index.html`                                 | ✅ done        | Session 1        |
| `site/assets/detect.js`                           | ✅ done        | Session 1        |
| `site/assets/releases.js`                         | ✅ done        | Session 1        |
| `site/assets/ui.js`                               | ✅ done        | Session 1        |
| `site/assets/styles.css`                          | ✅ done        | Session 1        |
| `site/assets/favicon.svg`                         | ✅ done        | Session 1        |
| `site/robots.txt`                                 | ✅ done        | Session 1        |
| UA detection tests (matrix in TESTING.md §2)      | ✅ done        | Session 1        |
| `build-release.yml` stub                          | ✅ done        | Session 1        |
| `deploy-site.yml`                                 | ✅ done        | Session 1        |
| Lighthouse run + scores recorded                  | ⏳ not started | —                |
| JS-disabled fallback verified                     | ✅ done (in HTML) | Session 1     |
| `NeelShah1505/Utter` placeholder resolved (Q-001)       | ❌ blocked on human | —            |
| Tauri app scaffold (`apps/desktop/`)              | ⏳ Phase 1     | —                |
| ASR engine code                                   | ⏳ Phase 2/3   | —                |
| Cleanup layer code                                | ⏳ Phase 4     | —                |
| Code signing / notarization                       | ⏳ Phase 5     | —                |

**Legend:** ✅ done · ⏳ pending · 🚧 in progress · ❌ blocked

---

## 7. Agent Self-Check (run before claiming any task is "done")

- [ ] Does the code match the canonical strings in §2?
- [ ] Does the code violate any invariant in §1?
- [ ] Are there empty `catch`/`Err(_)` swallowers? (Should be none.)
- [ ] Are there `TODO:` comments where logic is unimplemented? (Required if so.)
- [ ] Is the task in Phase 0 scope per §4? If not, did I confirm scope with the human?
- [ ] Did I hallucinate anything from §5? (Re-read §5 before answering.)
- [ ] Did I update §6 (State Tracker) for what I just did?
- [ ] Did I append to CONTEXT.md session log?

If any box is unchecked, the task is not done.

---

## 8. Questions To Ask The Human (instead of guessing)

- "What is the final GitHub org/repo name?" (Q-001)
- "What is the final product name?" (Q-002)
- "Is X in scope for this session, or a later phase?"
- "Should I assume the user has Ollama installed, or do we need to bundle a runtime?"
- "Is dark mode a hard requirement for the site, or nice-to-have?"
- "Are there specific browsers we must support beyond Chrome/Firefox/Safari/Edge latest?"

**Default behavior when unsure:** ask. Do not guess. A 30-second clarification is cheaper than a wrong implementation.
