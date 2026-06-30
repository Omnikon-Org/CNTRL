//! Google Gemini provider — uses the Gemini REST API.
//!
//! Tier 2 (Freemium). The free tier of Gemini Flash is sufficient for
//! most browser-automation tasks.

use reqwest::Client;
use serde_json::json;

use super::{CompletionRequest, CompletionResponse, Provider, Tier};
use crate::error::CntrlError;

/// Google Gemini provider using the `generateContent` REST API.
pub struct GeminiProvider {
    /// Gemini API key (held in memory only).
    api_key: String,
    /// Model identifier (e.g. `gemini-2.0-flash`).
    model: String,
    client: Client,
}

impl GeminiProvider {
    /// Create a new Gemini provider.
    ///
    /// # Arguments
    /// * `api_key` — Gemini API key.
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
impl Provider for GeminiProvider {
    fn name(&self) -> &str {
        "Gemini"
    }

    fn tier(&self) -> Tier {
        Tier::Freemium
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, CntrlError> {
        let mut parts = Vec::new();

        if let Some(ref system) = req.system {
            parts.push(json!({"text": system}));
        }
        if let Some(ref context) = req.context {
            parts.push(json!({"text": format!("Context:\n{context}")}));
        }
        parts.push(json!({"text": req.prompt}));

        let body = json!({
            "contents": [{
                "parts": parts,
            }],
        });

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let res = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| CntrlError::Ai(format!("Gemini HTTP error: {e}")))?;

        if !res.status().is_success() {
            let status = res.status();
            let err_text = res.text().await.unwrap_or_default();
            return Err(CntrlError::Ai(format!(
                "Gemini API error {status}: {err_text}"
            )));
        }

        let data: serde_json::Value = res
            .json()
            .await
            .map_err(|e| CntrlError::Ai(format!("Gemini JSON parse error: {e}")))?;

        let text = data["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("No response generated")
            .to_string();

        let tokens_used = data["usageMetadata"]["totalTokenCount"]
            .as_u64()
            .map(|n| n as u32);

        Ok(CompletionResponse {
            text,
            tokens_used,
            model: self.model.clone(),
            tier: Tier::Freemium,
        })
    }

    async fn health_check(&self) -> bool {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models?key={}",
            self.api_key
        );
        self.client
            .get(&url)
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
        let p = GeminiProvider::new("test-key".into(), "gemini-2.0-flash".into());
        assert_eq!(p.name(), "Gemini");
        assert_eq!(p.tier(), Tier::Freemium);
    }
}
