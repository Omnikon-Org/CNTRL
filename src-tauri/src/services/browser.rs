use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tauri::{Emitter, Manager, WebviewBuilder, WebviewUrl};
use uuid::Uuid;

use crate::error::CntrlError;

const CHROME_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tab {
    pub id: Uuid,
    pub url: String,
    pub title: String,
    pub favicon: Option<String>,
    pub is_background: bool,
    pub created_at: DateTime<Utc>,
    pub fallback_mode: bool,
    pub loaded: bool,
    #[serde(skip)]
    pub webview: Option<tauri::Webview<tauri::Wry>>,
}

#[derive(Default)]
pub struct BrowserState {
    pub tabs: Vec<Tab>,
    pub active_tab_id: Option<Uuid>,
}

#[derive(Clone)]
pub struct BrowserService {
    state: Arc<RwLock<BrowserState>>,
}

impl Default for BrowserService {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserService {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(BrowserState::default())),
        }
    }

    pub fn open_tab(
        &self,
        app: &tauri::AppHandle,
        url: String,
        is_background: bool,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> Result<Uuid, CntrlError> {
        let id = Uuid::new_v4();
        let label = format!("tab-{}", id);

        let mut tab = Tab {
            id,
            url: url.clone(),
            title: "New Tab".to_string(),
            favicon: None,
            is_background,
            created_at: Utc::now(),
            fallback_mode: false,
            loaded: false,
            webview: None,
        };

        if let Some(main_window) = app.get_window("main") {
            let parsed_url = url
                .parse()
                .unwrap_or_else(|_| "about:blank".parse().unwrap());

            // Initialization scripts for Fix 2
            let init_script = r#"
                Object.defineProperty(navigator, 'webdriver', { get: () => undefined });
                document.addEventListener('DOMContentLoaded', () => {
                  document.querySelectorAll('video, audio').forEach(el => {
                    el.autoplay = true;
                    el.muted = false;
                  });
                });
            "#;

            let state_clone = self.state.clone();
            let id_clone = id;

            let builder = WebviewBuilder::new(&label, WebviewUrl::External(parsed_url))
                .user_agent(CHROME_USER_AGENT)
                .initialization_script(init_script)
                .on_page_load(move |webview, _payload| {
                    let mut state = state_clone.write();
                    if let Some(t) = state.tabs.iter_mut().find(|t| t.id == id_clone) {
                        t.loaded = true;

                        let webview_clone = webview.clone();
                        tauri::async_runtime::spawn(async move {
                            tokio::time::sleep(Duration::from_millis(1000)).await;
                            let js = format!(r#"
                                (function() {{
                                    const data = {{
                                        id: '{}',
                                        title: document.title,
                                        favicon: document.querySelector('link[rel~="icon"]')?.href || ""
                                    }};
                                    window.__TAURI__.event.emit('tab-metadata', data);
                                }})()
                            "#, id_clone);
                            let _ = webview_clone.eval(&js);
                        });
                    }
                });

            let scale_factor = main_window.scale_factor().unwrap_or(1.0);
            
            if let Ok(webview) = main_window.add_child(
                builder,
                tauri::PhysicalPosition::new((x * scale_factor) as i32, (y * scale_factor) as i32),
                tauri::PhysicalSize::new((width.max(1.0) * scale_factor) as u32, (height.max(1.0) * scale_factor) as u32),
            ) {
                if is_background {
                    let _ = webview.hide();
                }
                tab.webview = Some(webview);
            }

            // Spawn 10s timeout to trigger fallback
            let url_clone = url.clone();
            if !url_clone.starts_with("cntrl://") && url_clone != "about:blank" {
                let state_clone2 = self.state.clone();
                let app_clone = app.clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(Duration::from_secs(10)).await;
                    let mut state = state_clone2.write();
                    if let Some(t) = state.tabs.iter_mut().find(|t| t.id == id_clone) {
                        if !t.loaded {
                            println!(
                                "Navigation timed out for {}. Triggering fallback.",
                                url_clone
                            );
                            t.fallback_mode = true;
                            if let Some(w) = app_clone.get_webview(&format!("tab-{}", id_clone)) {
                                let _ = w.hide(); // Hide native webview so iframe can show
                            }
                            let _ = app_clone.emit("tabs-updated", ());
                        }
                    }
                });
            } else {
                // Instantly mark internal pages as loaded
                let mut state = self.state.write();
                if let Some(t) = state.tabs.iter_mut().find(|t| t.id == id) {
                    t.loaded = true;
                }
            }
        }

        let mut state = self.state.write();
        state.tabs.push(tab);

        if !is_background {
            let prev_active = state.active_tab_id;
            state.active_tab_id = Some(id);
            // Hide previous active, show new
            if let Some(prev_id) = prev_active {
                if let Some(t) = state.tabs.iter().find(|t| t.id == prev_id) {
                    if let Some(w) = &t.webview {
                        let _ = w.hide();
                    }
                }
            }
            if let Some(t) = state.tabs.iter().find(|t| t.id == id) {
                if let Some(w) = &t.webview {
                    let _ = w.show();
                }
            }
        }

        Ok(id)
    }

    pub fn close_tab(
        &self,
        app: &tauri::AppHandle,
        id: Uuid,
    ) -> Result<(), CntrlError> {
        let mut state = self.state.write();
        let tab_to_close = state.tabs.iter().find(|t| t.id == id).cloned();
        state.tabs.retain(|t| t.id != id);

        if let Some(t) = tab_to_close {
            if let Some(w) = t.webview {
                let _ = w.close();
            }
        }

        if state.active_tab_id == Some(id) {
            state.active_tab_id = state.tabs.last().map(|t| t.id);
            if let Some(active_id) = state.active_tab_id {
                if let Some(t) = state.tabs.iter().find(|t| t.id == active_id) {
                    if let Some(w) = &t.webview {
                        let _ = w.show();
                    }
                }
            }
        }

        Ok(())
    }

    pub fn navigate(
        &self,
        app: &tauri::AppHandle,
        id: Uuid,
        url: String,
    ) -> Result<(), CntrlError> {
        let mut state = self.state.write();
        if let Some(tab) = state.tabs.iter_mut().find(|t| t.id == id) {
            tab.url = url.clone();
            tab.fallback_mode = false; // Reset fallback
            tab.loaded = false;

            if let Some(w) = &tab.webview {
                if url.starts_with("cntrl://") {
                    let _ = w.hide();
                } else if let Ok(parsed_url) = url.parse() {
                    let _ = w.navigate(parsed_url);
                    let _ = w.show(); // Ensure native is visible again since fallback might have hidden it
                }
            }

            // Spawn timeout for navigation
            let url_clone = url.clone();
            if !url_clone.starts_with("cntrl://") && url_clone != "about:blank" {
                let state_clone = self.state.clone();
                let app_clone = app.clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(Duration::from_secs(10)).await;
                    let mut state = state_clone.write();
                    if let Some(t) = state.tabs.iter_mut().find(|t| t.id == id) {
                        if !t.loaded {
                            println!(
                                "Navigation timed out for {}. Triggering fallback.",
                                url_clone
                            );
                            t.fallback_mode = true;
                            if let Some(w) = &t.webview {
                                let _ = w.hide();
                            }
                            let _ = app_clone.emit("tabs-updated", ());
                        }
                    }
                });
            } else {
                tab.loaded = true;
            }

            Ok(())
        } else {
            Err(CntrlError::Browser(format!("Tab {} not found", id)))
        }
    }

    pub fn get_tabs(&self) -> Result<Vec<Tab>, CntrlError> {
        let state = self.state.read();
        Ok(state.tabs.clone())
    }

    pub fn get_active_tab_id(&self) -> Result<Option<Uuid>, CntrlError> {
        let state = self.state.read();
        Ok(state.active_tab_id)
    }

    pub fn set_active_tab(
        &self,
        app: &tauri::AppHandle,
        id: Uuid,
    ) -> Result<(), CntrlError> {
        let mut state = self.state.write();
        if state.tabs.iter().any(|t| t.id == id) {
            let prev_active = state.active_tab_id;
            state.active_tab_id = Some(id);

            if let Some(prev_id) = prev_active {
                if prev_id != id {
                    if let Some(t) = state.tabs.iter().find(|t| t.id == prev_id) {
                        if let Some(w) = &t.webview {
                            let _ = w.hide();
                        }
                    }
                }
            }
            if let Some(tab) = state.tabs.iter().find(|t| t.id == id) {
                if !tab.url.starts_with("cntrl://") && !tab.fallback_mode {
                    if let Some(w) = &tab.webview {
                        let _ = w.show();
                    }
                } else if tab.url.starts_with("cntrl://") {
                    if let Some(w) = &tab.webview {
                        let _ = w.hide();
                    }
                }
            }
            Ok(())
        } else {
            Err(CntrlError::Browser(format!("Tab {} not found", id)))
        }
    }

    pub fn update_tab_bounds(
        &self,
        app: &tauri::AppHandle,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> Result<(), CntrlError> {
        let state = self.state.read();
        let scale_factor = app.get_window("main").map(|w| w.scale_factor().unwrap_or(1.0)).unwrap_or(1.0);
        
        if let Some(active_id) = state.active_tab_id {
            if let Some(t) = state.tabs.iter().find(|t| t.id == active_id) {
                if let Some(w) = &t.webview {
                    let w_clone = w.clone();
                    let _ = app.run_on_main_thread(move || {
                        if let Err(e) = w_clone.set_bounds(tauri::Rect {
                            position: tauri::Position::Physical(tauri::PhysicalPosition::new((x * scale_factor) as i32, (y * scale_factor) as i32)),
                            size: tauri::Size::Physical(tauri::PhysicalSize::new((width * scale_factor) as u32, (height * scale_factor) as u32)),
                        }) {
                            println!("Failed to set bounds on {}: {}", active_id, e);
                        }
                    });
                } else {
                    println!("Webview tab-{} NOT FOUND in struct for bounds update!", active_id);
                }
            }
        }
        Ok(())
    }

    pub fn go_back(
        &self,
        app: &tauri::AppHandle,
        id: Uuid,
    ) -> Result<(), CntrlError> {
        let state = self.state.read();
        if let Some(t) = state.tabs.iter().find(|t| t.id == id) {
            if let Some(w) = &t.webview {
                let _ = w.eval("window.history.back()");
            }
        }
        Ok(())
    }

    pub fn go_forward(
        &self,
        app: &tauri::AppHandle,
        id: Uuid,
    ) -> Result<(), CntrlError> {
        let state = self.state.read();
        if let Some(t) = state.tabs.iter().find(|t| t.id == id) {
            if let Some(w) = &t.webview {
                let _ = w.eval("window.history.forward()");
            }
        }
        Ok(())
    }

    pub fn reload(
        &self,
        app: &tauri::AppHandle,
        id: Uuid,
    ) -> Result<(), CntrlError> {
        let state = self.state.read();
        if let Some(t) = state.tabs.iter().find(|t| t.id == id) {
            if let Some(w) = &t.webview {
                let _ = w.eval("window.location.reload()");
            }
        }
        Ok(())
    }

    pub fn update_metadata(
        &self,
        id: Uuid,
        title: String,
        favicon: String,
    ) -> Result<(), CntrlError> {
        let mut state = self.state.write();
        if let Some(t) = state.tabs.iter_mut().find(|t| t.id == id) {
            if !title.is_empty() {
                t.title = title;
            }
            if !favicon.is_empty() {
                t.favicon = Some(favicon);
            }
        }
        Ok(())
    }
}
