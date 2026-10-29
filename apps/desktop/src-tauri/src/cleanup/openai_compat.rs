// cleanup/openai_compat.rs — OpenAI-compatible cleanup backend.
//
// The API key stays in memory for the request. It is not logged.
// Redirects are refused. The transcript is delimited and the reply is vetted.

use super::policy;
use super::{CleanupBackend, CleanupError};
use reqwest::Client;
use serde::Serialize;
use std::time::Duration;

pub struct OpenAiCompatBackend {
    url: String,
    model: String,
    api_key: String,
    prompt: Option<String>,
}

impl OpenAiCompatBackend {
    pub fn new(url: String, model: String, api_key: String, prompt: Option<String>) -> Self {
        Self {
            url,
            model,
            api_key,
            prompt,
        }
    }
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<Message<'a>>,
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

#[async_trait::async_trait]
impl CleanupBackend for OpenAiCompatBackend {
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
        };

        let client = Client::builder()
            .timeout(Duration::from_secs(45))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| CleanupError::Unreachable(format!("HTTP client: {e}")))?;

        let shown = policy::public_url(&self.url);
        let resp = client
            .post(&endpoint)
            .bearer_auth(&self.api_key)
            .json(&body)
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

        let mut buf = Vec::new();
        let mut resp = resp;
        loop {
            match resp.chunk().await {
                Ok(Some(chunk)) => {
                    if buf.len().saturating_add(chunk.len()) > policy::MAX_RESPONSE_BYTES {
                        return Err(CleanupError::Request("Cleanup reply was too large.".into()));
                    }
                    buf.extend_from_slice(&chunk);
                }
                Ok(None) => break,
                Err(e) => return Err(CleanupError::Request(format!("reading cleanup reply: {e}"))),
            }
        }

        let value: serde_json::Value = serde_json::from_slice(&buf)
            .map_err(|e| CleanupError::Request(format!("Cleanup reply was not JSON: {e}")))?;
        let text = value
            .pointer("/choices/0/message/content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                CleanupError::Request(
                    "Cleanup reply did not contain transcript text. The original text was kept."
                        .into(),
                )
            })?;
        policy::vet_output(raw, text)
    }
}
