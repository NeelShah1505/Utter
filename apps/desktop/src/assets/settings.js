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
const boxLocalOllama    = document.getElementById('box-LocalOllama');
const boxOpenAiCompat   = document.getElementById('box-OpenAiCompat');
const ollamaLocalUrl    = document.getElementById('ollama-local-url');
const ollamaLocalModel  = document.getElementById('ollama-local-model');
const oaiUrl            = document.getElementById('oai-url');
const oaiModel          = document.getElementById('oai-model');
const oaiKey            = document.getElementById('oai-key');
const boxCustomPrompt   = document.getElementById('box-CustomPrompt');
const aiCustomPrompt    = document.getElementById('ai-custom-prompt');
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
  if (boxLocalOllama) boxLocalOllama.hidden = (type !== 'LocalOllama');
  if (boxOpenAiCompat) boxOpenAiCompat.hidden = (type !== 'OpenAiCompat');
  if (boxCustomPrompt) boxCustomPrompt.hidden = (type === 'None');
  if (aiStatusNotice) aiStatusNotice.hidden = (type !== 'None');
}

engineRadios.forEach(radio => {
  radio.addEventListener('change', () => updateAiEngineVisibility(radio.value));
});

document.querySelectorAll('.btn-preset').forEach(btn => {
  btn.addEventListener('click', () => {
    if (oaiUrl) oaiUrl.value = btn.getAttribute('data-url');
    if (oaiModel) oaiModel.value = btn.getAttribute('data-model');
  });
});

const STYLE_PROMPTS = {
  email: "Format this voice dictation into a polished, professional email with polite greetings and clear structure.",
  chat: "Format this voice dictation into concise, direct chat messaging bullet points without fluff or salutations.",
  developer: "Format this dictation for a software engineer, preserving code variable names (camelCase, snake_case), technical lingo, and terminal commands."
};

document.querySelectorAll('.style-card').forEach(card => {
  card.addEventListener('click', () => {
    const styleKey = card.getAttribute('data-style');
    const promptText = STYLE_PROMPTS[styleKey] || '';
    
    document.querySelectorAll('.style-card').forEach(c => {
      c.classList.remove('active-style');
      const badge = c.querySelector('.badge-status');
      if (badge) {
        badge.classList.remove('enabled');
        badge.textContent = 'Optional';
      }
    });

    card.classList.add('active-style');
    const badge = card.querySelector('.badge-status');
    if (badge) {
      badge.classList.add('enabled');
      badge.textContent = 'Active';
    }

    if (aiCustomPrompt) {
      aiCustomPrompt.value = promptText;
    }
    saveSettingsSilently();
  });
});

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
    aiCustomPrompt.value   = '';
  }

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
    testBtn.disabled = true;
    testResult.textContent = 'Connecting to AI model…';
    testResult.className = 'test-result-msg';

    try {
      const settings = buildSettingsObject();
      if (settings.cleanup.type === 'OpenAiCompat' && oaiKey && oaiKey.value.trim()) {
        await invoke('set_keychain_secret', { 
          account: settings.cleanup.keychain_account, 
          secret: oaiKey.value.trim() 
        });
      }
      const result = await invoke('test_cleanup', { settings });
      testResult.textContent = `✓ Model Responded ("${result.output}") • Latency: ${result.latency_ms}ms`;
      testResult.className = 'test-result-msg success';
    } catch (e) {
      testResult.textContent = `✗ Connection Failed: ${e?.message ?? e ?? 'Ensure model is running'}`;
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
    if (isGranted) {
      accessBadge.textContent = 'Granted';
      accessBadge.className = 'badge-status';
      accessBadge.style.background = 'rgba(34, 197, 94, 0.15)';
      accessBadge.style.color = '#22c55e';
    } else {
      accessBadge.textContent = 'Action Required';
      accessBadge.className = 'badge-status';
      accessBadge.style.background = 'rgba(234, 179, 8, 0.15)';
      accessBadge.style.color = '#eab308';
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

updateAccessibilityUI();
window.addEventListener('focus', () => {
  updateAccessibilityUI();
  loadHistoryFromBackend();
});
