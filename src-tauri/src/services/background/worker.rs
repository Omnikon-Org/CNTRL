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

    let _ = browser.close_tab(app, task.id);

    final_status
}

async fn run_actions(
    app: AppHandle,
    browser: BrowserService,
    task: &BackgroundTask,
) -> Result<Option<String>, BackgroundError> {
    let initial_url = if let Some(BackgroundAction::Navigate(url)) = task.actions.first() {
        url.clone()
    } else {
        "about:blank".to_string()
    };

    let tab_id = browser
        .open_tab_with_id(&app, initial_url, true, task.id)
        .map_err(|e| BackgroundError::BrowserCreationFailed(e.to_string()))?;

    let mut last_result = None;

    for action in &task.actions {
        match action {
            BackgroundAction::Navigate(url) => {
                browser
                    .navigate(&app, tab_id, url.clone())
                    .map_err(|e| BackgroundError::NavigationFailed(e.to_string()))?;

                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            BackgroundAction::EvaluateJS(script) => {
                if let Some(w) = app.get_webview(&format!("tab-{}", tab_id)) {
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
