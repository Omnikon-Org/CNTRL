use std::path::PathBuf;
use tauri::{Emitter, Listener, Manager};

pub mod commands;
pub mod error;
pub mod services;

use services::ai::router::AiRouter;
use services::browser::BrowserService;

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


            app.manage(BrowserService::new());
            app.manage(AiRouter::new());

            let browser_service = app.state::<BrowserService>();
            let handle = app.handle().clone();
            let browser_service_clone = browser_service.inner().clone();
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

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::browser::open_tab,
            commands::browser::close_tab,
            commands::browser::navigate,
            commands::browser::get_tabs,
            commands::browser::get_active_tab_id,
            commands::browser::set_active_tab,
            commands::browser::fetch_fallback,
            commands::browser::update_tab_bounds,
            commands::browser::go_back,
            commands::browser::go_forward,
            commands::browser::reload,
            commands::ai::ask_ai,
            commands::ai::get_ai_config,
            commands::ai::update_ai_config,
            commands::ai::get_hf_models,
            commands::ai::get_openrouter_free_models,
            commands::ai::test_intent_router,
            commands::intent::execute_intent,
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
