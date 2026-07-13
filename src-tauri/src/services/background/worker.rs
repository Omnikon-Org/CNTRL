use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tokio::time::timeout;

use super::error::BackgroundError;
use super::task::{BackgroundAction, BackgroundTask, TaskResult, TaskStatus};
use crate::services::browser::BrowserService;

pub async fn execute_task(
    app: &AppHandle,
    browser: BrowserService,
    task: BackgroundTask,
) -> Result<TaskResult, BackgroundError> {
    let _ = app.emit("background://started", &task);

    // Run the execution with a timeout
    let duration = Duration::from_millis(task.timeout_ms);
    let result = timeout(duration, run_actions(app.clone(), browser.clone(), &task)).await;

    let final_status = match result {
        Ok(Ok(data)) => {
            let res = TaskResult {
                id: task.id,
                status: TaskStatus::Completed,
                data,
            };
            let _ = app.emit("background://completed", &res);
            Ok(res)
        }
        Ok(Err(e)) => {
            let _ = app.emit("background://failed", &e);
            Err(e)
        }
        Err(_) => {
            let res = TaskResult {
                id: task.id,
                status: TaskStatus::Timeout,
                data: None,
            };
            let _ = app.emit("background://timeout", &res);
            Err(BackgroundError::Timeout)
        }
    };

    // Guaranteed cleanup: the tab was created with `task.id`
    // If it hasn't been created yet, this is a no-op, which is safe.
    let _ = browser.close_tab(app, task.id);

    final_status
}

async fn run_actions(
    app: AppHandle,
    browser: BrowserService,
    task: &BackgroundTask,
) -> Result<Option<String>, BackgroundError> {
    // Determine initial URL if the first action is navigate
    let initial_url = if let Some(BackgroundAction::Navigate(url)) = task.actions.first() {
        url.clone()
    } else {
        "about:blank".to_string()
    };

    // The browser service uses Uuid for tab ids, but it generates them.
    // We want to force it to use our task.id or map it.
    // Let's modify BrowserService slightly to accept an optional ID,
    // or we can map task.id to the spawned tab_id.
    // For now, let's just open the tab and get its ID.
    let tab_id = browser
        .open_tab_with_id(&app, initial_url, true, task.id) // true for is_background
        .map_err(|e| BackgroundError::BrowserCreationFailed(e.to_string()))?;

    let mut last_result = None;

    for action in &task.actions {
        match action {
            BackgroundAction::Navigate(url) => {
                browser
                    .navigate(&app, tab_id, url.clone())
                    .map_err(|e| BackgroundError::NavigationFailed(e.to_string()))?;

                // wait for navigation (simplified)
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            BackgroundAction::EvaluateJS(script) => {
                if let Some(w) = app.get_webview(&format!("tab-{}", tab_id)) {
                    // Evaluate JS logic (we might not be able to get results back easily without events,
                    // but we can execute it)
                    w.eval(script)
                        .map_err(|e| BackgroundError::JavaScriptFailed(e.to_string()))?;
                    last_result = Some("Executed".to_string());
                } else {
                    return Err(BackgroundError::InternalError("Webview not found".into()));
                }
            }
        }
    }

    Ok(last_result)
}
