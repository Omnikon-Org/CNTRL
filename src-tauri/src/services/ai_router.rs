use parking_lot::RwLock;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use crate::error::CntrlError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelTier {
    Local,
    Freemium, // OpenRouter etc.
    Premium,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub tier: ModelTier,
    pub openrouter_key: Option<String>,
    pub ollama_url: String,
    pub selected_model: String,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            tier: ModelTier::Freemium,
            openrouter_key: None,
            ollama_url: "http://localhost:11434".to_string(),
            selected_model: "meta-llama/llama-3-8b-instruct:free".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct AiRouter {
    config: Arc<RwLock<ModelConfig>>,
    client: Client,
    storage_path: PathBuf,
}

impl AiRouter {
    pub fn new(storage_path: PathBuf) -> Self {
        // Ensure directory exists
        if let Some(parent) = storage_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let key_exists = storage_path.exists();

        let mut default_config = ModelConfig::default();
        if key_exists {
            default_config.openrouter_key = Some("sk-or-***".to_string());
        }

        Self {
            config: Arc::new(RwLock::new(default_config)),
            client: Client::new(),
            storage_path,
        }
    }

    pub fn update_config(&self, mut new_config: ModelConfig) {
        if let Some(key) = &new_config.openrouter_key {
            if key != "sk-or-***" {
                let _ = fs::write(&self.storage_path, key);
            }
        }

        if self.storage_path.exists() {
            new_config.openrouter_key = Some("sk-or-***".to_string());
        } else {
            new_config.openrouter_key = None;
        }

        let mut config = self.config.write();
        *config = new_config;
    }

    pub fn get_config(&self) -> ModelConfig {
        self.config.read().clone()
    }

    fn get_openrouter_key(&self) -> Result<String, CntrlError> {
        fs::read_to_string(&self.storage_path)
            .map_err(|_| CntrlError::Ai("OpenRouter API key not found in storage".to_string()))
    }

    pub async fn ask_model(&self, prompt: String) -> Result<String, CntrlError> {
        let config = self.config.read().clone();

        match config.tier {
            ModelTier::Local => self.call_ollama(&config, prompt).await,
            ModelTier::Freemium => self.call_openrouter(&config, prompt).await,
            ModelTier::Premium => self.call_openrouter(&config, prompt).await,
        }
    }

    async fn call_openrouter(
        &self,
        config: &ModelConfig,
        prompt: String,
    ) -> Result<String, CntrlError> {
        let key = self.get_openrouter_key()?;

        let body = json!({
            "model": config.selected_model,
            "messages": [
                {"role": "user", "content": prompt}
            ]
        });

        let res = self
            .client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .bearer_auth(key)
            .json(&body)
            .send()
            .await
            .map_err(|e| CntrlError::Ai(format!("HTTP error: {}", e)))?;

        if !res.status().is_success() {
            let status = res.status();
            let err_text = res.text().await.unwrap_or_else(|_| "".to_string());
            return Err(CntrlError::Ai(format!(
                "API error {}: {}",
                status, err_text
            )));
        }

        let data: serde_json::Value = res
            .json()
            .await
            .map_err(|e| CntrlError::Ai(format!("JSON parsing error: {}", e)))?;

        let content = data["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("No response generated")
            .to_string();

        Ok(content)
    }

    async fn call_ollama(
        &self,
        config: &ModelConfig,
        prompt: String,
    ) -> Result<String, CntrlError> {
        let body = json!({
            "model": config.selected_model.clone(),
            "prompt": prompt,
            "stream": false
        });

        let url = format!("{}/api/generate", config.ollama_url.trim_end_matches('/'));

        let res = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| CntrlError::Ai(format!("Ollama connection error: {}", e)))?;

        if !res.status().is_success() {
            return Err(CntrlError::Ai(format!("Ollama error: {}", res.status())));
        }

        let data: serde_json::Value = res
            .json()
            .await
            .map_err(|e| CntrlError::Ai(format!("JSON parsing error: {}", e)))?;

        let content = data["response"]
            .as_str()
            .unwrap_or("No response generated")
            .to_string();

        Ok(content)
    }

    pub fn score_intent(&self, intent: &str) -> ModelTier {
        let lower = intent.to_lowercase();
        if lower.contains("offline") || lower.contains("private") || lower.contains("local") {
            ModelTier::Local
        } else if lower.contains("code")
            || lower.contains("analyze")
            || lower.contains("complex")
            || lower.contains("reason")
        {
            ModelTier::Premium
        } else {
            ModelTier::Freemium
        }
    }

    pub fn score_sample_intents(&self, intents: Vec<String>) -> Vec<(String, ModelTier)> {
        intents
            .into_iter()
            .map(|intent| {
                let tier = self.score_intent(&intent);
                (intent, tier)
            })
            .collect()
    }

    pub async fn fetch_hf_models(&self) -> Result<Vec<String>, CntrlError> {
        let res = self.client.get("https://huggingface.co/api/models?pipeline_tag=text-generation&sort=downloads&direction=-1&limit=50")
            .send()
            .await
            .map_err(|e| CntrlError::Ai(format!("HTTP error: {}", e)))?;

        let data: serde_json::Value = res
            .json()
            .await
            .map_err(|e| CntrlError::Ai(format!("JSON parsing error: {}", e)))?;

        let mut models = Vec::new();
        if let Some(arr) = data.as_array() {
            for item in arr {
                if let Some(id) = item["id"].as_str() {
                    models.push(id.to_string());
                }
            }
        }
        Ok(models)
    }

    pub async fn fetch_openrouter_free_models(&self) -> Result<Vec<String>, CntrlError> {
        let res = self
            .client
            .get("https://openrouter.ai/api/v1/models")
            .send()
            .await
            .map_err(|e| CntrlError::Ai(format!("HTTP error: {}", e)))?;

        let data: serde_json::Value = res
            .json()
            .await
            .map_err(|e| CntrlError::Ai(format!("JSON parsing error: {}", e)))?;

        let mut models = Vec::new();
        if let Some(arr) = data["data"].as_array() {
            for item in arr {
                let prompt_pricing = item["pricing"]["prompt"].as_str().unwrap_or("1");
                if prompt_pricing == "0" {
                    if let Some(id) = item["id"].as_str() {
                        models.push(id.to_string());
                    }
                }
            }
        }
        Ok(models)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn router() -> AiRouter {
        AiRouter::new(PathBuf::from("/tmp/test-cntrl-key-do-not-use"))
    }

    #[test]
    fn offline_intent_routes_to_local() {
        assert_eq!(
            router().score_intent("search offline maps"),
            ModelTier::Local
        );
    }

    #[test]
    fn private_intent_routes_to_local() {
        assert_eq!(router().score_intent("browse privately"), ModelTier::Local);
    }

    #[test]
    fn local_intent_routes_to_local() {
        assert_eq!(
            router().score_intent("run a local server"),
            ModelTier::Local
        );
    }

    #[test]
    fn code_intent_routes_to_premium() {
        assert_eq!(
            router().score_intent("write code for a linked list"),
            ModelTier::Premium
        );
    }

    #[test]
    fn analyze_intent_routes_to_premium() {
        assert_eq!(
            router().score_intent("analyze this dataset"),
            ModelTier::Premium
        );
    }

    #[test]
    fn complex_intent_routes_to_premium() {
        assert_eq!(
            router().score_intent("solve this complex math problem"),
            ModelTier::Premium
        );
    }

    #[test]
    fn reason_intent_routes_to_premium() {
        assert_eq!(
            router().score_intent("reason through this argument"),
            ModelTier::Premium
        );
    }

    #[test]
    fn general_intent_routes_to_freemium() {
        assert_eq!(
            router().score_intent("find a good recipe for pasta"),
            ModelTier::Freemium
        );
    }

    #[test]
    fn empty_intent_routes_to_freemium() {
        assert_eq!(router().score_intent(""), ModelTier::Freemium);
    }

    #[test]
    fn case_insensitive_local_routing() {
        assert_eq!(
            router().score_intent("OFFLINE mode please"),
            ModelTier::Local
        );
    }
}
