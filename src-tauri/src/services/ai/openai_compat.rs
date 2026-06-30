//! Generic OpenAI-compatible provider.
//!
//! Works with any endpoint that follows the OpenAI chat completions API
//! format: OpenRouter, Groq, Together, Anyscale, local vLLM, etc.
//! Each concrete provider (Gemini adapter, Groq, OpenRouter) wraps this.

use reqwest::Client;
use serde_json::json;

use super::{CompletionRequest, CompletionResponse, Provider, Tier};
use crate::error::CntrlError;

/// A provider that speaks the OpenAI `/v1/chat/completions` protocol.
pub struct OpenAiCompatProvider {
    /// Human-readable name shown in the UI.
    display_name: String,
    /// Full endpoint URL (e.g. `https://openrouter.ai/api/v1/chat/completions`).
    endpoint: String,
    /// Bearer token for authentication (held in memory, never persisted).
    api_key: String,
    /// Model identifier sent in the request body.
    model: String,
    /// Tier this provider belongs to.
    provider_tier: Tier,
    client: Client,
}

impl OpenAiCompatProvider {
    /// Create a new OpenAI-compatible provider.
    ///
    /// # Arguments
    /// * `display_name` — Name shown in the settings UI.
    /// * `endpoint` — Chat completions endpoint URL.
    /// * `api_key` — Bearer token.
    /// * `model` — Model ID.
    /// * `tier` — Which complexity tier this provider serves.
    pub fn new(
        display_name: String,
        endpoint: String,
        api_key: String,
        model: String,
        tier: Tier,
    ) -> Self {
        Self {
            display_name,
            endpoint,
            api_key,
            model,
            provider_tier: tier,
            client: Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl Provider for OpenAiCompatProvider {
    fn name(&self) -> &str {
        &self.display_name
    }

    fn tier(&self) -> Tier {
        self.provider_tier
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, CntrlError> {
        let mut messages = Vec::new();

        if let Some(ref system) = req.system {
            messages.push(json!({"role": "system", "content": system}));
        }

        let mut user_content = String::new();
        if let Some(ref context) = req.context {
            user_content.push_str("Context:\n");
            user_content.push_str(context);
            user_content.push_str("\n\n");
        }
        user_content.push_str(&req.prompt);
        messages.push(json!({"role": "user", "content": user_content}));

        let mut body = json!({
            "model": self.model,
            "messages": messages,
        });

        if let Some(max_tokens) = req.max_tokens {
            body["max_tokens"] = json!(max_tokens);
        }

        let res = self
            .client
            .post(&self.endpoint)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| CntrlError::Ai(format!("{} HTTP error: {e}", self.display_name)))?;

        if !res.status().is_success() {
            let status = res.status();
            let err_text = res.text().await.unwrap_or_default();
            return Err(CntrlError::Ai(format!(
                "{} API error {status}: {err_text}",
                self.display_name
            )));
        }

        let data: serde_json::Value = res
            .json()
            .await
            .map_err(|e| CntrlError::Ai(format!("{} JSON parse error: {e}", self.display_name)))?;

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
        // Attempt a minimal request to verify connectivity.
        // We use the models endpoint if available, otherwise a tiny completion.
        let models_url = self.endpoint.replace("/chat/completions", "/models");
        self.client
            .get(&models_url)
            .bearer_auth(&self.api_key)
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
        let p = OpenAiCompatProvider::new(
            "TestProvider".into(),
            "https://example.com/v1/chat/completions".into(),
            "sk-test".into(),
            "gpt-4o".into(),
            Tier::Premium,
        );
        assert_eq!(p.name(), "TestProvider");
        assert_eq!(p.tier(), Tier::Premium);
    }
}
