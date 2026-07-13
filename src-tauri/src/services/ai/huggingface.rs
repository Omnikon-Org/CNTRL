//! HuggingFace Inference API provider — Tier 2 (Freemium).
//!
//! Calls the HuggingFace Inference API for both text-generation and
//! conversational pipelines. The HF access token is retrieved from the OS
//! keychain on each request.
//!
//! API reference: <https://huggingface.co/docs/api-inference/en/index>

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::CntrlError;
use crate::services::keychain;
use super::{CompletionRequest, CompletionResponse, Provider, Tier};

const HF_API_BASE: &str = "https://api-inference.huggingface.co/models";
const HF_MODELS_LIST_URL: &str =
    "https://huggingface.co/api/models?pipeline_tag=text-generation&sort=downloads&direction=-1&limit=50";

// ─────────────────────────────────────────────────────────────────────────────
// Wire types
// ─────────────────────────────────────────────────────────────────────────────

/// Request body for the HF text-generation pipeline.
#[derive(Serialize)]
struct HfTextGenRequest<'a> {
    inputs: &'a str,
    parameters: HfTextGenParameters,
}

#[derive(Serialize)]
struct HfTextGenParameters {
    max_new_tokens: u32,
    return_full_text: bool,
}

/// Single item in the HF text-generation response array.
#[derive(Deserialize)]
struct HfTextGenItem {
    generated_text: String,
}

/// Model metadata from the HF model list endpoint.
#[derive(Deserialize)]
struct HfModelEntry {
    id: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Provider implementation
// ─────────────────────────────────────────────────────────────────────────────

/// HuggingFace Inference API provider (Tier 2).
pub struct HuggingFaceProvider {
    client: Client,
    /// HF model ID, e.g. `"mistralai/Mistral-7B-Instruct-v0.2"`.
    model_id: String,
}

impl HuggingFaceProvider {
    /// Creates a new `HuggingFaceProvider`.
    ///
    /// # Arguments
    /// * `model_id` – The full HuggingFace model repository ID.
    #[must_use]
    pub fn new(model_id: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            model_id: model_id.into(),
        }
    }

    /// Fetches the list of popular text-generation models from the HF API.
    /// Does not require authentication.
    ///
    /// # Errors
    /// Returns [`CntrlError::Ai`] on network or parse failure.
    pub async fn fetch_model_list(&self) -> Result<Vec<String>, CntrlError> {
        let res = self
            .client
            .get(HF_MODELS_LIST_URL)
            .send()
            .await
            .map_err(|e| CntrlError::Ai(format!("HF model list fetch error: {e}")))?;

        let entries: Vec<HfModelEntry> = res
            .json()
            .await
            .map_err(|e| CntrlError::Ai(format!("HF model list parse error: {e}")))?;

        Ok(entries.into_iter().map(|m| m.id).collect())
    }
}

#[async_trait]
impl Provider for HuggingFaceProvider {
    fn name(&self) -> &str {
        "HuggingFace"
    }

    fn tier(&self) -> Tier {
        Tier::Freemium
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, CntrlError> {
        let token = keychain::retrieve_secret(keychain::KEY_HF_TOKEN)?;

        let url = format!("{HF_API_BASE}/{}", self.model_id);

        // Prepend system context if provided
        let full_prompt = match &req.system {
            Some(sys) => format!("{sys}\n\n{}", req.prompt),
            None => req.prompt.clone(),
        };

        let body = HfTextGenRequest {
            inputs: &full_prompt,
            parameters: HfTextGenParameters {
                max_new_tokens: 512,
                return_full_text: false,
            },
        };

        let res = self
            .client
            .post(&url)
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await
            .map_err(|e| CntrlError::Ai(format!("HuggingFace connection error: {e}")))?;

        if !res.status().is_success() {
            let status = res.status();
            let err_text = res.text().await.unwrap_or_default();
            return Err(CntrlError::Ai(format!(
                "HuggingFace API error {status}: {err_text}"
            )));
        }

        let items: Vec<HfTextGenItem> = res
            .json()
            .await
            .map_err(|e| CntrlError::Ai(format!("HuggingFace JSON parse error: {e}")))?;

        let text = items
            .into_iter()
            .next()
            .map(|i| i.generated_text)
            .unwrap_or_default();

        if text.is_empty() {
            return Err(CntrlError::Ai(
                "HuggingFace returned an empty response".to_string(),
            ));
        }

        Ok(CompletionResponse {
            text,
            tokens_used: None, // HF inference API doesn't return token counts
            provider: self.name().to_string(),
        })
    }

    async fn health_check(&self) -> bool {
        keychain::secret_exists(keychain::KEY_HF_TOKEN)
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
        let p = HuggingFaceProvider::new("mistralai/Mistral-7B-Instruct-v0.2");
        assert_eq!(p.name(), "HuggingFace");
        assert_eq!(p.tier(), Tier::Freemium);
    }

    /// Verifies the text-generation request body structure.
    #[test]
    fn request_body_structure() {
        let req = HfTextGenRequest {
            inputs: "Hello, world",
            parameters: HfTextGenParameters {
                max_new_tokens: 512,
                return_full_text: false,
            },
        };
        let json = serde_json::to_value(&req).expect("should serialise");
        assert_eq!(json["inputs"], "Hello, world");
        assert_eq!(json["parameters"]["max_new_tokens"], 512);
        assert_eq!(json["parameters"]["return_full_text"], false);
    }

    /// Verifies that system context is prepended to the prompt correctly.
    #[test]
    fn system_prompt_prepended() {
        let system = "Be concise.";
        let prompt = "What is 2+2?";
        let full = format!("{system}\n\n{prompt}");
        assert!(full.starts_with("Be concise.\n\nWhat is 2+2?"));
    }

    /// Verifies the URL is constructed correctly.
    #[test]
    fn inference_url_is_correct() {
        let model_id = "mistralai/Mistral-7B-Instruct-v0.2";
        let expected = format!("{HF_API_BASE}/{model_id}");
        assert_eq!(
            expected,
            "https://api-inference.huggingface.co/models/mistralai/Mistral-7B-Instruct-v0.2"
        );
    }
}
