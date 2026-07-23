// cleanup/mod.rs — Optional post-ASR text cleanup layer.
//
// The cleanup layer is OFF by default. When enabled, the raw transcript is sent
// to a user-configured backend for grammar/filler-word refinement.
//
// All backends implement the CleanupBackend trait with a single async method:
//   refine(raw: &str) -> Result<String, CleanupError>
//
// The shell calls refine() after end_session() if cleanup is enabled.
// The raw transcript is NEVER sent anywhere if cleanup is disabled (None).
//
// WHY async trait: network calls (Ollama, OpenAI) are inherently async.
// We use an async fn in a trait, which requires the `async-trait` pattern
// (in Rust 2024 edition, async fns in traits are stable).

pub mod noop;
pub mod ollama;
pub mod openai_compat;

use crate::error::AppError;
use crate::settings::CleanupConfig;

/// Errors from the cleanup layer. Mapped to AppError codes.
#[derive(Debug, thiserror::Error)]
pub enum CleanupError {
    #[error("Endpoint not reachable: {0}")]
    Unreachable(String),

    #[error("Authentication failed (401/403)")]
    Auth,

    #[error("Rate limited")]
    RateLimit,

    #[error("Request failed: {0}")]
    Request(String),
}

impl From<CleanupError> for AppError {
    fn from(e: CleanupError) -> Self {
        match e {
            CleanupError::Unreachable(u) => AppError::CleanupUnreachable(u),
            CleanupError::Auth           => AppError::CleanupAuth,
            CleanupError::RateLimit      => AppError::CleanupRate,
            CleanupError::Request(r)     => AppError::Internal(r),
        }
    }
}

/// The cleanup backend contract.
/// Every backend must be Send + Sync — they are shared across async tasks.
#[async_trait::async_trait]
pub trait CleanupBackend: Send + Sync {
    /// Refine the raw transcript. Returns the cleaned string.
    /// Must NOT store the transcript anywhere. Must NOT log it.
    async fn refine(&self, raw: &str) -> Result<String, CleanupError>;
}

/// Build a boxed CleanupBackend from the user's CleanupConfig.
/// Returns a NoOpCleanup if the config is None (default).
pub fn build_backend(config: &CleanupConfig) -> Box<dyn CleanupBackend> {
    match config {
        CleanupConfig::None => {
            Box::new(noop::NoOpCleanup)
        }
        CleanupConfig::LocalOllama { url, model } => {
            Box::new(ollama::OllamaBackend::new(url.clone(), model.clone(), None))
        }
        CleanupConfig::RemoteOllama { url, model, keychain_account } => {
            // Load bearer token from keychain — never stored in config file.
            let token = load_keychain_secret(keychain_account);
            Box::new(ollama::OllamaBackend::new(url.clone(), model.clone(), token))
        }
        CleanupConfig::OpenAiCompat { url, model, keychain_account } => {
            let api_key = load_keychain_secret(keychain_account).unwrap_or_default();
            Box::new(openai_compat::OpenAiCompatBackend::new(
                url.clone(),
                model.clone(),
                api_key,
            ))
        }
    }
}

/// Read a secret from the OS keychain.
/// Service name: "com.utter.app" (matches MEMORY.md §2.5).
/// Returns None if no secret is stored for the account.
pub fn load_keychain_secret(account: &str) -> Option<String> {
    if account.is_empty() {
        return None;
    }
    match keyring::Entry::new("com.utter.app", account) {
        Ok(entry) => match entry.get_password() {
            Ok(secret) => Some(secret),
            Err(keyring::Error::NoEntry) => None,
            Err(e) => {
                log::warn!("keychain: could not read account '{account}': {e}");
                None
            }
        },
        Err(e) => {
            log::warn!("keychain: could not create entry for '{account}': {e}");
            None
        }
    }
}

/// Store a secret in the OS keychain.
/// Service name: "com.utter.app".
pub fn store_keychain_secret(account: &str, secret: &str) -> Result<(), keyring::Error> {
    let entry = keyring::Entry::new("com.utter.app", account)?;
    entry.set_password(secret)?;
    log::info!("keychain: stored secret for account '{account}'");
    Ok(())
}

/// Delete a secret from the OS keychain.
pub fn delete_keychain_secret(account: &str) -> Result<(), keyring::Error> {
    let entry = keyring::Entry::new("com.utter.app", account)?;
    entry.delete_credential()?;
    log::info!("keychain: deleted secret for account '{account}'");
    Ok(())
}
