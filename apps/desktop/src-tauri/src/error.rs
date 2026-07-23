// error.rs — All application error types.
//
// WHY a dedicated error module:
//   Every error that can cross the Tauri IPC boundary must implement serde::Serialize
//   so Tauri can send it to the webview as JSON. We centralise all error codes here
//   so they stay consistent with ARCHITECTURE.md §3.3 and the JS side can match them
//   by string code without magic numbers.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// All error codes that can be emitted to the webview.
/// Codes match ARCHITECTURE.md §3.3 exactly.
#[derive(Debug, Error, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(tag = "code", content = "message")]
pub enum AppError {
    #[error("OS denied microphone access")]
    MicPermission,

    #[error("Another app holds the microphone")]
    MicBusy,

    #[error("macOS Accessibility permission not granted")]
    Accessibility,

    #[error("ASR model failed to load: {0}")]
    ModelLoad(String),

    #[error("Configured model file not found: {0}")]
    ModelNotFound(String),

    #[error("Cleanup endpoint not reachable: {0}")]
    CleanupUnreachable(String),

    #[error("Cleanup endpoint returned 401/403")]
    CleanupAuth,

    #[error("Cleanup endpoint rate-limited")]
    CleanupRate,

    #[error("Could not insert text into focused app")]
    InsertFailed,

    #[error("Internal error — see logs: {0}")]
    Internal(String),
}

impl AppError {
    /// The error code string as sent to the webview (matches ARCHITECTURE.md §3.3).
    pub fn code(&self) -> &'static str {
        match self {
            AppError::MicPermission         => "E_MIC_PERMISSION",
            AppError::MicBusy               => "E_MIC_BUSY",
            AppError::Accessibility         => "E_ACCESSIBILITY",
            AppError::ModelLoad(_)          => "E_MODEL_LOAD",
            AppError::ModelNotFound(_)      => "E_MODEL_NOT_FOUND",
            AppError::CleanupUnreachable(_) => "E_CLEANUP_UNREACHABLE",
            AppError::CleanupAuth           => "E_CLEANUP_AUTH",
            AppError::CleanupRate           => "E_CLEANUP_RATE",
            AppError::InsertFailed          => "E_INSERT_FAILED",
            AppError::Internal(_)           => "E_INTERNAL",
        }
    }
}

// Tauri requires that errors returned from #[tauri::command] functions
// implement serde::Serialize. Our AppError derives it above.

/// Convenience alias used throughout the codebase.
pub type Result<T> = std::result::Result<T, AppError>;
