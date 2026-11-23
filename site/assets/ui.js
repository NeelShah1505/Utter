/**
 * ui.js — Download wiring + motion for the Utter one-pager.
 *
 * Honesty rule: only assets in SHIPPED are offered as links. Everything else is
 * listed as not built. We never render a download that cannot run.
 *
 * detect.js and releases.js are covered by the node test suite and are used
 * here unchanged.
 *
 * Every effect below is optional: it is skipped on touch input or when the user
 * asks for reduced motion, and nothing throws if an element is absent.
 */

import { detectPlatform, ASSETS } from './detect.js';
import { fetchLatestRelease, getAssetUrl, getDirectDownloadUrl, RELEASES_PAGE_URL } from './releases.js';

/* ───────────────────────── download ───────────────────────── */

const ASSET_LABELS = Object.freeze({
  [ASSETS.MACOS_ARM64]:   'macOS · Apple Silicon',
  [ASSETS.MACOS_X64]:     'macOS · Intel',
  [ASSETS.WINDOWS_X64]:   'Windows',
  [ASSETS.WINDOWS_ARM64]: 'Windows · ARM',
});

/** Installers that exist and actually run. */
const SHIPPED = new Set([ASSETS.MACOS_ARM64]);

/** Rows shown in the platform table, in display order. */
const PLATFORM_ROWS = [
  { asset: ASSETS.MACOS_ARM64, name: 'macOS · Apple Silicon' },
  { asset: ASSETS.MACOS_X64,   name: 'macOS · Intel' },
  { asset: ASSETS.WINDOWS_X64, name: 'Windows' },
  { asset: null,               name: 'Linux' },
];

function formatSize(bytes) {
  if (!bytes || bytes <= 0) return null;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function sizeOf(release, filename) {
  if (!release || !Array.isArray(release.assets)) return null;
  const found = release.assets.find(a => a.name === filename);
  return formatSize(found && found.size);
}

/** Point a button at a real installer. */
function setButton(btn, meta, filename, url, size, version) {
  if (!btn) return;
  btn.href = url;
  btn.setAttribute('download', filename);
  const label = btn.querySelector('.btn__label');
  const text = `Download for ${ASSET_LABELS[filename] || filename}`;
  if (label) label.textContent = text; else btn.textContent = text;
  btn.removeAttribute('aria-disabled');
  if (meta) {
    const parts = [];
    if (version) parts.push(version);
    if (size) parts.push(size);
    parts.push('macOS 13+');
    meta.textContent = parts.join(' · ');
  }
}

/**
 * Nothing shippable matches this visitor (Windows/Linux/Intel, or detection
 * failed). Keep the button useful by sending them to the release page, and say
 * plainly that only Apple Silicon is published.
 */
function setButtonFallback(btn, meta) {
  if (!btn) return;
  btn.href = getDirectDownloadUrl(ASSETS.MACOS_ARM64);
  btn.setAttribute('download', ASSETS.MACOS_ARM64);
  const label = btn.querySelector('.btn__label');
  const text = 'Download for macOS · Apple Silicon';
  if (label) label.textContent = text; else btn.textContent = text;
  btn.removeAttribute('aria-disabled');
  if (meta) meta.textContent = 'Apple Silicon only · macOS 13+';
}

function renderPlatforms(list, release, detected) {
  if (!list) return;
  list.innerHTML = '';

  for (const row of PLATFORM_ROWS) {
    const li = document.createElement('li');
    const name = document.createElement('span');
    name.className = 'plat__name';
    const state = document.createElement('span');
    state.className = 'plat__state';

    if (row.asset && SHIPPED.has(row.asset)) {
      const url = (release ? getAssetUrl(release, row.asset) : null) || getDirectDownloadUrl(row.asset);
      const a = document.createElement('a');
      a.href = url;
      a.setAttribute('download', row.asset);
      a.textContent = row.name;
      name.appendChild(a);

      const size = sizeOf(release, row.asset);
      if (size) {
        const s = document.createElement('span');
        s.className = 'asset-size';
        s.textContent = ` ${size}`;
        name.appendChild(s);
      }

      state.textContent = 'Available';
      state.classList.add('plat__state--yes');
      if (row.asset === detected) {
        li.setAttribute('aria-current', 'true');
        const badge = document.createElement('span');
        badge.className = 'badge-recommended';
        badge.textContent = 'your Mac';
        name.appendChild(badge);
      }
    } else {
      name.textContent = row.name;
      state.textContent = 'Not built yet';
    }

    li.appendChild(name);
    li.appendChild(state);
    list.appendChild(li);
  }
}

async function initDownload() {
  const buttons = [
    { btn: document.getElementById('download-btn'),   meta: document.getElementById('download-meta') },
    { btn: document.getElementById('download-btn-2'), meta: document.getElementById('download-meta-2') },
  ];
  const list = document.getElementById('other-platforms-list');

  const [detected, result] = await Promise.all([
    detectPlatform(),
    fetchLatestRelease().then(d => ({ ok: true, data: d })).catch(e => ({ ok: false, error: e })),
  ]);
  const release = result.ok ? result.data : null;

  const shippable = detected && SHIPPED.has(detected) ? detected : null;
  for (const { btn, meta } of buttons) {
    if (shippable) {
      const url = (release ? getAssetUrl(release, shippable) : null) || getDirectDownloadUrl(shippable);
      setButton(btn, meta, shippable, url, sizeOf(release, shippable), release ? release.version : '');
    } else {
      setButtonFallback(btn, meta);
    }
  }

  renderPlatforms(list, release, detected);

  // Expose the releases page for anyone who wants every artifact.
  const gk = document.querySelector('.get__gatekeeper');
  if (gk && !gk.dataset.linked) {
    gk.dataset.linked = '1';
    const a = document.createElement('a');
    a.href = RELEASES_PAGE_URL;
    a.rel = 'noopener';
    a.textContent = 'All releases';
    gk.appendChild(document.createTextNode(' · '));
    gk.appendChild(a);
  }
}

/* ───────────────────────── motion ───────────────────────── */

const reduced = window.matchMedia('(prefers-reduced-motion: reduce)');
const finePointer = window.matchMedia('(hover: hover) and (pointer: fine)');
const wantsMotion = () => !reduced.matches;
const clamp = (v, lo, hi) => Math.min(hi, Math.max(lo, v));

/** Sticky header gets a background once you leave the top. */
function initHeader() {
  const header = document.getElementById('site-header');
  if (!header) return;
  const onScroll = () => header.classList.toggle('is-stuck', window.scrollY > 12);
  onScroll();
  window.addEventListener('scroll', onScroll, { passive: true });
}

/** Fade/slide elements in the first time they appear. */
function initReveal() {
  const items = document.querySelectorAll('[data-reveal]');
  if (!items.length) return;
  if (!('IntersectionObserver' in window) || !wantsMotion()) {
    items.forEach(el => el.classList.add('is-in'));
    return;
  }
  const io = new IntersectionObserver((entries) => {
    entries.forEach((e, i) => {
      if (!e.isIntersecting) return;
      const el = e.target;
      setTimeout(() => el.classList.add('is-in'), i * 70);
      io.unobserve(el);
    });
  }, { rootMargin: '0px 0px -12% 0px', threshold: 0.15 });
  items.forEach(el => io.observe(el));
}

/** A soft light that trails the pointer. */
function initCursor() {
  const glow = document.getElementById('cursor-glow');
  if (!glow || !finePointer.matches || !wantsMotion()) return;
  let x = window.innerWidth / 2, y = window.innerHeight / 2;
  let cx = x, cy = y, raf = 0;

  const loop = () => {
    cx += (x - cx) * 0.12;
    cy += (y - cy) * 0.12;
    glow.style.transform = `translate3d(${cx}px, ${cy}px, 0)`;
    raf = requestAnimationFrame(loop);
  };
  window.addEventListener('pointermove', (e) => {
    x = e.clientX; y = e.clientY;
    if (!glow.classList.contains('is-on')) glow.classList.add('is-on');
    if (!raf) raf = requestAnimationFrame(loop);
  }, { passive: true });
  document.addEventListener('pointerleave', () => glow.classList.remove('is-on'));
}

/** Buttons lean slightly toward the pointer. */
function initMagnetic() {
  if (!finePointer.matches || !wantsMotion()) return;
  document.querySelectorAll('[data-magnetic]').forEach((el) => {
    el.addEventListener('pointermove', (e) => {
      const r = el.getBoundingClientRect();
      const dx = (e.clientX - (r.left + r.width / 2)) / r.width;
      const dy = (e.clientY - (r.top + r.height / 2)) / r.height;
      el.style.transform = `translate3d(${dx * 10}px, ${dy * 6}px, 0)`;
    });
    el.addEventListener('pointerleave', () => { el.style.transform = ''; });
  });
}

/** Cards tilt in 3D and carry a spotlight under the pointer. */
function initCardTilt() {
  const cards = document.querySelectorAll('[data-tilt]');
  if (!cards.length) return;
  cards.forEach((card) => {
    card.addEventListener('pointermove', (e) => {
      const r = card.getBoundingClientRect();
      const px = (e.clientX - r.left) / r.width;
      const py = (e.clientY - r.top) / r.height;
      // Spotlight follows even with reduced motion (it is a colour change).
      card.style.setProperty('--mx', `${px * 100}%`);
      card.style.setProperty('--my', `${py * 100}%`);
      if (!finePointer.matches || !wantsMotion()) return;
      const ry = (px - 0.5) * 9;
      const rx = (0.5 - py) * 9;
      card.style.transform = `perspective(900px) rotateX(${rx}deg) rotateY(${ry}deg) translateZ(6px)`;
    });
    card.addEventListener('pointerleave', () => { card.style.transform = ''; });
  });
}

/** The hero block leans a little as the pointer crosses the viewport. */
function initHeroTilt() {
  const stage = document.querySelector('[data-tilt-scene]');
  if (!stage || !finePointer.matches || !wantsMotion()) return;
  let raf = 0, tx = 0, ty = 0, cxr = 0, cyr = 0;
  const loop = () => {
    cxr += (tx - cxr) * 0.08;
    cyr += (ty - cyr) * 0.08;
    stage.style.transform = `rotateX(${cyr}deg) rotateY(${cxr}deg)`;
    raf = requestAnimationFrame(loop);
  };
  window.addEventListener('pointermove', (e) => {
    tx = ((e.clientX / window.innerWidth) - 0.5) * 6;
    ty = ((e.clientY / window.innerHeight) - 0.5) * -4;
    if (!raf) raf = requestAnimationFrame(loop);
  }, { passive: true });
}

/**
 * The 3D journey. Cards travel from far away, through focus, then past the
 * viewer as the rail scrolls. Only engaged when motion is welcome and the
 * viewport is tall enough to pin a section.
 */
function initRail() {
  const rail = document.getElementById('rail');
  const depth = document.getElementById('rail-depth');
  if (!rail || !depth) return;
  const steps = Array.from(depth.querySelectorAll('.step'));
  if (!steps.length) return;

  if (!wantsMotion() || window.innerWidth < 760 || window.innerHeight < 560) return;
  rail.classList.add('is-3d');

  const n = steps.length;
  let raf = 0;

  const draw = () => {
    raf = 0;
    const rect = rail.getBoundingClientRect();
    const total = rail.offsetHeight - window.innerHeight;
    if (total <= 0) return;
    const p = clamp(-rect.top / total, 0, 1);

    steps.forEach((step, i) => {
      // u < 0 → still approaching; u ≈ 0 → in focus; u > 1 → gone past.
      const u = p * n - i;
      const z = -700 + u * 900;
      const fadeIn = clamp((u + 0.85) / 0.6, 0, 1);
      const fadeOut = 1 - clamp((u - 0.5) / 0.5, 0, 1);
      const opacity = clamp(fadeIn * fadeOut, 0, 1);
      step.style.transform =
        `translate3d(0, ${u * -34}px, ${z}px) rotateX(${u * -7}deg)`;
      step.style.opacity = String(opacity);
      step.style.zIndex = String(Math.round(1000 - Math.abs(u) * 100));
      step.style.pointerEvents = opacity > 0.6 ? 'auto' : 'none';
    });
  };

  const onScroll = () => { if (!raf) raf = requestAnimationFrame(draw); };
  window.addEventListener('scroll', onScroll, { passive: true });
  window.addEventListener('resize', onScroll, { passive: true });
  draw();
}

/**
 * Hero waveform. Layered sines with a drifting envelope — it reads as live
 * audio without pretending to be a real recording.
 */
function initWave() {
  const canvas = document.getElementById('wave');
  if (!canvas || !canvas.getContext) return;
  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  let w = 0, h = 0, dpr = 1, raf = 0, t = 0;

  const resize = () => {
    dpr = Math.min(window.devicePixelRatio || 1, 2);
    const r = canvas.getBoundingClientRect();
    w = Math.max(1, Math.round(r.width));
    h = Math.max(1, Math.round(r.height));
    canvas.width = Math.round(w * dpr);
    canvas.height = Math.round(h * dpr);
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  };

  const layer = (phase, amp, alpha, color) => {
    ctx.beginPath();
    for (let x = 0; x <= w; x += 2) {
      const nx = x / w;
      // Envelope keeps the ends quiet so it floats rather than bisects.
      const env = Math.sin(nx * Math.PI) ** 1.5;
      const y =
        h / 2 +
        Math.sin(nx * 7.5 + phase) * amp * env +
        Math.sin(nx * 17.3 - phase * 1.7) * amp * 0.35 * env +
        Math.sin(nx * 3.1 + phase * 0.6) * amp * 0.5 * env;
      if (x === 0) ctx.moveTo(x, y); else ctx.lineTo(x, y);
    }
    ctx.strokeStyle = color;
    ctx.globalAlpha = alpha;
    ctx.lineWidth = 1.5;
    ctx.stroke();
    ctx.globalAlpha = 1;
  };

  const frame = () => {
    ctx.clearRect(0, 0, w, h);
    // Monochrome, with the app's recording red as the only chroma.
    const a = h * 0.19;
    layer(t,             a,        0.92, '#0a0a0a');
    layer(t * 0.8 + 1.4, a * 0.78, 0.30, '#6b7280');
    layer(t * 1.2 + 3.1, a * 0.55, 0.80, '#ff3b30');
    t += 0.016;
    raf = requestAnimationFrame(frame);
  };

  resize();
  window.addEventListener('resize', resize, { passive: true });

  if (!wantsMotion()) {
    // Draw one static frame so the hero is not empty.
    ctx.clearRect(0, 0, w, h);
    const a = h * 0.19;
    layer(0.8, a, 0.9, '#0a0a0a');
    layer(2.2, a * 0.55, 0.8, '#ff3b30');
    return;
  }

  // Pause when the hero is off-screen; no point burning frames.
  if ('IntersectionObserver' in window) {
    const io = new IntersectionObserver((entries) => {
      entries.forEach((e) => {
        if (e.isIntersecting && !raf) raf = requestAnimationFrame(frame);
        else if (!e.isIntersecting && raf) { cancelAnimationFrame(raf); raf = 0; }
      });
    }, { threshold: 0.02 });
    io.observe(canvas);
  } else {
    raf = requestAnimationFrame(frame);
  }
}

/* ───────────────────────── boot ───────────────────────── */

function boot() {
  // Motion is cosmetic: never let it block the download wiring.
  try { initHeader(); }   catch (e) { console.warn('header', e); }
  try { initReveal(); }   catch (e) { console.warn('reveal', e); }
  try { initCursor(); }   catch (e) { console.warn('cursor', e); }
  try { initMagnetic(); } catch (e) { console.warn('magnetic', e); }
  try { initCardTilt(); } catch (e) { console.warn('tilt', e); }
  try { initHeroTilt(); } catch (e) { console.warn('heroTilt', e); }
  try { initRail(); }     catch (e) { console.warn('rail', e); }
  try { initWave(); }     catch (e) { console.warn('wave', e); }
  initDownload().catch(e => console.warn('download wiring failed', e));
}

if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', boot);
} else {
  boot();
}
