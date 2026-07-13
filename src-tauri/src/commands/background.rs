use tauri::State;
use uuid::Uuid;

use crate::services::background::{BackgroundRuntime, BackgroundTask, BackgroundAction};

#[tauri::command]
pub async fn spawn_background_task(
    url: String,
    script: Option<String>,
    timeout_ms: u64,
    background_runtime: State<'_, BackgroundRuntime>,
) -> Result<Uuid, String> {
    let id = Uuid::new_v4();
    
    let mut actions = vec![BackgroundAction::Navigate(url)];
    if let Some(s) = script {
        actions.push(BackgroundAction::EvaluateJS(s));
    }

    let task = BackgroundTask {
        id,
        actions,
        timeout_ms,
    };

    background_runtime
        .enqueue(task)
        .await
        .map_err(|e| e.to_string())?;

    Ok(id)
}
