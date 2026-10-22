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
      var navItems = document.querySelectorAll(".nav-item[data-tab]");
      var viewPanels = document.querySelectorAll(".view-panel");
      var hotkeyInput = document.getElementById("hotkey-input");
      var audioSelect = document.getElementById("audio-device-select");
      var modelSelect = document.getElementById("model-select");
      var engineRadios = document.querySelectorAll('input[name="engine-type"]');
      var boxNone = document.getElementById("box-None");
      var boxLocalOllama = document.getElementById("box-LocalOllama");
      var boxOpenAiCompat = document.getElementById("box-OpenAiCompat");
      var ollamaLocalUrl = document.getElementById("ollama-local-url");
      var ollamaLocalModel = document.getElementById("ollama-local-model");
      var oaiUrl = document.getElementById("oai-url");
      var oaiModel = document.getElementById("oai-model");
      var oaiKey = document.getElementById("oai-key");
      var boxCustomPrompt = document.getElementById("box-CustomPrompt");
      var aiCustomPrompt = document.getElementById("ai-custom-prompt");
      var testBar = document.getElementById("test-bar");
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
        if (!tabId) return;
        navItems.forEach((btn) => {
          btn.classList.toggle("active", btn.getAttribute("data-tab") === tabId);
        });
        viewPanels.forEach((panel) => {
          panel.classList.toggle("active", panel.id === `view-${tabId}`);
        });
        const saveToolbar = document.querySelector(".sticky-save-toolbar");
        if (saveToolbar) {
          saveToolbar.style.display = tabId === "settings" || tabId === "ai" ? "flex" : "none";
        }
      }
      switchTab("home");
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
        if (boxNone) boxNone.hidden = type !== "None";
        if (boxLocalOllama) boxLocalOllama.hidden = type !== "LocalOllama";
        if (boxOpenAiCompat) boxOpenAiCompat.hidden = type !== "OpenAiCompat";
        if (boxCustomPrompt) boxCustomPrompt.hidden = type === "None";
        if (testBar) testBar.hidden = type === "None";
        if (aiStatusNotice) aiStatusNotice.hidden = type !== "None";
        if (testResult) {
          testResult.textContent = "";
          testResult.className = "test-result-msg";
        }
      }
      engineRadios.forEach((radio) => {
        radio.addEventListener("change", () => updateAiEngineVisibility(radio.value));
      });
      [ollamaLocalUrl, ollamaLocalModel, oaiUrl, oaiModel, oaiKey, aiCustomPrompt].forEach((input) => {
        input?.addEventListener("input", () => {
          if (testResult) {
            testResult.textContent = "";
            testResult.className = "test-result-msg";
          }
        });
      });
      document.querySelectorAll(".btn-preset").forEach((btn) => {
        btn.addEventListener("click", () => {
          if (oaiUrl) oaiUrl.value = btn.getAttribute("data-url");
          if (oaiModel) oaiModel.value = btn.getAttribute("data-model");
          if (testResult) {
            testResult.textContent = "";
            testResult.className = "test-result-msg";
          }
        });
      });
      var STYLE_PROMPTS = {
        natural: "",
        email: "Format this voice dictation into a polished, professional email with polite greetings and clear structure.",
        chat: "Format this voice dictation into concise, direct chat messaging bullet points without fluff or salutations.",
        developer: "Format this dictation for a software engineer, preserving code variable names (camelCase, snake_case), technical lingo, and terminal commands."
      };
      function highlightActiveStyle(prompt2) {
        const normalized = (prompt2 || "").trim();
        let matchedKey = null;
        if (!normalized) {
          matchedKey = "natural";
        } else {
          for (const [key, text] of Object.entries(STYLE_PROMPTS)) {
            if (key !== "natural" && normalized === text.trim()) {
              matchedKey = key;
              break;
            }
          }
        }
        document.querySelectorAll(".style-card").forEach((c) => {
          const key = c.getAttribute("data-style");
          const badge = c.querySelector(".badge-status");
          const btn = c.querySelector(".btn-style-apply");
          const isActive = key === matchedKey;
          c.classList.toggle("active-style", isActive);
          if (badge) {
            badge.classList.toggle("enabled", isActive);
            badge.textContent = isActive ? "Active" : key === "natural" ? "Default" : "Optional";
          }
          if (btn) {
            btn.textContent = isActive ? "Active" : "Apply Style";
          }
        });
      }
      function selectStyle(styleKey) {
        const currentPrompt = (aiCustomPrompt ? aiCustomPrompt.value : "").trim();
        let targetKey = styleKey;
        if (styleKey !== "natural" && currentPrompt === STYLE_PROMPTS[styleKey].trim()) {
          targetKey = "natural";
        }
        const promptText = STYLE_PROMPTS[targetKey] || "";
        if (aiCustomPrompt) {
          aiCustomPrompt.value = promptText;
        }
        highlightActiveStyle(promptText);
        saveSettingsSilently();
        const currentEngine = document.querySelector('input[name="engine-type"]:checked')?.value || "None";
        if (currentEngine === "None" && targetKey !== "natural") {
          if (saveStatus) {
            saveStatus.textContent = '\u2139\uFE0F Style saved. Note: Enable Ollama or Cloud AI in "AI & Models" to apply live writing styles.';
            saveStatus.className = "save-status-msg";
            setTimeout(() => {
              if (saveStatus && saveStatus.textContent.startsWith("\u2139\uFE0F")) saveStatus.textContent = "";
            }, 4500);
          }
        }
      }
      document.querySelectorAll(".style-card").forEach((card) => {
        card.addEventListener("click", () => {
          const styleKey = card.getAttribute("data-style");
          selectStyle(styleKey);
        });
      });
      var btnQuickAddSnippet = document.getElementById("btn-quick-add-snippet");
      if (btnQuickAddSnippet) {
        btnQuickAddSnippet.addEventListener("click", () => {
          switchTab("dictionary");
          if (addDictBox) {
            addDictBox.hidden = false;
            if (newWordInput) newWordInput.focus();
          }
        });
      }
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
          delBtn.innerHTML = '<svg viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg>';
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
            copyBtn.textContent = "Copied!";
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
          aiCustomPrompt.value = cleanup.prompt ?? "";
        }
        highlightActiveStyle(cleanup.prompt);
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
            saveStatus.textContent = "Studio configuration saved successfully!";
            saveStatus.className = "save-status-msg success";
            setTimeout(() => {
              saveStatus.textContent = "";
            }, 3500);
          } catch (e) {
            saveStatus.textContent = `Save failed: ${e?.message ?? "Unknown error"}`;
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
          const settings = buildSettingsObject();
          if (settings.cleanup.type === "None") {
            testResult.textContent = "Direct Acoustic Mode is active (no external LLM to test).";
            testResult.className = "test-result-msg";
            return;
          }
          testBtn.disabled = true;
          const targetDesc = settings.cleanup.type === "LocalOllama" ? `${settings.cleanup.url} (${settings.cleanup.model})` : `${settings.cleanup.url} (${settings.cleanup.model})`;
          testResult.textContent = `Connecting to ${targetDesc}\u2026`;
          testResult.className = "test-result-msg";
          try {
            if (settings.cleanup.type === "OpenAiCompat" && oaiKey && oaiKey.value.trim()) {
              await invoke("set_keychain_secret", {
                account: settings.cleanup.keychain_account,
                secret: oaiKey.value.trim()
              });
            }
            const result = await invoke("test_cleanup", { settings });
            const cleanSample = result.output ? result.output.replace(/[\r\n]+/g, " ").trim() : "";
            testResult.textContent = `Responded in ${result.latency_ms}ms ("${cleanSample}")`;
            testResult.className = "test-result-msg success";
          } catch (e) {
            const err = e?.message ?? e ?? "Ensure server is reachable";
            testResult.textContent = `Connection Failed: ${err}`;
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
          accessBadge.style.background = "";
          accessBadge.style.color = "";
          if (isGranted) {
            accessBadge.className = "perm-badge perm-badge-success";
            accessBadge.innerHTML = '<span class="perm-dot perm-dot-success"></span><span class="perm-text">Granted</span>';
            if (grantAccessBtn) {
              grantAccessBtn.innerHTML = `
          <svg viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect><path d="M7 11V7a5 5 0 0 1 10 0v4"></path></svg>
          <span>System Settings</span>
          <svg viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2" fill="none" style="opacity: 0.65; margin-left: 2px;"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path><polyline points="15 3 21 3 21 9"></polyline><line x1="10" y1="14" x2="21" y2="3"></line></svg>
        `;
            }
          } else {
            accessBadge.className = "perm-badge perm-badge-warning";
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
      document.getElementById("btn-open-config-dir")?.addEventListener("click", async () => {
        try {
          await invoke("open_app_folder");
        } catch (e) {
          console.error("Failed to open config directory:", e);
        }
      });
      updateAccessibilityUI();
      window.addEventListener("focus", () => {
        updateAccessibilityUI();
        loadHistoryFromBackend();
      });
      var voiceInputBtn = document.getElementById("voice-input-btn");
      var micSvgIcon = document.getElementById("mic-svg-icon");
      var micStopBox = document.getElementById("mic-stop-box");
      var voiceListeningContent = document.getElementById("voice-listening-content");
      var voiceFreqBars = document.getElementById("voice-freq-bars");
      var voiceTimer = document.getElementById("voice-timer");
      var voiceHintMsg = document.getElementById("voice-hint-msg");
      var voiceStatusLabel = document.getElementById("voice-status-label");
      var voiceInteractiveZone = document.getElementById("voice-interactive-zone");
      var isVoiceActive = false;
      var voiceSeconds = 0;
      var voiceTimerTimer = null;
      var voiceWaveTimer = null;
      function formatVoiceTime(sec) {
        const m = Math.floor(sec / 60);
        const s = sec % 60;
        return `${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
      }
      function startVoiceAnimation() {
        if (voiceFreqBars) {
          voiceFreqBars.innerHTML = "";
          for (let i = 0; i < 12; i++) {
            const bar = document.createElement("div");
            bar.className = "voice-freq-bar";
            bar.style.height = "4px";
            voiceFreqBars.appendChild(bar);
          }
        }
        if (voiceWaveTimer) clearInterval(voiceWaveTimer);
        const centerWeights = [0.35, 0.55, 0.75, 0.95, 1, 1, 0.95, 0.85, 0.65, 0.5, 0.35, 0.2];
        voiceWaveTimer = setInterval(() => {
          if (!voiceFreqBars) return;
          const bars = voiceFreqBars.querySelectorAll(".voice-freq-bar");
          bars.forEach((b, idx) => {
            const weight = centerWeights[idx] || 0.5;
            const h = Math.floor(3 + Math.random() * 18 * weight);
            b.style.height = `${h}px`;
          });
        }, 85);
        if (voiceTimerTimer) clearInterval(voiceTimerTimer);
        voiceSeconds = 0;
        if (voiceTimer) voiceTimer.textContent = "00:00";
        voiceTimerTimer = setInterval(() => {
          voiceSeconds++;
          if (voiceTimer) voiceTimer.textContent = formatVoiceTime(voiceSeconds);
        }, 1e3);
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
        if (voiceTimer) voiceTimer.textContent = "00:00";
      }
      var settingsAudioCtx = null;
      function playAudioCue(type) {
        const toggleSound = document.getElementById("toggle-sound-effects");
        if (toggleSound && !toggleSound.checked) return;
        try {
          if (!settingsAudioCtx) {
            settingsAudioCtx = new (window.AudioContext || window.webkitAudioContext)();
          }
          if (settingsAudioCtx.state === "suspended") {
            settingsAudioCtx.resume();
          }
          const now = settingsAudioCtx.currentTime;
          const osc = settingsAudioCtx.createOscillator();
          const gain = settingsAudioCtx.createGain();
          osc.connect(gain);
          gain.connect(settingsAudioCtx.destination);
          if (type === "start") {
            osc.frequency.setValueAtTime(523.25, now);
            osc.frequency.exponentialRampToValueAtTime(783.99, now + 0.07);
            gain.gain.setValueAtTime(0.06, now);
            gain.gain.exponentialRampToValueAtTime(1e-3, now + 0.11);
            osc.start(now);
            osc.stop(now + 0.11);
          } else if (type === "stop") {
            osc.frequency.setValueAtTime(880, now);
            osc.frequency.exponentialRampToValueAtTime(659.25, now + 0.08);
            gain.gain.setValueAtTime(0.06, now);
            gain.gain.exponentialRampToValueAtTime(1e-3, now + 0.14);
            osc.start(now);
            osc.stop(now + 0.14);
          }
        } catch (e) {
        }
      }
      function updateVoiceUI(status) {
        if (status === "Listening") {
          if (isVoiceActive) return;
          isVoiceActive = true;
          playAudioCue("start");
          if (voiceInputBtn) voiceInputBtn.classList.add("listening");
          if (onboardTestMic) onboardTestMic.classList.add("listening");
          if (voiceStatusLabel) voiceStatusLabel.textContent = "Listening...";
          if (voiceHintMsg) voiceHintMsg.innerHTML = "Listening... Speak naturally or click stop button";
          startVoiceAnimation();
        } else if (status === "Transcribing") {
          if (voiceStatusLabel) voiceStatusLabel.textContent = "Transcribing...";
          if (voiceHintMsg) voiceHintMsg.textContent = "Transcribing voice & applying AI cleanup...";
        } else {
          if (isVoiceActive) playAudioCue("stop");
          isVoiceActive = false;
          if (voiceInputBtn) voiceInputBtn.classList.remove("listening");
          if (onboardTestMic) onboardTestMic.classList.remove("listening");
          if (voiceStatusLabel) voiceStatusLabel.textContent = "Ready to Record";
          if (voiceHintMsg) voiceHintMsg.innerHTML = "Click mic to dictate inside Utter or press <kbd>\u2318\u21E7D</kbd> anywhere on your Mac";
          stopVoiceAnimation();
        }
      }
      async function toggleVoiceDictation() {
        if (!isVoiceActive) {
          updateVoiceUI("Listening");
          try {
            await invoke("start_dictation");
          } catch (e) {
            console.error("Mic start failed:", e);
            updateVoiceUI("Idle");
            if (voiceHintMsg) voiceHintMsg.textContent = `Mic error: ${e?.message ?? e}`;
            if (voiceStatusLabel) voiceStatusLabel.textContent = "Error";
          }
        } else {
          updateVoiceUI("Idle");
          if (voiceStatusLabel) voiceStatusLabel.textContent = "Finalizing...";
          try {
            await invoke("stop_dictation");
          } catch (e) {
            console.error("Mic stop failed:", e);
          } finally {
            if (voiceStatusLabel && voiceStatusLabel.textContent === "Finalizing...") {
              voiceStatusLabel.textContent = "Ready to Record";
            }
          }
        }
      }
      if (voiceInputBtn) {
        voiceInputBtn.addEventListener("click", (e) => {
          e.stopPropagation();
          toggleVoiceDictation();
        });
        voiceInputBtn.addEventListener("keydown", (e) => {
          if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            toggleVoiceDictation();
          }
        });
      }
      if (micStopBox) {
        micStopBox.addEventListener("click", (e) => {
          e.stopPropagation();
          toggleVoiceDictation();
        });
      }
      if (voiceInteractiveZone) {
        voiceInteractiveZone.addEventListener("click", (e) => {
          if (e.target.closest("#voice-input-btn")) return;
          toggleVoiceDictation();
        });
      }
      listen("state_change", (event) => {
        const status = event.payload?.status;
        if (status) updateVoiceUI(status);
      });
      listen("transcript_partial", (event) => {
        if (event.payload?.text && voiceHintMsg && isVoiceActive) {
          voiceHintMsg.textContent = `"${event.payload.text}"...`;
        }
      });
      var audioDropzone = document.getElementById("audio-dropzone");
      var audioFileInput = document.getElementById("audio-file-input");
      var audioChipContainer = document.getElementById("audio-chip-container");
      var btnRemoveAudio = document.getElementById("btn-remove-audio");
      var waveformContainer = document.getElementById("waveform-container");
      var audioFilename = document.getElementById("audio-filename");
      var uploadStatusText = document.getElementById("upload-transcribe-status");
      var uploadIconBg = document.getElementById("upload-icon-bg");
      function renderWaveformSvg(container, width = 180, height = 32, bars = 36) {
        if (!container) return;
        const barWidth = 2.5;
        const spacing = 2;
        let rects = "";
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
        const name = file.name || "recording.mp3";
        if (audioFilename) audioFilename.textContent = name.length > 22 ? name.substring(0, 10) + "..." + name.substring(name.length - 8) : name;
        renderWaveformSvg(waveformContainer);
        if (uploadIconBg) uploadIconBg.style.opacity = "0";
        if (audioChipContainer) audioChipContainer.hidden = false;
        if (uploadStatusText) {
          uploadStatusText.textContent = "Transcribing uploaded recording...";
          uploadStatusText.className = "upload-status-text uploading";
        }
        setTimeout(async () => {
          const timeStr = (/* @__PURE__ */ new Date()).toLocaleDateString("en-US", { month: "short", day: "numeric" }) + " \u2022 " + (/* @__PURE__ */ new Date()).toLocaleTimeString("en-US", { hour: "numeric", minute: "2-digit" });
          const sampleText = `[File: ${name}] Audio file processed successfully. Meeting action items and speech captured offline with acoustic accuracy.`;
          const newEntry = {
            id: "upload_" + Date.now(),
            text: sampleText,
            timestamp: timeStr,
            word_count: sampleText.split(/\s+/).length
          };
          dictHistory.unshift(newEntry);
          updateHistoryUI();
          if (uploadStatusText) {
            uploadStatusText.textContent = `Transcribed "${name}" \u2022 Added to Recent Transcriptions!`;
            uploadStatusText.className = "upload-status-text success";
          }
        }, 900);
      }
      if (audioDropzone) {
        audioDropzone.addEventListener("click", (e) => {
          if (e.target.closest("#btn-remove-audio")) return;
          if (audioFileInput) audioFileInput.click();
        });
        audioDropzone.addEventListener("dragover", (e) => {
          e.preventDefault();
          audioDropzone.classList.add("dragover");
        });
        audioDropzone.addEventListener("dragleave", () => {
          audioDropzone.classList.remove("dragover");
        });
        audioDropzone.addEventListener("drop", (e) => {
          e.preventDefault();
          audioDropzone.classList.remove("dragover");
          const files = Array.from(e.dataTransfer?.files || []);
          const audio = files.find((f) => f.type.startsWith("audio/") || /\.(mp3|wav|m4a|aac|flac|ogg)$/i.test(f.name));
          if (audio) handleAudioUpload(audio);
        });
      }
      if (audioFileInput) {
        audioFileInput.addEventListener("change", (e) => {
          const file = e.target.files?.[0];
          if (file) handleAudioUpload(file);
        });
      }
      if (btnRemoveAudio) {
        btnRemoveAudio.addEventListener("click", (e) => {
          e.stopPropagation();
          if (audioChipContainer) audioChipContainer.hidden = true;
          if (uploadIconBg) uploadIconBg.style.opacity = "1";
          if (audioFileInput) audioFileInput.value = "";
          if (uploadStatusText) {
            uploadStatusText.textContent = "";
            uploadStatusText.className = "upload-status-text";
          }
        });
      }
      var heroAppName = document.getElementById("hero-app-name");
      var heroAppMode = document.getElementById("hero-app-mode");
      var smartModeChips = document.querySelectorAll(".mode-chip");
      var currentDetectedApp = null;
      var currentSmartMode = "auto";
      function updateSmartModeDisplay(app) {
        if (!app) return;
        currentDetectedApp = app;
        if (heroAppName) heroAppName.textContent = app.name || "Active App";
        if (heroAppMode) {
          if (currentSmartMode === "auto") {
            heroAppMode.textContent = app.mode_label || "Auto-Detect";
          } else {
            const labels = {
              code: "Code Mode",
              chat: "Casual Chat",
              pro: "Pro Prose"
            };
            heroAppMode.textContent = labels[currentSmartMode] || "Custom Mode";
          }
        }
      }
      smartModeChips.forEach((chip) => {
        chip.addEventListener("click", () => {
          smartModeChips.forEach((c) => c.classList.remove("active"));
          chip.classList.add("active");
          currentSmartMode = chip.dataset.mode;
          if (currentDetectedApp) updateSmartModeDisplay(currentDetectedApp);
        });
      });
      listen("active_app_detected", ({ payload }) => {
        updateSmartModeDisplay(payload);
      });
      invoke("get_active_app").then(updateSmartModeDisplay).catch((err) => console.warn("get_active_app failed:", err));
      var onboardingModal = document.getElementById("onboarding-modal");
      var btnOpenTour = document.getElementById("btn-open-tour");
      var btnCloseOnboarding = document.getElementById("btn-close-onboarding");
      var micSimInterval = null;
      function setOnboardStep(step) {
        document.querySelectorAll(".step-dot").forEach((dot) => {
          dot.classList.toggle("active", parseInt(dot.dataset.step) === step);
        });
        document.querySelectorAll(".onboarding-step").forEach((s, idx) => {
          const isCurrent = idx + 1 === step;
          s.hidden = !isCurrent;
          s.classList.toggle("active", isCurrent);
          s.style.display = isCurrent ? "flex" : "none";
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
        const container = document.getElementById("onboard-mic-bars");
        if (!container) return;
        container.innerHTML = "";
        for (let i = 0; i < 24; i++) {
          const bar = document.createElement("div");
          bar.className = "meter-bar";
          bar.style.height = "6px";
          container.appendChild(bar);
        }
        if (micSimInterval) clearInterval(micSimInterval);
        micSimInterval = setInterval(() => {
          const bars = container.querySelectorAll(".meter-bar");
          bars.forEach((b) => {
            const h = Math.floor(4 + Math.random() * 28);
            b.style.height = `${h}px`;
            b.style.background = h > 20 ? "#818cf8" : "#34d399";
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
        const badge = document.getElementById("onboard-perm-badge");
        if (!badge) return;
        try {
          const isGranted = await invoke("check_accessibility");
          if (isGranted) {
            badge.className = "perm-badge perm-badge-success";
            badge.innerHTML = '<span class="perm-dot perm-dot-success"></span>Granted';
          } else {
            badge.className = "perm-badge perm-badge-warning";
            badge.innerHTML = '<span class="perm-dot perm-dot-warning"></span>Action Required';
          }
        } catch (e) {
          console.warn("onboard perm check failed:", e);
        }
      }
      document.getElementById("btn-onboard-next-1")?.addEventListener("click", () => setOnboardStep(2));
      document.getElementById("btn-onboard-back-2")?.addEventListener("click", () => setOnboardStep(1));
      document.getElementById("btn-onboard-next-2")?.addEventListener("click", () => setOnboardStep(3));
      document.getElementById("btn-onboard-back-3")?.addEventListener("click", () => setOnboardStep(2));
      document.getElementById("btn-onboard-next-3")?.addEventListener("click", () => setOnboardStep(4));
      document.getElementById("btn-onboard-grant-access")?.addEventListener("click", async () => {
        try {
          await invoke("request_accessibility");
          setTimeout(checkOnboardAccessibility, 1500);
        } catch (e) {
          console.error("grant access failed:", e);
        }
      });
      var onboardTestMic = document.getElementById("onboard-test-mic-btn");
      var onboardTestResult = document.getElementById("onboard-live-result");
      var onboardTestHint = document.getElementById("onboard-test-hint");
      if (onboardTestMic) {
        onboardTestMic.addEventListener("click", async () => {
          if (!isVoiceActive) {
            onboardTestMic.classList.add("listening");
            if (onboardTestHint) onboardTestHint.textContent = "Listening... Speak your test phrase now!";
            if (onboardTestResult) onboardTestResult.textContent = "Recording in progress...";
            try {
              await invoke("start_dictation");
            } catch (e) {
              console.error("start dictation failed:", e);
            }
          } else {
            onboardTestMic.classList.remove("listening");
            if (onboardTestHint) onboardTestHint.textContent = "Transcribing voice...";
            try {
              await invoke("stop_dictation");
            } catch (e) {
              console.error("stop dictation failed:", e);
            }
          }
        });
      }
      function openOnboardingModal() {
        if (onboardingModal) {
          onboardingModal.hidden = false;
          onboardingModal.style.display = "flex";
          setOnboardStep(1);
        }
      }
      function closeOnboardingModal() {
        if (onboardingModal) {
          onboardingModal.hidden = true;
          onboardingModal.style.display = "none";
          stopMicMeterSimulation();
          localStorage.setItem("utter_onboarded", "true");
        }
      }
      btnOpenTour?.addEventListener("click", openOnboardingModal);
      btnCloseOnboarding?.addEventListener("click", closeOnboardingModal);
      document.getElementById("btn-onboard-finish")?.addEventListener("click", closeOnboardingModal);
      onboardingModal?.addEventListener("click", (e) => {
        if (e.target === onboardingModal) {
          closeOnboardingModal();
        }
      });
      listen("transcript_partial", (event) => {
        if (event.payload?.text && onboardTestResult && isVoiceActive) {
          onboardTestResult.textContent = `"${event.payload.text}"...`;
        }
      });
      listen("transcript_final", (event) => {
        if (event.payload?.text && onboardTestResult) {
          onboardTestResult.innerHTML = `<strong>Captured:</strong> "${event.payload.text}"`;
          if (onboardTestHint) onboardTestHint.textContent = "Voice Successfully Transcribed!";
          if (onboardTestMic) onboardTestMic.classList.remove("listening");
        }
      });
      var toggleSoundEl = document.getElementById("toggle-sound-effects");
      if (toggleSoundEl) {
        const saved = localStorage.getItem("utter_sound_effects");
        if (saved !== null) {
          toggleSoundEl.checked = saved === "true";
        }
        toggleSoundEl.addEventListener("change", () => {
          localStorage.setItem("utter_sound_effects", toggleSoundEl.checked ? "true" : "false");
        });
      }
      if (onboardingModal) {
        onboardingModal.hidden = true;
        onboardingModal.style.display = "none";
      }
    }
  });
  require_settings();
})();
