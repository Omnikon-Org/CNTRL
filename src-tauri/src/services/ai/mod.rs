//! AI services module.
//!
//! This module re-exports all provider implementations and exposes the
//! [`Provider`] trait, [`CompletionRequest`], [`CompletionResponse`], and
//! [`Tier`] types that glue them together.
//!
//! # Provider Hierarchy
//!
//! | Tier | Provider | Notes |
//! |------|----------|-------|
//! | 1 (Local)    | [`ollama`]         | Runs on `localhost:11434`, no API key |
//! | 2 (Freemium) | [`gemini`]         | Google Gemini Flash free tier |
//! | 2 (Freemium) | [`groq`]           | Groq free tier |
//! | 2 (Freemium) | [`huggingface`]    | HF Inference API |
//! | 2 (Freemium) | [`openrouter`]     | OpenRouter (routes to many free models) |
//! | 3 (Precision)| [`openai_compat`]  | Generic OpenAI-compatible endpoint |

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::CntrlError;

pub mod gemini;
pub mod groq;
pub mod huggingface;
pub mod ollama;
pub mod openai_compat;
pub mod openrouter;
pub mod router;

// ─────────────────────────────────────────────────────────────────────────────
// Shared types
// ─────────────────────────────────────────────────────────────────────────────

/// Provider capability tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tier {
    /// Tier 1 — local inference via Ollama. Zero network egress.
    Local,
    /// Tier 2 — free/freemium cloud providers (Gemini, Groq, HF, OpenRouter).
    Freemium,
    /// Tier 3 — precision cloud providers (Claude, GPT-4o, etc.) with paid keys.
    Premium,
}

/// A completion request sent to any [`Provider`].
#[derive(Debug, Clone)]
pub struct CompletionRequest {
    /// The user's input prompt.
    pub prompt: String,
    /// Optional system-level context prepended to the conversation.
    pub system: Option<String>,
}

/// A successful completion response from any [`Provider`].
#[derive(Debug, Clone)]
pub struct CompletionResponse {
    /// The model's generated text.
    pub text: String,
    /// Number of tokens used, if the provider returns this.
    pub tokens_used: Option<u32>,
    /// The provider name that generated this response.
    pub provider: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Provider trait
// ─────────────────────────────────────────────────────────────────────────────

/// Trait implemented by every AI provider.
///
/// All methods are `async` via `async_trait`. Providers are expected to be
/// `Send + Sync` so they can be stored behind an `Arc<dyn Provider>` and
/// shared across Tokio tasks.
#[async_trait]
pub trait Provider: Send + Sync {
    /// Human-readable provider name (e.g. `"Ollama"`, `"Gemini"`).
    fn name(&self) -> &str;

    /// The tier this provider belongs to.
    fn tier(&self) -> Tier;

    /// Send a completion request and return the response.
    ///
    /// # Errors
    /// Returns [`CntrlError::Ai`] if the request fails for any reason.
    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, CntrlError>;

    /// Returns `true` if the provider is currently reachable and can serve
    /// requests. This should be a cheap, fast probe (e.g. a HEAD request or
    /// a minimal list-models call).
    ///
    /// Must never panic — return `false` on any error.
    async fn health_check(&self) -> bool;
}

/// Human-readable provider info returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    /// Provider name.
    pub name: String,
    /// Tier as a string for serialisation.
    pub tier: String,
    /// Whether the provider passed its last health check.
    pub healthy: bool,
}
