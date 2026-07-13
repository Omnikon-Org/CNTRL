use std::path::PathBuf;
use tauri::{Emitter, Listener, Manager};

pub mod commands;
pub mod error;
pub mod services;

use services::ai_router::AiRouter;
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
            let app_data = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| PathBuf::from("."));
            let key_path = app_data.join(".cntrl_key");

            let browser_service = BrowserService::new();
            let config = browser_service.get_browser_config();
            
            let background_runtime = BackgroundRuntime::new(
                app.handle().clone(),
                browser_service.clone(),
                config.background_workers,
                config.background_queue_capacity,
            );

            app.manage(browser_service);
            app.manage(AiRouter::new(key_path));
            app.manage(background_runtime);
            
            let browser_state = app.state::<BrowserService>();
            let handle = app.handle().clone();
            let browser_service_clone = browser_state.inner().clone();
            let handle_clone = handle.clone();

            handle.listen("tab-metadata", move |event: tauri::Event| {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(event.payload()) {
                    if let (Some(id_str), Some(title), Some(favicon)) = (
                        data["id"].as_str(),
                        data["title"].as_str(),
                        data["favicon"].as_str(),
                    ) {
                        if let Ok(id) = uuid::Uuid::parse_str(id_str) {
                            let _ = browser_service_clone.update_metadata(
                                id,
                                title.to_string(),
                                favicon.to_string(),
                            );
                            let _ = handle_clone.emit("tabs-updated", ());
                        }
                    }
                }
            });

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
            commands::ai::ask_ai,
            commands::ai::get_ai_config,
            commands::ai::update_ai_config,
            commands::ai::get_hf_models,
            commands::ai::get_openrouter_free_models,
            commands::ai::test_intent_router,
            commands::background::spawn_background_task,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke_test() {
        assert_eq!(2 + 2, 4);
    }
}
