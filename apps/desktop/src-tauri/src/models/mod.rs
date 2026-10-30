// models/mod.rs — Whisper model catalog and downloader.
//
// Models are ggml .bin files stored in ~/Library/Application Support/Utter/models.
// Downloads stream to a .part file and are renamed on completion so a partial
// download is never mistaken for a usable model.

use crate::error::{AppError, Result};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCatalogEntry {
    pub id: String,
    pub label: String,
    pub file: String,
    pub url: String,
    pub approx_mb: u32,
    pub note: String,
    pub installed: bool,
}

/// The models Utter offers to download. base.en is the sensible default;
/// large-v3-turbo is the accuracy option for users who want the best.
fn catalog_raw() -> Vec<(
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    u32,
    &'static str,
)> {
    vec![
        (
            "base.en",
            "Base (English) — recommended",
            "ggml-base.en.bin",
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin",
            142,
            "Fast, good accuracy, English only. Best default.",
        ),
        (
            "small.en",
            "Small (English) — more accurate",
            "ggml-small.en.bin",
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en.bin",
            466,
            "Higher accuracy, a little slower. English only.",
        ),
        (
            "large-v3-turbo",
            "Large v3 Turbo — best accuracy",
            "ggml-large-v3-turbo.bin",
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin",
            1624,
            "Best accuracy and multilingual. Large download.",
        ),
    ]
}

pub fn models_dir() -> Result<PathBuf> {
    let base =
        dirs::config_dir().ok_or_else(|| AppError::Internal("config dir not found".into()))?;
    let dir = base.join("Utter").join("models");
    std::fs::create_dir_all(&dir).map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(dir)
}

pub fn catalog() -> Result<Vec<ModelCatalogEntry>> {
    let dir = models_dir()?;
    Ok(catalog_raw()
        .into_iter()
        .map(|(id, label, file, url, mb, note)| ModelCatalogEntry {
            id: id.into(),
            label: label.into(),
            file: file.into(),
            url: url.into(),
            approx_mb: mb,
            note: note.into(),
            installed: dir.join(file).exists(),
        })
        .collect())
}

/// Download a model by catalog id, streaming to `<file>.part` then renaming.
/// Emits `model_download_progress` events: { id, received, total, pct }.
pub async fn download(app: &AppHandle, id: &str) -> Result<String> {
    let entry = catalog()?
        .into_iter()
        .find(|e| e.id == id)
        .ok_or_else(|| AppError::Internal(format!("unknown model id {id}")))?;

    let dir = models_dir()?;
    let final_path = dir.join(&entry.file);
    if final_path.exists() {
        return Ok(final_path.to_string_lossy().into_owned());
    }
    let part_path = dir.join(format!("{}.part", entry.file));

    let client = reqwest::Client::builder()
        .build()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let resp = client
        .get(&entry.url)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("download request failed: {e}")))?;
    if !resp.status().is_success() {
        return Err(AppError::Internal(format!(
            "model download failed: HTTP {}",
            resp.status()
        )));
    }
    let total = resp.content_length().unwrap_or(0);

    use std::io::Write;
    let mut file =
        std::fs::File::create(&part_path).map_err(|e| AppError::Internal(e.to_string()))?;
    let mut received: u64 = 0;
    let mut last_pct = -1i64;
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| AppError::Internal(format!("download stream error: {e}")))?;
        file.write_all(&chunk)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        received += chunk.len() as u64;
        let pct = (received * 100)
            .checked_div(total)
            .map(|v| v as i64)
            .unwrap_or(-1);
        if pct != last_pct {
            last_pct = pct;
            let _ = app.emit(
                "model_download_progress",
                serde_json::json!({ "id": id, "received": received, "total": total, "pct": pct }),
            );
        }
    }
    file.flush()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    file.sync_all()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    drop(file);

    std::fs::rename(&part_path, &final_path)
        .map_err(|e| AppError::Internal(format!("finalize model: {e}")))?;
    log::info!("models: downloaded {} to {}", id, final_path.display());
    Ok(final_path.to_string_lossy().into_owned())
}
