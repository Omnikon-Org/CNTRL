use super::planner::Step;
use crate::services::ai::router::Router;
use crate::services::ai::CompletionRequest;
use crate::services::browser::BrowserService;
use serde::Serialize;
use std::process::Command;
use tauri::{AppHandle, Emitter, State};

#[derive(Clone, Serialize)]
pub struct StepStatusEvent {
    pub step_index: usize,
    pub total_steps: usize,
    pub status: String,
    pub result_markdown: Option<String>,
}

pub struct Executor;

impl Executor {
    pub async fn execute(
        plan: Vec<Step>,
        app_handle: &AppHandle,
        router: &State<'_, Router>,
        browser_store: &State<'_, BrowserService>,
        privacy_guard: &crate::services::privacy::PrivacyGuard,
        db: &crate::services::memory::db::AppDb,
    ) -> Result<String, String> {
        let total = plan.len();
        let mut final_output = String::new();

        for (i, step) in plan.into_iter().enumerate() {
            let _ = app_handle.emit(
                "intent://step-status",
                StepStatusEvent {
                    step_index: i,
                    total_steps: total,
                    status: "Running".to_string(),
                    result_markdown: None,
                },
            );

            let result = match step {
                Step::Navigate { url } => {
                    let final_url = if url.starts_with("http://")
                        || url.starts_with("https://")
                        || url.starts_with("cntrl://")
                    {
                        url.clone()
                    } else {
                        format!("https://{}", url)
                    };

                    let tabs = browser_store.get_tabs().map_err(|e| e.to_string())?;
                    if let Some(active_tab) = tabs.last() {
                        let _ =
                            browser_store.navigate(app_handle, active_tab.id, final_url.clone());
                    } else {
                        let _ = browser_store.open_tab(app_handle, final_url.clone(), false);
                    }
                    Ok(format!("Navigated to {}", final_url))
                }
                Step::AiQuery { prompt } => {
                    let req = CompletionRequest {
                        prompt: prompt.clone(),
                        system: None,
                    };
                    match router.route(&prompt, req, privacy_guard, db).await {
                        Ok(resp) => Ok(resp.text),
                        Err(e) => Err(e.to_string()),
                    }
                }
                Step::BuiltinCommand { command } => Self::execute_builtin(&command).await,
                Step::DisplayResult { markdown } => Ok(markdown),
            };

            match result {
                Ok(output) => {
                    final_output = output.clone();
                    let _ = app_handle.emit(
                        "intent://step-status",
                        StepStatusEvent {
                            step_index: i,
                            total_steps: total,
                            status: "Done".to_string(),
                            result_markdown: Some(output),
                        },
                    );
                }
                Err(err) => {
                    let _ = app_handle.emit(
                        "intent://step-status",
                        StepStatusEvent {
                            step_index: i,
                            total_steps: total,
                            status: "Failed".to_string(),
                            result_markdown: Some(err.clone()),
                        },
                    );
                    return Err(err);
                }
            }
        }

        Ok(final_output)
    }

    async fn execute_builtin(command: &str) -> Result<String, String> {
        match command {
            "bitcoin_price" => {
                let url =
                    "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd";
                match reqwest::get(url).await {
                    Ok(resp) => {
                        if let Ok(json) = resp.json::<serde_json::Value>().await {
                            if let Some(price) = json["bitcoin"]["usd"].as_f64() {
                                let whole = price as u64;
                                let frac = ((price - whole as f64) * 100.0).round() as u64;
                                let s: String = whole
                                    .to_string()
                                    .as_bytes()
                                    .rchunks(3)
                                    .rev()
                                    .map(|c| std::str::from_utf8(c).unwrap())
                                    .collect::<Vec<_>>()
                                    .join(",");
                                return Ok(format!(
                                    "Current Bitcoin Price: **${}.{:02}**",
                                    s, frac
                                ));
                            }
                        }
                        Err("Failed to parse Bitcoin price".to_string())
                    }
                    Err(e) => Err(format!("Network error: {}", e)),
                }
            }
            "screenshot" => {
                let output = Command::new("screencapture").arg("-i").arg("-c").output();
                match output {
                    Ok(_) => Ok("Screenshot taken and copied to clipboard.".to_string()),
                    Err(e) => Err(format!("Failed to take screenshot: {}", e)),
                }
            }
            "mute" => {
                let output = Command::new("osascript")
                    .arg("-e")
                    .arg("set volume with output muted")
                    .output();
                match output {
                    Ok(_) => Ok("System audio muted.".to_string()),
                    Err(e) => Err(format!("Failed to mute volume: {}", e)),
                }
            }
            _ => Err(format!("Unknown built-in command: {}", command)),
        }
    }
}
