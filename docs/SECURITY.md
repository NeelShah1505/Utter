# SECURITY — Threat Model & Privacy Guarantees

> This document is the authoritative privacy/security spec.
> If the code ever diverges from this document, the code is wrong — file an issue.

---

## 1. Privacy Guarantees (User-Facing Promise)

These are commitments we make to users. Violating any of them is a critical bug.

| Guarantee                                                              | Status |
|------------------------------------------------------------------------|--------|
| Audio never leaves the device for core dictation.                      | Enforced |
| Transcripts never leave the device unless cleanup backend is configured.| Enforced |
| No account required.                                                   | Enforced |
| No telemetry, no analytics, no crash reporting in the desktop app.     | Enforced |
| No third-party SDKs in the desktop app.                                | Enforced |
| API keys (if user provides for cleanup) stored in OS keychain only.    | Enforced |
| No microphone access until user explicitly starts dictation.           | Enforced |
| Site (marketing page) sets no cookies and runs no third-party scripts. | Enforced |

---

## 2. Threat Model

### 2.1 Assets (what we protect)
- **A1 — User audio.** The microphone input during dictation.
- **A2 — User transcripts.** The text transcribed from audio.
- **A3 — User API keys.** Credentials the user optionally provides for cleanup backends.
- **A4 — User system integrity.** No privilege escalation, no arbitrary code execution via the app.

### 2.2 Adversaries
- **T1 — Network attacker.** Observes/intercepts traffic. Mitigation: zero network calls in core dictation; HTTPS-only for any optional cleanup call.
- **T2 — Malicious local process.** Reads user files, attempts to read app memory. Mitigation: API keys in keychain (not files); transcripts not persisted to disk by default (user can opt in).
- **T3 — Supply chain attacker.** Pushes malicious dependency update. Mitigation: `cargo audit`, `npm audit`, pinned versions, SBOM generation in CI (Phase 5).
- **T4 — Malicious cleanup backend.** A user-configured remote Ollama or OpenAI-compatible endpoint that logs transcripts. Mitigation: clear UI warnings before enabling; user explicitly accepts risk.
- **T5 — Phishing clone.** Someone ships a fake "Utter" installer with malware. Mitigation: code signing (Phase 5); publish SHA-256 checksums alongside every release; official download is always the GitHub Release.

### 2.3 Out of Scope
- We are not defending against a fully compromised OS (root/admin). If the OS is owned, the app's process memory is readable regardless.
- We are not defending against the user intentionally configuring a malicious cleanup endpoint — we warn, we don't prevent.

---

## 3. OS Permissions Requested

| OS       | Permission              | Why needed                                                                                  | Requested when                                            |
|----------|-------------------------|---------------------------------------------------------------------------------------------|-----------------------------------------------------------|
| macOS    | Microphone              | Capture audio for ASR                                                                       | First time user starts dictation                          |
| macOS    | Accessibility           | Insert transcribed text into the focused app via synthetic input events                     | First time user starts dictation                          |
| macOS    | Input Monitoring        | Detect global hotkey (Cmd+Shift+D) without conflicting with app shortcuts                    | First time user starts dictation                          |
| Windows  | Microphone              | Capture audio for ASR                                                                       | First time user starts dictation (OS prompt)              |
| Windows  | (No accessibility perm) | Windows doesn't gate synthetic input the same way; we use `SendInput` which requires no perm | N/A                                                       |
| Windows  | (No input monitoring)   | Global hotkey via `RegisterHotKey` requires no special permission                            | N/A                                                       |

**Permissions we do NOT request:**
- Location
- Contacts
- Camera
- Full disk access
- Notifications (unless user enables error notifications — opt-in)

---

## 4. Data Flow

### 4.1 Core dictation (no cleanup)
```
Microphone → App process (RAM) → ASR engine (RAM) → Text → Synthetic input → Focused app
```
- Audio is processed in RAM and discarded immediately after transcription.
- Transcripts are NOT written to disk by default.
- Network: zero calls.

### 4.2 With cleanup backend enabled (opt-in)
```
Microphone → App process → ASR engine → Raw transcript
                                          ↓
                                          (only if cleanup ON)
                                          ↓
                               ┌───────────────────────────┐
                               │ User-configured endpoint  │
                               │ (local Ollama / remote    │
                               │  Ollama / OpenAI-compat)  │
                               └───────────────────────────┘
                                          ↓
                                   Cleaned transcript
                                          ↓
                                   Synthetic input
```
- The raw transcript is sent ONLY to the user-configured endpoint, over HTTPS (or localhost for local Ollama).
- The endpoint URL and any API key are stored in OS keychain.
- The user sees a one-time warning: "Enabling cleanup sends your transcripts to <endpoint>. Are you sure?"

### 4.3 Site (marketing page)
```
Visitor browser → GitHub Pages (static HTML/CSS/JS)
              → GitHub Releases API (client-side fetch, no server)
```
- No cookies set.
- No third-party scripts (no analytics, no fonts CDN, no CDN at all if avoidable).
- The visitor's User-Agent string is parsed client-side ONLY to pick the right download link. It is not sent anywhere.

---

## 5. Secret / Credential Storage

| Secret type               | macOS storage             | Windows storage                 | Plaintext anywhere? |
|---------------------------|---------------------------|---------------------------------|---------------------|
| Cleanup API key           | Keychain (`security` API) | Credential Manager (Win32 Cred) | **Never**           |
| Remote Ollama URL+token   | Keychain                  | Credential Manager              | **Never**           |
| Local Ollama URL          | Keychain (or defaults if no auth) | Credential Manager       | **Never** (defaults OK if `http://localhost:11434`) |
| App signing identities    | Keychain (build-time only) | Certificate store (build-time) | **Never in repo**   |
| GitHub release token      | GitHub Actions secret     | GitHub Actions secret           | **Never in repo**   |

**Forbidden storage locations (lint for these in CI):**
- `localStorage` / `sessionStorage` / `IndexedDB` in the webview
- Plaintext JSON / YAML / TOML in user config dir
- Environment variables (except for build-time CI secrets)
- Source code (obviously)
- Logs (any logging that could capture a key must redact)

---

## 6. Telemetry & Analytics Policy

### Desktop app
- **Telemetry:** None. No usage events, no crash reports, no "anonymous" stats.
- **Update checks:** None in v1. The app does not phone home. User checks GitHub for updates.
- **Error logging:** Errors are written to a local log file (user's choice to share). Default location: `~/Library/Logs/Utter/` on macOS, `%APPDATA%\Utter\logs\` on Windows. Log files never contain API keys (redacted) or audio (never logged).

### Marketing site
- **Analytics:** None by default. If we add analytics later, it MUST be:
  - Self-hosted (e.g., Plausible CE on our own infra, or a GitHub-hosted static counter), OR
  - A privacy-respecting hosted service (Plausible Cloud, Fathom) with a public `/privacy` page disclosing it.
  - Cookieless.
  - Disclosed in the site footer and in this document.
- **Current state:** No analytics. This section will be updated if that changes.

---

## 7. Dependency Policy

- **License audit:** `cargo license` and `npm license-checker` (or equivalent) run in CI. Build fails if any non-MIT/Apache-2.0/BSD/ISC dependency is introduced without explicit review.
- **Vulnerability audit:** `cargo audit` and `npm audit --omit=dev` run in CI. Build fails on high/critical advisories.
- **Pinning:** All dependencies pinned to exact versions in lockfiles. Lockfiles committed.
- **Review threshold:** Any new dependency with >10MB installed size or >500KB minified+gzipped requires maintainer review (documented in the PR).
- **No `postinstall` scripts:** Dependencies that require `postinstall` to run arbitrary code are rejected unless audited line-by-line.

---

## 8. Build & Release Security

- **Releases are reproducible:** Build commands documented in `ARCHITECTURE.md §CI/CD`. SHA-256 of each asset published alongside the release.
- **Signing (Phase 5):**
  - macOS: notarized with Apple Developer ID. Stapled ticket.
  - Windows: signed with EV cert if budget allows (Q-003); otherwise OV cert + documented SmartScreen warning for first-time users.
- **Release artifacts are immutable:** Once a GitHub Release is marked `latest`, its assets are not replaced. New fixes = new version.

---

## 9. Vulnerability Disclosure

- **Report address:** TBD — create a `security@` alias or use GitHub's private vulnerability reporting.
- **Response SLA:** Acknowledge within 72 hours. Fix or mitigation within 30 days for high-severity, 90 days for medium.
- **Credit:** Public credit in release notes unless reporter requests anonymity.

---

## 10. Known Security Considerations (Not Yet Implemented)

These are tracked here so they're not forgotten. They are Phase 5 hardening items:

- [ ] SBOM (CycloneDX) generated and attached to each release.
- [ ] Reproducible build verification script.
- [ ] Sandbox the ASR engine process (separate process, minimal permissions) — defense in depth against a malicious model file.
- [ ] Verify model file signatures (Parakeet TDT, Whisper GGML) against published hashes at load time.
- [ ] Rate-limit cleanup API calls to prevent accidental cost blowout on metered API keys.
