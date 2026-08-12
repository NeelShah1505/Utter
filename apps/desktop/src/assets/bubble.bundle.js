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

  // src/assets/bubble.js
  var require_bubble = __commonJS({
    "src/assets/bubble.js"() {
      init_event();
      init_core();
      var body = document.body;
      var transcriptEl = document.getElementById("transcript-text");
      var statusLabel = document.getElementById("status-label");
      var hintText = document.getElementById("hint-text");
      var errorBox = document.getElementById("error-box");
      var errorMessage = document.getElementById("error-message");
      var dismissBtn = document.getElementById("error-dismiss");
      var STATE_CLASSES = ["state-idle", "state-listening", "state-transcribing", "state-error"];
      function applyState(status, message = "") {
        STATE_CLASSES.forEach((c) => body.classList.remove(c));
        body.classList.add(`state-${status.toLowerCase()}`);
        const labels = {
          Idle: "Ready",
          Listening: "Listening\u2026",
          Transcribing: "Processing\u2026",
          Error: "Error"
        };
        statusLabel.textContent = labels[status] ?? status;
        hintText.hidden = status === "Listening" || status === "Transcribing";
        errorBox.hidden = status !== "Error";
        if (status === "Error" && message) {
          errorMessage.textContent = message;
        }
        if (status === "Idle") {
          transcriptEl.textContent = "";
        }
      }
      async function init() {
        await listen("state_change", ({ payload }) => {
          applyState(payload.status, payload.message);
        });
        await listen("transcript_partial", ({ payload }) => {
          transcriptEl.textContent = payload.text || "";
        });
        await listen("transcript_final", ({ payload }) => {
          transcriptEl.textContent = payload.text;
          setTimeout(() => applyState("Idle"), 1500);
        });
        try {
          const status = await invoke("get_status");
          applyState(status.status ?? "Idle");
        } catch (e) {
          console.warn("get_status failed:", e);
          applyState("Idle");
        }
      }
      dismissBtn.addEventListener("click", () => applyState("Idle"));
      init().catch(console.error);
    }
  });
  require_bubble();
})();
