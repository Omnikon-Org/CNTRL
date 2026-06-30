//! OpenRouter provider — routes to hundreds of models via the OpenRouter API.
//!
//! Tier 2 (Freemium) when using free models, Tier 3 (Premium) for paid.
//! Uses the OpenAI-compatible chat completions format.

use reqwest::Client;

use super::{CompletionRequest, CompletionResponse, Provider, Tier};
use crate::error::CntrlError;

/// OpenRouter provider wrapping the OpenAI-compatible endpoint.
pub struct OpenRouterProvider {
    /// Bearer token (held in memory only).
    api_key: String,
    /// Selected model ID (e.g. `meta-llama/llama-3-8b-instruct:free`).
    model: String,
    /// Which tier: Freemium for free models, Premium for paid.
    provider_tier: Tier,
    client: Client,
}

impl OpenRouterProvider {
    /// Create a new OpenRouter provider.
    ///
    /// # Arguments
    /// * `api_key` — OpenRouter bearer token.
    /// * `model` — Model identifier.
    /// * `tier` — Freemium or Premium depending on model pricing.
    pub fn new(api_key: String, model: String, tier: Tier) -> Self {
        Self {
            api_key,
            model,
            provider_tier: tier,
            client: Client::new(),
        }
    }

    /// Fetch the full model list from OpenRouter.
    pub async fn fetch_models(client: &Client) -> Result<Vec<OpenRouterModel>, CntrlError> {
        let res = client
            .get("https://openrouter.ai/api/v1/models")
            .send()
            .await
            .map_err(|e| CntrlError::Ai(format!("OpenRouter model list error: {e}")))?;

        let data: serde_json::Value = res
            .json()
            .await
            .map_err(|e| CntrlError::Ai(format!("OpenRouter JSON error: {e}")))?;

        let mut models = Vec::new();
        if let Some(arr) = data["data"].as_array() {
            for item in arr {
                let id = item["id"].as_str().unwrap_or_default().to_string();
                let name = item["name"].as_str().unwrap_or(&id).to_string();
                let context_length = item["context_length"].as_u64().unwrap_or(0) as u32;
                let prompt_price = item["pricing"]["prompt"]
                    .as_str()
                    .unwrap_or("0")
                    .to_string();
                let is_free = prompt_price == "0";

                models.push(OpenRouterModel {
                    id,
                    name,
                    context_length,
                    prompt_price,
                    is_free,
                });
            }
        }
        Ok(models)
    }

    /// Fetch only free models from OpenRouter.
    pub async fn fetch_free_models(client: &Client) -> Result<Vec<String>, CntrlError> {
        let models = Self::fetch_models(client).await?;
        Ok(models
            .into_iter()
            .filter(|m| m.is_free)
            .map(|m| m.id)
            .collect())
    }
}

/// Metadata about an OpenRouter model.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OpenRouterModel {
    /// Model identifier.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Maximum context window size.
    pub context_length: u32,
    /// Price per prompt token as a string.
    pub prompt_price: String,
    /// Whether this model is free to use.
    pub is_free: bool,
}

#[async_trait::async_trait]
impl Provider for OpenRouterProvider {
    fn name(&self) -> &str {
        "OpenRouter"
    }

    fn tier(&self) -> Tier {
        self.provider_tier
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, CntrlError> {
        let mut messages = Vec::new();

        if let Some(ref system) = req.system {
            messages.push(serde_json::json!({"role": "system", "content": system}));
        }

        let mut user_content = String::new();
        if let Some(ref context) = req.context {
            user_content.push_str("Context:\n");
            user_content.push_str(context);
            user_content.push_str("\n\n");
        }
        user_content.push_str(&req.prompt);
        messages.push(serde_json::json!({"role": "user", "content": user_content}));

        let mut body = serde_json::json!({
            "model": self.model,
            "messages": messages,
        });

        if let Some(max_tokens) = req.max_tokens {
            body["max_tokens"] = serde_json::json!(max_tokens);
        }

        let res = self
            .client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .header("HTTP-Referer", "https://cntrl-browser.app")
            .header("X-Title", "CNTRL Browser")
            .json(&body)
            .send()
            .await
            .map_err(|e| CntrlError::Ai(format!("OpenRouter HTTP error: {e}")))?;

        if !res.status().is_success() {
            let status = res.status();
            let err_text = res.text().await.unwrap_or_default();
            return Err(CntrlError::Ai(format!(
                "OpenRouter API error {status}: {err_text}"
            )));
        }

        let data: serde_json::Value = res
            .json()
            .await
            .map_err(|e| CntrlError::Ai(format!("OpenRouter JSON parse error: {e}")))?;

        let text = data["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("No response generated")
            .to_string();

        let tokens_used = data["usage"]["total_tokens"].as_u64().map(|n| n as u32);

        Ok(CompletionResponse {
            text,
            tokens_used,
            model: self.model.clone(),
            tier: self.provider_tier,
        })
    }

    async fn health_check(&self) -> bool {
        self.client
            .get("https://openrouter.ai/api/v1/models")
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_metadata() {
        let p = OpenRouterProvider::new("sk-test".into(), "test-model".into(), Tier::Freemium);
        assert_eq!(p.name(), "OpenRouter");
        assert_eq!(p.tier(), Tier::Freemium);
    }
}
