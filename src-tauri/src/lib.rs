//! CNTRL Browser — Tauri application entry point.
//!
//! This module is responsible for:
//! - Registering Tauri plugins.
//! - Initialising and managing application state (services).
//! - Wiring up the Tauri event system.
//! - Registering all Tauri commands via the invoke handler.
//!
//! No business logic lives here; all logic is in `services/`.

use tauri::{Emitter, Listener, Manager};

pub mod commands;
pub mod error;
pub mod services;

use services::ai::router::Router;
use services::browser::BrowserService;
use services::background::BackgroundRuntime;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .setup(|app| {
            // ── Browser service ────────────────────────────────────────────
            let browser_service = BrowserService::new();
            app.manage(browser_service);

            let browser_instance = app.state::<BrowserService>().inner().clone();
            let config = browser_instance.get_browser_config();
            
            let background_runtime = BackgroundRuntime::new(
                app.handle().clone(),
                browser_instance.clone(),
                config.background_workers,
                config.background_queue_capacity,
            );
            app.manage(background_runtime);

            // ── AI Router ─────────────────────────────────────────────────
            // The router is constructed with sensible defaults. Users configure
            // providers via the Settings UI; keys are stored/retrieved from the
            // OS keychain — never from config files.
            let router = Router::new(
                "http://localhost:11434",       // ollama_url
                "llama3",                       // ollama_model
                "meta-llama/llama-3-8b-instruct:free", // openrouter model
                "mistralai/Mistral-7B-Instruct-v0.2",  // hf model
                None,                           // compat endpoint (user-configured)
                None,                           // compat model
            );
            app.manage(router);

            // ── Tab metadata listener ──────────────────────────────────────
            let browser_service_ref = app.state::<BrowserService>();
            let handle = app.handle().clone();
            let browser_clone = browser_service_ref.inner().clone();
            let handle_clone = handle.clone();

            handle.listen("tab-metadata", move |event: tauri::Event| {
                if let Ok(data) =
                    serde_json::from_str::<serde_json::Value>(event.payload())
                {
                    if let (Some(id_str), Some(title), Some(favicon)) = (
                        data["id"].as_str(),
                        data["title"].as_str(),
                        data["favicon"].as_str(),
                    ) {
                        if let Ok(id) = uuid::Uuid::parse_str(id_str) {
                            let _ = browser_clone.update_metadata(
                                id,
                                title.to_string(),
                                favicon.to_string(),
                            );
                            let _ = handle_clone.emit("tabs-updated", ());
                        }
                    }
                }
            });

            // ── Window close → Cmd+W event ────────────────────────────────
            let main_window = app
                .get_webview_window("main")
                .expect("main window not found");
            let emit_handle = app.handle().clone();
            main_window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = emit_handle.emit("cmd-w", ());
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Browser commands
            commands::browser::open_tab,
            commands::browser::close_tab,
            commands::browser::navigate,
            commands::browser::get_tabs,
            commands::browser::set_active_tab,
            commands::browser::fetch_fallback,
            commands::browser::update_tab_bounds,
            commands::browser::go_back,
            commands::browser::go_forward,
            commands::browser::reload,
            commands::browser::get_browser_config,
            commands::browser::update_browser_config,
            // AI commands
            commands::ai::ask_ai,
            commands::ai::store_api_key,
            commands::ai::get_api_key_status,
            commands::ai::delete_api_key,
            commands::ai::health_check_all,
            commands::ai::get_available_providers,
            commands::ai::get_hf_models,
            commands::ai::get_openrouter_free_models,
            commands::ai::test_intent_router,
            commands::background::spawn_background_task,
            // Intent commands
            commands::intent::submit_intent,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    /// Smoke test — verifies the binary links correctly.
    #[test]
    fn smoke_test() {
        assert_eq!(2 + 2, 4);
    }
}
