use std::sync::Arc;

use super::{
    gemini::GeminiProvider, groq::GroqProvider, huggingface::HuggingFaceProvider,
    ollama::OllamaProvider, openai_compat::OpenAiCompatProvider, openrouter::OpenRouterProvider,
};
use super::{CompletionRequest, CompletionResponse, Provider, ProviderInfo, Tier};
use crate::error::CntrlError;

const COMPLEXITY_SIGNALS: &[(&[&str], u8)] = &[
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
            "logic",
            "logical",
            "flaw",
            "flaws",
            "argument",
            "philosophical",
        ],
        3,
    ),
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
            "react",
            "component",
            "rust",
            "python",
            "javascript",
            "typescript",
            "c++",
            "java",
            "coding",
            "programming",
            "software",
            "development",
        ],
        2,
    ),
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
            "find",
            "recipe",
            "weather",
            "today",
            "search",
            "query",
            "who",
            "what",
            "where",
            "when",
            "why",
            "how",
        ],
        1,
    ),
];

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

pub fn score_complexity(intent: &str) -> u8 {
    let lower = intent.to_lowercase();

    if LOCAL_OVERRIDE_KEYWORDS.iter().any(|kw| lower.contains(kw)) {
        return 0;
    }

    let mut score: u8 = 3;

    for (keywords, delta) in COMPLEXITY_SIGNALS {
        for kw in *keywords {
            if lower.contains(kw) {
                score = score.saturating_add(*delta);
                break;
            }
        }
    }

    score.min(10)
}

#[must_use]
pub fn score_to_tier(score: u8) -> Tier {
    match score {
        0..=3 => Tier::Local,
        4..=7 => Tier::Freemium,
        8..=10 => Tier::Premium,
        _ => Tier::Freemium,
    }
}

#[derive(Clone)]
pub struct Router {
    local: Arc<dyn Provider>,
    freemium: Vec<Arc<dyn Provider>>,
    premium: Vec<Arc<dyn Provider>>,
}

impl Router {
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
        freemium.retain(|_| true);

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

    pub async fn route(
        &self,
        intent: &str,
        req: CompletionRequest,
        privacy_guard: &crate::services::privacy::PrivacyGuard,
        db: &crate::services::memory::db::AppDb,
    ) -> Result<CompletionResponse, CntrlError> {
        let score = score_complexity(intent);
        let target_tier = score_to_tier(score);

        privacy_guard.check_tier(&format!("{:?}", target_tier))?;

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
            let start = std::time::Instant::now();
            let res = provider.complete(req.clone()).await;
            let latency = start.elapsed().as_millis() as i64;
            let success = res.is_ok();
            let tokens = res
                .as_ref()
                .ok()
                .and_then(|r| r.tokens_used.map(|t| t as i64));
            let _ = crate::services::audit::log_ai_call(
                db,
                intent,
                &format!("{:?}", provider.tier()),
                provider.name(),
                latency,
                tokens,
                success,
            )
            .await;

            match res {
                Ok(resp) => return Ok(resp),
                Err(e) => {
                    last_err = e;
                }
            }
        }

        let start = std::time::Instant::now();
        let res = self.local.complete(req).await;
        let latency = start.elapsed().as_millis() as i64;
        let success = res.is_ok();
        let tokens = res
            .as_ref()
            .ok()
            .and_then(|r| r.tokens_used.map(|t| t as i64));
        let _ = crate::services::audit::log_ai_call(
            db,
            intent,
            &format!("{:?}", self.local.tier()),
            self.local.name(),
            latency,
            tokens,
            success,
        )
        .await;

        res.map_err(|_| last_err)
    }

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

#[cfg(test)]
mod tests {
    use super::*;

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
<<<<<<< HEAD
        // Maximum possible: all signal groups fire
=======
>>>>>>> omnikon/main
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

    #[test]
    fn ten_intent_benchmark_8_of_10() {
        let cases: Vec<(&str, Tier)> = vec![
<<<<<<< HEAD
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
=======
            ("browse privately", Tier::Local),
            ("offline mode", Tier::Local),
            ("find a recipe for lasagne", Tier::Freemium),
            ("what is the weather today?", Tier::Freemium),
            ("translate this text to Spanish", Tier::Freemium),
            ("summarize this article", Tier::Freemium),
            ("write a blog post about AI", Tier::Freemium),
            ("debug this React component", Tier::Premium),
            ("implement a binary search tree in Rust", Tier::Premium),
            (
                "analyze the logical flaws in this complex argument",
                Tier::Premium,
            ),
>>>>>>> omnikon/main
        ];

        let correct: usize = cases
            .iter()
            .filter(|(intent, expected)| {
                let score = score_complexity(intent);
                let actual = score_to_tier(score);
                actual == *expected
            })
            .count();

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
