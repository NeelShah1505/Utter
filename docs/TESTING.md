# TESTING — Test Plan & QA Checklist

> Living document. Update when test cases are added or budgets change.
> All scores/results below must be re-verified per release.

---

## 1. Test Strategy

| Layer        | What                                                | Tools                         | Where                |
|--------------|-----------------------------------------------------|-------------------------------|----------------------|
| Unit         | Pure functions (UA detection, asset URL building)  | Node's built-in test runner   | `site/assets/*.test.js` |
| Integration  | Releases API fetch with mock JSON                   | Custom mock + fetch           | `site/assets/*.test.js` |
| E2E (site)   | Page renders, download links resolve, JS-disabled   | Manual + Lighthouse + Playwright (Phase 1+) | Local + CI |
| Lint         | HTML valid, CSS valid, JS lint-clean                | `html-validate`, `stylelint`, `eslint` | CI                   |
| A11y         | Keyboard nav, contrast, screen reader               | axe-core (Playwright)         | CI                   |
| Perf         | Lighthouse                                          | Lighthouse CLI                | CI on PRs to `site/` |
| Desktop (Phase 2+) | Engine unit + integration tests               | `cargo test`                  | CI                   |

---

## 2. UA Detection Matrix

Each row is a real User-Agent string. The detection function must return the expected asset.

| #  | UA string (abbreviated)                                                  | Expected platform       | Expected asset                |
|----|--------------------------------------------------------------------------|-------------------------|-------------------------------|
| 1  | `Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/... Safari/...` | macOS Intel | `app-macos-x64.dmg` |
| 2  | `Mozilla/5.0 (Macintosh; Intel Mac OS X 13_5) Chrome/120.0.0.0 Safari/537.36` + UA-CH `architecture: "arm", bitness: "64"` | macOS Apple Silicon | `app-macos-arm64.dmg` |
| 3  | `Mozilla/5.0 (Macintosh; Intel Mac OS X 14_0) Chrome/120.0.0.0 Safari/537.36` (no UA-CH — use WebGL fallback) | macOS Apple Silicon (via WebGL "Apple M" or "Apple GPU") | `app-macos-arm64.dmg` |
| 4  | `Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0.0.0 Safari/537.36` | Windows x64 | `app-windows-x64.msi` |
| 5  | `Mozilla/5.0 (Windows NT 10.0; Win64; x64) Gecko/... Firefox/120.0` | Windows x64 | `app-windows-x64.msi` |
| 6  | `Mozilla/5.0 (Windows NT 10.0; ARM; ...) Chrome/120.0.0.0` + UA-CH `architecture: "arm"` | Windows ARM64 | `app-windows-arm64.msi` |
| 7  | `Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X)` | iOS (not supported) | Show all options, highlight none |
| 8  | `Mozilla/5.0 (Linux; Android 13) Chrome/120` | Android (not supported) | Show all options, highlight none |
| 9  | `Mozilla/5.0 (X11; Linux x86_64) Gecko/... Firefox/120` | Linux (not supported) | Show all options, highlight none |
| 10 | `curl/8.0` (no browser, no platform) | Unknown | Show all options, highlight none |
| 11 | Empty UA string | Unknown | Show all options, highlight none |
| 12 | `Mozilla/5.0 (Windows NT 10.0; Win64; x64) Edge/120` + UA-CH `architecture: "x86", bitness: "64"` | Windows x64 | `app-windows-x64.msi` |

**Edge cases to handle gracefully (no thrown exceptions):**
- UA string shorter than 50 chars
- UA contains "Mac" but no version
- UA-CH platform differs from UA string (trust UA-CH if present — it's more reliable)
- WebGL unavailable (headless test env) — fall back to x64

---

## 3. Download Link Verification

For each of the four canonical asset filenames, verify:
1. The site builds the correct URL: `https://github.com/NeelShah1505/Utter/releases/download/latest/<filename>`
2. If a release exists, the URL returns HTTP 200.
3. If no release exists yet (current state), the URL returns 404 — and the site's fallback (link to `/releases/latest` page) is shown.
4. The "Other platforms" disclosure contains all four filenames as visible text.

**Test method:** `site/assets/verify-links.test.js` — runs in CI, hits GitHub for the latest release, asserts every expected asset name is present in the response (or, if no release, asserts the fallback UI renders).

---

## 4. Lighthouse Targets

| Metric                  | Target  | Current (this session) | Notes                                  |
|-------------------------|---------|------------------------|----------------------------------------|
| Performance             | ≥ 95    | TBD — run after build  |                                        |
| Accessibility           | ≥ 95    | TBD                    |                                        |
| Best Practices          | ≥ 95    | TBD                    |                                        |
| SEO                     | ≥ 95    | TBD                    |                                        |
| LCP                     | < 1.5s  | TBD                    |                                        |
| CLS                     | < 0.1   | TBD                    |                                        |
| TBT                     | < 200ms | TBD                    |                                        |

**How to run:**
```bash
python3 -m http.server 8000 --directory site
npx lighthouse http://localhost:8000 --output=html --output-path=./lighthouse-report.html --preset=desktop
```

**Record actual numbers in this table after first run.** If a target is missed, document why and either fix or justify.

---

## 5. JavaScript-Disabled Behavior

**Test:** Disable JS in browser (or use `curl` to fetch `index.html`), verify:
- [x] Hero headline is visible
- [x] Subhead is visible
- [x] All four download links are present (in a `<noscript>` block) with their canonical filenames
- [x] GitHub repo link is visible and clickable
- [x] No "Download" button is shown that does nothing when clicked (no broken affordances)
- [x] Features / How it works / System requirements sections are visible (CSS-only, no JS-dependent content)

---

## 6. Accessibility Checklist

- [ ] All interactive elements reachable by keyboard (Tab order sensible)
- [ ] Download button has visible focus ring
- [ ] "Other platforms" disclosure is operable with Enter/Space
- [ ] Color contrast meets WCAG AA (verified via axe-core)
- [ ] Page has a single `<h1>`; section headings use `<h2>`/`<h3>` correctly
- [ ] All images (if any) have alt text; decorative SVGs have `aria-hidden="true"`
- [ ] `prefers-reduced-motion` respected (we have no motion, but verify none is introduced)
- [ ] `prefers-color-scheme: dark` supported (site has dark variant)
- [ ] Page title is descriptive: "Utter — Type with your voice. Offline."
- [ ] Meta description present and accurate

---

## 7. CI YAML Validation

**Test:** Run `yamllint` on `.github/workflows/*.yml` and `actionlint` to verify:
- [ ] `build-release.yml` is syntactically valid
- [ ] Job names match: `build-macos-arm64`, `build-macos-x64`, `build-windows-x64`, `build-windows-arm64`, `release`
- [ ] Each build job uploads an artifact named exactly `app-macos-arm64.dmg` / etc.
- [ ] `release` job depends on all four build jobs
- [ ] `deploy-site.yml` triggers on push to `main` when `site/**` changes

---

## 8. Cross-Browser Compatibility

| Browser              | Tested? | Notes                                            |
|----------------------|---------|--------------------------------------------------|
| Chrome 120+          | TBD     | Primary test target                              |
| Firefox 120+         | TBD     | No `navigator.userAgentData` — exercises fallback |
| Safari 17+           | TBD     | macOS default                                    |
| Edge 120+            | TBD     | Chromium-based                                   |
| Safari iOS 17        | TBD     | Should show "not supported" gracefully           |
| Chrome Android       | TBD     | Same as iOS                                      |

---

## 9. Test Commands (one-liners for CI)

```bash
# Site tests (no framework — Node's built-in runner)
node --test site/assets/detect.test.js
node --test site/assets/releases.test.js

# Lint
npx html-validate site/index.html
npx stylelint "site/assets/*.css"
npx eslint site/assets/

# Lighthouse
python3 -m http.server 8000 --directory site &
npx lighthouse http://localhost:8000 --preset=desktop --output=json --output-path=lh.json

# Workflow lint
npx actionlint .github/workflows/*.yml

# Link check (after release exists)
node site/assets/verify-links.test.js
```

---

## 10. Known Test Gaps (to fill in later phases)

- [ ] Desktop app unit tests (none until Phase 2 — engine trait doesn't exist yet)
- [ ] ASR accuracy tests (need a labeled audio dataset — deferred to Phase 2)
- [ ] Cross-OS hotkey tests (manual only until Phase 1+)
- [ ] Real-device microphone capture tests (manual only)
- [ ] Notarization / signing verification (Phase 5)

---

## 11. Release Readiness Checklist (use before tagging a release)

- [ ] All tests green in CI
- [ ] Lighthouse scores meet targets (§4)
- [ ] No `TODO:` comments in shipped code paths (allowed in docs)
- [ ] CHANGELOG.md updated
- [ ] `docs/CONTEXT.md` session entry appended
- [ ] SECURITY.md still accurate vs. actual code behavior
- [ ] Four assets produced, SHA-256 computed
- [ ] Release notes mention any breaking changes
- [ ] Site's GitHub Releases API fetch returns the new release as `latest`
