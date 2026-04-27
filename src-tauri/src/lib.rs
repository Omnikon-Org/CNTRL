use tauri::Manager;

pub mod commands;
pub mod error;
pub mod services;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            app.manage(services::browser::BrowserService::new());
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
