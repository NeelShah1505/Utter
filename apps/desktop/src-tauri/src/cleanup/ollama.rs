// cleanup/ollama.rs — Ollama cleanup backend (local and remote).
//
// Posts to the OpenAI-compatible /v1/chat/completions path, and also accepts
// Ollama's native `{ "message": { "content" } }` body if a proxy returns that.

use super::policy;
use super::{CleanupBackend, CleanupError};
use reqwest::Client;
use serde::Serialize;
use std::time::Duration;

pub struct OllamaBackend {
    url: String,
    model: String,
    bearer_token: Option<String>,
    prompt: Option<String>,
}

impl OllamaBackend {
    pub fn new(
        url: String,
        model: String,
        bearer_token: Option<String>,
        prompt: Option<String>,
    ) -> Self {
        Self {
            url,
            model,
            bearer_token,
            prompt,
        }
    }

    fn client() -> Result<Client, CleanupError> {
        Client::builder()
            .timeout(Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| CleanupError::Unreachable(format!("HTTP client: {e}")))
    }
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<Message<'a>>,
    stream: bool,
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

#[async_trait::async_trait]
impl CleanupBackend for OllamaBackend {
    async fn refine(&self, raw: &str) -> Result<String, CleanupError> {
        policy::validate_endpoint(&self.url)?;

        let clean_url = self.url.trim().trim_end_matches('/');
        let endpoint = if clean_url.ends_with("/chat/completions") {
            clean_url.to_string()
        } else if clean_url.ends_with("/v1") {
            format!("{clean_url}/chat/completions")
        } else {
            format!("{clean_url}/v1/chat/completions")
        };

        let system = policy::guarded_system(self.prompt.as_deref().unwrap_or(""));
        let user = policy::user_message(raw);
        let body = ChatRequest {
            model: &self.model,
            messages: vec![
                Message {
                    role: "system",
                    content: &system,
                },
                Message {
                    role: "user",
                    content: &user,
                },
            ],
            stream: false,
        };

        let client = Self::client()?;
        let mut req = client.post(&endpoint).json(&body);
        if let Some(token) = &self.bearer_token {
            req = req.bearer_auth(token);
        }

        let shown = policy::public_url(&self.url);
        let resp = req
            .send()
            .await
            .map_err(|e| CleanupError::Unreachable(format!("{shown}: {e}")))?;

        match resp.status().as_u16() {
            200 => {}
            301 | 302 | 303 | 307 | 308 => {
                return Err(CleanupError::Request(
                    "The cleanup server tried to redirect. Utter refused.".into(),
                ));
            }
            401 | 403 => return Err(CleanupError::Auth),
            429 => return Err(CleanupError::RateLimit),
            other => {
                return Err(CleanupError::Request(format!(
                    "The cleanup server returned HTTP {other}."
                )))
            }
        }

        let bytes = read_limited(resp).await?;
        let text = extract_text(&bytes)?;
        policy::vet_output(raw, &text)
    }
}

async fn read_limited(mut resp: reqwest::Response) -> Result<Vec<u8>, CleanupError> {
    let mut buf = Vec::new();
    loop {
        match resp.chunk().await {
            Ok(Some(chunk)) => {
                if buf.len().saturating_add(chunk.len()) > policy::MAX_RESPONSE_BYTES {
                    return Err(CleanupError::Request("Cleanup reply was too large.".into()));
                }
                buf.extend_from_slice(&chunk);
            }
            Ok(None) => return Ok(buf),
            Err(e) => return Err(CleanupError::Request(format!("reading cleanup reply: {e}"))),
        }
    }
}

fn extract_text(bytes: &[u8]) -> Result<String, CleanupError> {
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|e| CleanupError::Request(format!("Cleanup reply was not JSON: {e}")))?;
    if let Some(text) = value
        .pointer("/choices/0/message/content")
        .and_then(|v| v.as_str())
    {
        return Ok(text.to_string());
    }
    if let Some(text) = value.pointer("/message/content").and_then(|v| v.as_str()) {
        return Ok(text.to_string());
    }
    Err(CleanupError::Request(
        "Cleanup reply did not contain transcript text. The original text was kept.".into(),
    ))
}
