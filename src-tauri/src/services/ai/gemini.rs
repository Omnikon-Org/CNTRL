//! Google Gemini provider — Tier 2 (Freemium).
//!
//! Uses the Gemini 1.5 Flash API (free tier). The API key is retrieved
//! from the OS keychain via [`crate::services::keychain`] and is never
//! stored in memory longer than the duration of a single request.
//!
//! API reference: <https://ai.google.dev/api/generate-content>

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::CntrlError;
use crate::services::keychain;
use super::{CompletionRequest, CompletionResponse, Provider, Tier};

const GEMINI_API_BASE: &str = "https://generativelanguage.googleapis.com/v1beta/models";
const GEMINI_MODEL: &str = "gemini-1.5-flash";

// ─────────────────────────────────────────────────────────────────────────────
// Wire types
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct GeminiPart<'a> {
    text: &'a str,
}

#[derive(Serialize)]
struct GeminiContent<'a> {
    parts: Vec<GeminiPart<'a>>,
}

#[derive(Serialize)]
struct GeminiRequest<'a> {
    contents: Vec<GeminiContent<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiContent<'a>>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiResponseContent,
}

#[derive(Deserialize)]
struct GeminiResponseContent {
    parts: Vec<GeminiResponsePart>,
}

#[derive(Deserialize)]
struct GeminiResponsePart {
    text: String,
}

#[derive(Deserialize)]
struct GeminiUsageMetadata {
    #[serde(rename = "totalTokenCount")]
    total_token_count: Option<u32>,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<GeminiUsageMetadata>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Provider implementation
// ─────────────────────────────────────────────────────────────────────────────

/// Google Gemini Flash provider (Tier 2).
pub struct GeminiProvider {
    client: Client,
}

impl GeminiProvider {
    /// Creates a new `GeminiProvider`.
    /// The API key is read from the OS keychain on each request — it is not
    /// cached in memory.
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl Default for GeminiProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for GeminiProvider {
    fn name(&self) -> &str {
        "Gemini"
    }

    fn tier(&self) -> Tier {
        Tier::Freemium
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, CntrlError> {
        let api_key = keychain::retrieve_secret(keychain::KEY_GEMINI)?;

        let url = format!(
            "{GEMINI_API_BASE}/{GEMINI_MODEL}:generateContent?key={api_key}"
        );

        let body = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart { text: &req.prompt }],
            }],
            system_instruction: req.system.as_deref().map(|s| GeminiContent {
                parts: vec![GeminiPart { text: s }],
            }),
        };

        let res = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| CntrlError::Ai(format!("Gemini connection error: {e}")))?;

        if !res.status().is_success() {
            let status = res.status();
            let err_text = res.text().await.unwrap_or_default();
            return Err(CntrlError::Ai(format!("Gemini API error {status}: {err_text}")));
        }

        let data: GeminiResponse = res
            .json()
            .await
            .map_err(|e| CntrlError::Ai(format!("Gemini JSON parse error: {e}")))?;

        let text = data
            .candidates
            .into_iter()
            .next()
            .and_then(|c| c.content.parts.into_iter().next())
            .map(|p| p.text)
            .unwrap_or_default();

        if text.is_empty() {
            return Err(CntrlError::Ai("Gemini returned an empty response".to_string()));
        }

        let tokens_used = data
            .usage_metadata
            .and_then(|u| u.total_token_count);

        Ok(CompletionResponse {
            text,
            tokens_used,
            provider: self.name().to_string(),
        })
    }

    async fn health_check(&self) -> bool {
        // Check if the API key is stored; a real network ping would consume quota.
        keychain::secret_exists(keychain::KEY_GEMINI)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies the request body serialises to the format Gemini expects.
    #[test]
    fn request_body_structure_is_correct() {
        let req = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart { text: "Hello" }],
            }],
            system_instruction: None,
        };

        let json = serde_json::to_value(&req).expect("should serialise");
        let contents = &json["contents"];
        assert!(contents.is_array(), "contents must be an array");
        assert_eq!(contents[0]["parts"][0]["text"], "Hello");
        assert!(
            json.get("system_instruction").is_none(),
            "system_instruction must be omitted when None"
        );
    }

    #[test]
    fn request_body_with_system_instruction() {
        let req = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart { text: "Hi" }],
            }],
            system_instruction: Some(GeminiContent {
                parts: vec![GeminiPart { text: "Be concise." }],
            }),
        };

        let json = serde_json::to_value(&req).expect("should serialise");
        assert_eq!(json["system_instruction"]["parts"][0]["text"], "Be concise.");
    }

    #[test]
    fn provider_metadata() {
        let p = GeminiProvider::new();
        assert_eq!(p.name(), "Gemini");
        assert_eq!(p.tier(), Tier::Freemium);
    }
}
