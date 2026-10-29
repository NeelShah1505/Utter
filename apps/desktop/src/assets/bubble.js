/**
 * bubble.js — Transcript bubble logic.
 * Listens for Tauri events and updates the DOM.
 *
 * Tauri events consumed:
 *   state_change      { status: 'Idle'|'Listening'|'Transcribing'|'Error', message?: string }
 *   transcript_partial { text: string }
 *   transcript_final   { text: string }
 */

import { listen, emit } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

const body          = document.body;
const transcriptEl   = document.getElementById('transcript-text');
const statusLabel    = document.getElementById('status-label');
const hintText       = document.getElementById('hint-text');
const errorBox       = document.getElementById('error-box');
const errorMessage   = document.getElementById('error-message');
const dismissBtn     = document.getElementById('error-dismiss');
const targetAppChip  = document.getElementById('target-app-chip');
const targetAppIcon  = document.getElementById('target-app-icon');
const targetAppName  = document.getElementById('target-app-name');
const bubbleWaveform = document.getElementById('bubble-waveform');

// ---------------------------------------------------------------------------
// Synthesizer Audio Cues (Earcons)
// ---------------------------------------------------------------------------
let audioCtx = null;
function playAudioCue(type) {
  try {
    if (!audioCtx) {
      audioCtx = new (window.AudioContext || window.webkitAudioContext)();
    }
    if (audioCtx.state === 'suspended') {
      audioCtx.resume();
    }
    const now = audioCtx.currentTime;
    const osc = audioCtx.createOscillator();
    const gain = audioCtx.createGain();
    osc.connect(gain);
    gain.connect(audioCtx.destination);

    if (type === 'start') {
      // Pleasant rising chime (C5 -> G5)
      osc.frequency.setValueAtTime(523.25, now);
      osc.frequency.exponentialRampToValueAtTime(783.99, now + 0.07);
      gain.gain.setValueAtTime(0.06, now);
      gain.gain.exponentialRampToValueAtTime(0.001, now + 0.11);
      osc.start(now);
      osc.stop(now + 0.11);
    } else if (type === 'stop') {
      // Gentle resolving chime (A5 -> E5)
      osc.frequency.setValueAtTime(880.00, now);
      osc.frequency.exponentialRampToValueAtTime(659.25, now + 0.08);
      gain.gain.setValueAtTime(0.06, now);
      gain.gain.exponentialRampToValueAtTime(0.001, now + 0.14);
      osc.start(now);
      osc.stop(now + 0.14);
    }
  } catch (e) {
    // Audio contexts might be blocked until user gesture, ignore gracefully
  }
}

// ---------------------------------------------------------------------------
// State → DOM mapping
// ---------------------------------------------------------------------------

const STATE_CLASSES = ['state-idle', 'state-listening', 'state-transcribing', 'state-error'];

function updateAppBadge(app) {
  if (!app) return;
  if (targetAppIcon) {
    if (app.mode === 'code') {
      targetAppIcon.innerHTML = '<svg viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2" fill="none"><polyline points="16 18 22 12 16 6"></polyline><polyline points="8 6 2 12 8 18"></polyline></svg>';
    } else if (app.mode === 'chat') {
      targetAppIcon.innerHTML = '<svg viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2" fill="none"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path></svg>';
    } else {
      targetAppIcon.innerHTML = '<svg viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2" fill="none"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon></svg>';
    }
  }
  if (targetAppName) targetAppName.textContent = `${app.name} • ${app.mode_label || 'Auto'}`;
}

function applyState(status, message = '', app = null) {
  STATE_CLASSES.forEach(c => body.classList.remove(c));
  body.classList.add(`state-${status.toLowerCase()}`);

  const labels = {
    Idle:         'Ready',
    Listening:    'Listening…',
    Transcribing: 'Refining…',
    Error:        'Error',
  };
  statusLabel.textContent = labels[status] ?? status;

  if (bubbleWaveform) {
    bubbleWaveform.hidden = (status !== 'Listening');
  }

  if (app) updateAppBadge(app);

  if (status === 'Listening') {
    playAudioCue('start');
  } else if (status === 'Idle') {
    playAudioCue('stop');
  }

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
    applyState(payload.status, payload.message, payload.app);
  });

  // active_app_detected event
  await listen('active_app_detected', ({ payload }) => {
    updateAppBadge(payload);
  });

  // transcript_partial — stream of partial results while listening
  await listen('transcript_partial', ({ payload }) => {
    transcriptEl.textContent = payload.text || '';
  });

  // transcript_final — final result after processing
  await listen('transcript_final', ({ payload }) => {
    transcriptEl.textContent = payload.text || '';
    if (payload.insert_error) {
      applyState('Error', payload.insert_error);
      return;
    }
    setTimeout(() => applyState('Idle'), 1400);
  });

  // Query initial active app
  try {
    const app = await invoke('get_active_app');
    updateAppBadge(app);
  } catch (e) {
    // Ignore
  }

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


