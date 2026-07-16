//! Generic OpenAI-compatible provider — Tier 3 (Precision).
//!
//! Supports any endpoint that implements the OpenAI Chat Completions API,
//! including Anthropic's Claude (via API proxy), OpenAI GPT-4o, and any
//! self-hosted OpenAI-compatible server.
//!
//! The endpoint URL and API key are retrieved from the OS keychain / config.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{CompletionRequest, CompletionResponse, Provider, Tier};
use crate::error::CntrlError;
use crate::services::keychain;

// ─────────────────────────────────────────────────────────────────────────────
// Wire types — standard OpenAI chat completions
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct OpenAiRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
}

#[derive(Deserialize)]
struct OpenAiMessage {
    content: String,
}

#[derive(Deserialize)]
struct OpenAiUsage {
    total_tokens: Option<u32>,
}

#[derive(Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
    usage: Option<OpenAiUsage>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Provider implementation
// ─────────────────────────────────────────────────────────────────────────────

/// Generic OpenAI-compatible Tier 3 provider.
pub struct OpenAiCompatProvider {
    client: Client,
    /// Full endpoint URL, e.g. `"https://api.openai.com/v1/chat/completions"`.
    endpoint: String,
    /// Model identifier, e.g. `"gpt-4o"` or `"claude-3-opus-20240229"`.
    model: String,
}

impl OpenAiCompatProvider {
    /// Creates a new `OpenAiCompatProvider`.
    ///
    /// # Arguments
    /// * `endpoint` – The full chat-completions endpoint URL.
    /// * `model`    – The model identifier to pass in requests.
    #[must_use]
    pub fn new(endpoint: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            endpoint: endpoint.into(),
            model: model.into(),
        }
    }
}

#[async_trait]
impl Provider for OpenAiCompatProvider {
    fn name(&self) -> &str {
        "OpenAI-Compatible"
    }

    fn tier(&self) -> Tier {
        Tier::Premium
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, CntrlError> {
        let api_key = keychain::retrieve_secret(keychain::KEY_OPENAI_COMPAT)?;

        let mut messages = vec![];
        if let Some(system) = &req.system {
            messages.push(ChatMessage {
                role: "system",
                content: system.as_str(),
            });
        }
        messages.push(ChatMessage {
            role: "user",
            content: &req.prompt,
        });

        let body = OpenAiRequest {
            model: &self.model,
            messages,
        };

        let res = self
            .client
            .post(&self.endpoint)
            .bearer_auth(&api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| CntrlError::Ai(format!("OpenAI-compat connection error: {e}")))?;

        if !res.status().is_success() {
            let status = res.status();
            let err_text = res.text().await.unwrap_or_default();
            return Err(CntrlError::Ai(format!(
                "OpenAI-compat API error {status}: {err_text}"
            )));
        }

        let data: OpenAiResponse = res
            .json()
            .await
            .map_err(|e| CntrlError::Ai(format!("OpenAI-compat JSON parse error: {e}")))?;

        let text = data
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default();

        if text.is_empty() {
            return Err(CntrlError::Ai(
                "OpenAI-compat endpoint returned an empty response".to_string(),
            ));
        }

        let tokens_used = data.usage.and_then(|u| u.total_tokens);

        Ok(CompletionResponse {
            text,
            tokens_used,
            provider: self.name().to_string(),
        })
    }

    async fn health_check(&self) -> bool {
        keychain::secret_exists(keychain::KEY_OPENAI_COMPAT)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_metadata() {
        let p = OpenAiCompatProvider::new("https://api.openai.com/v1/chat/completions", "gpt-4o");
        assert_eq!(p.name(), "OpenAI-Compatible");
        assert_eq!(p.tier(), Tier::Premium);
    }

    /// Verifies the request body serialises to the standard OpenAI format.
    #[test]
    fn request_body_standard_format() {
        let messages = vec![
            ChatMessage {
                role: "system",
                content: "You are a helpful assistant.",
            },
            ChatMessage {
                role: "user",
                content: "Hello",
            },
        ];
        let req = OpenAiRequest {
            model: "gpt-4o",
            messages,
        };
        let json = serde_json::to_value(&req).expect("should serialise");
        assert_eq!(json["model"], "gpt-4o");
        assert_eq!(json["messages"][0]["role"], "system");
        assert_eq!(json["messages"][1]["role"], "user");
        assert_eq!(json["messages"][1]["content"], "Hello");
    }

    #[test]
    fn endpoint_stored_correctly() {
        let endpoint = "https://api.anthropic.com/v1/messages";
        let p = OpenAiCompatProvider::new(endpoint, "claude-3-opus");
        assert_eq!(p.endpoint, endpoint);
        assert_eq!(p.model, "claude-3-opus");
    }
}
