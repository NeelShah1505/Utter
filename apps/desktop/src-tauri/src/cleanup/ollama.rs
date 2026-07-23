// cleanup/ollama.rs — Ollama cleanup backend (local and remote).
//
// Speaks the Ollama OpenAI-compatible /api/chat endpoint.
// Works for both LocalOllama (http://localhost:11434) and RemoteOllama.
//
// WHY not the Ollama-native /api/generate:
//   /api/chat with a system prompt gives us a cleaner system/user separation.
//   Using the OpenAI-compat format also makes it easy to swap backends.
//
// Security:
//   - The bearer token (if any) is read from the OS keychain at build time.
//     It is NEVER logged. It is NOT stored in the Settings struct (which is written to disk).
//   - The raw transcript is sent ONLY to the configured endpoint.

use super::{CleanupBackend, CleanupError};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// System prompt sent to the model for every cleanup request.
const SYSTEM_PROMPT: &str =
    "You are a transcript editor. The user will give you a raw speech-to-text transcript. \
     Your job is to fix grammar, remove filler words (um, uh, like, you know), and correct \
     obvious transcription errors. Return ONLY the corrected transcript — no explanations, \
     no comments, no surrounding quotes. Preserve meaning exactly. Do not add content.";

pub struct OllamaBackend {
    client:       Client,
    url:          String,
    model:        String,
    bearer_token: Option<String>,
}

impl OllamaBackend {
    pub fn new(url: String, model: String, bearer_token: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            // reqwest::ClientBuilder::build only fails on TLS init errors,
            // which would be a fatal misconfiguration — panic is appropriate.
            .expect("HTTP client init failed");
        Self { client, url, model, bearer_token }
    }
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model:    &'a str,
    messages: Vec<Message<'a>>,
    stream:   bool,
}

#[derive(Serialize)]
struct Message<'a> {
    role:    &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct ChatResponse {
    message: MessageContent,
}

#[derive(Deserialize)]
struct MessageContent {
    content: String,
}

#[async_trait::async_trait]
impl CleanupBackend for OllamaBackend {
    async fn refine(&self, raw: &str) -> Result<String, CleanupError> {
        let endpoint = format!("{}/v1/chat/completions", self.url.trim_end_matches('/'));

        let body = ChatRequest {
            model:    &self.model,
            messages: vec![
                Message { role: "system", content: SYSTEM_PROMPT },
                Message { role: "user",   content: raw },
            ],
            stream: false,
        };

        let mut req = self.client.post(&endpoint).json(&body);

        if let Some(token) = &self.bearer_token {
            req = req.bearer_auth(token);
        }

        let resp = req.send().await.map_err(|e| {
            CleanupError::Unreachable(format!("{}: {e}", self.url))
        })?;

        match resp.status().as_u16() {
            200        => {}
            401 | 403  => return Err(CleanupError::Auth),
            429        => return Err(CleanupError::RateLimit),
            other      => return Err(CleanupError::Request(format!("HTTP {other}"))),
        }

        let chat: ChatResponse = resp.json().await.map_err(|e| {
            CleanupError::Request(format!("parse response: {e}"))
        })?;

        Ok(chat.message.content.trim().to_owned())
    }
}
