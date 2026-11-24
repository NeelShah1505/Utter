/**
 * demo.js — Playable Utter, in the browser.
 *
 * Three interactive pieces, all driven by behaviour measured from the real app
 * (see docs/AUDIT/03-release-readiness.md). Nothing here invents a result:
 *
 *  1. Dictation stage — runs the real state machine (idle → listening →
 *     transcribing → idle) through the actual bubble component, then types the
 *     finished text into a mock window. Switching the target app changes the
 *     routing, which is what the app's smart routing really does: it biases the
 *     cleanup prompt, so the wording changes only because cleanup is on.
 *  2. Dictionary demo — the measured difference when the user's vocabulary is
 *     passed to whisper as initial_prompt:
 *       off → "U-State / Qubectl / Snake Case"
 *       on  → "useState / kubectl / snake_case"
 *  3. Cleanup playground — real llama3.1:8b outputs, including the
 *     self-correction case.
 *
 * Reduced motion is honoured throughout: the sequences still run, they just
 * stop animating character by character.
 */

const reduce = window.matchMedia('(prefers-reduced-motion: reduce)');
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

/* ─────────────── 1. Dictation stage ─────────────── */

const RAW = 'remind me to uh send the quarterly report to priya before friday';

const TARGETS = {
  notes: {
    name: 'Notes',
    title: 'Notes',
    placeholder: 'Start a note…',
    icon: '<svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 4h16v16H4z"></path><path d="M8 9h8M8 13h5"></path></svg>',
    final: 'Remind me to send the quarterly report to Priya before Friday.',
  },
  slack: {
    name: 'Slack',
    title: '#team — Slack',
    placeholder: 'Message #team',
    icon: '<svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 11.5a8.4 8.4 0 0 1-9 8.4 9 9 0 0 1-3.9-.9L3 21l1.9-4.1A8.4 8.4 0 0 1 12 3a8.4 8.4 0 0 1 9 8.5Z"></path></svg>',
    final: "Hey — I'll send the quarterly report to Priya before Friday.",
  },
  code: {
    name: 'VS Code',
    title: 'notes.ts — utter',
    placeholder: '// write something…',
    icon: '<svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2"><path d="m8 6-6 6 6 6M16 6l6 6-6 6"></path></svg>',
    final: '// TODO: send the quarterly report to Priya before Friday',
  },
};

function initStage() {
  const stage = document.getElementById('stage');
  if (!stage) return;

  const doc = document.getElementById('win-doc');
  const title = document.getElementById('win-title');
  const label = document.getElementById('status-label');
  const text = document.getElementById('transcript-text');
  const chipName = document.getElementById('target-app-name');
  const chipIcon = document.getElementById('target-app-icon');
  const trigger = document.getElementById('stage-trigger');
  const appBtns = Array.from(stage.querySelectorAll('[data-target]'));
  if (!doc || !label || !text || !trigger) return;

  let key = 'notes';
  let running = false;
  let token = 0;

  const setIdleDoc = () => {
    const t = TARGETS[key];
    doc.innerHTML = `<span class="ph">${t.placeholder}</span><span class="win__caret"></span>`;
  };

  const applyTarget = (next) => {
    key = next;
    const t = TARGETS[key];
    if (title) title.textContent = t.title;
    if (chipName) chipName.textContent = t.name;
    if (chipIcon) chipIcon.innerHTML = t.icon;
    appBtns.forEach((b) => b.setAttribute('aria-pressed', String(b.dataset.target === key)));
    if (!running) setIdleDoc();
  };

  const state = (s) => {
    stage.classList.remove('is-listening', 'is-transcribing');
    stage.classList.toggle('is-live', s !== 'idle');
    if (s === 'listening' || s === 'transcribing') stage.classList.add(`is-${s}`);
  };

  /** Type `str` into the bubble transcript, word by word. */
  async function speak(str, mine) {
    const words = str.split(' ');
    text.textContent = '';
    for (let i = 0; i < words.length; i++) {
      if (mine !== token) return false;
      text.textContent = words.slice(0, i + 1).join(' ');
      await sleep(reduce.matches ? 0 : 78);
    }
    return mine === token;
  }

  /** Type the finished sentence into the mock window. */
  async function typeDoc(str, mine) {
    for (let i = 0; i <= str.length; i++) {
      if (mine !== token) return false;
      doc.innerHTML = `${escapeHtml(str.slice(0, i))}<span class="win__caret"></span>`;
      await sleep(reduce.matches ? 0 : 16);
    }
    return mine === token;
  }

  async function run() {
    if (running) return;
    running = true;
    const mine = ++token;
    trigger.disabled = true;

    try {
      const t = TARGETS[key];
      doc.innerHTML = `<span class="ph">${t.placeholder}</span><span class="win__caret"></span>`;

      state('listening');
      label.textContent = 'Listening';
      text.textContent = '';
      await sleep(reduce.matches ? 0 : 320);
      if (!(await speak(RAW, mine))) return;
      await sleep(reduce.matches ? 0 : 420);

      state('transcribing');
      label.textContent = 'Transcribing';
      await sleep(reduce.matches ? 0 : 620);
      if (mine !== token) return;

      label.textContent = 'Inserted';
      text.textContent = t.final;
      if (!(await typeDoc(t.final, mine))) return;

      await sleep(reduce.matches ? 0 : 900);
      if (mine !== token) return;
      state('idle');
      label.textContent = 'Ready';
    } finally {
      if (mine === token) {
        running = false;
        trigger.disabled = false;
      }
    }
  }

  trigger.addEventListener('click', run);
  appBtns.forEach((b) => b.addEventListener('click', () => applyTarget(b.dataset.target)));

  // The real hotkey, on the page.
  window.addEventListener('keydown', (e) => {
    const combo = (e.metaKey || e.ctrlKey) && e.shiftKey && (e.key === 'd' || e.key === 'D');
    if (!combo) return;
    e.preventDefault();
    run();
  });

  applyTarget('notes');
  state('idle');
  label.textContent = 'Ready';
}

function escapeHtml(s) {
  return s.replace(/[&<>"']/g, (c) => (
    { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]
  ));
}

/* ─────────────── 2. Dictionary demo ─────────────── */

const DICT = {
  off: 'Refactor the U-State hook and call Qubectl apply on the Snake Case config.',
  on:  'Refactor the useState hook and call kubectl apply on the snake_case config.',
};
const DICT_MARK = ['useState', 'kubectl', 'snake_case'];

function initDictionary() {
  const sw = document.getElementById('dict-switch');
  const out = document.getElementById('dict-out');
  const state = document.getElementById('dict-state');
  if (!sw || !out) return;

  let on = true;

  const paint = () => {
    sw.classList.toggle('on', on);
    sw.setAttribute('aria-checked', String(on));
    if (state) state.textContent = on ? 'ON' : 'OFF';
    const src = on ? DICT.on : DICT.off;
    if (on) {
      let html = escapeHtml(src);
      DICT_MARK.forEach((w) => {
        html = html.replace(escapeHtml(w), `<mark>${escapeHtml(w)}</mark>`);
      });
      out.innerHTML = html;
    } else {
      out.textContent = src;
    }
    out.classList.remove('flash');
    void out.offsetWidth; // restart the flash
    out.classList.add('flash');
  };

  const toggle = () => { on = !on; paint(); };
  sw.addEventListener('click', toggle);
  sw.addEventListener('keydown', (e) => {
    if (e.key === ' ' || e.key === 'Enter') { e.preventDefault(); toggle(); }
  });
  paint();
}

/* ─────────────── 3. Cleanup playground ─────────────── */

const CLEAN = [
  {
    id: 'correct',
    tag: 'Self-correction',
    raw: 'book an appointment for nine a.m. oh no sorry book an appointment for ten p.m.',
    out: 'Book an appointment for 10 p.m.',
  },
  {
    id: 'filler',
    tag: 'Filler words',
    raw: 'um hello uh this is a test you know like a quick one okay',
    out: 'Hello, this is a test.',
  },
  {
    id: 'grammar',
    tag: 'Grammar',
    raw: 'me and him was going to the store yesterday and buyed some stuffs',
    out: 'My friend and I went to the store yesterday and bought some things.',
  },
];

function initCleanup() {
  const root = document.getElementById('clean');
  if (!root) return;
  const chips = Array.from(root.querySelectorAll('[data-clean]'));
  const rawEl = document.getElementById('clean-raw');
  const outEl = document.getElementById('clean-out');
  const runBtn = document.getElementById('clean-run');
  if (!rawEl || !outEl || !runBtn) return;

  let current = CLEAN[0];
  let busy = false;
  let token = 0;

  const select = (id) => {
    current = CLEAN.find((c) => c.id === id) || CLEAN[0];
    chips.forEach((c) => c.setAttribute('aria-pressed', String(c.dataset.clean === current.id)));
    rawEl.textContent = `“${current.raw}”`;
    outEl.textContent = '';
    outEl.classList.remove('is-done');
    runBtn.disabled = false;
    runBtn.textContent = 'RUN CLEANUP';
  };

  async function run() {
    if (busy) return;
    busy = true;
    const mine = ++token;
    runBtn.disabled = true;
    runBtn.textContent = 'THINKING…';
    outEl.textContent = '';
    outEl.classList.remove('is-done');

    await sleep(reduce.matches ? 0 : 520);
    const str = `“${current.out}”`;
    for (let i = 0; i <= str.length; i++) {
      if (mine !== token) { busy = false; return; }
      outEl.textContent = str.slice(0, i);
      await sleep(reduce.matches ? 0 : 18);
    }
    if (mine !== token) { busy = false; return; }
    outEl.classList.add('is-done');
    runBtn.textContent = 'RUN AGAIN';
    runBtn.disabled = false;
    busy = false;
  }

  chips.forEach((c) => c.addEventListener('click', () => select(c.dataset.clean)));
  runBtn.addEventListener('click', run);
  select(CLEAN[0].id);
}

/* ─────────────── 4. Studio tabs ─────────────── */

function initStudio() {
  const nav = document.getElementById('studio-nav');
  if (!nav) return;
  const btns = Array.from(nav.querySelectorAll('button'));
  const panes = Array.from(document.querySelectorAll('[data-pane]'));
  btns.forEach((b) => {
    b.addEventListener('click', () => {
      btns.forEach((x) => x.setAttribute('aria-selected', String(x === b)));
      panes.forEach((p) => { p.hidden = p.dataset.pane !== b.dataset.tab; });
    });
  });
  // Toggles inside Studio are demonstrative: flip them for feedback.
  document.querySelectorAll('.studio .sw').forEach((sw) => {
    sw.addEventListener('click', () => {
      const on = sw.classList.toggle('on');
      sw.setAttribute('aria-checked', String(on));
    });
  });
}

/* ─────────────── boot ─────────────── */

function boot() {
  try { initStage(); }      catch (e) { console.warn('stage', e); }
  try { initDictionary(); } catch (e) { console.warn('dictionary', e); }
  try { initCleanup(); }    catch (e) { console.warn('cleanup', e); }
  try { initStudio(); }     catch (e) { console.warn('studio', e); }
}

if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', boot);
} else {
  boot();
}
