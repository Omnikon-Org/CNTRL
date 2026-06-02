use tauri::State;
use uuid::Uuid;

use crate::services::browser::{BrowserService, Tab};

#[tauri::command]
pub fn open_tab(
    url: String,
    is_background: bool,
    app: tauri::AppHandle,
    browser_service: State<'_, BrowserService>,
) -> Result<Uuid, String> {
    browser_service
        .open_tab(&app, url, is_background)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn close_tab(
    id: Uuid,
    app: tauri::AppHandle,
    browser_service: State<'_, BrowserService>,
) -> Result<(), String> {
    browser_service
        .close_tab(&app, id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn navigate(
    id: Uuid,
    url: String,
    app: tauri::AppHandle,
    browser_service: State<'_, BrowserService>,
) -> Result<(), String> {
    browser_service
        .navigate(&app, id, url)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_tabs(browser_service: State<'_, BrowserService>) -> Result<Vec<Tab>, String> {
    browser_service.get_tabs().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_active_tab(
    id: Uuid,
    app: tauri::AppHandle,
    browser_service: State<'_, BrowserService>,
) -> Result<(), String> {
    browser_service
        .set_active_tab(&app, id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn fetch_fallback(url: String, app: tauri::AppHandle) -> Result<String, String> {
    crate::services::fallback::fetch_fallback_html(&app, &url)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_tab_bounds(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    app: tauri::AppHandle,
    browser_service: State<'_, BrowserService>,
) -> Result<(), String> {
    browser_service
        .update_tab_bounds(&app, x, y, width, height)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn go_back(
    id: Uuid,
    app: tauri::AppHandle,
    browser_service: State<'_, BrowserService>,
) -> Result<(), String> {
    browser_service.go_back(&app, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn go_forward(
    id: Uuid,
    app: tauri::AppHandle,
    browser_service: State<'_, BrowserService>,
) -> Result<(), String> {
    browser_service
        .go_forward(&app, id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reload(
    id: Uuid,
    app: tauri::AppHandle,
    browser_service: State<'_, BrowserService>,
) -> Result<(), String> {
    browser_service.reload(&app, id).map_err(|e| e.to_string())
}
