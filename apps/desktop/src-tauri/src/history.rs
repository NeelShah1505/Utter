// history.rs — Persistent transcript history on disk.
//
// Stored as JSON at:
//   macOS:   ~/Library/Application Support/Utter/history.json
//   Windows: %APPDATA%\Utter\history.json

use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub text: String,
    pub timestamp: String,
    pub word_count: usize,
}

fn format_current_time() -> String {
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("date").arg("+%b %d • %I:%M %p").output() {
            let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !s.is_empty() {
                return s;
            }
        }
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{now}")
}

pub fn history_path() -> std::result::Result<PathBuf, std::io::Error> {
    let base = dirs::config_dir()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "config dir not found"))?;
    let dir = base.join("Utter");
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("history.json"))
}

pub fn load() -> Vec<HistoryEntry> {
    let path = match history_path() {
        Ok(p) => p,
        Err(e) => {
            log::warn!("history: could not determine history path: {e}");
            return vec![];
        }
    };

    if !path.exists() {
        return vec![];
    }

    let raw = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            log::warn!("history: could not read {}: {e}", path.display());
            return vec![];
        }
    };

    serde_json::from_str::<Vec<HistoryEntry>>(&raw).unwrap_or_else(|e| {
        log::warn!("history: parse error in {}: {e}", path.display());
        vec![]
    })
}

pub fn save(entries: &[HistoryEntry]) -> Result<()> {
    let path = history_path().map_err(|e| AppError::Internal(e.to_string()))?;
    let raw = serde_json::to_string_pretty(entries)
        .map_err(|e| AppError::Internal(format!("history serialize: {e}")))?;
    std::fs::write(&path, raw)
        .map_err(|e| AppError::Internal(format!("history write {}: {e}", path.display())))?;
    Ok(())
}

pub fn add_entry(text: &str) -> Result<HistoryEntry> {
    let clean_text = text.trim();
    if clean_text.is_empty() {
        return Err(AppError::Internal("Empty transcript text".into()));
    }

    let mut entries = load();
    let word_count = clean_text.split_whitespace().count();
    let id = format!(
        "hist_{}_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        entries.len() + 1
    );

    let entry = HistoryEntry {
        id,
        text: clean_text.to_string(),
        timestamp: format_current_time(),
        word_count,
    };

    entries.insert(0, entry.clone());
    if entries.len() > 100 {
        entries.truncate(100);
    }

    save(&entries)?;
    log::info!("history: added entry with {} words", word_count);
    Ok(entry)
}

pub fn delete_entry(id: &str) -> Result<Vec<HistoryEntry>> {
    let mut entries = load();
    entries.retain(|e| e.id != id);
    save(&entries)?;
    Ok(entries)
}

pub fn update_entry(id: &str, new_text: &str) -> Result<Vec<HistoryEntry>> {
    let mut entries = load();
    for entry in &mut entries {
        if entry.id == id {
            entry.text = new_text.trim().to_string();
            entry.word_count = entry.text.split_whitespace().count();
            break;
        }
    }
    save(&entries)?;
    Ok(entries)
}

pub fn clear_all() -> Result<()> {
    save(&[])
}
