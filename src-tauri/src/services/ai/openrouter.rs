//! OpenRouter provider — Tier 2 (Freemium).
//!
//! OpenRouter is an OpenAI-compatible gateway that routes to hundreds of
//! models including many free ones. The API key is retrieved from the OS
//! keychain on each request.
//!
//! API reference: <https://openrouter.ai/docs>

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{CompletionRequest, CompletionResponse, Provider, Tier};
use crate::error::CntrlError;
use crate::services::keychain;

const OPENROUTER_API_BASE: &str = "https://openrouter.ai/api/v1/chat/completions";
const OPENROUTER_MODELS_URL: &str = "https://openrouter.ai/api/v1/models";

// ─────────────────────────────────────────────────────────────────────────────
// Wire types
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct OpenRouterRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
}

#[derive(Deserialize)]
struct OrChoice {
    message: OrMessage,
}

#[derive(Deserialize)]
struct OrMessage {
    content: String,
}

#[derive(Deserialize)]
struct OrUsage {
    total_tokens: Option<u32>,
}

#[derive(Deserialize)]
struct OrResponse {
    choices: Vec<OrChoice>,
    usage: Option<OrUsage>,
}

/// Model entry from the OpenRouter models list endpoint.
#[derive(Debug, Deserialize)]
pub struct OrModelEntry {
    pub id: String,
    pub name: Option<String>,
    /// Prompt price per token (as a string, e.g. `"0"` for free).
    pub pricing: OrPricing,
}

#[derive(Debug, Deserialize)]
pub struct OrPricing {
    pub prompt: String,
}

#[derive(Deserialize)]
struct OrModelsResponse {
    data: Vec<OrModelEntry>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Provider implementation
// ─────────────────────────────────────────────────────────────────────────────

/// OpenRouter provider (Tier 2).
pub struct OpenRouterProvider {
    client: Client,
    /// Model ID used for completions, e.g. `"meta-llama/llama-3-8b-instruct:free"`.
    model: String,
}

impl OpenRouterProvider {
    /// Creates a new `OpenRouterProvider`.
    ///
    /// # Arguments
    /// * `model` – The OpenRouter model identifier.
    #[must_use]
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            model: model.into(),
        }
    }

    /// Returns the list of all free models on OpenRouter (prompt price = `"0"`).
    ///
    /// # Errors
    /// Returns [`CntrlError::Ai`] on network or parse failure.
    pub async fn fetch_free_models(&self) -> Result<Vec<String>, CntrlError> {
        let res = self
            .client
            .get(OPENROUTER_MODELS_URL)
            .send()
            .await
            .map_err(|e| CntrlError::Ai(format!("OpenRouter models fetch error: {e}")))?;

        let data: OrModelsResponse = res
            .json()
            .await
            .map_err(|e| CntrlError::Ai(format!("OpenRouter models parse error: {e}")))?;

        let free_models = data
            .data
            .into_iter()
            .filter(|m| m.pricing.prompt == "0")
            .map(|m| m.id)
            .collect();

        Ok(free_models)
    }
}

#[async_trait]
impl Provider for OpenRouterProvider {
    fn name(&self) -> &str {
        "OpenRouter"
    }

    fn tier(&self) -> Tier {
        Tier::Freemium
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, CntrlError> {
        let api_key = keychain::retrieve_secret(keychain::KEY_OPENROUTER)?;

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

        let body = OpenRouterRequest {
            model: &self.model,
            messages,
        };

        let res = self
            .client
            .post(OPENROUTER_API_BASE)
            .bearer_auth(&api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| CntrlError::Ai(format!("OpenRouter connection error: {e}")))?;

        if !res.status().is_success() {
            let status = res.status();
            let err_text = res.text().await.unwrap_or_default();
            return Err(CntrlError::Ai(format!(
                "OpenRouter API error {status}: {err_text}"
            )));
        }

        let data: OrResponse = res
            .json()
            .await
            .map_err(|e| CntrlError::Ai(format!("OpenRouter JSON parse error: {e}")))?;

        let text = data
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default();

        if text.is_empty() {
            return Err(CntrlError::Ai(
                "OpenRouter returned an empty response".to_string(),
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
        keychain::secret_exists(keychain::KEY_OPENROUTER)
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
        let p = OpenRouterProvider::new("meta-llama/llama-3-8b-instruct:free");
        assert_eq!(p.name(), "OpenRouter");
        assert_eq!(p.tier(), Tier::Freemium);
    }

    /// Verifies the request body is OpenAI-compatible.
    #[test]
    fn request_body_openai_compatible() {
        let messages = vec![ChatMessage {
            role: "user",
            content: "Hello",
        }];
        let body = OpenRouterRequest {
            model: "meta-llama/llama-3-8b-instruct:free",
            messages,
        };
        let json = serde_json::to_value(&body).expect("should serialise");
        assert_eq!(json["model"], "meta-llama/llama-3-8b-instruct:free");
        assert_eq!(json["messages"][0]["role"], "user");
    }

    /// Verifies that the free-model filter works on mock data.
    #[test]
    fn free_model_filter_logic() {
        let entries = vec![
            OrModelEntry {
                id: "free-model-1".to_string(),
                name: None,
                pricing: OrPricing {
                    prompt: "0".to_string(),
                },
            },
            OrModelEntry {
                id: "paid-model-1".to_string(),
                name: None,
                pricing: OrPricing {
                    prompt: "0.000002".to_string(),
                },
            },
            OrModelEntry {
                id: "free-model-2".to_string(),
                name: Some("Free Model Two".to_string()),
                pricing: OrPricing {
                    prompt: "0".to_string(),
                },
            },
        ];

        let free: Vec<String> = entries
            .into_iter()
            .filter(|m| m.pricing.prompt == "0")
            .map(|m| m.id)
            .collect();

        assert_eq!(free.len(), 2);
        assert!(free.contains(&"free-model-1".to_string()));
        assert!(free.contains(&"free-model-2".to_string()));
        assert!(!free.contains(&"paid-model-1".to_string()));
    }
}
