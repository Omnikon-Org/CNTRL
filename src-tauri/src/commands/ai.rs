//! Tauri commands for the AI subsystem.
//!
//! Each command is a thin adapter that delegates to the appropriate service.
//! All AI API calls happen in the Rust backend — no keys or API calls ever
//! touch the frontend.

use std::collections::HashMap;

use tauri::State;

use crate::services::ai::huggingface::HuggingFaceProvider;
use crate::services::ai::openrouter::OpenRouterProvider;
use crate::services::ai::{
    router::{score_complexity, score_to_tier, Router},
    CompletionRequest, ProviderInfo,
};
use crate::services::keychain::{
    self, KEY_GEMINI, KEY_GROQ, KEY_HF_TOKEN, KEY_OPENAI_COMPAT, KEY_OPENROUTER, MASKED_SENTINEL,
};

// ─────────────────────────────────────────────────────────────────────────────
// Completion
// ─────────────────────────────────────────────────────────────────────────────

/// Sends a prompt to the router, which selects the best provider based on
/// complexity scoring and tier availability.
///
/// # Arguments
/// * `prompt`  – The user's natural language prompt.
/// * `context` – Optional system-level context to prepend.
#[tauri::command]
pub async fn ask_ai(
    prompt: String,
    context: Option<String>,
    router: State<'_, Router>,
) -> Result<String, String> {
    let req = CompletionRequest {
        prompt: prompt.clone(),
        system: context,
    };
    router
        .route(&prompt, req)
        .await
        .map(|r| r.text)
        .map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// Key management (via OS keychain only — no plaintext in config)
// ─────────────────────────────────────────────────────────────────────────────

/// Stores an API key in the OS keychain.
///
/// The `provider` parameter maps to one of the well-known keychain constants.
/// If the value is the masked sentinel (`"***stored***"`), the call is a no-op
/// so the UI can safely round-trip the masked value without overwriting the key.
#[tauri::command]
pub fn store_api_key(provider: String, value: String) -> Result<(), String> {
    if value == MASKED_SENTINEL || value.is_empty() {
        return Ok(());
    }
    let key = provider_to_keychain_key(&provider)?;
    keychain::store_secret(key, &value).map_err(|e| e.to_string())
}

/// Returns whether a key is stored for the given provider (returns the masked
/// sentinel if stored, empty string if not), so the UI can show "key is set".
#[tauri::command]
pub fn get_api_key_status(provider: String) -> Result<String, String> {
    let key = provider_to_keychain_key(&provider)?;
    if keychain::secret_exists(key) {
        Ok(MASKED_SENTINEL.to_string())
    } else {
        Ok(String::new())
    }
}

/// Deletes a stored API key from the OS keychain.
#[tauri::command]
pub fn delete_api_key(provider: String) -> Result<(), String> {
    let key = provider_to_keychain_key(&provider)?;
    keychain::delete_secret(key).map_err(|e| e.to_string())
}

fn provider_to_keychain_key(provider: &str) -> Result<&'static str, String> {
    match provider {
        "openrouter" => Ok(KEY_OPENROUTER),
        "gemini" => Ok(KEY_GEMINI),
        "groq" => Ok(KEY_GROQ),
        "huggingface" => Ok(KEY_HF_TOKEN),
        "openai_compat" => Ok(KEY_OPENAI_COMPAT),
        other => Err(format!("Unknown provider: {other}")),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Health checks
// ─────────────────────────────────────────────────────────────────────────────

/// Returns health status for all configured providers.
#[tauri::command]
pub async fn health_check_all(router: State<'_, Router>) -> Result<HashMap<String, bool>, String> {
    let infos: Vec<ProviderInfo> = router.health_check_all().await;
    Ok(infos.into_iter().map(|p| (p.name, p.healthy)).collect())
}

/// Returns detailed provider info (name, tier, health) for all providers.
#[tauri::command]
pub async fn get_available_providers(
    router: State<'_, Router>,
) -> Result<Vec<ProviderInfo>, String> {
    Ok(router.health_check_all().await)
}

// ─────────────────────────────────────────────────────────────────────────────
// Model lists
// ─────────────────────────────────────────────────────────────────────────────

/// Fetches the list of popular HuggingFace text-generation models.
#[tauri::command]
pub async fn get_hf_models() -> Result<Vec<String>, String> {
    let provider = HuggingFaceProvider::new("mistralai/Mistral-7B-Instruct-v0.2");
    provider.fetch_model_list().await.map_err(|e| e.to_string())
}

/// Fetches the list of free models from OpenRouter.
#[tauri::command]
pub async fn get_openrouter_free_models() -> Result<Vec<String>, String> {
    let provider = OpenRouterProvider::new("meta-llama/llama-3-8b-instruct:free");
    provider
        .fetch_free_models()
        .await
        .map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// Router diagnostics
// ─────────────────────────────────────────────────────────────────────────────

/// Scores a list of intent strings and returns `(intent, score, tier)` tuples.
/// Used by the Settings UI to demonstrate the router's behaviour.
#[tauri::command]
pub fn test_intent_router(intents: Vec<String>) -> Result<Vec<(String, u8, String)>, String> {
    Ok(intents
        .into_iter()
        .map(|intent| {
            let score = score_complexity(&intent);
            let tier = format!("{:?}", score_to_tier(score));
            (intent, score, tier)
        })
        .collect())
}
