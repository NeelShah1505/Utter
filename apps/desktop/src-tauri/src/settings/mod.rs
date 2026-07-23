// settings/mod.rs — User settings: load, save, defaults.
//
// Config is stored as TOML at:
//   macOS:   ~/Library/Application Support/Utter/config.toml
//   Windows: %APPDATA%\Utter\config.toml
//
// API keys (cleanup backend credentials) are NEVER stored here.
// They live in the OS keychain — see cleanup/mod.rs.
//
// WHY TOML: human-readable, human-editable, round-trips without schema surprises.
// WHY not SQLite/JSON: overkill for a flat settings file with <20 fields.

use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// The full application settings struct.
/// All fields have serde defaults so a partially-written config file still loads.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Global hotkey string. Default: "CmdOrCtrl+Shift+D"
    pub hotkey: String,

    /// Path to the ASR model file. Empty = use bundled default.
    pub model_path: String,

    /// Audio input device name. Empty = system default.
    pub audio_device: String,

    /// Cleanup backend configuration.
    pub cleanup: CleanupConfig,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            hotkey:       "CmdOrCtrl+Shift+D".into(),
            model_path:   String::new(),
            audio_device: String::new(),
            cleanup:      CleanupConfig::None,
        }
    }
}

/// Which cleanup backend is active. OFF by default (None).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum CleanupConfig {
    /// No cleanup — raw ASR output is inserted as-is. The default.
    None,

    /// Local Ollama instance (http://localhost:11434 by default).
    LocalOllama {
        #[serde(default = "default_ollama_url")]
        url:   String,
        /// Model name, e.g. "llama3" or "qwen2.5".
        model: String,
    },

    /// Remote Ollama instance. URL + optional bearer token stored in keychain.
    RemoteOllama {
        url:   String,
        model: String,
        /// keychain account name for the bearer token (if any)
        #[serde(default)]
        keychain_account: String,
    },

    /// Any OpenAI-compatible endpoint. API key stored in keychain.
    OpenAiCompat {
        url:   String,
        model: String,
        /// keychain account name for the API key
        keychain_account: String,
    },
}

fn default_ollama_url() -> String {
    "http://localhost:11434".into()
}

/// Returns the OS-specific path to config.toml.
/// Creates parent directories if they do not exist.
pub fn config_path() -> std::result::Result<PathBuf, std::io::Error> {
    let base = dirs::config_dir()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "config dir not found"))?;
    let dir = base.join("Utter");
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("config.toml"))
}

/// Load settings from disk. Returns `Settings::default()` if the file is absent
/// or cannot be parsed (we don't fail-hard on a bad config — we warn and reset).
pub fn load() -> Settings {
    let path = match config_path() {
        Ok(p)  => p,
        Err(e) => {
            log::warn!("settings: could not determine config path: {e}");
            return Settings::default();
        }
    };

    if !path.exists() {
        return Settings::default();
    }

    let raw = match std::fs::read_to_string(&path) {
        Ok(s)  => s,
        Err(e) => {
            log::warn!("settings: could not read {}: {e}", path.display());
            return Settings::default();
        }
    };

    match toml::from_str::<Settings>(&raw) {
        Ok(s)  => s,
        Err(e) => {
            log::warn!("settings: parse error in {}: {e} — using defaults", path.display());
            Settings::default()
        }
    }
}

/// Persist settings to disk.
/// Fails with `AppError::Internal` if serialisation or write fails.
pub fn save(settings: &Settings) -> Result<()> {
    let path = config_path().map_err(|e| AppError::Internal(e.to_string()))?;

    let raw = toml::to_string_pretty(settings)
        .map_err(|e| AppError::Internal(format!("settings serialise: {e}")))?;

    std::fs::write(&path, raw)
        .map_err(|e| AppError::Internal(format!("settings write {}: {e}", path.display())))?;

    log::info!("settings: saved to {}", path.display());
    Ok(())
}
