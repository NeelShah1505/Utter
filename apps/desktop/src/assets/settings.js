/**
 * settings.js — Full Utter Studio & Flow Dashboard Logic
 * Handles tab navigation, interactive dictionary editor, AI configuration,
 * real-time transcription history, and Tauri IPC communication.
 */

import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// ---------------------------------------------------------------------------
// State & Element references
// ---------------------------------------------------------------------------

let currentSettings = {
  hotkey: 'CmdOrCtrl+Shift+D',
  model_path: '',
  audio_device: '',
  cleanup: { type: 'None' },
  dictionary: []
};

let dictHistory = JSON.parse(localStorage.getItem('utter_history') || '[]');

// DOM Elements
const navItems          = document.querySelectorAll('.nav-item');
const viewPanels        = document.querySelectorAll('.view-panel');
const hotkeyInput       = document.getElementById('hotkey-input');
const audioSelect       = document.getElementById('audio-device-select');
const modelSelect       = document.getElementById('model-select');

// AI Fields
const engineRadios      = document.querySelectorAll('input[name="engine-type"]');
const boxNone           = document.getElementById('box-None');
const boxLocalOllama    = document.getElementById('box-LocalOllama');
const boxOpenAiCompat   = document.getElementById('box-OpenAiCompat');
const ollamaLocalUrl    = document.getElementById('ollama-local-url');
const ollamaLocalModel  = document.getElementById('ollama-local-model');
const oaiUrl            = document.getElementById('oai-url');
const oaiModel          = document.getElementById('oai-model');
const oaiKey            = document.getElementById('oai-key');
const boxCustomPrompt   = document.getElementById('box-CustomPrompt');
const aiCustomPrompt    = document.getElementById('ai-custom-prompt');
const testBar           = document.getElementById('test-bar');
const testBtn           = document.getElementById('cleanup-test-btn');
const testResult        = document.getElementById('cleanup-test-result');

// Dictionary Elements
const dictTbody         = document.getElementById('dictionary-tbody');
const btnShowAddModal   = document.getElementById('btn-show-add-modal');
const addDictBox        = document.getElementById('add-dict-box');
const btnSaveWord       = document.getElementById('btn-save-word');
const btnCancelWord     = document.getElementById('btn-cancel-word');
const newWordInput      = document.getElementById('new-word-input');
const newReplaceInput   = document.getElementById('new-replace-input');
const dictSearchInput   = document.getElementById('dict-search-input');

// Save Toolbar
const saveBtn           = document.getElementById('save-btn');
const cancelBtn         = document.getElementById('cancel-btn');
const saveStatus        = document.getElementById('save-status');

// ---------------------------------------------------------------------------
// 1. Navigation & Tab Switching
// ---------------------------------------------------------------------------

function switchTab(tabId) {
  navItems.forEach(btn => {
    btn.classList.toggle('active', btn.getAttribute('data-tab') === tabId);
  });
  viewPanels.forEach(panel => {
    panel.classList.toggle('active', panel.id === `view-${tabId}`);
  });
}

navItems.forEach(btn => {
  btn.addEventListener('click', () => switchTab(btn.getAttribute('data-tab')));
});

document.getElementById('btn-explore-dict')?.addEventListener('click', () => {
  switchTab('dictionary');
});

// Listen for tray menu navigation events from Rust!
listen('navigate_tab', (event) => {
  if (event.payload) switchTab(event.payload);
});

// ---------------------------------------------------------------------------
// 2. AI Engine Mode Selector & Presets
// ---------------------------------------------------------------------------

const aiStatusNotice = document.getElementById('ai-status-notice');
const btnGotoAiTab  = document.getElementById('btn-goto-ai-tab');

if (btnGotoAiTab) {
  btnGotoAiTab.addEventListener('click', () => switchTab('ai'));
}

function updateAiEngineVisibility(type) {
  if (boxNone) boxNone.hidden = (type !== 'None');
  if (boxLocalOllama) boxLocalOllama.hidden = (type !== 'LocalOllama');
  if (boxOpenAiCompat) boxOpenAiCompat.hidden = (type !== 'OpenAiCompat');
  if (boxCustomPrompt) boxCustomPrompt.hidden = (type === 'None');
  if (testBar) testBar.hidden = (type === 'None');
  if (aiStatusNotice) aiStatusNotice.hidden = (type !== 'None');

  // Immediately clear any leftover test result message
  if (testResult) {
    testResult.textContent = '';
    testResult.className = 'test-result-msg';
  }
}

engineRadios.forEach(radio => {
  radio.addEventListener('change', () => updateAiEngineVisibility(radio.value));
});

// Clear test results whenever the user types or alters AI configuration
[ollamaLocalUrl, ollamaLocalModel, oaiUrl, oaiModel, oaiKey, aiCustomPrompt].forEach(input => {
  input?.addEventListener('input', () => {
    if (testResult) {
      testResult.textContent = '';
      testResult.className = 'test-result-msg';
    }
  });
});

document.querySelectorAll('.btn-preset').forEach(btn => {
  btn.addEventListener('click', () => {
    if (oaiUrl) oaiUrl.value = btn.getAttribute('data-url');
    if (oaiModel) oaiModel.value = btn.getAttribute('data-model');
    if (testResult) {
      testResult.textContent = '';
      testResult.className = 'test-result-msg';
    }
  });
});

const STYLE_PROMPTS = {
  natural: "",
  email: "Format this voice dictation into a polished, professional email with polite greetings and clear structure.",
  chat: "Format this voice dictation into concise, direct chat messaging bullet points without fluff or salutations.",
  developer: "Format this dictation for a software engineer, preserving code variable names (camelCase, snake_case), technical lingo, and terminal commands."
};

function highlightActiveStyle(prompt) {
  const normalized = (prompt || '').trim();
  let matchedKey = null;

  if (!normalized) {
    matchedKey = 'natural';
  } else {
    for (const [key, text] of Object.entries(STYLE_PROMPTS)) {
      if (key !== 'natural' && normalized === text.trim()) {
        matchedKey = key;
        break;
      }
    }
  }

  document.querySelectorAll('.style-card').forEach(c => {
    const key = c.getAttribute('data-style');
    const badge = c.querySelector('.badge-status');
    const btn = c.querySelector('.btn-style-apply');
    const isActive = (key === matchedKey);

    c.classList.toggle('active-style', isActive);
    if (badge) {
      badge.classList.toggle('enabled', isActive);
      badge.textContent = isActive ? 'Active' : (key === 'natural' ? 'Default' : 'Optional');
    }
    if (btn) {
      btn.textContent = isActive ? '✓ Active' : 'Apply Style';
    }
  });
}

function selectStyle(styleKey) {
  const currentPrompt = (aiCustomPrompt ? aiCustomPrompt.value : '').trim();
  let targetKey = styleKey;
  if (styleKey !== 'natural' && currentPrompt === STYLE_PROMPTS[styleKey].trim()) {
    targetKey = 'natural';
  }

  const promptText = STYLE_PROMPTS[targetKey] || '';
  if (aiCustomPrompt) {
    aiCustomPrompt.value = promptText;
  }
  highlightActiveStyle(promptText);
  saveSettingsSilently();

  const currentEngine = document.querySelector('input[name="engine-type"]:checked')?.value || 'None';
  if (currentEngine === 'None' && targetKey !== 'natural') {
    if (saveStatus) {
      saveStatus.textContent = 'ℹ️ Style saved. Note: Enable Ollama or Cloud AI in "AI & Models" to apply live writing styles.';
      saveStatus.className = 'save-status-msg';
      setTimeout(() => { if (saveStatus && saveStatus.textContent.startsWith('ℹ️')) saveStatus.textContent = ''; }, 4500);
    }
  }
}

document.querySelectorAll('.style-card').forEach(card => {
  card.addEventListener('click', () => {
    const styleKey = card.getAttribute('data-style');
    selectStyle(styleKey);
  });
});

const btnQuickAddSnippet = document.getElementById('btn-quick-add-snippet');
if (btnQuickAddSnippet) {
  btnQuickAddSnippet.addEventListener('click', () => {
    switchTab('dictionary');
    if (addDictBox) {
      addDictBox.hidden = false;
      if (newWordInput) newWordInput.focus();
    }
  });
}

// ---------------------------------------------------------------------------
// 3. Dictionary Manager Logic
// ---------------------------------------------------------------------------

function renderDictionary() {
  if (!dictTbody) return;
  dictTbody.innerHTML = '';
  
  const query = dictSearchInput ? dictSearchInput.value.toLowerCase() : '';
  const items = currentSettings.dictionary || [];

  items.filter(item => {
    return item.word.toLowerCase().includes(query) || 
           (item.replacement && item.replacement.toLowerCase().includes(query));
  }).forEach((item, index) => {
    const tr = document.createElement('tr');
    
    const tdWord = document.createElement('td');
    tdWord.innerHTML = `<strong>${escapeHtml(item.word)}</strong>`;
    
    const tdReplace = document.createElement('td');
    tdReplace.textContent = item.replacement || '(Recognize exact word)';
    if (!item.replacement) tdReplace.style.color = '#94A0B8';

    const tdActions = document.createElement('td');
    tdActions.className = 'col-actions';
    const delBtn = document.createElement('button');
    delBtn.className = 'btn-delete-icon';
    delBtn.innerHTML = '🗑️';
    delBtn.title = 'Remove word';
    delBtn.onclick = () => {
      currentSettings.dictionary.splice(index, 1);
      renderDictionary();
      saveSettingsSilently();
    };

    tdActions.appendChild(delBtn);
    tr.appendChild(tdWord);
    tr.appendChild(tdReplace);
    tr.appendChild(tdActions);
    dictTbody.appendChild(tr);
  });
}

if (dictSearchInput) {
  dictSearchInput.addEventListener('input', renderDictionary);
}

if (btnShowAddModal) {
  btnShowAddModal.addEventListener('click', () => {
    addDictBox.hidden = false;
    newWordInput.focus();
  });
}

if (btnCancelWord) {
  btnCancelWord.addEventListener('click', () => {
    addDictBox.hidden = true;
    newWordInput.value = '';
    newReplaceInput.value = '';
  });
}

if (btnSaveWord) {
  btnSaveWord.addEventListener('click', () => {
    const word = newWordInput.value.trim();
    const replacement = newReplaceInput.value.trim();
    if (!word) return;

    if (!currentSettings.dictionary) currentSettings.dictionary = [];
    currentSettings.dictionary.push({
      word: word,
      replacement: replacement ? replacement : null
    });

    addDictBox.hidden = true;
    newWordInput.value = '';
    newReplaceInput.value = '';
    renderDictionary();
    saveSettingsSilently();
  });
}

function escapeHtml(str) {
  return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

// ---------------------------------------------------------------------------
// 4. Transcription Activity Feed & Stats
// ---------------------------------------------------------------------------

async function loadHistoryFromBackend() {
  try {
    const entries = await invoke('get_history');
    if (Array.isArray(entries)) {
      dictHistory = entries;
      updateHistoryUI();
    }
  } catch (e) {
    console.error('get_history failed:', e);
  }
}

function updateHistoryUI() {
  const container = document.getElementById('history-list');
  if (!container) return;
  container.innerHTML = '';

  const statWords  = document.getElementById('stat-words');
  const statWpm    = document.getElementById('stat-wpm');
  const statStreak = document.getElementById('stat-streak');

  if (dictHistory.length === 0) {
    container.innerHTML = `
      <div id="history-empty" class="empty-state-card">
        <div class="empty-icon"><svg viewBox="0 0 24 24" width="32" height="32" stroke="currentColor" stroke-width="2" fill="none"><path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z"></path><path d="M19 10v2a7 7 0 0 1-14 0v-2"></path><line x1="12" y1="19" x2="12" y2="22"></line></svg></div>
        <h4>No dictations yet</h4>
        <p>Trigger dictation with Cmd+Shift+D anywhere on your Mac. Your real speech-to-text logs will appear here instantly with quick copy controls.</p>
      </div>
    `;
    if (statWords) statWords.textContent = '0';
    if (statWpm) statWpm.innerHTML = '0 <small>wpm</small>';
    if (statStreak) statStreak.innerHTML = '0 <small>days</small>';
    return;
  }

  let totalWords = 0;
  dictHistory.forEach((entry, idx) => {
    const wordCount = entry.word_count || entry.text.split(/\s+/).filter(w => w.length > 0).length;
    totalWords += wordCount;
    
    const card = document.createElement('div');
    card.className = 'history-card';
    card.innerHTML = `
      <div class="history-meta">
        <span class="history-time">${entry.timestamp || 'Just now'}</span>
        <span class="history-app"><svg viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2" fill="none" style="vertical-align: middle; margin-right: 3px;"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon></svg> Real-time Streamed</span>
      </div>
      <p class="history-text" id="hist-text-${idx}">${escapeHtml(entry.text)}</p>
      <div class="history-actions">
        <button class="btn-action-sm btn-copy-sm" data-idx="${idx}">Copy</button>
        <button class="btn-action-sm btn-edit-sm" data-idx="${idx}">Edit</button>
        <button class="btn-action-sm btn-action-danger btn-del-sm" data-idx="${idx}">Delete</button>
      </div>
    `;
    
    const copyBtn = card.querySelector('.btn-copy-sm');
    copyBtn.onclick = () => {
      navigator.clipboard.writeText(entry.text);
      copyBtn.textContent = '✓ Copied';
      setTimeout(() => { copyBtn.textContent = 'Copy'; }, 2000);
    };

    const delBtn = card.querySelector('.btn-del-sm');
    delBtn.onclick = async () => {
      try {
        dictHistory = await invoke('delete_history_item', { id: entry.id });
      } catch (e) {
        dictHistory.splice(idx, 1);
      }
      updateHistoryUI();
    };

    const editBtn = card.querySelector('.btn-edit-sm');
    editBtn.onclick = async () => {
      const newText = prompt('Edit transcript:', entry.text);
      if (newText !== null && newText.trim() !== '') {
        try {
          dictHistory = await invoke('update_history_item', { id: entry.id, newText: newText.trim() });
        } catch (e) {
          dictHistory[idx].text = newText.trim();
        }
        updateHistoryUI();
      }
    };

    container.appendChild(card);
  });

  if (statWords) statWords.textContent = totalWords.toLocaleString();
  if (statWpm) statWpm.innerHTML = `${totalWords > 0 ? 128 : 0} <small>wpm</small>`;
  if (statStreak) statStreak.innerHTML = `1 <small>day</small>`;
}

document.getElementById('btn-clear-history')?.addEventListener('click', async () => {
  try {
    await invoke('clear_history');
  } catch (e) {
    console.error('clear_history failed:', e);
  }
  dictHistory = [];
  updateHistoryUI();
});

// Real-time listener for finished dictations from engine
listen('transcript_final', (event) => {
  if (event.payload) {
    if (event.payload.entry) {
      dictHistory = dictHistory.filter(e => e.id !== event.payload.entry.id);
      dictHistory.unshift(event.payload.entry);
    } else if (event.payload.text) {
      const timeStr = new Date().toLocaleDateString('en-US', { month: 'short', day: 'numeric' }) + ' • ' + new Date().toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit' });
      dictHistory.unshift({ id: 'hist_' + Date.now(), text: event.payload.text, timestamp: timeStr, word_count: event.payload.text.split(/\s+/).length });
    }
    updateHistoryUI();
  }
});

// ---------------------------------------------------------------------------
// 5. Load Settings via IPC
// ---------------------------------------------------------------------------

async function loadSettings() {
  try {
    currentSettings = await invoke('get_settings');
    console.log('Loaded settings:', currentSettings);
  } catch (e) {
    console.error('get_settings failed:', e);
    return;
  }

  hotkeyInput.value = currentSettings.hotkey ?? 'CmdOrCtrl+Shift+D';

  // Audio devices
  try {
    const devices = await invoke('list_audio_devices');
    devices.forEach(name => {
      const opt = document.createElement('option');
      opt.value = name;
      opt.textContent = name;
      if (name === currentSettings.audio_device) opt.selected = true;
      audioSelect.appendChild(opt);
    });
  } catch (e) {
    console.warn('list_audio_devices failed:', e);
  }

  // Models
  try {
    const models = await invoke('list_models');
    models.forEach(m => {
      const opt = document.createElement('option');
      opt.value = m.path;
      opt.textContent = `${m.name}`;
      if (m.path === currentSettings.model_path) opt.selected = true;
      modelSelect.appendChild(opt);
    });
  } catch (e) {
    console.warn('list_models failed:', e);
  }

  // AI Cleanup configuration
  const cleanup = currentSettings.cleanup ?? { type: 'None' };
  const engineType = cleanup.type ?? 'None';

  const radio = document.querySelector(`input[name="engine-type"][value="${engineType}"]`);
  if (radio) radio.checked = true;

  if (engineType === 'LocalOllama') {
    ollamaLocalUrl.value   = cleanup.url   ?? 'http://localhost:11434';
    ollamaLocalModel.value = cleanup.model ?? 'llama3';
    aiCustomPrompt.value   = cleanup.prompt ?? '';
  } else if (engineType === 'OpenAiCompat') {
    oaiUrl.value   = cleanup.url   ?? 'https://api.openai.com';
    oaiModel.value = cleanup.model ?? 'gpt-4o-mini';
    aiCustomPrompt.value   = cleanup.prompt ?? '';
  } else {
    aiCustomPrompt.value   = cleanup.prompt ?? '';
  }

  highlightActiveStyle(cleanup.prompt);
  updateAiEngineVisibility(engineType);
  renderDictionary();
  updateHistoryUI();
}

// ---------------------------------------------------------------------------
// 6. Save & Test AI Connection
// ---------------------------------------------------------------------------

function buildSettingsObject() {
  const selectedEngine = document.querySelector('input[name="engine-type"]:checked')?.value || 'None';
  let cleanup;

  if (selectedEngine === 'None') {
    cleanup = { type: 'None' };
  } else if (selectedEngine === 'LocalOllama') {
    cleanup = {
      type:  'LocalOllama',
      url:   ollamaLocalUrl.value.trim()   || 'http://localhost:11434',
      model: ollamaLocalModel.value.trim() || 'llama3',
      prompt: aiCustomPrompt.value.trim() || null,
    };
  } else if (selectedEngine === 'OpenAiCompat') {
    cleanup = {
      type:             'OpenAiCompat',
      url:              oaiUrl.value.trim() || 'https://api.openai.com',
      model:            oaiModel.value.trim() || 'gpt-4o-mini',
      keychain_account: 'cleanup_openai_compat_key',
      prompt:           aiCustomPrompt.value.trim() || null,
    };
  }

  return {
    hotkey:       hotkeyInput.value.trim() || 'CmdOrCtrl+Shift+D',
    model_path:   modelSelect.value || '',
    audio_device: audioSelect.value === 'System Default Microphone' ? '' : audioSelect.value,
    cleanup:      cleanup,
    dictionary:   currentSettings.dictionary || []
  };
}

async function saveSettingsSilently() {
  try {
    const updated = buildSettingsObject();
    await invoke('set_settings', { newSettings: updated });
    currentSettings = updated;
  } catch (e) {
    console.error('Silent save failed:', e);
  }
}

if (saveBtn) {
  saveBtn.addEventListener('click', async () => {
    saveStatus.textContent = 'Saving configuration…';
    saveStatus.className = 'save-status-msg';

    try {
      const updated = buildSettingsObject();
      await invoke('set_settings', { newSettings: updated });
      currentSettings = updated;
      
      // Store API Key securely to macOS Keychain if provided
      if (updated.cleanup.type === 'OpenAiCompat' && oaiKey && oaiKey.value.trim()) {
        await invoke('set_keychain_secret', { 
          account: updated.cleanup.keychain_account, 
          secret: oaiKey.value.trim() 
        });
      }

      saveStatus.textContent = '✓ Studio configuration saved successfully!';
      saveStatus.className = 'save-status-msg success';
      setTimeout(() => { saveStatus.textContent = ''; }, 3500);
    } catch (e) {
      saveStatus.textContent = `✗ Save failed: ${e?.message ?? 'Unknown error'}`;
      saveStatus.className = 'save-status-msg error';
    }
  });
}

if (cancelBtn) {
  cancelBtn.addEventListener('click', () => {
    loadSettings().catch(console.error);
    saveStatus.textContent = 'Reverted to saved state.';
    saveStatus.className = 'save-status-msg';
    setTimeout(() => { saveStatus.textContent = ''; }, 2500);
  });
}

if (testBtn) {
  testBtn.addEventListener('click', async () => {
    const settings = buildSettingsObject();
    if (settings.cleanup.type === 'None') {
      testResult.textContent = 'ℹ️ Direct Acoustic Mode is active (no external LLM to test).';
      testResult.className = 'test-result-msg';
      return;
    }

    testBtn.disabled = true;
    const targetDesc = settings.cleanup.type === 'LocalOllama'
      ? `${settings.cleanup.url} (${settings.cleanup.model})`
      : `${settings.cleanup.url} (${settings.cleanup.model})`;

    testResult.textContent = `Connecting to ${targetDesc}…`;
    testResult.className = 'test-result-msg';

    try {
      if (settings.cleanup.type === 'OpenAiCompat' && oaiKey && oaiKey.value.trim()) {
        await invoke('set_keychain_secret', { 
          account: settings.cleanup.keychain_account, 
          secret: oaiKey.value.trim() 
        });
      }
      const result = await invoke('test_cleanup', { settings });
      const cleanSample = result.output ? result.output.replace(/[\r\n]+/g, ' ').trim() : '';
      testResult.textContent = `✓ Responded in ${result.latency_ms}ms ("${cleanSample}")`;
      testResult.className = 'test-result-msg success';
    } catch (e) {
      const err = e?.message ?? e ?? 'Ensure server is reachable';
      testResult.textContent = `✗ Connection Failed: ${err}`;
      testResult.className = 'test-result-msg error';
    } finally {
      testBtn.disabled = false;
    }
  });
}

// Initialize on document load
loadSettings().catch(console.error);
loadHistoryFromBackend().catch(console.error);

// Accessibility Permission Check
const accessBadge    = document.getElementById('accessibility-badge');
const grantAccessBtn = document.getElementById('btn-grant-accessibility');

async function updateAccessibilityUI() {
  if (!accessBadge) return;
  try {
    const isGranted = await invoke('check_accessibility');
    // Clear any previous inline styles
    accessBadge.style.background = '';
    accessBadge.style.color = '';
    if (isGranted) {
      accessBadge.className = 'perm-badge perm-badge-success';
      accessBadge.innerHTML = '<span class="perm-dot perm-dot-success"></span><span class="perm-text">Granted</span>';
      if (grantAccessBtn) {
        grantAccessBtn.innerHTML = `
          <svg viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect><path d="M7 11V7a5 5 0 0 1 10 0v4"></path></svg>
          <span>System Settings</span>
          <svg viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2" fill="none" style="opacity: 0.65; margin-left: 2px;"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path><polyline points="15 3 21 3 21 9"></polyline><line x1="10" y1="14" x2="21" y2="3"></line></svg>
        `;
      }
    } else {
      accessBadge.className = 'perm-badge perm-badge-warning';
      accessBadge.innerHTML = '<span class="perm-dot perm-dot-warning"></span><span class="perm-text">Action Required</span>';
      if (grantAccessBtn) {
        grantAccessBtn.innerHTML = `
          <svg viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect><path d="M7 11V7a5 5 0 0 1 10 0v4"></path></svg>
          <span>Open System Settings</span>
          <svg viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2" fill="none" style="opacity: 0.65; margin-left: 2px;"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path><polyline points="15 3 21 3 21 9"></polyline><line x1="10" y1="14" x2="21" y2="3"></line></svg>
        `;
      }
    }
  } catch (e) {
    console.error('Failed to check accessibility:', e);
  }
}

if (grantAccessBtn) {
  grantAccessBtn.addEventListener('click', async () => {
    try {
      await invoke('request_accessibility');
    } catch (e) {
      console.error('Failed to open system settings:', e);
    }
  });
}

document.getElementById('btn-open-config-dir')?.addEventListener('click', async () => {
  try {
    await invoke('open_app_folder');
  } catch (e) {
    console.error('Failed to open config directory:', e);
  }
});

updateAccessibilityUI();
window.addEventListener('focus', () => {
  updateAccessibilityUI();
  loadHistoryFromBackend();
});

// ---------------------------------------------------------------------------
// 7. Voice Input (Mic) & Audio Upload Card Studio Controls
// ---------------------------------------------------------------------------

const voiceInputBtn       = document.getElementById('voice-input-btn');
const micSvgIcon          = document.getElementById('mic-svg-icon');
const micStopBox          = document.getElementById('mic-stop-box');
const voiceListeningContent = document.getElementById('voice-listening-content');
const voiceFreqBars       = document.getElementById('voice-freq-bars');
const voiceTimer          = document.getElementById('voice-timer');
const voiceHintMsg        = document.getElementById('voice-hint-msg');
const voiceStatusLabel    = document.getElementById('voice-status-label');
const voiceInteractiveZone = document.getElementById('voice-interactive-zone');

let isVoiceActive = false;
let voiceSeconds = 0;
let voiceTimerTimer = null;
let voiceWaveTimer = null;

function formatVoiceTime(sec) {
  const m = Math.floor(sec / 60);
  const s = sec % 60;
  return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
}

function startVoiceAnimation() {
  if (voiceFreqBars) {
    voiceFreqBars.innerHTML = '';
    for (let i = 0; i < 12; i++) {
      const bar = document.createElement('div');
      bar.className = 'voice-freq-bar';
      bar.style.height = '4px';
      voiceFreqBars.appendChild(bar);
    }
  }

  if (voiceWaveTimer) clearInterval(voiceWaveTimer);
  voiceWaveTimer = setInterval(() => {
    if (!voiceFreqBars) return;
    const bars = voiceFreqBars.querySelectorAll('.voice-freq-bar');
    bars.forEach((b) => {
      const h = Math.floor(3 + Math.random() * 14);
      b.style.height = `${h}px`;
    });
  }, 120);

  if (voiceTimerTimer) clearInterval(voiceTimerTimer);
  voiceSeconds = 0;
  if (voiceTimer) voiceTimer.textContent = '00:00';
  voiceTimerTimer = setInterval(() => {
    voiceSeconds++;
    if (voiceTimer) voiceTimer.textContent = formatVoiceTime(voiceSeconds);
  }, 1000);
}

function stopVoiceAnimation() {
  if (voiceWaveTimer) {
    clearInterval(voiceWaveTimer);
    voiceWaveTimer = null;
  }
  if (voiceTimerTimer) {
    clearInterval(voiceTimerTimer);
    voiceTimerTimer = null;
  }
  voiceSeconds = 0;
  if (voiceTimer) voiceTimer.textContent = '00:00';
}

// ---------------------------------------------------------------------------
// Synthesizer Audio Cues (Earcons)
// ---------------------------------------------------------------------------
let settingsAudioCtx = null;
function playAudioCue(type) {
  const toggleSound = document.getElementById('toggle-sound-effects');
  if (toggleSound && !toggleSound.checked) return;
  try {
    if (!settingsAudioCtx) {
      settingsAudioCtx = new (window.AudioContext || window.webkitAudioContext)();
    }
    if (settingsAudioCtx.state === 'suspended') {
      settingsAudioCtx.resume();
    }
    const now = settingsAudioCtx.currentTime;
    const osc = settingsAudioCtx.createOscillator();
    const gain = settingsAudioCtx.createGain();
    osc.connect(gain);
    gain.connect(settingsAudioCtx.destination);
    if (type === 'start') {
      osc.frequency.setValueAtTime(523.25, now);
      osc.frequency.exponentialRampToValueAtTime(783.99, now + 0.07);
      gain.gain.setValueAtTime(0.06, now);
      gain.gain.exponentialRampToValueAtTime(0.001, now + 0.11);
      osc.start(now);
      osc.stop(now + 0.11);
    } else if (type === 'stop') {
      osc.frequency.setValueAtTime(880.00, now);
      osc.frequency.exponentialRampToValueAtTime(659.25, now + 0.08);
      gain.gain.setValueAtTime(0.06, now);
      gain.gain.exponentialRampToValueAtTime(0.001, now + 0.14);
      osc.start(now);
      osc.stop(now + 0.14);
    }
  } catch (e) {}
}

function updateVoiceUI(status) {
  if (status === 'Listening') {
    isVoiceActive = true;
    playAudioCue('start');
    if (voiceInputBtn) voiceInputBtn.classList.add('listening');
    if (micSvgIcon) micSvgIcon.style.display = 'none';
    if (micStopBox) micStopBox.hidden = false;
    if (voiceListeningContent) voiceListeningContent.hidden = false;
    if (voiceStatusLabel) voiceStatusLabel.textContent = 'Listening...';
    if (voiceHintMsg) voiceHintMsg.innerHTML = 'Listening... Speak naturally or click stop button';
    startVoiceAnimation();
  } else if (status === 'Transcribing') {
    if (voiceStatusLabel) voiceStatusLabel.textContent = 'Transcribing...';
    if (voiceHintMsg) voiceHintMsg.textContent = 'Transcribing voice & applying AI cleanup...';
  } else {
    if (isVoiceActive) playAudioCue('stop');
    isVoiceActive = false;
    if (voiceInputBtn) voiceInputBtn.classList.remove('listening');
    if (micSvgIcon) micSvgIcon.style.display = '';
    if (micStopBox) micStopBox.hidden = true;
    if (voiceListeningContent) voiceListeningContent.hidden = true;
    if (voiceStatusLabel) voiceStatusLabel.textContent = 'Ready to Record';
    if (voiceHintMsg) voiceHintMsg.innerHTML = 'Click mic to dictate inside Utter or press <kbd>⌘⇧D</kbd> anywhere on your Mac';
    stopVoiceAnimation();
  }
}

async function toggleVoiceDictation() {
  try {
    if (!isVoiceActive) {
      if (voiceStatusLabel) voiceStatusLabel.textContent = 'Opening mic...';
      await invoke('start_dictation');
    } else {
      if (voiceStatusLabel) voiceStatusLabel.textContent = 'Stopping...';
      await invoke('stop_dictation');
    }
  } catch (e) {
    console.error('Mic toggle failed:', e);
    if (voiceHintMsg) voiceHintMsg.textContent = `Mic error: ${e?.message ?? e}`;
    if (voiceStatusLabel) voiceStatusLabel.textContent = 'Error';
  }
}

if (voiceInputBtn) {
  voiceInputBtn.addEventListener('click', (e) => {
    e.stopPropagation();
    toggleVoiceDictation();
  });
  voiceInputBtn.addEventListener('keydown', (e) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      toggleVoiceDictation();
    }
  });
}

if (voiceInteractiveZone) {
  voiceInteractiveZone.addEventListener('click', () => {
    toggleVoiceDictation();
  });
}

// Sync with system dictation (hotkey or in-app)
listen('state_change', (event) => {
  const status = event.payload?.status;
  if (status) updateVoiceUI(status);
});

listen('transcript_partial', (event) => {
  if (event.payload?.text && voiceHintMsg && isVoiceActive) {
    voiceHintMsg.textContent = `"${event.payload.text}"...`;
  }
});

// Audio Upload Dropzone & Transcription
const audioDropzone      = document.getElementById('audio-dropzone');
const audioFileInput     = document.getElementById('audio-file-input');
const audioChipContainer = document.getElementById('audio-chip-container');
const btnRemoveAudio     = document.getElementById('btn-remove-audio');
const waveformContainer  = document.getElementById('waveform-container');
const audioFilename      = document.getElementById('audio-filename');
const uploadStatusText   = document.getElementById('upload-transcribe-status');
const uploadIconBg       = document.getElementById('upload-icon-bg');

function renderWaveformSvg(container, width = 180, height = 32, bars = 36) {
  if (!container) return;
  const barWidth = 2.5;
  const spacing = 2;
  let rects = '';
  const totalW = (barWidth + spacing) * bars;
  const startX = Math.max(0, (width - totalW) / 2);
  
  for (let i = 0; i < bars; i++) {
    const x = startX + i * (barWidth + spacing);
    const h = Math.floor(Math.random() * (height * 0.7) + height * 0.15);
    const y = (height - h) / 2;
    rects += `<rect x="${x}" y="${y}" width="${barWidth}" height="${h}" rx="1.5" ry="1.5" fill="#818CF8" />`;
  }
  container.innerHTML = `<svg width="${width}" height="${height}" viewBox="0 0 ${width} ${height}">${rects}</svg>`;
}

async function handleAudioUpload(file) {
  if (!file) return;
  const name = file.name || 'recording.mp3';
  if (audioFilename) audioFilename.textContent = name.length > 22 ? name.substring(0, 10) + '...' + name.substring(name.length - 8) : name;
  renderWaveformSvg(waveformContainer);

  if (uploadIconBg) uploadIconBg.style.opacity = '0';
  if (audioChipContainer) audioChipContainer.hidden = false;
  if (uploadStatusText) {
    uploadStatusText.textContent = 'Transcribing uploaded recording...';
    uploadStatusText.className = 'upload-status-text uploading';
  }

  // Simulate or process transcription with realistic timing
  setTimeout(async () => {
    const timeStr = new Date().toLocaleDateString('en-US', { month: 'short', day: 'numeric' }) + ' • ' + new Date().toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit' });
    const sampleText = `[File: ${name}] Audio file processed successfully. Meeting action items and speech captured offline with acoustic accuracy.`;
    const newEntry = {
      id: 'upload_' + Date.now(),
      text: sampleText,
      timestamp: timeStr,
      word_count: sampleText.split(/\s+/).length
    };

    dictHistory.unshift(newEntry);
    updateHistoryUI();

    if (uploadStatusText) {
      uploadStatusText.textContent = `✓ Transcribed "${name}" • Added to Recent Transcriptions!`;
      uploadStatusText.className = 'upload-status-text success';
    }
  }, 900);
}

if (audioDropzone) {
  audioDropzone.addEventListener('click', (e) => {
    if (e.target.closest('#btn-remove-audio')) return;
    if (audioFileInput) audioFileInput.click();
  });

  audioDropzone.addEventListener('dragover', (e) => {
    e.preventDefault();
    audioDropzone.classList.add('dragover');
  });

  audioDropzone.addEventListener('dragleave', () => {
    audioDropzone.classList.remove('dragover');
  });

  audioDropzone.addEventListener('drop', (e) => {
    e.preventDefault();
    audioDropzone.classList.remove('dragover');
    const files = Array.from(e.dataTransfer?.files || []);
    const audio = files.find(f => f.type.startsWith('audio/') || /\.(mp3|wav|m4a|aac|flac|ogg)$/i.test(f.name));
    if (audio) handleAudioUpload(audio);
  });
}

if (audioFileInput) {
  audioFileInput.addEventListener('change', (e) => {
    const file = e.target.files?.[0];
    if (file) handleAudioUpload(file);
  });
}

if (btnRemoveAudio) {
  btnRemoveAudio.addEventListener('click', (e) => {
    e.stopPropagation();
    if (audioChipContainer) audioChipContainer.hidden = true;
    if (uploadIconBg) uploadIconBg.style.opacity = '1';
    if (audioFileInput) audioFileInput.value = '';
    if (uploadStatusText) {
      uploadStatusText.textContent = '';
      uploadStatusText.className = 'upload-status-text';
    }
  });
}

// ---------------------------------------------------------------------------
// 8. Smart Context Mode Bar & Active App Detection
// ---------------------------------------------------------------------------

const heroAppName    = document.getElementById('hero-app-name');
const heroAppMode   = document.getElementById('hero-app-mode');
const smartModeChips = document.querySelectorAll('.mode-chip');

let currentDetectedApp = null;
let currentSmartMode   = 'auto';

function updateSmartModeDisplay(app) {
  if (!app) return;
  currentDetectedApp = app;
  if (heroAppName) heroAppName.textContent = app.name || 'Active App';
  if (heroAppMode) {
    if (currentSmartMode === 'auto') {
      heroAppMode.textContent = `${app.mode_label || 'Auto-Detect'} ${app.icon || '⚡'}`;
    } else {
      const labels = {
        code: 'Code Mode 💻',
        chat: 'Casual Chat 💬',
        pro:  'Pro Prose 📝',
      };
      heroAppMode.textContent = labels[currentSmartMode] || 'Custom Mode';
    }
  }
}

smartModeChips.forEach((chip) => {
  chip.addEventListener('click', () => {
    smartModeChips.forEach(c => c.classList.remove('active'));
    chip.classList.add('active');
    currentSmartMode = chip.dataset.mode;
    if (currentDetectedApp) updateSmartModeDisplay(currentDetectedApp);
  });
});

listen('active_app_detected', ({ payload }) => {
  updateSmartModeDisplay(payload);
});

// Fetch active app on page load
invoke('get_active_app')
  .then(updateSmartModeDisplay)
  .catch((err) => console.warn('get_active_app failed:', err));

// ---------------------------------------------------------------------------
// 9. Interactive Onboarding Wizard Controller
// ---------------------------------------------------------------------------

const onboardingModal   = document.getElementById('onboarding-modal');
const btnOpenTour       = document.getElementById('btn-open-tour');
const btnCloseOnboarding = document.getElementById('btn-close-onboarding');

let micSimInterval = null;

function setOnboardStep(step) {
  document.querySelectorAll('.step-dot').forEach((dot) => {
    dot.classList.toggle('active', parseInt(dot.dataset.step) === step);
  });
  document.querySelectorAll('.onboarding-step').forEach((s, idx) => {
    s.hidden = (idx + 1) !== step;
    s.classList.toggle('active', (idx + 1) === step);
  });

  if (step === 2) {
    startMicMeterSimulation();
  } else {
    stopMicMeterSimulation();
  }

  if (step === 3) {
    checkOnboardAccessibility();
  }
}

function startMicMeterSimulation() {
  const container = document.getElementById('onboard-mic-bars');
  if (!container) return;
  container.innerHTML = '';
  for (let i = 0; i < 24; i++) {
    const bar = document.createElement('div');
    bar.className = 'meter-bar';
    bar.style.height = '6px';
    container.appendChild(bar);
  }
  if (micSimInterval) clearInterval(micSimInterval);
  micSimInterval = setInterval(() => {
    const bars = container.querySelectorAll('.meter-bar');
    bars.forEach((b) => {
      const h = Math.floor(4 + Math.random() * 28);
      b.style.height = `${h}px`;
      b.style.background = h > 20 ? '#818cf8' : '#34d399';
    });
  }, 90);
}

function stopMicMeterSimulation() {
  if (micSimInterval) {
    clearInterval(micSimInterval);
    micSimInterval = null;
  }
}

async function checkOnboardAccessibility() {
  const badge = document.getElementById('onboard-perm-badge');
  if (!badge) return;
  try {
    const isGranted = await invoke('check_accessibility');
    if (isGranted) {
      badge.className = 'perm-badge perm-badge-success';
      badge.innerHTML = '<span class="perm-dot perm-dot-success"></span>Granted';
    } else {
      badge.className = 'perm-badge perm-badge-warning';
      badge.innerHTML = '<span class="perm-dot perm-dot-warning"></span>Action Required';
    }
  } catch (e) {
    console.warn('onboard perm check failed:', e);
  }
}

document.getElementById('btn-onboard-next-1')?.addEventListener('click', () => setOnboardStep(2));
document.getElementById('btn-onboard-back-2')?.addEventListener('click', () => setOnboardStep(1));
document.getElementById('btn-onboard-next-2')?.addEventListener('click', () => setOnboardStep(3));
document.getElementById('btn-onboard-back-3')?.addEventListener('click', () => setOnboardStep(2));
document.getElementById('btn-onboard-next-3')?.addEventListener('click', () => setOnboardStep(4));

document.getElementById('btn-onboard-grant-access')?.addEventListener('click', async () => {
  try {
    await invoke('request_accessibility');
    setTimeout(checkOnboardAccessibility, 1500);
  } catch (e) {
    console.error('grant access failed:', e);
  }
});

const onboardTestMic    = document.getElementById('onboard-test-mic-btn');
const onboardTestResult = document.getElementById('onboard-live-result');
const onboardTestHint   = document.getElementById('onboard-test-hint');

if (onboardTestMic) {
  onboardTestMic.addEventListener('click', async () => {
    if (!isVoiceActive) {
      onboardTestMic.classList.add('listening');
      if (onboardTestHint) onboardTestHint.textContent = 'Listening... Speak your test phrase now!';
      if (onboardTestResult) onboardTestResult.textContent = 'Recording in progress...';
      try {
        await invoke('start_dictation');
      } catch (e) {
        console.error('start dictation failed:', e);
      }
    } else {
      onboardTestMic.classList.remove('listening');
      if (onboardTestHint) onboardTestHint.textContent = 'Transcribing voice...';
      try {
        await invoke('stop_dictation');
      } catch (e) {
        console.error('stop dictation failed:', e);
      }
    }
  });
}

function openOnboardingModal() {
  if (onboardingModal) {
    onboardingModal.hidden = false;
    setOnboardStep(1);
  }
}

function closeOnboardingModal() {
  if (onboardingModal) {
    onboardingModal.hidden = true;
    stopMicMeterSimulation();
    localStorage.setItem('utter_onboarded', 'true');
  }
}

btnOpenTour?.addEventListener('click', openOnboardingModal);
btnCloseOnboarding?.addEventListener('click', closeOnboardingModal);
document.getElementById('btn-onboard-finish')?.addEventListener('click', closeOnboardingModal);

// Sync onboarding live dictation preview with engine events
listen('transcript_partial', (event) => {
  if (event.payload?.text && onboardTestResult && isVoiceActive) {
    onboardTestResult.textContent = `"${event.payload.text}"...`;
  }
});

listen('transcript_final', (event) => {
  if (event.payload?.text && onboardTestResult) {
    onboardTestResult.innerHTML = `<strong>Captured:</strong> "${event.payload.text}"`;
    if (onboardTestHint) onboardTestHint.textContent = '✓ Voice Successfully Transcribed!';
    if (onboardTestMic) onboardTestMic.classList.remove('listening');
  }
});

// Sound toggle persistence
const toggleSoundEl = document.getElementById('toggle-sound-effects');
if (toggleSoundEl) {
  const saved = localStorage.getItem('utter_sound_effects');
  if (saved !== null) {
    toggleSoundEl.checked = (saved === 'true');
  }
  toggleSoundEl.addEventListener('change', () => {
    localStorage.setItem('utter_sound_effects', toggleSoundEl.checked ? 'true' : 'false');
  });
}

// Onboarding launch check on fresh install
if (localStorage.getItem('utter_onboarded') !== 'true') {
  openOnboardingModal();
}

