//! Tauri command adapter for the intent pipeline.
//!
//! Orchestrates the full cycle:
//! 1. Parse the raw user input into a structured [`IntentResult`].
//! 2. Recall relevant past tasks and prepend them as context for the AI.
//! 3. Plan steps from the intent (using the decorated prompt where applicable).
//! 4. Record the task as `"running"` in `task_history`.
//! 5. Execute the plan.
//! 6. Update `task_history` with the final status and record site habits.

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
    // 1. Recall relevant context from past tasks (best-effort; never blocks).
    let context_entries = find_relevant_context(db.inner(), &input, 3)
        .await
        .unwrap_or_default();

    // 2. Build a decorated prompt that prepends recalled context when available.
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

    // 3. Parse intent from the original (undecorated) input so classification
    //    is not confused by the context preamble.
    let intent = IntentResult::parse(&input);

    // 4. Plan using the decorated prompt (the planner forwards it to the AI).
    let plan = Planner::plan_with_context(intent.clone(), &decorated_input);

    // 5. Save task as "running".
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

    // 6. Execute.
    let execution_result = Executor::execute(
        plan,
        &app_handle,
        &router,
        &browser_store,
        &privacy_guard,
        db.inner(),
    )
    .await;

    // 7. Update task history with final status and record habits.
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
