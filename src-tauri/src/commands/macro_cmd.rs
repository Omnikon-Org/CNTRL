// commands/macro_cmd.rs — Tauri command handlers for Phase 6 macro system

use std::sync::Arc;
use tauri::{AppHandle, State};
use tauri_plugin_notification::NotificationExt;

use crate::services::agent::{run_macro, DbHandle};
use crate::services::ai::router::Router;
use crate::services::browser::BrowserService;
use crate::services::macro_dir;
use crate::services::macro_format::{MacroTrigger, Vibemacro};
use crate::services::memory::db::AppDb;
use crate::services::privacy::PrivacyGuard;
use crate::services::recorder::Recorder;
use crate::services::scheduler::MacroScheduler;

// ── Frontend-facing DTOs ──────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize)]
pub struct MacroSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub step_count: usize,
    pub created_at: String,
    pub triggers: Vec<String>, // human-readable: "manual" | "cron: <expr>"
}

impl From<&Vibemacro> for MacroSummary {
    fn from(m: &Vibemacro) -> Self {
        let triggers = m
            .triggers
            .iter()
            .map(|t| match t {
                MacroTrigger::Manual => "manual".to_string(),
                MacroTrigger::Cron { cron } => format!("cron: {cron}"),
            })
            .collect();
        Self {
            id: m.id.clone(),
            name: m.name.clone(),
            description: m.description.clone(),
            step_count: m.steps.len(),
            created_at: m.created_at.to_rfc3339(),
            triggers,
        }
    }
}

// ── Recording commands ────────────────────────────────────────────────────────

/// Begin recording a new macro session.
#[tauri::command]
pub fn start_recording(recorder: State<'_, Recorder>) -> Result<String, String> {
    recorder.start().map_err(|e| e.to_string())
}

/// Stop recording and save the macro to ~/.vibe/macros/.
#[tauri::command]
pub fn stop_recording(name: String, recorder: State<'_, Recorder>) -> Result<MacroSummary, String> {
    let mac = recorder.stop(&name).map_err(|e| e.to_string())?;
    macro_dir::save_macro(&mac).map_err(|e| e.to_string())?;
    Ok(MacroSummary::from(&mac))
}

/// Cancel the current recording without saving.
#[tauri::command]
pub fn cancel_recording(recorder: State<'_, Recorder>) -> Result<(), String> {
    recorder.cancel().map_err(|e| e.to_string())
}

/// Returns `true` if a recording is currently in progress.
#[tauri::command]
pub fn is_recording(recorder: State<'_, Recorder>) -> Result<bool, String> {
    recorder.is_recording().map_err(|e| e.to_string())
}

/// Capture a single intent into the active recording session.
/// Called by `submit_intent` when the recorder is active.
#[tauri::command]
pub fn capture_intent(
    intent: String,
    delay_ms: u64,
    recorder: State<'_, Recorder>,
) -> Result<(), String> {
    recorder.capture(&intent, delay_ms).map_err(|e| e.to_string())
}

// ── Library commands ──────────────────────────────────────────────────────────

/// List all saved macros (newest first).
#[tauri::command]
pub fn list_macros() -> Result<Vec<MacroSummary>, String> {
    macro_dir::list_macros()
        .map(|v| v.iter().map(MacroSummary::from).collect())
        .map_err(|e| e.to_string())
}

/// Delete a macro by ID.
#[tauri::command]
pub fn delete_macro(macro_id: String) -> Result<(), String> {
    macro_dir::delete_macro(&macro_id).map_err(|e| e.to_string())
}

// ── Playback commands ─────────────────────────────────────────────────────────

/// Run a macro immediately in the background.
#[tauri::command]
pub async fn run_macro_cmd(
    macro_id: String,
    app_handle: AppHandle,
    router: State<'_, Router>,
    browser: State<'_, BrowserService>,
    privacy: State<'_, PrivacyGuard>,
    db: State<'_, AppDb>,
) -> Result<(), String> {
    let mac = macro_dir::get_macro(&macro_id).map_err(|e| e.to_string())?;

    let router_arc = Arc::new(router.inner().clone());
    let browser_clone = browser.inner().clone();
    let privacy_clone = privacy.inner().clone();
    let db_clone: DbHandle = db.inner().clone();
    let app_clone = app_handle.clone();

    tauri::async_runtime::spawn(async move {
        run_macro(mac, app_clone, router_arc, browser_clone, privacy_clone, db_clone).await;
    });

    Ok(())
}

// ── Scheduling commands ───────────────────────────────────────────────────────

/// Schedule a macro to run on a cron expression.
#[tauri::command]
pub async fn schedule_macro(
    macro_id: String,
    cron: String,
    app_handle: AppHandle,
    scheduler: State<'_, MacroScheduler>,
    router: State<'_, Router>,
    browser: State<'_, BrowserService>,
    privacy: State<'_, PrivacyGuard>,
    db: State<'_, AppDb>,
) -> Result<String, String> {
    let router_arc = Arc::new(router.inner().clone());
    let browser_clone = browser.inner().clone();
    let privacy_clone = privacy.inner().clone();
    let db_clone: DbHandle = db.inner().clone();
    let app_clone = app_handle.clone();

    let app_notify = app_handle.clone();

    let job_uuid = scheduler
        .schedule(macro_id.clone(), &cron, move |mid| {
            let mac = match macro_dir::get_macro(&mid) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("[scheduler] Cannot load macro '{mid}': {e}");
                    let _ = app_notify
                        .notification()
                        .builder()
                        .title("CNTRL — Scheduled macro failed")
                        .body(format!("Could not load macro '{mid}': {e}"))
                        .show();
                    return;
                }
            };

            let ra = router_arc.clone();
            let bc = browser_clone.clone();
            let pc = privacy_clone.clone();
            let dc = db_clone.clone();
            let ac = app_clone.clone();

            // Notify the trigger fired
            let _ = ac
                .notification()
                .builder()
                .title("CNTRL — Scheduled macro triggered")
                .body(format!("⏰ Running: {}", mac.name))
                .show();

            tauri::async_runtime::spawn(async move {
                run_macro(mac, ac, ra, bc, pc, dc).await;
            });
        })
        .await
        .map_err(|e| e.to_string())?;

    Ok(job_uuid.to_string())
}

/// Remove a macro's cron schedule.
#[tauri::command]
pub async fn unschedule_macro(
    macro_id: String,
    scheduler: State<'_, MacroScheduler>,
) -> Result<(), String> {
    scheduler.unschedule(&macro_id).await.map_err(|e| e.to_string())
}

/// List all currently scheduled macros.
#[tauri::command]
pub fn list_scheduled_macros(scheduler: State<'_, MacroScheduler>) -> Vec<serde_json::Value> {
    scheduler
        .list_scheduled()
        .into_iter()
        .map(|j| {
            serde_json::json!({
                "macro_id": j.macro_id,
                "cron": j.cron,
                "job_uuid": j.job_uuid.to_string(),
            })
        })
        .collect()
}
