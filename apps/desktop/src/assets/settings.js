/**
 * settings.js — Settings panel logic.
 * Reads settings via get_settings IPC, writes via set_settings.
 * Handles cleanup backend field show/hide and test_cleanup.
 */

const { invoke } = window.__TAURI__?.core ?? mockTauriCore();

// ---------------------------------------------------------------------------
// Element refs
// ---------------------------------------------------------------------------
const hotkeyInput       = document.getElementById('hotkey-input');
const audioSelect       = document.getElementById('audio-device-select');
const modelSelect       = document.getElementById('model-select');
const cleanupType       = document.getElementById('cleanup-type');
const ollamaLocalUrl    = document.getElementById('ollama-local-url');
const ollamaLocalModel  = document.getElementById('ollama-local-model');
const ollamaRemoteUrl   = document.getElementById('ollama-remote-url');
const ollamaRemoteModel = document.getElementById('ollama-remote-model');
const ollamaRemoteToken = document.getElementById('ollama-remote-token');
const oaiUrl            = document.getElementById('oai-url');
const oaiModel          = document.getElementById('oai-model');
const oaiKey            = document.getElementById('oai-key');
const testBtn           = document.getElementById('cleanup-test-btn');
const testResult        = document.getElementById('cleanup-test-result');
const cleanupTestRow    = document.getElementById('cleanup-test-row');
const saveBtn           = document.getElementById('save-btn');
const cancelBtn         = document.getElementById('cancel-btn');
const saveStatus        = document.getElementById('save-status');
const aboutVersion      = document.getElementById('about-version');

// ---------------------------------------------------------------------------
// Cleanup field visibility
// ---------------------------------------------------------------------------

const CLEANUP_FIELDS = ['LocalOllama', 'RemoteOllama', 'OpenAiCompat'];

function updateCleanupFields() {
  const val = cleanupType.value;
  CLEANUP_FIELDS.forEach(type => {
    const el = document.getElementById(`fields-${type}`);
    if (el) el.hidden = (val !== type);
  });
  cleanupTestRow.hidden = (val === 'None');
}

cleanupType.addEventListener('change', updateCleanupFields);

// ---------------------------------------------------------------------------
// Load settings from backend
// ---------------------------------------------------------------------------

async function loadSettings() {
  let settings;
  try {
    settings = await invoke('get_settings');
  } catch (e) {
    console.error('get_settings failed:', e);
    return;
  }

  hotkeyInput.value = settings.hotkey ?? 'CmdOrCtrl+Shift+D';

  // Audio devices
  try {
    const devices = await invoke('list_audio_devices');
    devices.forEach(name => {
      const opt = document.createElement('option');
      opt.value = name;
      opt.textContent = name;
      if (name === settings.audio_device) opt.selected = true;
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
      opt.textContent = `${m.name} (${formatBytes(m.size_bytes)})`;
      if (m.path === settings.model_path) opt.selected = true;
      modelSelect.appendChild(opt);
    });
  } catch (e) {
    console.warn('list_models failed:', e);
  }

  // Cleanup config
  const cleanup = settings.cleanup ?? { type: 'None' };
  cleanupType.value = cleanup.type ?? 'None';

  if (cleanup.type === 'LocalOllama') {
    ollamaLocalUrl.value   = cleanup.url   ?? 'http://localhost:11434';
    ollamaLocalModel.value = cleanup.model ?? '';
  } else if (cleanup.type === 'RemoteOllama') {
    ollamaRemoteUrl.value   = cleanup.url   ?? '';
    ollamaRemoteModel.value = cleanup.model ?? '';
    // Don't pre-fill password fields — they come from keychain
  } else if (cleanup.type === 'OpenAiCompat') {
    oaiUrl.value   = cleanup.url   ?? '';
    oaiModel.value = cleanup.model ?? '';
    // Don't pre-fill API key
  }

  updateCleanupFields();
}

// ---------------------------------------------------------------------------
// Build settings object from form
// ---------------------------------------------------------------------------

function buildSettings() {
  const cleanupTypeVal = cleanupType.value;
  let cleanup;

  if (cleanupTypeVal === 'None') {
    cleanup = { type: 'None' };
  } else if (cleanupTypeVal === 'LocalOllama') {
    cleanup = {
      type:  'LocalOllama',
      url:   ollamaLocalUrl.value.trim()   || 'http://localhost:11434',
      model: ollamaLocalModel.value.trim() || 'llama3',
    };
  } else if (cleanupTypeVal === 'RemoteOllama') {
    cleanup = {
      type:             'RemoteOllama',
      url:              ollamaRemoteUrl.value.trim(),
      model:            ollamaRemoteModel.value.trim(),
      keychain_account: 'cleanup_remote_ollama_token',
    };
  } else if (cleanupTypeVal === 'OpenAiCompat') {
    cleanup = {
      type:             'OpenAiCompat',
      url:              oaiUrl.value.trim(),
      model:            oaiModel.value.trim(),
      keychain_account: 'cleanup_openai_compat_key',
    };
  }

  return {
    hotkey:       hotkeyInput.value.trim() || 'CmdOrCtrl+Shift+D',
    model_path:   modelSelect.value,
    audio_device: audioSelect.value === 'System default' ? '' : audioSelect.value,
    cleanup,
  };
}

// ---------------------------------------------------------------------------
// Test cleanup connection
// ---------------------------------------------------------------------------

testBtn.addEventListener('click', async () => {
  testBtn.disabled = true;
  testResult.textContent = 'Testing…';
  testResult.className = 'test-result';

  try {
    const settings = buildSettings();
    const result = await invoke('test_cleanup', { settings });
    testResult.textContent =
      `✓ "${result.output}" (${result.latency_ms}ms)`;
    testResult.className = 'test-result success';
  } catch (e) {
    testResult.textContent = `✗ ${e?.message ?? e?.code ?? 'Connection failed'}`;
    testResult.className = 'test-result error';
  } finally {
    testBtn.disabled = false;
  }
});

// ---------------------------------------------------------------------------
// Save / Cancel
// ---------------------------------------------------------------------------

saveBtn.addEventListener('click', async () => {
  saveStatus.textContent = 'Saving…';
  saveStatus.className = 'save-status';

  try {
    const settings = buildSettings();
    await invoke('set_settings', { newSettings: settings });
    saveStatus.textContent = '✓ Saved';
    saveStatus.className = 'save-status success';
    setTimeout(() => { saveStatus.textContent = ''; }, 3000);
  } catch (e) {
    saveStatus.textContent = `✗ ${e?.message ?? 'Save failed'}`;
    saveStatus.className = 'save-status error';
  }
});

cancelBtn.addEventListener('click', () => {
  // Re-load from disk to discard unsaved changes
  loadSettings().catch(console.error);
  saveStatus.textContent = '';
});

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function formatBytes(bytes) {
  if (!bytes) return '?';
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

// ---------------------------------------------------------------------------
// Init
// ---------------------------------------------------------------------------

loadSettings().catch(console.error);

function mockTauriCore() {
  return {
    invoke: async (cmd, args) => {
      console.log(`[mock] invoke ${cmd}:`, args);
      if (cmd === 'get_settings') {
        return {
          hotkey: 'CmdOrCtrl+Shift+D',
          model_path: '',
          audio_device: '',
          cleanup: { type: 'None' },
        };
      }
      if (cmd === 'list_audio_devices') return ['Default Microphone'];
      if (cmd === 'list_models') return [];
      if (cmd === 'test_cleanup') return { input: 'test', output: 'test', latency_ms: 0 };
      return null;
    },
  };
}
