//! Complexity-scoring router that picks the best AI provider for each task.
//!
//! The router scores each intent 0-10, maps it to a tier, and selects
//! the best available provider. Falls back down the tier chain if the
//! target tier has no healthy providers.

use parking_lot::RwLock;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::gemini::GeminiProvider;
use super::groq::GroqProvider;
use super::huggingface::HuggingFaceProvider;
use super::ollama::OllamaProvider;
use super::openai_compat::OpenAiCompatProvider;
use super::openrouter::OpenRouterProvider;
use super::{CompletionRequest, CompletionResponse, Provider, ProviderInfo, Tier};
use crate::error::CntrlError;

// ── Configuration ──────────────────────────────────────────────────────

/// Per-provider configuration stored in memory. Keys are never persisted to disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    /// Active tier preference (user override).
    pub active_tier: Tier,
    /// Ollama base URL.
    pub ollama_url: String,
    /// Ollama model name.
    pub ollama_model: String,
    /// OpenRouter API key (in-memory only, shown as masked in UI).
    pub openrouter_key: Option<String>,
    /// OpenRouter selected model.
    pub openrouter_model: String,
    /// Gemini API key.
    pub gemini_key: Option<String>,
    /// Gemini model name.
    pub gemini_model: String,
    /// Groq API key.
    pub groq_key: Option<String>,
    /// Groq model name.
    pub groq_model: String,
    /// HuggingFace token.
    pub hf_token: Option<String>,
    /// HuggingFace model ID.
    pub hf_model: String,
    /// Custom Tier 3 endpoint URL.
    pub custom_endpoint: Option<String>,
    /// Custom Tier 3 API key.
    pub custom_key: Option<String>,
    /// Custom Tier 3 model.
    pub custom_model: String,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            active_tier: Tier::Freemium,
            ollama_url: "http://localhost:11434".to_string(),
            ollama_model: "llama3".to_string(),
            openrouter_key: None,
            openrouter_model: "meta-llama/llama-3-8b-instruct:free".to_string(),
            gemini_key: None,
            gemini_model: "gemini-2.0-flash".to_string(),
            groq_key: None,
            groq_model: "llama3-8b-8192".to_string(),
            hf_token: None,
            hf_model: "mistralai/Mistral-7B-Instruct-v0.2".to_string(),
            custom_endpoint: None,
            custom_key: None,
            custom_model: "gpt-4o".to_string(),
        }
    }
}

/// Masked config returned to the frontend (keys shown as `***`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfigMasked {
    /// Active tier preference.
    pub active_tier: Tier,
    /// Ollama base URL.
    pub ollama_url: String,
    /// Ollama model name.
    pub ollama_model: String,
    /// Whether an OpenRouter key is configured.
    pub openrouter_key_set: bool,
    /// OpenRouter selected model.
    pub openrouter_model: String,
    /// Whether a Gemini key is configured.
    pub gemini_key_set: bool,
    /// Gemini model name.
    pub gemini_model: String,
    /// Whether a Groq key is configured.
    pub groq_key_set: bool,
    /// Groq model name.
    pub groq_model: String,
    /// Whether an HF token is configured.
    pub hf_token_set: bool,
    /// HF model ID.
    pub hf_model: String,
    /// Whether a custom endpoint is configured.
    pub custom_endpoint_set: bool,
    /// Custom model name.
    pub custom_model: String,
}

// ── Router ─────────────────────────────────────────────────────────────

/// The AI router manages provider configuration and complexity-based routing.
#[derive(Clone)]
pub struct AiRouter {
    config: Arc<RwLock<AiConfig>>,
    client: Client,
}

impl AiRouter {
    /// Create a new router with default configuration.
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(AiConfig::default())),
            client: Client::new(),
        }
    }

    /// Update the full configuration. Keys are held in memory only.
    pub fn update_config(&self, new_config: AiConfig) {
        let mut config = self.config.write();
        *config = new_config;
    }

    /// Get a masked copy of the config (safe to send to the frontend).
    pub fn get_config_masked(&self) -> AiConfigMasked {
        let config = self.config.read();
        AiConfigMasked {
            active_tier: config.active_tier,
            ollama_url: config.ollama_url.clone(),
            ollama_model: config.ollama_model.clone(),
            openrouter_key_set: config.openrouter_key.is_some(),
            openrouter_model: config.openrouter_model.clone(),
            gemini_key_set: config.gemini_key.is_some(),
            gemini_model: config.gemini_model.clone(),
            groq_key_set: config.groq_key.is_some(),
            groq_model: config.groq_model.clone(),
            hf_token_set: config.hf_token.is_some(),
            hf_model: config.hf_model.clone(),
            custom_endpoint_set: config.custom_endpoint.is_some(),
            custom_model: config.custom_model.clone(),
        }
    }

    // ── Complexity scoring ─────────────────────────────────────────────

    /// Score an intent's complexity from 0-10.
    ///
    /// 0-3 → Tier 1 (Local), 4-7 → Tier 2 (Freemium), 8-10 → Tier 3 (Premium).
    pub fn score_complexity(intent: &str) -> u8 {
        let lower = intent.to_lowercase();
        let mut score: u8 = 4; // Default: Tier 2

        // Tier 1 indicators (simple / local / privacy)
        let local_keywords = [
            "offline", "private", "local", "mute", "unmute", "close tab",
            "new tab", "screenshot", "reload", "go back", "go forward",
        ];
        for kw in &local_keywords {
            if lower.contains(kw) {
                return score.min(2); // Force low score
            }
        }

        // Tier 3 indicators (complex reasoning)
        let complex_keywords = [
            "analyze", "reason", "explain step by step", "code review",
            "refactor", "debug", "write complex", "compare and contrast",
            "evaluate", "synthesize", "critique",
        ];
        for kw in &complex_keywords {
            if lower.contains(kw) {
                score = score.saturating_add(3);
            }
        }

        // Length heuristic: longer prompts are more complex
        let word_count = lower.split_whitespace().count();
        if word_count > 50 {
            score = score.saturating_add(2);
        } else if word_count > 20 {
            score = score.saturating_add(1);
        }

        // Nested conditional indicators
        if lower.contains("if") && lower.contains("then") {
            score = score.saturating_add(1);
        }

        score.min(10)
    }

    /// Map a complexity score to a tier.
    pub fn tier_for_score(score: u8) -> Tier {
        match score {
            0..=3 => Tier::Local,
            4..=7 => Tier::Freemium,
            _ => Tier::Premium,
        }
    }

    /// Score an intent and return the recommended tier.
    pub fn score_intent(intent: &str) -> Tier {
        Self::tier_for_score(Self::score_complexity(intent))
    }

    /// Score a batch of intents (for testing / UI display).
    pub fn score_sample_intents(intents: Vec<String>) -> Vec<(String, Tier, u8)> {
        intents
            .into_iter()
            .map(|intent| {
                let score = Self::score_complexity(&intent);
                let tier = Self::tier_for_score(score);
                (intent, tier, score)
            })
            .collect()
    }

    // ── Provider construction ──────────────────────────────────────────

    /// Build the best available provider for the given tier, falling back
    /// down the tier chain if the target tier has no configured provider.
    fn build_provider_for_tier(&self, tier: Tier) -> Option<Box<dyn Provider>> {
        let config = self.config.read();

        match tier {
            Tier::Premium => {
                // Try custom endpoint first
                if let (Some(ref endpoint), Some(ref key)) =
                    (&config.custom_endpoint, &config.custom_key)
                {
                    return Some(Box::new(OpenAiCompatProvider::new(
                        "Custom (Tier 3)".into(),
                        endpoint.clone(),
                        key.clone(),
                        config.custom_model.clone(),
                        Tier::Premium,
                    )));
                }
                // Fall back to OpenRouter with paid model
                if let Some(ref key) = config.openrouter_key {
                    return Some(Box::new(OpenRouterProvider::new(
                        key.clone(),
                        config.openrouter_model.clone(),
                        Tier::Premium,
                    )));
                }
                // Fall back to Tier 2
                drop(config);
                self.build_provider_for_tier(Tier::Freemium)
            }
            Tier::Freemium => {
                // Try OpenRouter free
                if let Some(ref key) = config.openrouter_key {
                    return Some(Box::new(OpenRouterProvider::new(
                        key.clone(),
                        config.openrouter_model.clone(),
                        Tier::Freemium,
                    )));
                }
                // Try Gemini
                if let Some(ref key) = config.gemini_key {
                    return Some(Box::new(GeminiProvider::new(
                        key.clone(),
                        config.gemini_model.clone(),
                    )));
                }
                // Try Groq
                if let Some(ref key) = config.groq_key {
                    return Some(Box::new(GroqProvider::new(
                        key.clone(),
                        config.groq_model.clone(),
                    )));
                }
                // Try HuggingFace
                if let Some(ref token) = config.hf_token {
                    return Some(Box::new(HuggingFaceProvider::new(
                        token.clone(),
                        config.hf_model.clone(),
                    )));
                }
                // Fall back to Tier 1
                drop(config);
                self.build_provider_for_tier(Tier::Local)
            }
            Tier::Local => Some(Box::new(OllamaProvider::new(
                config.ollama_url.clone(),
                config.ollama_model.clone(),
            ))),
        }
    }

    // ── Public API ─────────────────────────────────────────────────────

    /// Route a completion request to the best available provider.
    ///
    /// Uses the user's active tier preference. If that tier has no
    /// configured provider, falls back down the chain.
    pub async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, CntrlError> {
        let tier = self.config.read().active_tier;
        let provider = self
            .build_provider_for_tier(tier)
            .ok_or_else(|| CntrlError::Ai("No AI provider configured".into()))?;
        provider.complete(req).await
    }

    /// Route using complexity scoring (auto-tier selection).
    pub async fn complete_auto(
        &self,
        intent: &str,
        req: CompletionRequest,
    ) -> Result<CompletionResponse, CntrlError> {
        let tier = Self::score_intent(intent);
        let provider = self
            .build_provider_for_tier(tier)
            .ok_or_else(|| CntrlError::Ai("No AI provider configured".into()))?;
        provider.complete(req).await
    }

    /// Simple prompt completion using the active tier.
    pub async fn ask(&self, prompt: String) -> Result<String, CntrlError> {
        let req = CompletionRequest {
            prompt,
            system: None,
            context: None,
            max_tokens: None,
        };
        let resp = self.complete(req).await?;
        Ok(resp.text)
    }

    /// Get info about all potentially available providers.
    pub async fn get_available_providers(&self) -> Vec<ProviderInfo> {
        let config = self.config.read().clone();
        let mut providers = Vec::new();

        // Ollama (always listed — health check determines availability)
        let ollama = OllamaProvider::new(config.ollama_url.clone(), config.ollama_model.clone());
        let ollama_healthy = ollama.health_check().await;
        providers.push(ProviderInfo {
            name: "Ollama".into(),
            tier: Tier::Local,
            healthy: ollama_healthy,
        });

        // OpenRouter
        if config.openrouter_key.is_some() {
            providers.push(ProviderInfo {
                name: "OpenRouter".into(),
                tier: Tier::Freemium,
                healthy: true, // Assume reachable if key is set
            });
        }

        // Gemini
        if config.gemini_key.is_some() {
            providers.push(ProviderInfo {
                name: "Gemini".into(),
                tier: Tier::Freemium,
                healthy: true,
            });
        }

        // Groq
        if config.groq_key.is_some() {
            providers.push(ProviderInfo {
                name: "Groq".into(),
                tier: Tier::Freemium,
                healthy: true,
            });
        }

        // HuggingFace
        if config.hf_token.is_some() {
            providers.push(ProviderInfo {
                name: "Hugging Face".into(),
                tier: Tier::Freemium,
                healthy: true,
            });
        }

        // Custom Tier 3
        if config.custom_endpoint.is_some() && config.custom_key.is_some() {
            providers.push(ProviderInfo {
                name: "Custom (Tier 3)".into(),
                tier: Tier::Premium,
                healthy: true,
            });
        }

        providers
    }

    /// Fetch HuggingFace model list.
    pub async fn fetch_hf_models(&self) -> Result<Vec<String>, CntrlError> {
        let models = HuggingFaceProvider::fetch_models(&self.client).await?;
        Ok(models.into_iter().map(|m| m.id).collect())
    }

    /// Fetch OpenRouter free models.
    pub async fn fetch_openrouter_free_models(&self) -> Result<Vec<String>, CntrlError> {
        OpenRouterProvider::fetch_free_models(&self.client).await
    }
}

impl Default for AiRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complexity_scoring_local() {
        // Simple system commands → Tier 1
        assert!(AiRouter::score_complexity("mute") <= 3);
        assert!(AiRouter::score_complexity("close tab") <= 3);
        assert!(AiRouter::score_complexity("take a screenshot") <= 3);
        assert!(AiRouter::score_complexity("go back") <= 3);
        assert!(AiRouter::score_complexity("keep it private and offline") <= 3);
    }

    #[test]
    fn complexity_scoring_freemium() {
        // Medium complexity → Tier 2
        let score = AiRouter::score_complexity("summarize this page for me");
        assert!((4..=7).contains(&score), "score was {score}");
        let score = AiRouter::score_complexity("search for the best lo-fi playlists");
        assert!((4..=7).contains(&score), "score was {score}");
    }

    #[test]
    fn complexity_scoring_premium() {
        // Complex reasoning → Tier 3
        let score = AiRouter::score_complexity(
            "analyze the logical fallacies in this argument and explain step by step",
        );
        assert!(score >= 8, "score was {score}");
    }

    #[test]
    fn tier_for_score_boundaries() {
        assert_eq!(AiRouter::tier_for_score(0), Tier::Local);
        assert_eq!(AiRouter::tier_for_score(3), Tier::Local);
        assert_eq!(AiRouter::tier_for_score(4), Tier::Freemium);
        assert_eq!(AiRouter::tier_for_score(7), Tier::Freemium);
        assert_eq!(AiRouter::tier_for_score(8), Tier::Premium);
        assert_eq!(AiRouter::tier_for_score(10), Tier::Premium);
    }

    #[test]
    fn sample_intent_scoring() {
        let intents = vec![
            "go to reddit".to_string(),
            "search for best lo-fi playlists".to_string(),
            "analyze this code and debug it".to_string(),
            "mute".to_string(),
            "take a screenshot".to_string(),
            "summarize this page".to_string(),
            "translate this to Spanish".to_string(),
            "write complex react hook for debouncing".to_string(),
            "what is the weather today".to_string(),
            "offline private browsing mode".to_string(),
        ];

        let results = AiRouter::score_sample_intents(intents);
        let mut correct = 0;

        // Expected: navigate=local/freemium, search=freemium, analyze=premium,
        // mute=local, screenshot=local, summarize=freemium, translate=freemium,
        // write complex=premium, weather=freemium, offline=local
        let expected_tiers = [
            Tier::Freemium, // "go to reddit" — no local keyword
            Tier::Freemium, // "search for best lo-fi playlists"
            Tier::Premium,  // "analyze this code and debug it"
            Tier::Local,    // "mute"
            Tier::Local,    // "take a screenshot" — contains "screenshot"
            Tier::Freemium, // "summarize this page"
            Tier::Freemium, // "translate this to Spanish"
            Tier::Premium,  // "write complex react hook"
            Tier::Freemium, // "what is the weather today"
            Tier::Local,    // "offline private browsing mode"
        ];

        for (i, (_, tier, _)) in results.iter().enumerate() {
            if let Some(expected) = expected_tiers.get(i) {
                if tier == expected {
                    correct += 1;
                }
            }
        }

        assert!(
            correct >= 8,
            "Only {correct}/10 intents matched expected tier (need ≥ 8)"
        );
    }

    #[test]
    fn fallback_builds_ollama_when_no_keys() {
        let router = AiRouter::new();
        // With no keys, Freemium should fall back to Local (Ollama)
        let provider = router.build_provider_for_tier(Tier::Freemium);
        assert!(provider.is_some());
        let p = provider.expect("should have a provider");
        assert_eq!(p.name(), "Ollama");
    }

    #[test]
    fn router_defaults() {
        let router = AiRouter::new();
        let masked = router.get_config_masked();
        assert_eq!(masked.active_tier, Tier::Freemium);
        assert!(!masked.openrouter_key_set);
        assert!(!masked.gemini_key_set);
    }
}
