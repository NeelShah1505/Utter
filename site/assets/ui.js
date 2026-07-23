/**
 * ui.js — Wires detect.js + releases.js to the DOM.
 * Populates the download button and "Other platforms" list.
 * Handles all failure modes (network down, no release, rate-limited).
 * INVARIANT: filenames must match MEMORY.md §2.1 exactly.
 */

import { detectPlatform, ASSETS } from './detect.js';
import { fetchLatestRelease, getAssetUrl, RELEASES_PAGE_URL } from './releases.js';

/** Human-readable labels for each canonical asset filename. */
const ASSET_LABELS = Object.freeze({
  [ASSETS.MACOS_ARM64]:   'macOS (Apple Silicon)',
  [ASSETS.MACOS_X64]:     'macOS (Intel)',
  [ASSETS.WINDOWS_X64]:   'Windows x64',
  [ASSETS.WINDOWS_ARM64]: 'Windows ARM64 (experimental)',
});

/** All four build targets in display order. */
const ALL_ASSETS = [
  ASSETS.MACOS_ARM64,
  ASSETS.MACOS_X64,
  ASSETS.WINDOWS_X64,
  ASSETS.WINDOWS_ARM64,
];

/** Format bytes as human-readable string. Returns null if unknown. */
function formatSize(bytes) {
  if (!bytes || bytes <= 0) return null;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

/** Update the primary download button for the detected platform. */
function setPrimaryButton(btn, filename, url, size, version) {
  const label = ASSET_LABELS[filename] || filename;
  btn.href = url;
  btn.setAttribute('download', filename);
  btn.textContent = `Download for ${label}`;

  // Show version and size as accessible subtitle if available
  const meta = document.getElementById('download-meta');
  if (meta) {
    const parts = [];
    if (version) parts.push(version);
    if (size)    parts.push(size);
    meta.textContent = parts.length ? parts.join(' · ') : '';
    meta.hidden = parts.length === 0;
  }

  btn.removeAttribute('aria-disabled');
  btn.classList.add('is-ready');
}

/** Set primary button to fallback state — points to GitHub releases page. */
function setPrimaryButtonFallback(btn) {
  btn.href = RELEASES_PAGE_URL;
  btn.textContent = 'Download from GitHub';
  btn.removeAttribute('download');
  btn.removeAttribute('aria-disabled');
  btn.classList.add('is-ready');

  const meta = document.getElementById('download-meta');
  if (meta) {
    meta.textContent = 'Choose your platform on GitHub';
    meta.hidden = false;
  }
}

/** Populate the "Other platforms" list with all four download links. */
function populateOtherPlatformsList(list, release, primaryAsset) {
  list.innerHTML = '';

  for (const filename of ALL_ASSETS) {
    const li = document.createElement('li');
    const a  = document.createElement('a');

    const url  = release ? getAssetUrl(release, filename) : null;
    const size = release ? formatSize((release.assets.find(a => a.name === filename) || {}).size) : null;

    if (url) {
      a.href = url;
      a.setAttribute('download', filename);
    } else {
      // No release or asset not in release — link to releases page
      a.href = RELEASES_PAGE_URL;
    }

    const label = ASSET_LABELS[filename] || filename;
    a.textContent = label;

    if (size) {
      const sizeSpan = document.createElement('span');
      sizeSpan.className = 'asset-size';
      sizeSpan.textContent = ` (${size})`;
      sizeSpan.setAttribute('aria-label', `, ${size}`);
      a.appendChild(sizeSpan);
    }

    if (filename === primaryAsset) {
      li.setAttribute('aria-current', 'true');
      const badge = document.createElement('span');
      badge.className = 'badge-recommended';
      badge.textContent = 'Recommended for your system';
      badge.setAttribute('aria-label', '— recommended for your system');
      li.appendChild(a);
      li.appendChild(badge);
    } else {
      li.appendChild(a);
    }

    list.appendChild(li);
  }
}

/** Main entry point — runs detection and release fetching concurrently. */
async function init() {
  const btn  = /** @type {HTMLAnchorElement|null} */ (document.getElementById('download-btn'));
  const list = /** @type {HTMLElement|null} */ (document.getElementById('other-platforms-list'));

  if (!btn) return; // JS-disabled path: noscript block handles this, no btn in DOM

  // Disable the button until we have data (prevents click on an empty href)
  btn.setAttribute('aria-disabled', 'true');

  // Run detection and API fetch concurrently — they're independent.
  // We intentionally don't await them sequentially; instead we await both at once.
  const [detectedAsset, releaseResult] = await Promise.all([
    detectPlatform(),
    fetchLatestRelease().then(r => ({ ok: true, data: r }))
                        .catch(err => ({ ok: false, error: err })),
  ]);

  const release = releaseResult.ok ? releaseResult.data : null;

  // Determine download URL for the primary button
  if (detectedAsset) {
    const url = release ? getAssetUrl(release, detectedAsset) : null;
    if (url) {
      const size    = release ? formatSize((release.assets.find(a => a.name === detectedAsset) || {}).size) : null;
      const version = release ? release.version : '';
      setPrimaryButton(btn, detectedAsset, url, size, version);
    } else {
      // Detected platform but no matching asset in the release (or no release yet)
      setPrimaryButtonFallback(btn);
    }
  } else {
    // Inconclusive detection — show generic "Download from GitHub"
    setPrimaryButtonFallback(btn);
  }

  // Populate "Other platforms" regardless of detection outcome
  if (list) {
    populateOtherPlatformsList(list, release, detectedAsset);
  }

  // Show the "Other platforms" details element (hidden until JS populates it)
  const detailsEl = document.getElementById('other-platforms');
  if (detailsEl) detailsEl.hidden = false;
}

// Run after DOM is fully parsed. Use DOMContentLoaded to avoid waiting for images.
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', init);
} else {
  // DOMContentLoaded already fired (e.g., script is deferred)
  init();
}
