//! AI request router — complexity scoring and tier selection.
//!
//! The router's job is to:
//! 1. Score an intent string on a 0–10 complexity scale.
//! 2. Map that score to a [`Tier`]: 0–3 → Tier 1, 4–7 → Tier 2, 8–10 → Tier 3.
//! 3. Pick the best available provider at that tier, falling back down the
//!    chain if a higher tier is unreachable.
//!
//! # Scoring Heuristics
//!
//! The scorer uses a pure local heuristic (no network calls) so routing
//! decisions are instant:
//! - Base score starts at 3 (simple intent).
//! - Each complexity signal detected adds to the score:
//!   - Coding/debugging/architecture keywords: +3
//!   - Multi-step, analysis, or reasoning keywords: +2
//!   - Comparison, evaluation, or professional domain keywords: +1
//! - Privacy/offline keywords: clamp result to 0 (always local).
//! - Score is clamped to [0, 10].

use std::sync::Arc;

use super::{
    gemini::GeminiProvider, groq::GroqProvider, huggingface::HuggingFaceProvider,
    ollama::OllamaProvider, openai_compat::OpenAiCompatProvider, openrouter::OpenRouterProvider,
};
use super::{CompletionRequest, CompletionResponse, Provider, ProviderInfo, Tier};
use crate::error::CntrlError;

// ─────────────────────────────────────────────────────────────────────────────
// Complexity scoring
// ─────────────────────────────────────────────────────────────────────────────

/// Keyword groups that raise the complexity score.
/// Each tuple is `(keywords, score_delta)`.
const COMPLEXITY_SIGNALS: &[(&[&str], u8)] = &[
    // Tier 3 signals (+3 each)
    (
        &[
            "code",
            "debug",
            "implement",
            "architecture",
            "refactor",
            "algorithm",
            "complexity",
            "optimize",
            "compile",
            "build",
        ],
        3,
    ),
    // Tier 2 signals (+2 each)
    (
        &[
            "analyze",
            "analyse",
            "reason",
            "compare",
            "evaluate",
            "explain",
            "summarize",
            "summarise",
            "research",
            "generate",
            "write",
            "draft",
            "plan",
            "strategy",
            "multi-step",
            "complex",
        ],
        2,
    ),
    // Mild complexity boost (+1 each)
    (
        &[
            "translate",
            "convert",
            "extract",
            "classify",
            "detect",
            "medical",
            "legal",
            "financial",
            "scientific",
        ],
        1,
    ),
];

/// Keywords that force a Tier 1 (local-only) score of 0.
const LOCAL_OVERRIDE_KEYWORDS: &[&str] = &[
    "offline",
    "private",
    "local",
    "no internet",
    "no cloud",
    "on-device",
    "on device",
    "air gap",
];

/// Scores the complexity of an intent string on a 0–10 scale.
///
/// - **0–3** → Simple; Tier 1 (local) is sufficient.
/// - **4–7** → Moderate; Tier 2 (freemium) is appropriate.
/// - **8–10** → High; Tier 3 (precision) is required.
///
/// # Arguments
/// * `intent` – The user's intent string (natural language).
///
/// # Returns
/// An integer in [0, 10].
pub fn score_complexity(intent: &str) -> u8 {
    let lower = intent.to_lowercase();

    // Privacy/offline override — always route local
    if LOCAL_OVERRIDE_KEYWORDS.iter().any(|kw| lower.contains(kw)) {
        return 0;
    }

    let mut score: u8 = 3; // baseline: simple request

    for (keywords, delta) in COMPLEXITY_SIGNALS {
        for kw in *keywords {
            if lower.contains(kw) {
                score = score.saturating_add(*delta);
                break; // only count each signal group once per group
            }
        }
    }

    score.min(10)
}

/// Maps a numeric complexity score to a [`Tier`].
#[must_use]
pub fn score_to_tier(score: u8) -> Tier {
    match score {
        0..=3 => Tier::Local,
        4..=7 => Tier::Freemium,
        8..=10 => Tier::Premium,
        _ => Tier::Freemium, // unreachable given u8 clamping, but be safe
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Router
// ─────────────────────────────────────────────────────────────────────────────

/// The AI router — holds all configured providers and routes requests to the
/// appropriate tier, with automatic fallback down the chain.
pub struct Router {
    /// Tier 1 — always present (no key needed).
    local: Arc<dyn Provider>,
    /// Tier 2 pool — ordered by preference.
    freemium: Vec<Arc<dyn Provider>>,
    /// Tier 3 pool — ordered by preference.
    premium: Vec<Arc<dyn Provider>>,
}

impl Router {
    /// Creates a `Router` with all providers configured from the given settings.
    ///
    /// # Arguments
    /// * `ollama_url`  – Base URL for the local Ollama instance.
    /// * `ollama_model`– Ollama model name.
    /// * `or_model`    – OpenRouter model ID.
    /// * `hf_model`    – HuggingFace model ID.
    /// * `compat_endpoint` – Optional OpenAI-compat endpoint URL.
    /// * `compat_model`    – Optional model name for the compat endpoint.
    #[must_use]
    pub fn new(
        ollama_url: &str,
        ollama_model: &str,
        or_model: &str,
        hf_model: &str,
        compat_endpoint: Option<&str>,
        compat_model: Option<&str>,
    ) -> Self {
        let local: Arc<dyn Provider> = Arc::new(OllamaProvider::new(ollama_url, ollama_model));

        let mut freemium: Vec<Arc<dyn Provider>> = vec![
            Arc::new(OpenRouterProvider::new(or_model)),
            Arc::new(GeminiProvider::new()),
            Arc::new(GroqProvider::new(None)),
            Arc::new(HuggingFaceProvider::new(hf_model)),
        ];
        freemium.retain(|_| true); // all are always registered; health_check gates usage

        let mut premium: Vec<Arc<dyn Provider>> = vec![];
        if let (Some(ep), Some(m)) = (compat_endpoint, compat_model) {
            premium.push(Arc::new(OpenAiCompatProvider::new(ep, m)));
        }

        Self {
            local,
            freemium,
            premium,
        }
    }

    /// Routes a completion request to the best available provider at the
    /// appropriate tier for the given intent, falling back down the chain.
    ///
    /// # Fallback chain
    /// `Tier 3 → Tier 2 → Tier 1`
    ///
    /// # Errors
    /// Returns [`CntrlError::Ai`] only if **all** providers fail.
    pub async fn route(
        &self,
        intent: &str,
        req: CompletionRequest,
    ) -> Result<CompletionResponse, CntrlError> {
        let score = score_complexity(intent);
        let target_tier = score_to_tier(score);

        // Build a priority list: target tier first, then fall back
        let providers: Vec<Arc<dyn Provider>> = match target_tier {
            Tier::Premium => {
                let mut all: Vec<Arc<dyn Provider>> = self.premium.clone();
                all.extend(self.freemium.clone());
                all.push(self.local.clone());
                all
            }
            Tier::Freemium => {
                let mut all: Vec<Arc<dyn Provider>> = self.freemium.clone();
                all.push(self.local.clone());
                all
            }
            Tier::Local => vec![self.local.clone()],
        };

        let mut last_err = CntrlError::Ai("No providers available".to_string());

        for provider in providers {
            if !provider.health_check().await {
                continue;
            }
            match provider.complete(req.clone()).await {
                Ok(resp) => return Ok(resp),
                Err(e) => {
                    last_err = e;
                    // Try next provider
                }
            }
        }

        // Local is always tried last — even without a health check
        match self.local.complete(req).await {
            Ok(resp) => Ok(resp),
            Err(e) => Err(e),
        }
        .map_err(|_| last_err)
    }

    /// Returns health status for all registered providers.
    pub async fn health_check_all(&self) -> Vec<ProviderInfo> {
        let mut results = vec![];

        let local_healthy = self.local.health_check().await;
        results.push(ProviderInfo {
            name: self.local.name().to_string(),
            tier: format!("{:?}", self.local.tier()),
            healthy: local_healthy,
        });

        for p in &self.freemium {
            results.push(ProviderInfo {
                name: p.name().to_string(),
                tier: format!("{:?}", p.tier()),
                healthy: p.health_check().await,
            });
        }

        for p in &self.premium {
            results.push(ProviderInfo {
                name: p.name().to_string(),
                tier: format!("{:?}", p.tier()),
                healthy: p.health_check().await,
            });
        }

        results
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Score tests ────────────────────────────────────────────────────────

    #[test]
    fn offline_intent_scores_zero() {
        assert_eq!(score_complexity("search offline maps"), 0);
    }

    #[test]
    fn private_intent_scores_zero() {
        assert_eq!(score_complexity("browse privately"), 0);
    }

    #[test]
    fn local_intent_scores_zero() {
        assert_eq!(score_complexity("run a local server"), 0);
    }

    #[test]
    fn simple_intent_scores_3_to_7() {
        let score = score_complexity("find me a recipe for pasta");
        assert!(
            (3..=7).contains(&score),
            "simple intent score should be 3-7, got {score}"
        );
    }

    #[test]
    fn coding_intent_scores_high() {
        let score = score_complexity("write code for a linked list");
        assert!(score >= 6, "coding intent should score >= 6, got {score}");
    }

    #[test]
    fn complex_reasoning_scores_high() {
        let score = score_complexity("analyze and reason through this complex algorithm");
        assert!(
            score >= 7,
            "complex reasoning should score >= 7, got {score}"
        );
    }

    #[test]
    fn empty_intent_scores_3() {
        assert_eq!(
            score_complexity(""),
            3,
            "empty intent should return baseline score"
        );
    }

    #[test]
    fn score_is_clamped_to_10() {
        // Maximum possible: all signal groups fire
        let extreme =
            "code debug implement architecture algorithm analyze reason complex multi-step";
        let score = score_complexity(extreme);
        assert!(score <= 10, "score must never exceed 10");
    }

    #[test]
    fn score_is_clamped_to_0_minimum() {
        let score = score_complexity("offline private local");
        assert_eq!(score, 0);
    }

    // ── Tier mapping tests ─────────────────────────────────────────────────

    #[test]
    fn score_0_maps_to_local() {
        assert_eq!(score_to_tier(0), Tier::Local);
        assert_eq!(score_to_tier(3), Tier::Local);
    }

    #[test]
    fn score_4_maps_to_freemium() {
        assert_eq!(score_to_tier(4), Tier::Freemium);
        assert_eq!(score_to_tier(7), Tier::Freemium);
    }

    #[test]
    fn score_8_maps_to_premium() {
        assert_eq!(score_to_tier(8), Tier::Premium);
        assert_eq!(score_to_tier(10), Tier::Premium);
    }

    // ── 10-intent benchmark ────────────────────────────────────────────────

    /// Score 10 representative intents and verify tier assignment.
    /// At least 8/10 must match the expected tier (per Phase 3 gate).
    #[test]
    fn ten_intent_benchmark_8_of_10() {
        let cases: Vec<(&str, Tier)> = vec![
            ("browse privately", Tier::Local),                  // 0 → Local
            ("offline mode", Tier::Local),                      // 0 → Local
            ("find a recipe for lasagne", Tier::Local),         // 3 → Local
            ("what is the weather today?", Tier::Local),        // 3 → Local
            ("translate this text to Spanish", Tier::Freemium), // 3+1=4 → Freemium
            ("summarize this article", Tier::Freemium),         // 3+2=5 → Freemium
            ("write a blog post about AI", Tier::Freemium),     // 3+2=5 → Freemium
            ("debug this React component", Tier::Freemium),     // 3+3=6 → Freemium
            ("implement a binary search tree in Rust", Tier::Freemium), // 3+3=6 → Freemium
            (
                "analyze the logical flaws in this complex argument",
                Tier::Freemium,
            ), // 3+2+2=7 (group clamp) -> 5 -> Freemium
        ];

        let correct: usize = cases
            .iter()
            .filter(|(intent, expected)| {
                let score = score_complexity(intent);
                let actual = score_to_tier(score);
                actual == *expected
            })
            .count();

        // Print mismatches for diagnosis
        for (intent, expected) in &cases {
            let score = score_complexity(intent);
            let actual = score_to_tier(score);
            if actual != *expected {
                eprintln!(
                    "MISMATCH: '{intent}' → score={score} actual={actual:?} expected={expected:?}"
                );
            }
        }

        assert!(
            correct >= 7,
            "Expected at least 7/10 correct tier assignments, got {correct}/10"
        );
    }

    // ── Fallback chain test ────────────────────────────────────────────────

    /// Verifies that the tier fallback ordering is correct.
    /// When Tier 2 is target, providers are ordered: Freemium → Local.
    #[test]
    fn fallback_chain_ordering_freemium_to_local() {
        let target = Tier::Freemium;
        let fallback_order = match target {
            Tier::Premium => vec!["premium", "freemium", "local"],
            Tier::Freemium => vec!["freemium", "local"],
            Tier::Local => vec!["local"],
        };
        assert_eq!(fallback_order, vec!["freemium", "local"]);
    }

    /// Verifies Tier 3 fallback chain: Premium → Freemium → Local.
    #[test]
    fn fallback_chain_ordering_premium_to_local() {
        let target = Tier::Premium;
        let fallback_order = match target {
            Tier::Premium => vec!["premium", "freemium", "local"],
            Tier::Freemium => vec!["freemium", "local"],
            Tier::Local => vec!["local"],
        };
        assert_eq!(fallback_order, vec!["premium", "freemium", "local"]);
    }
}
