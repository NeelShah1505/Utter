/**
 * releases.js — GitHub Releases API fetcher for the Dictate download page.
 *
 * Fetches the latest release from GitHub, caches it in sessionStorage for 5 minutes
 * to be polite to the unauthenticated API (60 req/hr per IP), and falls back
 * gracefully if the API is unavailable or no release exists yet.
 *
 * WHY sessionStorage and not localStorage:
 *   - sessionStorage is cleared when the tab closes, so stale data doesn't persist.
 *   - We only need the cache within a single browsing session.
 *   - localStorage would require explicit expiry logic and could persist across sessions.
 *
 * WHY 5 minutes:
 *   - GitHub's unauthenticated rate limit is 60 req/hr per IP.
 *   - A marketing page with multiple visitors from the same office/VPN could hit that.
 *   - 5 minutes is conservative; new releases happen infrequently.
 *
 * INVARIANT: REPO must be set to 'NeelShah1505/Utter' until Q-001 is resolved (CONTEXT.md).
 *            Do not invent an org or repo name.
 */

// TODO: Replace 'NeelShah1505/Utter' with real value once Q-001 (CONTEXT.md) is resolved.
const REPO = 'NeelShah1505/Utter';
const API_URL = `https://api.github.com/repos/${REPO}/releases/latest`;
const CACHE_KEY = 'dictate_latest_release';
const CACHE_TTL_MS = 5 * 60 * 1000; // 5 minutes

/**
 * @typedef {Object} ReleaseAsset
 * @property {string} name            - Canonical filename (e.g. 'app-macos-arm64.dmg')
 * @property {string} url             - Direct download URL
 * @property {number} size            - Size in bytes
 * @property {number} downloadCount   - Number of downloads
 */

/**
 * @typedef {Object} ReleaseInfo
 * @property {string}         version       - Tag name (e.g. 'v1.0.0')
 * @property {string}         publishedAt   - ISO 8601 timestamp
 * @property {ReleaseAsset[]} assets        - List of release assets
 * @property {string}         releaseNotes  - Raw release body markdown
 */

/**
 * @typedef {Object} CacheEntry
 * @property {number}      ts   - Unix timestamp when cached (ms)
 * @property {ReleaseInfo} data - The cached release data
 */

/**
 * Read a cache entry from sessionStorage. Returns null if absent, expired, or unparseable.
 *
 * @returns {ReleaseInfo|null}
 */
function readCache() {
  try {
    const raw = sessionStorage.getItem(CACHE_KEY);
    if (!raw) return null;

    const entry = /** @type {CacheEntry} */ (JSON.parse(raw));
    if (!entry || typeof entry.ts !== 'number' || !entry.data) return null;

    const age = Date.now() - entry.ts;
    if (age > CACHE_TTL_MS) {
      // Stale — remove it so next read doesn't try to parse it again
      sessionStorage.removeItem(CACHE_KEY);
      return null;
    }

    return entry.data;
  } catch (_err) {
    // sessionStorage may throw in certain privacy modes — treat as cache miss
    return null;
  }
}

/**
 * Write a release to the sessionStorage cache.
 * Silently ignores errors (e.g. storage quota exceeded or privacy mode).
 *
 * @param {ReleaseInfo} data
 */
function writeCache(data) {
  try {
    const entry = /** @type {CacheEntry} */ ({ ts: Date.now(), data });
    sessionStorage.setItem(CACHE_KEY, JSON.stringify(entry));
  } catch (_err) {
    // Non-fatal — operate without caching
  }
}

/**
 * Parse the raw GitHub Releases API response into a typed ReleaseInfo object.
 * Defensive — every field access guards against null/undefined.
 *
 * @param {Record<string, any>} raw - Parsed JSON from GitHub API
 * @returns {ReleaseInfo}
 */
function parseRelease(raw) {
  const assets = Array.isArray(raw.assets)
    ? raw.assets.map(a => ({
        name:          String(a.name            || ''),
        url:           String(a.browser_download_url || ''),
        size:          Number(a.size            || 0),
        downloadCount: Number(a.download_count  || 0),
      }))
    : [];

  return {
    version:      String(raw.tag_name     || ''),
    publishedAt:  String(raw.published_at || ''),
    assets,
    releaseNotes: String(raw.body         || ''),
  };
}

/**
 * Fetch the latest release from the GitHub Releases API.
 *
 * Returns a ReleaseInfo on success, or throws a typed error on failure.
 * Callers MUST handle the thrown error and display the fallback UI.
 *
 * Error types thrown:
 *   - { code: 'RATE_LIMITED' }  — HTTP 403 (GitHub rate limit)
 *   - { code: 'NO_RELEASE' }    — HTTP 404 (no releases published yet)
 *   - { code: 'API_ERROR', status: number } — other HTTP error
 *   - { code: 'NETWORK_ERROR', cause: Error } — fetch threw (offline, DNS, etc.)
 *
 * WHY we throw typed errors instead of returning null:
 *   The caller needs to know WHY it failed to show an appropriate fallback message.
 *   A null return conflates all failure modes.
 *
 * @param {{ fetchOverride?: Function }} [opts] - Test hook to inject a mock fetch.
 * @returns {Promise<ReleaseInfo>}
 */
export async function fetchLatestRelease(opts = {}) {
  // Check cache first
  const cached = readCache();
  if (cached) return cached;

  const fetchFn = (opts.fetchOverride) || (typeof fetch !== 'undefined' ? fetch : null);
  if (!fetchFn) {
    throw { code: 'NETWORK_ERROR', cause: new Error('fetch not available') };
  }

  let response;
  try {
    response = await fetchFn(API_URL, {
      headers: { 'Accept': 'application/vnd.github+json' },
    });
  } catch (cause) {
    throw { code: 'NETWORK_ERROR', cause };
  }

  if (response.status === 403) throw { code: 'RATE_LIMITED' };
  if (response.status === 404) throw { code: 'NO_RELEASE' };
  if (!response.ok) throw { code: 'API_ERROR', status: response.status };

  let raw;
  try {
    raw = await response.json();
  } catch (cause) {
    throw { code: 'NETWORK_ERROR', cause };
  }

  const release = parseRelease(raw);
  writeCache(release);
  return release;
}

/**
 * Given a release and a canonical filename, find the matching asset's download URL.
 * Returns null if the asset isn't present in the release (e.g. build didn't run yet).
 *
 * @param {ReleaseInfo} release
 * @param {string} filename - Canonical asset filename (e.g. 'app-macos-arm64.dmg')
 * @returns {string|null}
 */
export function getAssetUrl(release, filename) {
  const asset = release.assets.find(a => a.name === filename);
  return asset ? asset.url : null;
}

/**
 * The GitHub releases page URL to fall back to when API fails or no assets exist.
 * Always available as a static link.
 *
 * @type {string}
 */
export const RELEASES_PAGE_URL = `https://github.com/${REPO}/releases/latest`;

export { REPO };
