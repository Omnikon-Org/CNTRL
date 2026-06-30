use tauri::AppHandle;
use crate::services::intent::parse_intent;
use crate::services::planner::{plan, TaskResult};
use crate::services::executor::execute_steps;

#[tauri::command]
pub async fn execute_intent(
    input: String,
    app_handle: AppHandle,
) -> Result<TaskResult, String> {
    // 1. Parse intent
    let intent_result = parse_intent(&input).map_err(|e| e.to_string())?;

    // 2. Plan steps
    let steps = plan(&intent_result).map_err(|e| e.to_string())?;

    // 3. Execute steps
    let result = execute_steps(&app_handle, intent_result.id, steps).await.map_err(|e| e.to_string())?;

    Ok(result)
}
