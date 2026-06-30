//! Groq provider — fast inference via the Groq cloud API.
//!
//! Tier 2 (Freemium). Uses the OpenAI-compatible chat completions format.

use reqwest::Client;

use super::{CompletionRequest, CompletionResponse, Provider, Tier};
use crate::error::CntrlError;

/// Groq provider using their OpenAI-compatible endpoint.
pub struct GroqProvider {
    /// Groq API key (held in memory only).
    api_key: String,
    /// Model identifier (e.g. `llama3-8b-8192`).
    model: String,
    client: Client,
}

impl GroqProvider {
    /// Create a new Groq provider.
    ///
    /// # Arguments
    /// * `api_key` — Groq API key.
    /// * `model` — Model identifier.
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            client: Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl Provider for GroqProvider {
    fn name(&self) -> &str {
        "Groq"
    }

    fn tier(&self) -> Tier {
        Tier::Freemium
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
            .post("https://api.groq.com/openai/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| CntrlError::Ai(format!("Groq HTTP error: {e}")))?;

        if !res.status().is_success() {
            let status = res.status();
            let err_text = res.text().await.unwrap_or_default();
            return Err(CntrlError::Ai(format!(
                "Groq API error {status}: {err_text}"
            )));
        }

        let data: serde_json::Value = res
            .json()
            .await
            .map_err(|e| CntrlError::Ai(format!("Groq JSON parse error: {e}")))?;

        let text = data["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("No response generated")
            .to_string();

        let tokens_used = data["usage"]["total_tokens"].as_u64().map(|n| n as u32);

        Ok(CompletionResponse {
            text,
            tokens_used,
            model: self.model.clone(),
            tier: Tier::Freemium,
        })
    }

    async fn health_check(&self) -> bool {
        self.client
            .get("https://api.groq.com/openai/v1/models")
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
        let p = GroqProvider::new("test-key".into(), "llama3-8b-8192".into());
        assert_eq!(p.name(), "Groq");
        assert_eq!(p.tier(), Tier::Freemium);
    }
}
