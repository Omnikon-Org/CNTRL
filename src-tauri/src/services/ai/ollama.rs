//! Ollama provider — connects to a local Ollama instance on localhost.
//!
//! Tier 1 (Local). Zero API key required. If Ollama is not running the
//! health check returns `false` and the router falls back to Tier 2.

use reqwest::Client;
use serde_json::json;

use super::{CompletionRequest, CompletionResponse, Provider, Tier};
use crate::error::CntrlError;

/// Ollama provider targeting `http://localhost:11434` by default.
pub struct OllamaProvider {
    /// Base URL of the Ollama API (e.g. `http://localhost:11434`).
    pub base_url: String,
    /// Model name to use (e.g. `llama3`).
    pub model: String,
    client: Client,
}

impl OllamaProvider {
    /// Create a new Ollama provider.
    ///
    /// # Arguments
    /// * `base_url` — Ollama API base URL.
    /// * `model` — Model name to use for completions.
    pub fn new(base_url: String, model: String) -> Self {
        Self {
            base_url,
            model,
            client: Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl Provider for OllamaProvider {
    fn name(&self) -> &str {
        "Ollama"
    }

    fn tier(&self) -> Tier {
        Tier::Local
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, CntrlError> {
        let mut prompt = String::new();
        if let Some(ref system) = req.system {
            prompt.push_str(system);
            prompt.push_str("\n\n");
        }
        if let Some(ref context) = req.context {
            prompt.push_str("Context:\n");
            prompt.push_str(context);
            prompt.push_str("\n\n");
        }
        prompt.push_str(&req.prompt);

        let body = json!({
            "model": self.model,
            "prompt": prompt,
            "stream": false,
        });

        let url = format!(
            "{}/api/generate",
            self.base_url.trim_end_matches('/')
        );

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

        let data: serde_json::Value = res
            .json()
            .await
            .map_err(|e| CntrlError::Ai(format!("Ollama JSON parse error: {e}")))?;

        let text = data["response"]
            .as_str()
            .unwrap_or("No response generated")
            .to_string();

        Ok(CompletionResponse {
            text,
            tokens_used: data["eval_count"].as_u64().map(|n| n as u32),
            model: self.model.clone(),
            tier: Tier::Local,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_metadata() {
        let p = OllamaProvider::new("http://localhost:11434".into(), "llama3".into());
        assert_eq!(p.name(), "Ollama");
        assert_eq!(p.tier(), Tier::Local);
    }
}
