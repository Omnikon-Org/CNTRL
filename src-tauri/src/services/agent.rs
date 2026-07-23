// services/agent.rs
//
// Background macro execution agent.
// Wraps the existing Phase 4 Planner + Executor to run macros step-by-step
// on a dedicated Tokio task, independently of the main UI thread.
// Fires OS notifications (via tauri-plugin-notification) on start/done/fail.

use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;
use tokio::time::{sleep, Duration};

use crate::services::ai::router::Router;
use crate::services::browser::BrowserService;
use crate::services::intent::IntentResult;
use crate::services::macro_format::Vibemacro;
use crate::services::planner::Planner;
use crate::services::privacy::PrivacyGuard;

/// Emitted to the frontend as each macro step runs.
#[derive(Clone, serde::Serialize)]
pub struct MacroStepEvent {
    pub macro_id: String,
    pub macro_name: String,
    pub step_index: usize,
    pub total_steps: usize,
    pub intent: String,
    pub status: String, // "running" | "done" | "failed"
}

/// Execute a macro's steps sequentially, leveraging the Phase 4 pipeline.
/// This function is designed to be spawned with `tauri::async_runtime::spawn`.
pub async fn run_macro(
    mac: Vibemacro,
    app: AppHandle,
    router: std::sync::Arc<Router>,
    browser: BrowserService,
    privacy: PrivacyGuard,
    db: DbHandle,
) {
    let macro_id = mac.id.clone();
    let macro_name = mac.name.clone();
    let total = mac.steps.len();

    // ── Notification: started ────────────────────────────────────────────────
    let _ = app
        .notification()
        .builder()
        .title("CNTRL — Macro started")
        .body(format!("▶ {macro_name}"))
        .show();

    let _ = app.emit(
        "macro://started",
        serde_json::json!({ "macro_id": macro_id, "name": macro_name }),
    );

    let mut failed = false;

    for (i, step) in mac.steps.iter().enumerate() {
        // Optional inter-step delay
        if step.delay_ms > 0 {
            sleep(Duration::from_millis(step.delay_ms)).await;
        }

        let _ = app.emit(
            "macro://step",
            MacroStepEvent {
                macro_id: macro_id.clone(),
                macro_name: macro_name.clone(),
                step_index: i,
                total_steps: total,
                intent: step.intent.clone(),
                status: "running".into(),
            },
        );

        let intent = IntentResult::parse(&step.intent);

        // Unknown macro_id referenced intent → log and skip gracefully
        if matches!(
            intent.intent_type,
            crate::services::intent::IntentType::MacroTrigger
        ) {
            let inner_id = intent
                .parameters
                .get("macro_id")
                .cloned()
                .unwrap_or_default();
            eprintln!(
                "[agent] Step {i}: macro trigger '{inner_id}' inside a macro is not supported; skipping."
            );
            let _ = app.emit(
                "macro://step",
                MacroStepEvent {
                    macro_id: macro_id.clone(),
                    macro_name: macro_name.clone(),
                    step_index: i,
                    total_steps: total,
                    intent: step.intent.clone(),
                    status: "skipped".into(),
                },
            );
            continue;
        }

        let plan = Planner::plan(intent);

        let router_ref: &Router = &router;
        // We use the State-less variant of execute by constructing references directly.
        let result = execute_plan_direct(plan, &app, router_ref, &browser, &privacy, &db).await;

        match result {
            Ok(_) => {
                let _ = app.emit(
                    "macro://step",
                    MacroStepEvent {
                        macro_id: macro_id.clone(),
                        macro_name: macro_name.clone(),
                        step_index: i,
                        total_steps: total,
                        intent: step.intent.clone(),
                        status: "done".into(),
                    },
                );
            }
            Err(ref e) => {
                eprintln!("[agent] Step {i} failed: {e}");
                let _ = app.emit(
                    "macro://step",
                    MacroStepEvent {
                        macro_id: macro_id.clone(),
                        macro_name: macro_name.clone(),
                        step_index: i,
                        total_steps: total,
                        intent: step.intent.clone(),
                        status: format!("failed: {e}"),
                    },
                );
                failed = true;
                break; // stop macro on first failure
            }
        }
    }

    // ── Notification: complete or failed ─────────────────────────────────────
    if failed {
        let _ = app
            .notification()
            .builder()
            .title("CNTRL — Macro failed")
            .body(format!("✗ {macro_name} encountered an error"))
            .show();
        let _ = app.emit(
            "macro://failed",
            serde_json::json!({ "macro_id": macro_id, "name": macro_name }),
        );
    } else {
        let _ = app
            .notification()
            .builder()
            .title("CNTRL — Macro complete")
            .body(format!("✓ {macro_name} finished successfully"))
            .show();
        let _ = app.emit(
            "macro://completed",
            serde_json::json!({ "macro_id": macro_id, "name": macro_name }),
        );
    }
}

/// Type alias — the AppDb pool can be passed directly since it's Clone.
pub type DbHandle = crate::services::memory::db::AppDb;

/// Execute a plan using direct references (avoids needing `tauri::State`
/// wrappers which are only valid on the command handler thread).
async fn execute_plan_direct(
    plan: Vec<crate::services::planner::Step>,
    app: &AppHandle,
    router: &Router,
    browser: &BrowserService,
    privacy: &PrivacyGuard,
    db: &DbHandle,
) -> Result<String, String> {
    use crate::services::ai::CompletionRequest;
    use crate::services::planner::Step;

    let total = plan.len();
    let mut final_output = String::new();

    for (i, step) in plan.into_iter().enumerate() {
        let result: Result<String, String> = match step {
            Step::Navigate { url } => {
                let final_url = if url.starts_with("http://")
                    || url.starts_with("https://")
                    || url.starts_with("cntrl://")
                {
                    url.clone()
                } else {
                    format!("https://{}", url)
                };
                let tabs = browser.get_tabs().map_err(|e| e.to_string())?;
                if let Some(tab) = tabs.last() {
                    let _ = browser.navigate(app, tab.id, final_url.clone());
                } else {
                    let _ = browser.open_tab(app, final_url.clone(), false);
                }
                Ok(format!("Navigated to {final_url}"))
            }
            Step::AiQuery { prompt } => {
                let req = CompletionRequest {
                    prompt: prompt.clone(),
                    system: None,
                };
                router
                    .route(&prompt, req, privacy, db)
                    .await
                    .map(|r| r.text)
                    .map_err(|e| e.to_string())
            }
            Step::BuiltinCommand { command } => {
                crate::services::executor::Executor::execute_builtin_cmd(&command).await
            }
            Step::DisplayResult { markdown } => Ok(markdown),
        };

        match result {
            Ok(out) => {
                final_output = out;
            }
            Err(e) => return Err(e),
        }

        let _ = i;
        let _ = total;
    }

    Ok(final_output)
}
