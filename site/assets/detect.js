/**
 * detect.js — OS and CPU architecture detection for the Dictate download page.
 *
 * Detection order (most → least reliable):
 *   1. navigator.userAgentData (Chromium 90+, HTTPS-only)
 *   2. navigator.userAgent string parsing
 *   3. WebGL renderer string (heuristic for Apple Silicon when UA-CH unavailable)
 *
 * Returns one of the four canonical asset filenames, or null when detection is
 * inconclusive (caller should show all options with none highlighted).
 *
 * WHY this order:
 *   userAgentData is the official successor to navigator.userAgent for platform
 *   detection; it's unambiguous and opt-in. We fall back to UA string parsing
 *   because Firefox and Safari don't expose userAgentData. The WebGL renderer
 *   heuristic is a last resort for distinguishing Apple Silicon from Intel Mac
 *   when no UA-CH hint is available — it is explicitly documented as heuristic.
 *
 * INVARIANT: canonical filenames must match MEMORY.md §2.1 exactly.
 */

/** @typedef {'app-macos-arm64.dmg'|'app-macos-x64.dmg'|'app-windows-x64.msi'|'app-windows-arm64.msi'|null} Asset */

const ASSETS = Object.freeze({
  MACOS_ARM64:   'app-macos-arm64.dmg',
  MACOS_X64:     'app-macos-x64.dmg',
  WINDOWS_X64:   'app-windows-x64.msi',
  WINDOWS_ARM64: 'app-windows-arm64.msi',
});

/**
 * Attempt detection via the User-Agent Client Hints API.
 * Only available in Chromium 90+ over HTTPS. Returns a Promise that resolves to
 * an Asset string, or null if detection is inconclusive or the API is absent.
 *
 * @returns {Promise<Asset>}
 */
async function detectViaUAClientHints() {
  const nav = /** @type {any} */ (navigator);
  if (!nav.userAgentData) return null;

  const platform = nav.userAgentData.platform || '';

  // We need high-entropy values to determine architecture.
  // getHighEntropyValues is async and may throw if the browser blocks it.
  let hints;
  try {
    hints = await nav.userAgentData.getHighEntropyValues(['architecture', 'bitness']);
  } catch (_err) {
    // Browser denied high-entropy values — fall through to UA string parsing.
    // We still know the platform from the low-entropy hint if it's set.
    if (/^macOS$/i.test(platform)) {
      // Can't determine arch without high-entropy values; signal inconclusive.
      return null;
    }
    if (/^Windows$/i.test(platform)) {
      // Without arch hint on Windows, default to x64 (vast majority of Windows).
      return ASSETS.WINDOWS_X64;
    }
    return null;
  }

  const arch   = (hints.architecture || '').toLowerCase();
  const plat   = (hints.platform    || platform).toLowerCase();

  if (plat === 'macos') {
    // 'arm' covers arm64. 'x86' covers x86_64.
    if (arch === 'arm')  return ASSETS.MACOS_ARM64;
    if (arch === 'x86')  return ASSETS.MACOS_X64;
    return null; // Unknown arch on macOS — inconclusive
  }

  if (plat === 'windows') {
    if (arch === 'arm')  return ASSETS.WINDOWS_ARM64;
    if (arch === 'x86')  return ASSETS.WINDOWS_X64;
    // Default Windows to x64 if arch unknown
    return ASSETS.WINDOWS_X64;
  }

  // Linux, Android, iOS, unknown — not supported for a primary download
  return null;
}

/**
 * Attempt detection via the WebGL renderer string.
 * Used only to distinguish Apple Silicon from Intel Mac when UA-CH is unavailable.
 * This is a heuristic — "Apple M" or "Apple GPU" in the renderer indicates ANE/M-series.
 *
 * WHY WebGL and not WASM SIMD or other signals: WebGL renderer is available in all
 * major browsers synchronously, doesn't require user permission, and is reasonably
 * reliable for identifying Apple GPUs without being a tracking vector (it's a
 * browser-level string, not unique to the user).
 *
 * @returns {'arm64'|'x64'|null} null if WebGL is unavailable (e.g., headless env)
 */
function detectMacArchViaWebGL() {
  try {
    const canvas = document.createElement('canvas');
    const gl = canvas.getContext('webgl') || canvas.getContext('experimental-webgl');
    if (!gl) return null;

    const glAny = /** @type {any} */ (gl);
    const ext = glAny.getExtension('WEBGL_debug_renderer_info');
    if (!ext) return null;

    const renderer = (gl.getParameter(ext.UNMASKED_RENDERER_WEBGL) || '').toString();
    // "Apple M1", "Apple M2", "Apple M3", "Apple M4", "Apple GPU" all indicate ANE hardware.
    if (/Apple\s+(M\d|GPU)/i.test(renderer)) return 'arm64';

    // Intel GPU strings: "Intel Iris", "Intel UHD", "ATI Radeon" etc.
    return 'x64';
  } catch (_err) {
    // WebGL unavailable or threw — return null (inconclusive)
    return null;
  }
}

/**
 * Attempt detection via navigator.userAgent string parsing.
 * This is the fallback for browsers that don't expose userAgentData (Firefox, Safari).
 *
 * WHY we still support UA string parsing:
 *   Firefox 120 and Safari 17 do not implement navigator.userAgentData. They represent
 *   a significant share of Mac users (Safari is the default macOS browser). We must
 *   handle them gracefully.
 *
 * @param {string} [uaOverride] - Override for testing; uses navigator.userAgent if omitted.
 * @returns {Promise<Asset>} Promise because arch detection for Mac may need WebGL (async-safe).
 */
async function detectViaUAString(uaOverride) {
  const ua = typeof uaOverride === 'string'
    ? uaOverride
    : (typeof navigator !== 'undefined' ? navigator.userAgent : '');

  // iOS must be checked before macOS: iOS UA strings contain "Mac OS X" too.
  if (/iPhone|iPad/i.test(ua)) return null;   // iOS — not supported
  if (/Android/i.test(ua))     return null;   // Android — not supported

  if (/Mac/i.test(ua)) {
    // Try WebGL heuristic to distinguish Apple Silicon from Intel.
    // In a test environment (Node, headless), detectMacArchViaWebGL returns null → x64.
    const arch = (typeof document !== 'undefined') ? detectMacArchViaWebGL() : null;
    return arch === 'arm64' ? ASSETS.MACOS_ARM64 : ASSETS.MACOS_X64;
  }

  if (/Windows/i.test(ua)) {
    // UA-CH wasn't available (we're in the UA-string fallback path). Look for ARM hints
    // in the UA string itself. This is rare — Surface users on Chrome have UA-CH, so
    // this path is mostly for Edge/Firefox on ARM Windows.
    if (/ARM/i.test(ua)) return ASSETS.WINDOWS_ARM64;
    return ASSETS.WINDOWS_X64;
  }

  // Linux, curl, unknown — not supported for a primary download
  return null;
}

/**
 * Detect the visitor's OS and CPU architecture, returning the canonical asset filename
 * to highlight as the primary download, or null if detection is inconclusive.
 *
 * Call order: userAgentData → UA string → (WebGL is called inside UA string path for Mac).
 *
 * This function never throws. All failures are caught and result in null (show all options).
 *
 * @param {{ uaOverride?: string, skipUACH?: boolean }} [opts] - Test hooks only.
 * @returns {Promise<Asset>}
 */
export async function detectPlatform(opts = {}) {
  try {
    if (!opts.skipUACH) {
      const uach = await detectViaUAClientHints();
      if (uach !== null) return uach;
    }

    // UA-CH not available, inconclusive, or access denied — fall back to UA string.
    return await detectViaUAString(opts.uaOverride);
  } catch (_err) {
    // Defensive catch: if anything unexpected throws, degrade gracefully.
    return null;
  }
}

// Export individual helpers for testing
export { ASSETS, detectViaUAString, detectMacArchViaWebGL };
