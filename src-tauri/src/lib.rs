use tauri::{Emitter, Listener, Manager};

pub mod commands;
pub mod error;
pub mod services;

use services::ai::router::Router;
use services::background::BackgroundRuntime;
use services::browser::BrowserService;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_keyring::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let app_data = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir");
            std::fs::create_dir_all(&app_data).expect("Failed to create app data dir");
            let db_path = app_data.join("vibe-browser.db");
            let db_path_str = db_path.to_string_lossy();

            let db = tauri::async_runtime::block_on(async {
                services::memory::db::open(&db_path_str).await
            })
            .expect("Failed to open SQLite database");

            let privacy_guard = tauri::async_runtime::block_on(async {
                services::privacy::PrivacyGuard::load(&db).await
            })
            .expect("Failed to load privacy guard");

            services::keychain::init_audit_db(db.clone());

            let lance_path = app_data.to_string_lossy().to_string();
            services::memory::recall::init_lance_db(&lance_path);

            app.manage(db);
            app.manage(privacy_guard);

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

            let router = Router::new(
                "http://localhost:11434",
                "llama3",
                "meta-llama/llama-3-8b-instruct:free",
                "mistralai/Mistral-7B-Instruct-v0.2",
                None,
                None,
            );
            app.manage(router);

            let scheduler = tauri::async_runtime::block_on(async {
                services::scheduler::MacroScheduler::new().await
            })
            .expect("Failed to create macro scheduler");
            app.manage(scheduler);

            let recorder = services::recorder::Recorder::new();
            app.manage(recorder);

            let browser_service_ref = app.state::<BrowserService>();
            let handle = app.handle().clone();
            let browser_clone = browser_service_ref.inner().clone();
            let handle_clone = handle.clone();

            handle.listen("tab-metadata", move |event: tauri::Event| {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(event.payload()) {
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
            commands::ai::store_api_key,
            commands::ai::get_api_key_status,
            commands::ai::delete_api_key,
            commands::ai::health_check_all,
            commands::ai::get_available_providers,
            commands::ai::get_hf_models,
            commands::ai::get_openrouter_free_models,
            commands::ai::test_intent_router,
            commands::background::spawn_background_task,
            commands::intent::submit_intent,
            commands::memory::get_preference,
            commands::memory::set_preference,
            commands::memory::is_privacy_mode_enabled,
            commands::memory::set_privacy_mode,
            commands::memory::get_recent_audit_log,
            commands::memory::get_audit_log_count,
            commands::memory::get_site_habits,
            commands::macro_cmd::start_recording,
            commands::macro_cmd::stop_recording,
            commands::macro_cmd::cancel_recording,
            commands::macro_cmd::is_recording,
            commands::macro_cmd::capture_intent,
            commands::macro_cmd::list_macros,
            commands::macro_cmd::delete_macro,
            commands::macro_cmd::run_macro_cmd,
            commands::macro_cmd::schedule_macro,
            commands::macro_cmd::unschedule_macro,
            commands::macro_cmd::list_scheduled_macros,
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
