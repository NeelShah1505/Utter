// error.rs — All application error types.
//
// WHY a dedicated error module:
//   Every error that can cross the Tauri IPC boundary must implement serde::Serialize
//   so Tauri can send it to the webview as JSON. We centralise all error codes here
//   so they stay consistent with ARCHITECTURE.md §3.3 and the JS side can match them
//   by string code without magic numbers.

use serde::Serialize;
use thiserror::Error;

/// All error codes that can be emitted to the webview.
/// The serialized form is always `{ "code", "message" }` where `message`
/// is a plain sentence the UI can show.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AppError {
    #[error("Microphone access is off. Turn on Utter in System Settings → Privacy & Security → Microphone, then try again.")]
    MicPermission,

    #[error("Another app holds the microphone")]
    MicBusy,

    #[error("Accessibility is off, so the text was not typed into the other app. It is saved in History. Turn on Utter in System Settings → Privacy & Security → Accessibility, then dictate again.")]
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

    #[error("Could not insert text into the focused app. Click the text field and try again.")]
    InsertFailed,

    #[error("Utter cannot dictate on this computer. {0}")]
    EngineUnavailable(String),

    #[error("{0}")]
    Internal(String),
}

impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("code", self.code())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
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
            AppError::EngineUnavailable(_)  => "E_ENGINE_UNAVAILABLE",
            AppError::Internal(_)           => "E_INTERNAL",
        }
    }
}

// Tauri requires that errors returned from #[tauri::command] functions
// implement serde::Serialize. Our AppError derives it above.

/// Convenience alias used throughout the codebase.
pub type Result<T> = std::result::Result<T, AppError>;
