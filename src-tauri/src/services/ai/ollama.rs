//! Ollama provider — Tier 1 (Local).
//!
//! Communicates with a locally-running Ollama instance at the configured base
//! URL (default: `http://localhost:11434`). No API key is required. If Ollama
//! is not running, [`health_check`] returns `false` and the router falls back
//! to Tier 2.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{CompletionRequest, CompletionResponse, Provider, Tier};
use crate::error::CntrlError;

// ─────────────────────────────────────────────────────────────────────────────
// Wire types (private to this module)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct OllamaGenerateRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<&'a str>,
}

#[derive(Deserialize)]
struct OllamaGenerateResponse {
    response: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Provider implementation
// ─────────────────────────────────────────────────────────────────────────────

/// Ollama local inference provider.
pub struct OllamaProvider {
    client: Client,
    /// Base URL of the Ollama API, e.g. `http://localhost:11434`.
    base_url: String,
    /// Model name, e.g. `"llama3"` or `"mistral"`.
    model: String,
}

impl OllamaProvider {
    /// Creates a new `OllamaProvider`.
    ///
    /// # Arguments
    /// * `base_url` – Ollama server base URL (no trailing slash needed).
    /// * `model`    – Model name to use for completions.
    #[must_use]
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
            model: model.into(),
        }
    }
}

#[async_trait]
impl Provider for OllamaProvider {
    fn name(&self) -> &str {
        "Ollama"
    }

    fn tier(&self) -> Tier {
        Tier::Local
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, CntrlError> {
        let url = format!("{}/api/generate", self.base_url.trim_end_matches('/'));

        let body = OllamaGenerateRequest {
            model: &self.model,
            prompt: &req.prompt,
            stream: false,
            system: req.system.as_deref(),
        };

        let res = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| CntrlError::Ai(format!("Ollama connection error: {e}")))?;

        if !res.status().is_success() {
            return Err(CntrlError::Ai(format!("Ollama error: {}", res.status())));
        }

        let data: OllamaGenerateResponse = res
            .json()
            .await
            .map_err(|e| CntrlError::Ai(format!("Ollama JSON parse error: {e}")))?;

        Ok(CompletionResponse {
            text: data.response,
            tokens_used: None,
            provider: self.name().to_string(),
        })
    }

    async fn health_check(&self) -> bool {
        let url = format!("{}/api/tags", self.base_url.trim_end_matches('/'));
        self.client
            .get(&url)
            .timeout(std::time::Duration::from_secs(3))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that the correct URL and JSON body are sent to Ollama.
    /// Uses a mock HTTP server (mockito) pattern via `wiremock` if available,
    /// or simply asserts the request shape by constructing the struct.
    #[test]
    fn request_body_is_correct() {
        let provider = OllamaProvider::new("http://localhost:11434", "llama3");
        assert_eq!(provider.name(), "Ollama");
        assert_eq!(provider.tier(), Tier::Local);

        // Verify the serialised request shape
        let req_body = OllamaGenerateRequest {
            model: "llama3",
            prompt: "Hello",
            stream: false,
            system: None,
        };
        let json = serde_json::to_value(&req_body).expect("should serialise");
        assert_eq!(json["model"], "llama3");
        assert_eq!(json["prompt"], "Hello");
        assert_eq!(json["stream"], false);
        // `system` is skipped when None
        assert!(
            json.get("system").is_none(),
            "system must be omitted when None"
        );
    }

    #[test]
    fn request_body_with_system_is_correct() {
        let req_body = OllamaGenerateRequest {
            model: "llama3",
            prompt: "Hello",
            stream: false,
            system: Some("You are a helpful assistant."),
        };
        let json = serde_json::to_value(&req_body).expect("should serialise");
        assert_eq!(json["system"], "You are a helpful assistant.");
    }
}
