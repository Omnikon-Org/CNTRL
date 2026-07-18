use crate::services::ai::router::Router;
use crate::services::browser::BrowserService;
use crate::services::executor::Executor;
use crate::services::intent::IntentResult;
use crate::services::memory::recall::{find_relevant_context, save_task};
use crate::services::planner::Planner;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn submit_intent(
    input: String,
    app_handle: AppHandle,
    router: State<'_, Router>,
    browser_store: State<'_, BrowserService>,
    privacy_guard: State<'_, crate::services::privacy::PrivacyGuard>,
    db: State<'_, crate::services::memory::db::AppDb>,
) -> Result<String, String> {
    let context_entries = find_relevant_context(db.inner(), &input, 3)
        .await
        .unwrap_or_default();

    let decorated_input: String = if context_entries.is_empty() {
        input.clone()
    } else {
        let context_block: String = context_entries
            .iter()
            .enumerate()
            .map(|(i, e)| {
                let result_str = e.result.as_deref().unwrap_or("(no result)");
                format!(
                    "{}. [{}] \"{}\" → {}",
                    i + 1,
                    e.intent_type,
                    e.intent_raw,
                    result_str
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!("Relevant context from past tasks:\n{context_block}\n\nCurrent request: {input}")
    };

    let intent = IntentResult::parse(&input);

    let plan = Planner::plan_with_context(intent.clone(), &decorated_input);

    let task_id = uuid::Uuid::new_v4().to_string();
    let slots_json = serde_json::to_string(&intent.parameters).unwrap_or_default();
    let intent_type_str = format!("{:?}", intent.intent_type);
    let _ = save_task(
        db.inner(),
        &task_id,
        &input,
        &intent_type_str,
        &slots_json,
        "running",
        None,
    )
    .await;

    let execution_result = Executor::execute(
        plan,
        &app_handle,
        &router,
        &browser_store,
        &privacy_guard,
        db.inner(),
    )
    .await;

    let status = if execution_result.is_ok() {
        "done"
    } else {
        "failed"
    };
    let result_str = match &execution_result {
        Ok(s) => s.as_str(),
        Err(e) => e.as_str(),
    };
    let _ = save_task(
        db.inner(),
        &task_id,
        &input,
        &intent_type_str,
        &slots_json,
        status,
        Some(result_str),
    )
    .await;

    if execution_result.is_ok() {
        if let Some(url) = intent.parameters.get("url") {
            let _ = crate::services::memory::habits::record_outcome(
                db.inner(),
                &intent_type_str,
                &input,
                url,
            )
            .await;
        }
    }

    execution_result
}
