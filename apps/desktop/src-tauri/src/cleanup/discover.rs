// cleanup/discover.rs — Ask a cleanup provider which models it can serve.
//
// Different servers expose different listing endpoints, so we probe in order
// and accept whichever shape answers:
//   Ollama native      GET {base}/api/tags      -> { models: [ { name } ] }
//   OpenAI-compatible  GET {base}/v1/models     -> { data:   [ { id   } ] }
//   Already-versioned  GET {base}/models        -> { data:   [ { id   } ] }
//
// The same URL policy as the chat call applies (https, or http on localhost),
// and the API key is sent as a bearer token but never logged.

use super::policy;
use super::CleanupError;
use reqwest::Client;
use std::time::Duration;

/// Probe `url` for a model list. Returns model ids, sorted and deduped.
pub async fn list_models(url: &str, bearer: Option<&str>) -> Result<Vec<String>, CleanupError> {
    policy::validate_endpoint(url)?;
    let base = url.trim().trim_end_matches('/');

    // If the user already pointed at a chat endpoint, step back to its root.
    let root = base
        .strip_suffix("/chat/completions")
        .unwrap_or(base)
        .trim_end_matches('/');
    let root_no_v1 = root
        .strip_suffix("/v1")
        .unwrap_or(root)
        .trim_end_matches('/');

    let candidates = [
        format!("{root_no_v1}/api/tags"),
        format!("{root_no_v1}/v1/models"),
        format!("{root}/models"),
    ];

    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| CleanupError::Unreachable(format!("HTTP client: {e}")))?;

    let shown = policy::public_url(url);
    let mut last_err: Option<CleanupError> = None;

    for endpoint in candidates.iter() {
        let mut req = client.get(endpoint);
        if let Some(key) = bearer.filter(|k| !k.is_empty()) {
            req = req.bearer_auth(key);
        }
        let resp = match req.send().await {
            Ok(r) => r,
            Err(e) => {
                last_err = Some(CleanupError::Unreachable(format!("{shown}: {e}")));
                continue;
            }
        };
        match resp.status().as_u16() {
            200 => {}
            401 | 403 => return Err(CleanupError::Auth),
            429 => return Err(CleanupError::RateLimit),
            _ => {
                last_err = Some(CleanupError::Request(format!(
                    "{shown} returned HTTP {}",
                    resp.status().as_u16()
                )));
                continue;
            }
        }

        let bytes = match resp.bytes().await {
            Ok(b) => b,
            Err(e) => {
                last_err = Some(CleanupError::Request(format!("reading model list: {e}")));
                continue;
            }
        };
        if bytes.len() > policy::MAX_RESPONSE_BYTES {
            last_err = Some(CleanupError::Request("Model list was too large.".into()));
            continue;
        }
        let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
            last_err = Some(CleanupError::Request("Model list was not JSON.".into()));
            continue;
        };

        let mut out = parse_models(&value);
        if !out.is_empty() {
            out.retain(|m| is_chat_capable(m));
            out.sort();
            out.dedup();
            return Ok(out);
        }
        last_err = Some(CleanupError::Request(
            "The server answered but listed no models.".into(),
        ));
    }

    Err(last_err.unwrap_or_else(|| {
        CleanupError::Request("Could not find a model list on that server.".into())
    }))
}

/// Drop models that cannot answer a chat completion, so the picker never offers
/// something that will just error (embeddings, rerankers, audio/image models).
fn is_chat_capable(id: &str) -> bool {
    let l = id.to_lowercase();
    const NOT_CHAT: &[&str] = &[
        "embed",
        "rerank",
        "whisper",
        "tts",
        "dall-e",
        "stable-diffusion",
        "clip",
        "moderation",
    ];
    !NOT_CHAT.iter().any(|bad| l.contains(bad))
}

/// Accept either the Ollama (`models[].name`) or OpenAI (`data[].id`) shape.
fn parse_models(value: &serde_json::Value) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(arr) = value.get("models").and_then(|v| v.as_array()) {
        for m in arr {
            if let Some(name) = m.get("name").and_then(|v| v.as_str()) {
                out.push(name.to_string());
            } else if let Some(id) = m.get("id").and_then(|v| v.as_str()) {
                out.push(id.to_string());
            }
        }
    }
    if out.is_empty() {
        if let Some(arr) = value.get("data").and_then(|v| v.as_array()) {
            for m in arr {
                if let Some(id) = m.get("id").and_then(|v| v.as_str()) {
                    out.push(id.to_string());
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ollama_shape() {
        let v = serde_json::json!({"models":[{"name":"llama3.1:8b"},{"name":"qwen2.5"}]});
        assert_eq!(parse_models(&v), vec!["llama3.1:8b", "qwen2.5"]);
    }

    #[test]
    fn parses_openai_shape() {
        let v = serde_json::json!({"data":[{"id":"gpt-4o-mini"},{"id":"gpt-4o"}]});
        assert_eq!(parse_models(&v), vec!["gpt-4o-mini", "gpt-4o"]);
    }

    #[test]
    fn ignores_unknown_shape() {
        assert!(parse_models(&serde_json::json!({"whatever": 1})).is_empty());
    }

    #[test]
    fn filters_non_chat_models() {
        assert!(is_chat_capable("llama3.1:8b"));
        assert!(is_chat_capable("gpt-4o-mini"));
        assert!(!is_chat_capable("nomic-embed-text-v2-moe:latest"));
        assert!(!is_chat_capable("text-embedding-3-large"));
        assert!(!is_chat_capable("whisper-1"));
        assert!(!is_chat_capable("bge-reranker-base"));
    }
}
