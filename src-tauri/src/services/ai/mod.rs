//! AI subsystem — provider trait, shared types, and tier-based routing.
//!
//! All AI API calls are made from the Rust backend. The frontend never
//! contacts an AI endpoint directly. Secrets are held in memory only;
//! Phase 5 moves them to the OS keychain.

pub mod gemini;
pub mod groq;
pub mod huggingface;
pub mod ollama;
pub mod openai_compat;
pub mod openrouter;
pub mod router;

use serde::{Deserialize, Serialize};

use crate::error::CntrlError;

// ── Tier classification ────────────────────────────────────────────────

/// The three AI tiers used for complexity-based routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tier {
    /// Tier 1 — local models (Ollama). Zero network calls.
    Local,
    /// Tier 2 — free/freemium cloud models (Gemini Flash, Groq, HF, OpenRouter free).
    Freemium,
    /// Tier 3 — precision / paid models (OpenAI-compatible, OpenRouter paid).
    Premium,
}

// ── Request / Response ─────────────────────────────────────────────────

/// A completion request sent to any provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// The user prompt or instruction.
    pub prompt: String,
    /// Optional system message prepended to the conversation.
    pub system: Option<String>,
    /// Optional context from memory / page content.
    pub context: Option<String>,
    /// Maximum tokens to generate (provider may ignore if unsupported).
    pub max_tokens: Option<u32>,
}

/// A completion response returned by any provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// The generated text.
    pub text: String,
    /// Tokens consumed (if the provider reports it).
    pub tokens_used: Option<u32>,
    /// Provider-reported model identifier.
    pub model: String,
    /// Which tier handled this request.
    pub tier: Tier,
}

/// Summary info about a configured provider (exposed to the frontend).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    /// Human-readable name, e.g. "Ollama", "Gemini Flash".
    pub name: String,
    /// Which tier this provider belongs to.
    pub tier: Tier,
    /// Whether the provider is currently reachable.
    pub healthy: bool,
}

// ── Provider trait ─────────────────────────────────────────────────────

/// Every AI backend implements this trait. The router picks the best
/// available provider at the tier determined by complexity scoring.
#[async_trait::async_trait]
pub trait Provider: Send + Sync {
    /// Human-readable provider name.
    fn name(&self) -> &str;

    /// Which tier this provider belongs to.
    fn tier(&self) -> Tier;

    /// Run a completion request. Returns the generated text + metadata.
    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, CntrlError>;

    /// Quick connectivity check. Must never panic.
    async fn health_check(&self) -> bool;
}
