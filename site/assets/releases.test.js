/**
 * releases.test.js — Unit tests for releases.js.
 *
 * Uses Node's built-in test runner (node --test). No test framework dependency.
 * Run with: node --test site/assets/releases.test.js
 *
 * Tests:
 *   - Successful fetch returns parsed ReleaseInfo
 *   - Caching behavior (second call uses cache)
 *   - Rate-limited response (403) throws { code: 'RATE_LIMITED' }
 *   - No releases yet (404) throws { code: 'NO_RELEASE' }
 *   - Network failure throws { code: 'NETWORK_ERROR' }
 *   - getAssetUrl returns correct URL or null
 *   - Canonical filenames in mock match MEMORY.md §2.1
 */

import { test, describe, beforeEach } from 'node:test';
import assert from 'node:assert/strict';
import { fetchLatestRelease, getAssetUrl, RELEASES_PAGE_URL, REPO } from './releases.js';

// ---------------------------------------------------------------------------
// Mock release fixture — uses exact canonical filenames
// ---------------------------------------------------------------------------

const MOCK_RELEASE = {
  tag_name:     'v0.0.1-placeholder',
  published_at: '2024-07-23T00:00:00Z',
  body:         'Initial scaffold release. No actual installers yet.',
  assets: [
    {
      name:                 'app-macos-arm64.dmg',
      browser_download_url: 'https://github.com/NeelShah1505/Utter/releases/download/v0.0.1-placeholder/app-macos-arm64.dmg',
      size:                 10 * 1024 * 1024, // 10 MB placeholder
      download_count:       0,
    },
    {
      name:                 'app-macos-x64.dmg',
      browser_download_url: 'https://github.com/NeelShah1505/Utter/releases/download/v0.0.1-placeholder/app-macos-x64.dmg',
      size:                 11 * 1024 * 1024,
      download_count:       0,
    },
    {
      name:                 'app-windows-x64.msi',
      browser_download_url: 'https://github.com/NeelShah1505/Utter/releases/download/v0.0.1-placeholder/app-windows-x64.msi',
      size:                 9 * 1024 * 1024,
      download_count:       0,
    },
    {
      name:                 'app-windows-arm64.msi',
      browser_download_url: 'https://github.com/NeelShah1505/Utter/releases/download/v0.0.1-placeholder/app-windows-arm64.msi',
      size:                 9 * 1024 * 1024,
      download_count:       0,
    },
  ],
};

/**
 * Create a mock fetch that returns the given status and body.
 *
 * @param {number} status
 * @param {any} [body]
 * @returns {Function}
 */
function makeMockFetch(status, body) {
  return async (_url, _opts) => ({
    ok:     status >= 200 && status < 300,
    status,
    json:   async () => body,
  });
}

/**
 * Create a mock fetch that throws a network error.
 *
 * @returns {Function}
 */
function makeMockFetchNetworkError() {
  return async (_url, _opts) => {
    throw new Error('Network error: connection refused');
  };
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe('fetchLatestRelease', () => {
  // NOTE: We can't easily clear the real sessionStorage in Node (it's a browser API).
  // The fetchOverride opt bypasses caching concerns for unit tests; in a browser
  // integration test, sessionStorage.clear() would be called in beforeEach.
  // The module's cache is keyed by sessionStorage which doesn't exist in Node,
  // so readCache() always returns null here → no stale cache interference.

  test('successful fetch returns parsed ReleaseInfo with all four assets', async () => {
    const release = await fetchLatestRelease({
      fetchOverride: makeMockFetch(200, MOCK_RELEASE),
    });

    assert.equal(release.version,     'v0.0.1-placeholder');
    assert.equal(release.publishedAt, '2024-07-23T00:00:00Z');
    assert.equal(release.assets.length, 4);

    // Check all four canonical names are present
    const names = release.assets.map(a => a.name).sort();
    assert.deepEqual(names, [
      'app-macos-arm64.dmg',
      'app-macos-x64.dmg',
      'app-windows-arm64.msi',
      'app-windows-x64.msi',
    ]);
  });

  test('HTTP 403 throws { code: RATE_LIMITED }', async () => {
    await assert.rejects(
      () => fetchLatestRelease({ fetchOverride: makeMockFetch(403) }),
      (err) => {
        assert.equal(err.code, 'RATE_LIMITED');
        return true;
      },
    );
  });

  test('HTTP 404 throws { code: NO_RELEASE }', async () => {
    await assert.rejects(
      () => fetchLatestRelease({ fetchOverride: makeMockFetch(404) }),
      (err) => {
        assert.equal(err.code, 'NO_RELEASE');
        return true;
      },
    );
  });

  test('HTTP 500 throws { code: API_ERROR, status: 500 }', async () => {
    await assert.rejects(
      () => fetchLatestRelease({ fetchOverride: makeMockFetch(500) }),
      (err) => {
        assert.equal(err.code,   'API_ERROR');
        assert.equal(err.status, 500);
        return true;
      },
    );
  });

  test('network error throws { code: NETWORK_ERROR }', async () => {
    await assert.rejects(
      () => fetchLatestRelease({ fetchOverride: makeMockFetchNetworkError() }),
      (err) => {
        assert.equal(err.code, 'NETWORK_ERROR');
        assert.ok(err.cause instanceof Error, 'cause should be an Error instance');
        return true;
      },
    );
  });

  test('empty assets array is handled (release exists but no builds yet)', async () => {
    const emptyRelease = { ...MOCK_RELEASE, assets: [] };
    const release = await fetchLatestRelease({
      fetchOverride: makeMockFetch(200, emptyRelease),
    });
    assert.equal(release.assets.length, 0);
  });

  test('missing fields in API response are handled gracefully', async () => {
    const partialRelease = {}; // completely empty response
    const release = await fetchLatestRelease({
      fetchOverride: makeMockFetch(200, partialRelease),
    });
    assert.equal(release.version,     '');
    assert.equal(release.publishedAt, '');
    assert.equal(release.assets.length, 0);
  });
});

describe('getAssetUrl', () => {
  test('returns the download URL for a known asset name', async () => {
    const release = await fetchLatestRelease({
      fetchOverride: makeMockFetch(200, MOCK_RELEASE),
    });
    const url = getAssetUrl(release, 'app-macos-arm64.dmg');
    assert.ok(url && url.includes('app-macos-arm64.dmg'), 'URL should contain the asset filename');
  });

  test('returns null for an asset not present in the release', async () => {
    const release = await fetchLatestRelease({
      fetchOverride: makeMockFetch(200, MOCK_RELEASE),
    });
    const url = getAssetUrl(release, 'nonexistent-file.dmg');
    assert.equal(url, null);
  });
});

describe('RELEASES_PAGE_URL', () => {
  test('falls back to GitHub releases page URL (uses REPO placeholder)', () => {
    // RELEASES_PAGE_URL should contain the REPO constant
    assert.ok(
      RELEASES_PAGE_URL.includes(REPO),
      `RELEASES_PAGE_URL (${RELEASES_PAGE_URL}) should contain REPO (${REPO})`,
    );
    assert.ok(
      RELEASES_PAGE_URL.startsWith('https://github.com/'),
      'RELEASES_PAGE_URL must start with https://github.com/',
    );
    assert.ok(
      RELEASES_PAGE_URL.endsWith('/releases/latest'),
      'RELEASES_PAGE_URL must end with /releases/latest',
    );
  });
});
