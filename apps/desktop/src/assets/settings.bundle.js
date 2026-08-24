(() => {
  var __getOwnPropNames = Object.getOwnPropertyNames;
  var __esm = (fn, res, err) => function __init() {
    if (err) throw err[0];
    try {
      return fn && (res = (0, fn[__getOwnPropNames(fn)[0]])(fn = 0)), res;
    } catch (e) {
      throw err = [e], e;
    }
  };
  var __commonJS = (cb, mod) => function __require() {
    try {
      return mod || (0, cb[__getOwnPropNames(cb)[0]])((mod = { exports: {} }).exports, mod), mod.exports;
    } catch (e) {
      throw mod = 0, e;
    }
  };

  // node_modules/@tauri-apps/api/external/tslib/tslib.es6.js
  function __classPrivateFieldGet(receiver, state, kind, f) {
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a getter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot read private member from an object whose class did not declare it");
    return kind === "m" ? f : kind === "a" ? f.call(receiver) : f ? f.value : state.get(receiver);
  }
  function __classPrivateFieldSet(receiver, state, value, kind, f) {
    if (kind === "m") throw new TypeError("Private method is not writable");
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a setter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot write private member to an object whose class did not declare it");
    return kind === "a" ? f.call(receiver, value) : f ? f.value = value : state.set(receiver, value), value;
  }
  var init_tslib_es6 = __esm({
    "node_modules/@tauri-apps/api/external/tslib/tslib.es6.js"() {
    }
  });

  // node_modules/@tauri-apps/api/core.js
  function transformCallback(callback, once = false) {
    return window.__TAURI_INTERNALS__.transformCallback(callback, once);
  }
  async function invoke(cmd, args = {}, options) {
    return window.__TAURI_INTERNALS__.invoke(cmd, args, options);
  }
  var _Channel_onmessage, _Channel_nextMessageIndex, _Channel_pendingMessages, _Channel_messageEndIndex, _Resource_rid, SERIALIZE_TO_IPC_FN, Channel;
  var init_core = __esm({
    "node_modules/@tauri-apps/api/core.js"() {
      init_tslib_es6();
      SERIALIZE_TO_IPC_FN = "__TAURI_TO_IPC_KEY__";
      Channel = class {
        constructor(onmessage) {
          _Channel_onmessage.set(this, void 0);
          _Channel_nextMessageIndex.set(this, 0);
          _Channel_pendingMessages.set(this, []);
          _Channel_messageEndIndex.set(this, void 0);
          __classPrivateFieldSet(this, _Channel_onmessage, onmessage || (() => {
          }), "f");
          this.id = transformCallback((rawMessage) => {
            const index = rawMessage.index;
            if ("end" in rawMessage) {
              if (index == __classPrivateFieldGet(this, _Channel_nextMessageIndex, "f")) {
                this.cleanupCallback();
              } else {
                __classPrivateFieldSet(this, _Channel_messageEndIndex, index, "f");
              }
              return;
            }
            const message = rawMessage.message;
            if (index == __classPrivateFieldGet(this, _Channel_nextMessageIndex, "f")) {
              __classPrivateFieldGet(this, _Channel_onmessage, "f").call(this, message);
              __classPrivateFieldSet(this, _Channel_nextMessageIndex, __classPrivateFieldGet(this, _Channel_nextMessageIndex, "f") + 1, "f");
              while (__classPrivateFieldGet(this, _Channel_nextMessageIndex, "f") in __classPrivateFieldGet(this, _Channel_pendingMessages, "f")) {
                const message2 = __classPrivateFieldGet(this, _Channel_pendingMessages, "f")[__classPrivateFieldGet(this, _Channel_nextMessageIndex, "f")];
                __classPrivateFieldGet(this, _Channel_onmessage, "f").call(this, message2);
                delete __classPrivateFieldGet(this, _Channel_pendingMessages, "f")[__classPrivateFieldGet(this, _Channel_nextMessageIndex, "f")];
                __classPrivateFieldSet(this, _Channel_nextMessageIndex, __classPrivateFieldGet(this, _Channel_nextMessageIndex, "f") + 1, "f");
              }
              if (__classPrivateFieldGet(this, _Channel_nextMessageIndex, "f") === __classPrivateFieldGet(this, _Channel_messageEndIndex, "f")) {
                this.cleanupCallback();
              }
            } else {
              __classPrivateFieldGet(this, _Channel_pendingMessages, "f")[index] = message;
            }
          });
        }
        cleanupCallback() {
          window.__TAURI_INTERNALS__.unregisterCallback(this.id);
        }
        set onmessage(handler) {
          __classPrivateFieldSet(this, _Channel_onmessage, handler, "f");
        }
        get onmessage() {
          return __classPrivateFieldGet(this, _Channel_onmessage, "f");
        }
        [(_Channel_onmessage = /* @__PURE__ */ new WeakMap(), _Channel_nextMessageIndex = /* @__PURE__ */ new WeakMap(), _Channel_pendingMessages = /* @__PURE__ */ new WeakMap(), _Channel_messageEndIndex = /* @__PURE__ */ new WeakMap(), SERIALIZE_TO_IPC_FN)]() {
          return `__CHANNEL__:${this.id}`;
        }
        toJSON() {
          return this[SERIALIZE_TO_IPC_FN]();
        }
      };
      _Resource_rid = /* @__PURE__ */ new WeakMap();
    }
  });

  // node_modules/@tauri-apps/api/event.js
  async function _unlisten(event, eventId) {
    window.__TAURI_EVENT_PLUGIN_INTERNALS__.unregisterListener(event, eventId);
    await invoke("plugin:event|unlisten", {
      event,
      eventId
    });
  }
  async function listen(event, handler, options) {
    var _a;
    const target = typeof (options === null || options === void 0 ? void 0 : options.target) === "string" ? { kind: "AnyLabel", label: options.target } : (_a = options === null || options === void 0 ? void 0 : options.target) !== null && _a !== void 0 ? _a : { kind: "Any" };
    return invoke("plugin:event|listen", {
      event,
      target,
      handler: transformCallback(handler)
    }).then((eventId) => {
      return async () => _unlisten(event, eventId);
    });
  }
  var TauriEvent;
  var init_event = __esm({
    "node_modules/@tauri-apps/api/event.js"() {
      init_core();
      (function(TauriEvent2) {
        TauriEvent2["WINDOW_RESIZED"] = "tauri://resize";
        TauriEvent2["WINDOW_MOVED"] = "tauri://move";
        TauriEvent2["WINDOW_CLOSE_REQUESTED"] = "tauri://close-requested";
        TauriEvent2["WINDOW_DESTROYED"] = "tauri://destroyed";
        TauriEvent2["WINDOW_FOCUS"] = "tauri://focus";
        TauriEvent2["WINDOW_BLUR"] = "tauri://blur";
        TauriEvent2["WINDOW_SCALE_FACTOR_CHANGED"] = "tauri://scale-change";
        TauriEvent2["WINDOW_THEME_CHANGED"] = "tauri://theme-changed";
        TauriEvent2["WINDOW_CREATED"] = "tauri://window-created";
        TauriEvent2["WINDOW_SUSPENDED"] = "tauri://suspended";
        TauriEvent2["WINDOW_RESUMED"] = "tauri://resumed";
        TauriEvent2["WEBVIEW_CREATED"] = "tauri://webview-created";
        TauriEvent2["DRAG_ENTER"] = "tauri://drag-enter";
        TauriEvent2["DRAG_OVER"] = "tauri://drag-over";
        TauriEvent2["DRAG_DROP"] = "tauri://drag-drop";
        TauriEvent2["DRAG_LEAVE"] = "tauri://drag-leave";
      })(TauriEvent || (TauriEvent = {}));
    }
  });

  // src/assets/settings.js
  var require_settings = __commonJS({
    "src/assets/settings.js"() {
      init_core();
      init_event();
      var currentSettings = {
        hotkey: "CmdOrCtrl+Shift+D",
        model_path: "",
        audio_device: "",
        cleanup: { type: "None" },
        dictionary: []
      };
      var dictHistory = JSON.parse(localStorage.getItem("utter_history") || "[]");
      var navItems = document.querySelectorAll(".nav-item");
      var viewPanels = document.querySelectorAll(".view-panel");
      var hotkeyInput = document.getElementById("hotkey-input");
      var audioSelect = document.getElementById("audio-device-select");
      var modelSelect = document.getElementById("model-select");
      var engineRadios = document.querySelectorAll('input[name="engine-type"]');
      var boxLocalOllama = document.getElementById("box-LocalOllama");
      var boxOpenAiCompat = document.getElementById("box-OpenAiCompat");
      var ollamaLocalUrl = document.getElementById("ollama-local-url");
      var ollamaLocalModel = document.getElementById("ollama-local-model");
      var oaiUrl = document.getElementById("oai-url");
      var oaiModel = document.getElementById("oai-model");
      var oaiKey = document.getElementById("oai-key");
      var boxCustomPrompt = document.getElementById("box-CustomPrompt");
      var aiCustomPrompt = document.getElementById("ai-custom-prompt");
      var testBtn = document.getElementById("cleanup-test-btn");
      var testResult = document.getElementById("cleanup-test-result");
      var dictTbody = document.getElementById("dictionary-tbody");
      var btnShowAddModal = document.getElementById("btn-show-add-modal");
      var addDictBox = document.getElementById("add-dict-box");
      var btnSaveWord = document.getElementById("btn-save-word");
      var btnCancelWord = document.getElementById("btn-cancel-word");
      var newWordInput = document.getElementById("new-word-input");
      var newReplaceInput = document.getElementById("new-replace-input");
      var dictSearchInput = document.getElementById("dict-search-input");
      var saveBtn = document.getElementById("save-btn");
      var cancelBtn = document.getElementById("cancel-btn");
      var saveStatus = document.getElementById("save-status");
      function switchTab(tabId) {
        navItems.forEach((btn) => {
          btn.classList.toggle("active", btn.getAttribute("data-tab") === tabId);
        });
        viewPanels.forEach((panel) => {
          panel.classList.toggle("active", panel.id === `view-${tabId}`);
        });
      }
      navItems.forEach((btn) => {
        btn.addEventListener("click", () => switchTab(btn.getAttribute("data-tab")));
      });
      document.getElementById("btn-explore-dict")?.addEventListener("click", () => {
        switchTab("dictionary");
      });
      listen("navigate_tab", (event) => {
        if (event.payload) switchTab(event.payload);
      });
      var aiStatusNotice = document.getElementById("ai-status-notice");
      var btnGotoAiTab = document.getElementById("btn-goto-ai-tab");
      if (btnGotoAiTab) {
        btnGotoAiTab.addEventListener("click", () => switchTab("ai"));
      }
      function updateAiEngineVisibility(type) {
        if (boxLocalOllama) boxLocalOllama.hidden = type !== "LocalOllama";
        if (boxOpenAiCompat) boxOpenAiCompat.hidden = type !== "OpenAiCompat";
        if (boxCustomPrompt) boxCustomPrompt.hidden = type === "None";
        if (aiStatusNotice) aiStatusNotice.hidden = type !== "None";
      }
      engineRadios.forEach((radio) => {
        radio.addEventListener("change", () => updateAiEngineVisibility(radio.value));
      });
      document.querySelectorAll(".btn-preset").forEach((btn) => {
        btn.addEventListener("click", () => {
          if (oaiUrl) oaiUrl.value = btn.getAttribute("data-url");
          if (oaiModel) oaiModel.value = btn.getAttribute("data-model");
        });
      });
      var STYLE_PROMPTS = {
        email: "Format this voice dictation into a polished, professional email with polite greetings and clear structure.",
        chat: "Format this voice dictation into concise, direct chat messaging bullet points without fluff or salutations.",
        developer: "Format this dictation for a software engineer, preserving code variable names (camelCase, snake_case), technical lingo, and terminal commands."
      };
      document.querySelectorAll(".style-card").forEach((card) => {
        card.addEventListener("click", () => {
          const styleKey = card.getAttribute("data-style");
          const promptText = STYLE_PROMPTS[styleKey] || "";
          document.querySelectorAll(".style-card").forEach((c) => {
            c.classList.remove("active-style");
            const badge2 = c.querySelector(".badge-status");
            if (badge2) {
              badge2.classList.remove("enabled");
              badge2.textContent = "Optional";
            }
          });
          card.classList.add("active-style");
          const badge = card.querySelector(".badge-status");
          if (badge) {
            badge.classList.add("enabled");
            badge.textContent = "Active";
          }
          if (aiCustomPrompt) {
            aiCustomPrompt.value = promptText;
          }
          saveSettingsSilently();
        });
      });
      function renderDictionary() {
        if (!dictTbody) return;
        dictTbody.innerHTML = "";
        const query = dictSearchInput ? dictSearchInput.value.toLowerCase() : "";
        const items = currentSettings.dictionary || [];
        items.filter((item) => {
          return item.word.toLowerCase().includes(query) || item.replacement && item.replacement.toLowerCase().includes(query);
        }).forEach((item, index) => {
          const tr = document.createElement("tr");
          const tdWord = document.createElement("td");
          tdWord.innerHTML = `<strong>${escapeHtml(item.word)}</strong>`;
          const tdReplace = document.createElement("td");
          tdReplace.textContent = item.replacement || "(Recognize exact word)";
          if (!item.replacement) tdReplace.style.color = "#94A0B8";
          const tdActions = document.createElement("td");
          tdActions.className = "col-actions";
          const delBtn = document.createElement("button");
          delBtn.className = "btn-delete-icon";
          delBtn.innerHTML = "\u{1F5D1}\uFE0F";
          delBtn.title = "Remove word";
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
        dictSearchInput.addEventListener("input", renderDictionary);
      }
      if (btnShowAddModal) {
        btnShowAddModal.addEventListener("click", () => {
          addDictBox.hidden = false;
          newWordInput.focus();
        });
      }
      if (btnCancelWord) {
        btnCancelWord.addEventListener("click", () => {
          addDictBox.hidden = true;
          newWordInput.value = "";
          newReplaceInput.value = "";
        });
      }
      if (btnSaveWord) {
        btnSaveWord.addEventListener("click", () => {
          const word = newWordInput.value.trim();
          const replacement = newReplaceInput.value.trim();
          if (!word) return;
          if (!currentSettings.dictionary) currentSettings.dictionary = [];
          currentSettings.dictionary.push({
            word,
            replacement: replacement ? replacement : null
          });
          addDictBox.hidden = true;
          newWordInput.value = "";
          newReplaceInput.value = "";
          renderDictionary();
          saveSettingsSilently();
        });
      }
      function escapeHtml(str) {
        return str.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
      }
      async function loadHistoryFromBackend() {
        try {
          const entries = await invoke("get_history");
          if (Array.isArray(entries)) {
            dictHistory = entries;
            updateHistoryUI();
          }
        } catch (e) {
          console.error("get_history failed:", e);
        }
      }
      function updateHistoryUI() {
        const container = document.getElementById("history-list");
        if (!container) return;
        container.innerHTML = "";
        const statWords = document.getElementById("stat-words");
        const statWpm = document.getElementById("stat-wpm");
        const statStreak = document.getElementById("stat-streak");
        if (dictHistory.length === 0) {
          container.innerHTML = `
      <div id="history-empty" class="empty-state-card">
        <div class="empty-icon"><svg viewBox="0 0 24 24" width="32" height="32" stroke="currentColor" stroke-width="2" fill="none"><path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z"></path><path d="M19 10v2a7 7 0 0 1-14 0v-2"></path><line x1="12" y1="19" x2="12" y2="22"></line></svg></div>
        <h4>No dictations yet</h4>
        <p>Trigger dictation with Cmd+Shift+D anywhere on your Mac. Your real speech-to-text logs will appear here instantly with quick copy controls.</p>
      </div>
    `;
          if (statWords) statWords.textContent = "0";
          if (statWpm) statWpm.innerHTML = "0 <small>wpm</small>";
          if (statStreak) statStreak.innerHTML = "0 <small>days</small>";
          return;
        }
        let totalWords = 0;
        dictHistory.forEach((entry, idx) => {
          const wordCount = entry.word_count || entry.text.split(/\s+/).filter((w) => w.length > 0).length;
          totalWords += wordCount;
          const card = document.createElement("div");
          card.className = "history-card";
          card.innerHTML = `
      <div class="history-meta">
        <span class="history-time">${entry.timestamp || "Just now"}</span>
        <span class="history-app"><svg viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2" fill="none" style="vertical-align: middle; margin-right: 3px;"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon></svg> Real-time Streamed</span>
      </div>
      <p class="history-text" id="hist-text-${idx}">${escapeHtml(entry.text)}</p>
      <div class="history-actions">
        <button class="btn-action-sm btn-copy-sm" data-idx="${idx}">Copy</button>
        <button class="btn-action-sm btn-edit-sm" data-idx="${idx}">Edit</button>
        <button class="btn-action-sm btn-action-danger btn-del-sm" data-idx="${idx}">Delete</button>
      </div>
    `;
          const copyBtn = card.querySelector(".btn-copy-sm");
          copyBtn.onclick = () => {
            navigator.clipboard.writeText(entry.text);
            copyBtn.textContent = "\u2713 Copied";
            setTimeout(() => {
              copyBtn.textContent = "Copy";
            }, 2e3);
          };
          const delBtn = card.querySelector(".btn-del-sm");
          delBtn.onclick = async () => {
            try {
              dictHistory = await invoke("delete_history_item", { id: entry.id });
            } catch (e) {
              dictHistory.splice(idx, 1);
            }
            updateHistoryUI();
          };
          const editBtn = card.querySelector(".btn-edit-sm");
          editBtn.onclick = async () => {
            const newText = prompt("Edit transcript:", entry.text);
            if (newText !== null && newText.trim() !== "") {
              try {
                dictHistory = await invoke("update_history_item", { id: entry.id, newText: newText.trim() });
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
      document.getElementById("btn-clear-history")?.addEventListener("click", async () => {
        try {
          await invoke("clear_history");
        } catch (e) {
          console.error("clear_history failed:", e);
        }
        dictHistory = [];
        updateHistoryUI();
      });
      listen("transcript_final", (event) => {
        if (event.payload) {
          if (event.payload.entry) {
            dictHistory = dictHistory.filter((e) => e.id !== event.payload.entry.id);
            dictHistory.unshift(event.payload.entry);
          } else if (event.payload.text) {
            const timeStr = (/* @__PURE__ */ new Date()).toLocaleDateString("en-US", { month: "short", day: "numeric" }) + " \u2022 " + (/* @__PURE__ */ new Date()).toLocaleTimeString("en-US", { hour: "numeric", minute: "2-digit" });
            dictHistory.unshift({ id: "hist_" + Date.now(), text: event.payload.text, timestamp: timeStr, word_count: event.payload.text.split(/\s+/).length });
          }
          updateHistoryUI();
        }
      });
      async function loadSettings() {
        try {
          currentSettings = await invoke("get_settings");
          console.log("Loaded settings:", currentSettings);
        } catch (e) {
          console.error("get_settings failed:", e);
          return;
        }
        hotkeyInput.value = currentSettings.hotkey ?? "CmdOrCtrl+Shift+D";
        try {
          const devices = await invoke("list_audio_devices");
          devices.forEach((name) => {
            const opt = document.createElement("option");
            opt.value = name;
            opt.textContent = name;
            if (name === currentSettings.audio_device) opt.selected = true;
            audioSelect.appendChild(opt);
          });
        } catch (e) {
          console.warn("list_audio_devices failed:", e);
        }
        try {
          const models = await invoke("list_models");
          models.forEach((m) => {
            const opt = document.createElement("option");
            opt.value = m.path;
            opt.textContent = `${m.name}`;
            if (m.path === currentSettings.model_path) opt.selected = true;
            modelSelect.appendChild(opt);
          });
        } catch (e) {
          console.warn("list_models failed:", e);
        }
        const cleanup = currentSettings.cleanup ?? { type: "None" };
        const engineType = cleanup.type ?? "None";
        const radio = document.querySelector(`input[name="engine-type"][value="${engineType}"]`);
        if (radio) radio.checked = true;
        if (engineType === "LocalOllama") {
          ollamaLocalUrl.value = cleanup.url ?? "http://localhost:11434";
          ollamaLocalModel.value = cleanup.model ?? "llama3";
          aiCustomPrompt.value = cleanup.prompt ?? "";
        } else if (engineType === "OpenAiCompat") {
          oaiUrl.value = cleanup.url ?? "https://api.openai.com";
          oaiModel.value = cleanup.model ?? "gpt-4o-mini";
          aiCustomPrompt.value = cleanup.prompt ?? "";
        } else {
          aiCustomPrompt.value = "";
        }
        updateAiEngineVisibility(engineType);
        renderDictionary();
        updateHistoryUI();
      }
      function buildSettingsObject() {
        const selectedEngine = document.querySelector('input[name="engine-type"]:checked')?.value || "None";
        let cleanup;
        if (selectedEngine === "None") {
          cleanup = { type: "None" };
        } else if (selectedEngine === "LocalOllama") {
          cleanup = {
            type: "LocalOllama",
            url: ollamaLocalUrl.value.trim() || "http://localhost:11434",
            model: ollamaLocalModel.value.trim() || "llama3",
            prompt: aiCustomPrompt.value.trim() || null
          };
        } else if (selectedEngine === "OpenAiCompat") {
          cleanup = {
            type: "OpenAiCompat",
            url: oaiUrl.value.trim() || "https://api.openai.com",
            model: oaiModel.value.trim() || "gpt-4o-mini",
            keychain_account: "cleanup_openai_compat_key",
            prompt: aiCustomPrompt.value.trim() || null
          };
        }
        return {
          hotkey: hotkeyInput.value.trim() || "CmdOrCtrl+Shift+D",
          model_path: modelSelect.value || "",
          audio_device: audioSelect.value === "System Default Microphone" ? "" : audioSelect.value,
          cleanup,
          dictionary: currentSettings.dictionary || []
        };
      }
      async function saveSettingsSilently() {
        try {
          const updated = buildSettingsObject();
          await invoke("set_settings", { newSettings: updated });
          currentSettings = updated;
        } catch (e) {
          console.error("Silent save failed:", e);
        }
      }
      if (saveBtn) {
        saveBtn.addEventListener("click", async () => {
          saveStatus.textContent = "Saving configuration\u2026";
          saveStatus.className = "save-status-msg";
          try {
            const updated = buildSettingsObject();
            await invoke("set_settings", { newSettings: updated });
            currentSettings = updated;
            if (updated.cleanup.type === "OpenAiCompat" && oaiKey && oaiKey.value.trim()) {
              await invoke("set_keychain_secret", {
                account: updated.cleanup.keychain_account,
                secret: oaiKey.value.trim()
              });
            }
            saveStatus.textContent = "\u2713 Studio configuration saved successfully!";
            saveStatus.className = "save-status-msg success";
            setTimeout(() => {
              saveStatus.textContent = "";
            }, 3500);
          } catch (e) {
            saveStatus.textContent = `\u2717 Save failed: ${e?.message ?? "Unknown error"}`;
            saveStatus.className = "save-status-msg error";
          }
        });
      }
      if (cancelBtn) {
        cancelBtn.addEventListener("click", () => {
          loadSettings().catch(console.error);
          saveStatus.textContent = "Reverted to saved state.";
          saveStatus.className = "save-status-msg";
          setTimeout(() => {
            saveStatus.textContent = "";
          }, 2500);
        });
      }
      if (testBtn) {
        testBtn.addEventListener("click", async () => {
          testBtn.disabled = true;
          testResult.textContent = "Connecting to AI model\u2026";
          testResult.className = "test-result-msg";
          try {
            const settings = buildSettingsObject();
            if (settings.cleanup.type === "OpenAiCompat" && oaiKey && oaiKey.value.trim()) {
              await invoke("set_keychain_secret", {
                account: settings.cleanup.keychain_account,
                secret: oaiKey.value.trim()
              });
            }
            const result = await invoke("test_cleanup", { settings });
            testResult.textContent = `\u2713 Model Responded ("${result.output}") \u2022 Latency: ${result.latency_ms}ms`;
            testResult.className = "test-result-msg success";
          } catch (e) {
            testResult.textContent = `\u2717 Connection Failed: ${e?.message ?? e ?? "Ensure model is running"}`;
            testResult.className = "test-result-msg error";
          } finally {
            testBtn.disabled = false;
          }
        });
      }
      loadSettings().catch(console.error);
      loadHistoryFromBackend().catch(console.error);
      var accessBadge = document.getElementById("accessibility-badge");
      var grantAccessBtn = document.getElementById("btn-grant-accessibility");
      async function updateAccessibilityUI() {
        if (!accessBadge) return;
        try {
          const isGranted = await invoke("check_accessibility");
          if (isGranted) {
            accessBadge.textContent = "Granted";
            accessBadge.className = "badge-status";
            accessBadge.style.background = "rgba(34, 197, 94, 0.15)";
            accessBadge.style.color = "#22c55e";
          } else {
            accessBadge.textContent = "Action Required";
            accessBadge.className = "badge-status";
            accessBadge.style.background = "rgba(234, 179, 8, 0.15)";
            accessBadge.style.color = "#eab308";
          }
        } catch (e) {
          console.error("Failed to check accessibility:", e);
        }
      }
      if (grantAccessBtn) {
        grantAccessBtn.addEventListener("click", async () => {
          try {
            await invoke("request_accessibility");
          } catch (e) {
            console.error("Failed to open system settings:", e);
          }
        });
      }
      updateAccessibilityUI();
      window.addEventListener("focus", () => {
        updateAccessibilityUI();
        loadHistoryFromBackend();
      });
    }
  });
  require_settings();
})();
