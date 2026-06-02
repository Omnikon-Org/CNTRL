use crate::services::ai_router::{AiRouter, ModelConfig};
use tauri::State;

#[tauri::command]
pub async fn ask_ai(prompt: String, ai_router: State<'_, AiRouter>) -> Result<String, String> {
    ai_router.ask_model(prompt).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_ai_config(ai_router: State<'_, AiRouter>) -> Result<ModelConfig, String> {
    Ok(ai_router.get_config())
}

#[tauri::command]
pub fn update_ai_config(config: ModelConfig, ai_router: State<'_, AiRouter>) -> Result<(), String> {
    ai_router.update_config(config);
    Ok(())
}

#[tauri::command]
pub async fn get_hf_models(ai_router: State<'_, AiRouter>) -> Result<Vec<String>, String> {
    ai_router.fetch_hf_models().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_openrouter_free_models(
    ai_router: State<'_, AiRouter>,
) -> Result<Vec<String>, String> {
    ai_router
        .fetch_openrouter_free_models()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn test_intent_router(
    intents: Vec<String>,
    ai_router: State<'_, AiRouter>,
) -> Result<Vec<(String, String)>, String> {
    let scores = ai_router.score_sample_intents(intents);
    Ok(scores
        .into_iter()
        .map(|(intent, tier)| (intent, format!("{:?}", tier)))
        .collect())
}
