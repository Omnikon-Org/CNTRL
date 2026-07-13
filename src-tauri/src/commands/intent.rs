use tauri::{AppHandle, State};
use crate::services::intent::IntentResult;
use crate::services::planner::Planner;
use crate::services::executor::Executor;
use crate::services::ai::router::Router;
use crate::services::browser::BrowserStore;

#[tauri::command]
pub async fn submit_intent(
    input: String,
    app_handle: AppHandle,
    router: State<'_, Router>,
    browser_store: State<'_, BrowserStore>,
) -> Result<String, String> {
    // 1. Parse intent
    let intent = IntentResult::parse(&input);

    // 2. Plan
    let plan = Planner::plan(intent);

    // 3. Execute
    Executor::execute(plan, &app_handle, &router, &browser_store).await
}
