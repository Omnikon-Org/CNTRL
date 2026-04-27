use tauri::Manager;
use std::path::PathBuf;

pub mod commands;
pub mod error;
pub mod services;

use services::browser::BrowserService;
use services::ai_router::AiRouter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
            let key_path = app_data.join(".vibe_key");
            
            app.manage(BrowserService::new());
            app.manage(AiRouter::new(key_path));
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
            commands::ai::ask_ai,
            commands::ai::get_ai_config,
            commands::ai::update_ai_config,
            commands::ai::get_hf_models,
            commands::ai::get_openrouter_free_models,
            commands::ai::test_intent_router,
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
