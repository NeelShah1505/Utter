/**
 * bubble.js — Transcript bubble logic.
 * Listens for Tauri events and updates the DOM.
 *
 * Tauri events consumed:
 *   state_change      { status: 'Idle'|'Listening'|'Transcribing'|'Error', message?: string }
 *   transcript_partial { text: string }
 *   transcript_final   { text: string }
 */

const { listen, emit } = window.__TAURI__?.event ?? mockTauriEvent();
const { invoke }       = window.__TAURI__?.core   ?? mockTauriCore();

const body          = document.body;
const transcriptEl  = document.getElementById('transcript-text');
const statusLabel   = document.getElementById('status-label');
const hintText      = document.getElementById('hint-text');
const errorBox      = document.getElementById('error-box');
const errorMessage  = document.getElementById('error-message');
const dismissBtn    = document.getElementById('error-dismiss');

// ---------------------------------------------------------------------------
// State → DOM mapping
// ---------------------------------------------------------------------------

const STATE_CLASSES = ['state-idle', 'state-listening', 'state-transcribing', 'state-error'];

function applyState(status, message = '') {
  STATE_CLASSES.forEach(c => body.classList.remove(c));
  body.classList.add(`state-${status.toLowerCase()}`);

  const labels = {
    Idle:         'Ready',
    Listening:    'Listening…',
    Transcribing: 'Processing…',
    Error:        'Error',
  };
  statusLabel.textContent = labels[status] ?? status;

  hintText.hidden = (status === 'Listening' || status === 'Transcribing');

  errorBox.hidden = (status !== 'Error');
  if (status === 'Error' && message) {
    errorMessage.textContent = message;
  }

  // Clear transcript text on transition back to Idle
  if (status === 'Idle') {
    transcriptEl.textContent = '';
  }
}

// ---------------------------------------------------------------------------
// Tauri event listeners
// ---------------------------------------------------------------------------

async function init() {
  // state_change event — from hotkey handler and IPC commands
  await listen('state_change', ({ payload }) => {
    applyState(payload.status, payload.message);
  });

  // transcript_partial — stream of partial results while listening
  await listen('transcript_partial', ({ payload }) => {
    transcriptEl.textContent = payload.text;
  });

  // transcript_final — final result after processing
  await listen('transcript_final', ({ payload }) => {
    transcriptEl.textContent = payload.text;
    // Brief flash of the final text before clearing
    setTimeout(() => applyState('Idle'), 1500);
  });

  // Load initial status from backend
  try {
    const status = await invoke('get_status');
    applyState(status.status ?? 'Idle');
  } catch (e) {
    console.warn('get_status failed:', e);
    applyState('Idle');
  }
}

dismissBtn.addEventListener('click', () => applyState('Idle'));

init().catch(console.error);

// ---------------------------------------------------------------------------
// Dev fallback — allows opening index.html in a browser without Tauri
// ---------------------------------------------------------------------------

function mockTauriEvent() {
  console.warn('Tauri not available — using mock event system');
  return {
    listen: async (event, cb) => {
      console.log(`[mock] listening for event: ${event}`);
      return () => {};
    },
    emit: async (event, payload) => {
      console.log(`[mock] emit ${event}:`, payload);
    },
  };
}

function mockTauriCore() {
  return {
    invoke: async (cmd, args) => {
      console.log(`[mock] invoke ${cmd}:`, args);
      return { status: 'Idle' };
    },
  };
}
