use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{CompletionRequest, CompletionResponse, Provider, Tier};
use crate::error::CntrlError;
use crate::services::keychain;

const HF_API_BASE: &str = "https://api-inference.huggingface.co/models";
const HF_MODELS_LIST_URL: &str =
    "https://huggingface.co/api/models?pipeline_tag=text-generation&sort=downloads&direction=-1&limit=50";

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

#[derive(Deserialize)]
struct HfTextGenItem {
    generated_text: String,
}

#[derive(Deserialize)]
struct HfModelEntry {
    id: String,
}

pub struct HuggingFaceProvider {
    client: Client,
    model_id: String,
}

impl HuggingFaceProvider {
    #[must_use]
    pub fn new(model_id: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            model_id: model_id.into(),
        }
    }

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
            tokens_used: None,
            provider: self.name().to_string(),
        })
    }

    async fn health_check(&self) -> bool {
        keychain::secret_exists(keychain::KEY_HF_TOKEN)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_metadata() {
        let p = HuggingFaceProvider::new("mistralai/Mistral-7B-Instruct-v0.2");
        assert_eq!(p.name(), "HuggingFace");
        assert_eq!(p.tier(), Tier::Freemium);
    }

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

    #[test]
    fn system_prompt_prepended() {
        let system = "Be concise.";
        let prompt = "What is 2+2?";
        let full = format!("{system}\n\n{prompt}");
        assert!(full.starts_with("Be concise.\n\nWhat is 2+2?"));
    }

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
