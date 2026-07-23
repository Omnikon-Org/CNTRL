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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tier {
    Local,
    Freemium,
    Premium,
}

#[derive(Debug, Clone)]
pub struct CompletionRequest {
    pub prompt: String,
    pub system: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CompletionResponse {
    pub text: String,
    pub tokens_used: Option<u32>,
    pub provider: String,
}

#[async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;

    fn tier(&self) -> Tier;

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, CntrlError>;

    async fn health_check(&self) -> bool;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub name: String,
    pub tier: String,
    pub healthy: bool,
}
