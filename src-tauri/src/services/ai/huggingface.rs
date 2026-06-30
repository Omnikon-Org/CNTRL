//! Hugging Face Inference API provider.
//!
//! Tier 2 (Freemium). Supports both text-generation and conversational
//! pipeline types via the HF Inference API.

use reqwest::Client;
use serde_json::json;

use super::{CompletionRequest, CompletionResponse, Provider, Tier};
use crate::error::CntrlError;

/// Hugging Face Inference API provider.
pub struct HuggingFaceProvider {
    /// HF user token (held in memory only).
    api_token: String,
    /// Model identifier (e.g. `mistralai/Mistral-7B-Instruct-v0.2`).
    model: String,
    client: Client,
}

impl HuggingFaceProvider {
    /// Create a new Hugging Face provider.
    ///
    /// # Arguments
    /// * `api_token` — HF bearer token.
    /// * `model` — Model identifier on HF Hub.
    pub fn new(api_token: String, model: String) -> Self {
        Self {
            api_token,
            model,
            client: Client::new(),
        }
    }

    /// Fetch popular text-generation models from HF Hub.
    pub async fn fetch_models(client: &Client) -> Result<Vec<HfModelInfo>, CntrlError> {
        let res = client
            .get("https://huggingface.co/api/models?pipeline_tag=text-generation&sort=downloads&direction=-1&limit=50")
            .send()
            .await
            .map_err(|e| CntrlError::Ai(format!("HF model list error: {e}")))?;

        let data: serde_json::Value = res
            .json()
            .await
            .map_err(|e| CntrlError::Ai(format!("HF JSON error: {e}")))?;

        let mut models = Vec::new();
        if let Some(arr) = data.as_array() {
            for item in arr {
                let id = item["id"].as_str().unwrap_or_default().to_string();
                let downloads = item["downloads"].as_u64().unwrap_or(0);
                models.push(HfModelInfo { id, downloads });
            }
        }
        Ok(models)
    }
}

/// Metadata about a Hugging Face model.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HfModelInfo {
    /// Model identifier on HF Hub.
    pub id: String,
    /// Total download count.
    pub downloads: u64,
}

#[async_trait::async_trait]
impl Provider for HuggingFaceProvider {
    fn name(&self) -> &str {
        "Hugging Face"
    }

    fn tier(&self) -> Tier {
        Tier::Freemium
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
            "inputs": prompt,
            "parameters": {
                "max_new_tokens": req.max_tokens.unwrap_or(512),
                "return_full_text": false,
            },
        });

        let url = format!(
            "https://api-inference.huggingface.co/models/{}",
            self.model
        );

        let res = self
            .client
            .post(&url)
            .bearer_auth(&self.api_token)
            .json(&body)
            .send()
            .await
            .map_err(|e| CntrlError::Ai(format!("HF HTTP error: {e}")))?;

        if !res.status().is_success() {
            let status = res.status();
            let err_text = res.text().await.unwrap_or_default();
            return Err(CntrlError::Ai(format!(
                "HF API error {status}: {err_text}"
            )));
        }

        let data: serde_json::Value = res
            .json()
            .await
            .map_err(|e| CntrlError::Ai(format!("HF JSON parse error: {e}")))?;

        // HF inference returns an array with generated_text
        let text = if let Some(arr) = data.as_array() {
            arr.first()
                .and_then(|v| v["generated_text"].as_str())
                .unwrap_or("No response generated")
                .to_string()
        } else {
            data["generated_text"]
                .as_str()
                .unwrap_or("No response generated")
                .to_string()
        };

        Ok(CompletionResponse {
            text,
            tokens_used: None, // HF doesn't always return token counts
            model: self.model.clone(),
            tier: Tier::Freemium,
        })
    }

    async fn health_check(&self) -> bool {
        let url = format!(
            "https://api-inference.huggingface.co/models/{}",
            self.model
        );
        self.client
            .get(&url)
            .bearer_auth(&self.api_token)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map(|r| r.status().is_success() || r.status().as_u16() == 503)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_metadata() {
        let p = HuggingFaceProvider::new("hf_test".into(), "test-model".into());
        assert_eq!(p.name(), "Hugging Face");
        assert_eq!(p.tier(), Tier::Freemium);
    }
}
