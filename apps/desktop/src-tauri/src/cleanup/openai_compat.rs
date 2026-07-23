// cleanup/openai_compat.rs — OpenAI-compatible cleanup backend.
//
// Works with any endpoint that speaks the OpenAI /v1/chat/completions API:
//   - OpenAI (api.openai.com)
//   - LM Studio (localhost:1234)
//   - vLLM (any host)
//   - Ollama with OpenAI compat shim
//
// Security:
//   - The API key is loaded from the OS keychain by cleanup/mod.rs.
//     It is passed in as a String and held in memory only for the duration
//     of the request. It is NEVER logged.
//   - The raw transcript is sent ONLY to the configured endpoint over HTTPS.

use super::{CleanupBackend, CleanupError};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const SYSTEM_PROMPT: &str =
    "You are a transcript editor. Fix grammar, remove filler words (um, uh, like, you know), \
     and correct obvious transcription errors. Return ONLY the corrected transcript — no \
     explanations, no comments, no surrounding quotes. Preserve meaning exactly.";

pub struct OpenAiCompatBackend {
    client:  Client,
    url:     String,
    model:   String,
    api_key: String,
}

impl OpenAiCompatBackend {
    pub fn new(url: String, model: String, api_key: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60)) // cloud APIs can be slower
            .https_only(true) // enforce HTTPS — never send API keys over plain HTTP
            .build()
            .expect("HTTP client init failed");
        Self { client, url, model, api_key }
    }
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model:    &'a str,
    messages: Vec<Message<'a>>,
}

#[derive(Serialize)]
struct Message<'a> {
    role:    &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: MessageContent,
}

#[derive(Deserialize)]
struct MessageContent {
    content: String,
}

#[async_trait::async_trait]
impl CleanupBackend for OpenAiCompatBackend {
    async fn refine(&self, raw: &str) -> Result<String, CleanupError> {
        let endpoint = format!("{}/v1/chat/completions", self.url.trim_end_matches('/'));

        let body = ChatRequest {
            model:    &self.model,
            messages: vec![
                Message { role: "system", content: SYSTEM_PROMPT },
                Message { role: "user",   content: raw },
            ],
        };

        let resp = self.client
            .post(&endpoint)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| CleanupError::Unreachable(format!("{}: {e}", self.url)))?;

        match resp.status().as_u16() {
            200       => {}
            401 | 403 => return Err(CleanupError::Auth),
            429       => return Err(CleanupError::RateLimit),
            other     => return Err(CleanupError::Request(format!("HTTP {other}"))),
        }

        let chat: ChatResponse = resp.json().await.map_err(|e| {
            CleanupError::Request(format!("parse response: {e}"))
        })?;

        let content = chat
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content.trim().to_owned())
            .unwrap_or_else(|| raw.to_owned()); // fallback: return raw if no choice

        Ok(content)
    }
}
