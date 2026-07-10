//! Integration tests for `BrowserService`.
//!
//! These tests use `tauri::test::mock_app()` to obtain a real `AppHandle`
//! without spinning up a visible window, so they can exercise the full
//! service-layer logic including webview registration bookkeeping.
//!
//! Coverage requirements (Phase 2 gate):
//! - `open_tab` returns a valid UUID
//! - `navigate` updates the tab's URL field
//! - `close_tab` removes the tab from the list
//! - Background tab is fetchable (is_background = true) without becoming active

use cntrl_browser_lib::services::browser::BrowserService;

/// Helper: create a mock app and a fresh BrowserService.
fn setup() -> (tauri::App<tauri::test::MockRuntime>, BrowserService) {
    let app = tauri::test::mock_app();
    let service = BrowserService::new();
    (app, service)
}

// ─────────────────────────────────────────────────────────────────────────────
// open_tab
// ─────────────────────────────────────────────────────────────────────────────

/// `open_tab` must return a UUID and register the tab in the tab list.
#[test]
fn open_tab_returns_valid_uuid_and_registers_tab() {
    let (app, service) = setup();
    let handle = app.handle();

    let id = service
        .open_tab(handle, "https://example.com".to_string(), false)
        .expect("open_tab should succeed");

    let tabs = service.get_tabs().expect("get_tabs should succeed");
    assert_eq!(tabs.len(), 1, "exactly one tab should exist");
    assert_eq!(tabs[0].id, id, "tab id must match the returned uuid");
    assert_eq!(tabs[0].url, "https://example.com", "url must be set correctly");
    assert!(!tabs[0].is_background, "tab should not be a background tab");
}

/// Opening a second foreground tab should give a distinct UUID.
#[test]
fn open_two_foreground_tabs_have_distinct_ids() {
    let (app, service) = setup();
    let handle = app.handle();

    let id1 = service
        .open_tab(handle, "https://one.example.com".to_string(), false)
        .expect("first open_tab should succeed");
    let id2 = service
        .open_tab(handle, "https://two.example.com".to_string(), false)
        .expect("second open_tab should succeed");

    assert_ne!(id1, id2, "tab ids must be unique");
    let tabs = service.get_tabs().expect("get_tabs should succeed");
    assert_eq!(tabs.len(), 2, "two tabs should exist");
}

// ─────────────────────────────────────────────────────────────────────────────
// navigate
// ─────────────────────────────────────────────────────────────────────────────

/// `navigate` must update the tab's URL field and reset `fallback_mode`.
#[test]
fn navigate_updates_tab_url() {
    let (app, service) = setup();
    let handle = app.handle();

    let id = service
        .open_tab(handle, "https://example.com".to_string(), false)
        .expect("open_tab should succeed");

    service
        .navigate(handle, id, "https://news.ycombinator.com".to_string())
        .expect("navigate should succeed");

    let tabs = service.get_tabs().expect("get_tabs should succeed");
    assert_eq!(
        tabs[0].url, "https://news.ycombinator.com",
        "URL must be updated after navigate"
    );
    assert!(
        !tabs[0].fallback_mode,
        "fallback_mode must be reset on navigate"
    );
}

/// Navigating a non-existent tab ID must return an error, not panic.
#[test]
fn navigate_unknown_tab_returns_error() {
    let (app, service) = setup();
    let handle = app.handle();
    let fake_id = uuid::Uuid::new_v4();

    let result = service.navigate(handle, fake_id, "https://example.com".to_string());
    assert!(result.is_err(), "navigate to unknown tab must return Err");
}

// ─────────────────────────────────────────────────────────────────────────────
// close_tab
// ─────────────────────────────────────────────────────────────────────────────

/// `close_tab` must remove the tab from the list.
#[test]
fn close_tab_removes_it_from_list() {
    let (app, service) = setup();
    let handle = app.handle();

    let id = service
        .open_tab(handle, "https://example.com".to_string(), false)
        .expect("open_tab should succeed");

    service.close_tab(handle, id).expect("close_tab should succeed");

    let tabs = service.get_tabs().expect("get_tabs should succeed");
    assert!(tabs.is_empty(), "tab list must be empty after close");
}

/// Closing one of two tabs leaves the other intact.
#[test]
fn close_tab_leaves_other_tab_intact() {
    let (app, service) = setup();
    let handle = app.handle();

    let id1 = service
        .open_tab(handle, "https://first.example.com".to_string(), false)
        .expect("open first tab");
    let id2 = service
        .open_tab(handle, "https://second.example.com".to_string(), false)
        .expect("open second tab");

    service.close_tab(handle, id1).expect("close first tab");

    let tabs = service.get_tabs().expect("get_tabs should succeed");
    assert_eq!(tabs.len(), 1, "one tab must remain");
    assert_eq!(tabs[0].id, id2, "the remaining tab must be the second one");
}

// ─────────────────────────────────────────────────────────────────────────────
// Background tabs
// ─────────────────────────────────────────────────────────────────────────────

/// Opening a background tab must set `is_background = true` and keep it
/// accessible via `get_tabs` without it becoming the active tab.
#[test]
fn background_tab_is_registered_and_fetchable() {
    let (app, service) = setup();
    let handle = app.handle();

    let fg_id = service
        .open_tab(handle, "https://foreground.example.com".to_string(), false)
        .expect("open foreground tab");

    let bg_id = service
        .open_tab(handle, "https://background.example.com".to_string(), true)
        .expect("open background tab");

    let tabs = service.get_tabs().expect("get_tabs should succeed");
    assert_eq!(tabs.len(), 2, "both tabs must be in the list");

    let bg_tab = tabs
        .iter()
        .find(|t| t.id == bg_id)
        .expect("background tab must be in the list");
    assert!(bg_tab.is_background, "background tab must have is_background=true");
    assert_eq!(
        bg_tab.url, "https://background.example.com",
        "background tab URL must be set"
    );

    // The foreground tab must still be accessible too
    let fg_tab = tabs
        .iter()
        .find(|t| t.id == fg_id)
        .expect("foreground tab must still be in the list");
    assert!(!fg_tab.is_background, "foreground tab must not be a background tab");
}

/// A background tab must be closeable independently of foreground tabs.
#[test]
fn close_background_tab_leaves_foreground_tab() {
    let (app, service) = setup();
    let handle = app.handle();

    service
        .open_tab(handle, "https://foreground.example.com".to_string(), false)
        .expect("open foreground tab");
    let bg_id = service
        .open_tab(handle, "https://background.example.com".to_string(), true)
        .expect("open background tab");

    service.close_tab(handle, bg_id).expect("close background tab");

    let tabs = service.get_tabs().expect("get_tabs should succeed");
    assert_eq!(tabs.len(), 1, "only the foreground tab must remain");
    assert!(!tabs[0].is_background);
}

// ─────────────────────────────────────────────────────────────────────────────
// Full lifecycle (original combined test kept and expanded)
// ─────────────────────────────────────────────────────────────────────────────

/// Exercises the complete tab lifecycle: open → navigate → open background → close.
#[test]
fn browser_service_full_lifecycle() {
    let (app, service) = setup();
    let handle = app.handle();

    // 1. Open foreground tab
    let id = service
        .open_tab(handle, "https://example.com".to_string(), false)
        .expect("open_tab should succeed");
    let tabs = service.get_tabs().expect("get_tabs should succeed");
    assert_eq!(tabs.len(), 1);
    assert_eq!(tabs[0].id, id);
    assert_eq!(tabs[0].url, "https://example.com");

    // 2. Navigate
    service
        .navigate(handle, id, "https://news.ycombinator.com".to_string())
        .expect("navigate should succeed");
    let tabs = service.get_tabs().expect("get_tabs should succeed");
    assert_eq!(tabs[0].url, "https://news.ycombinator.com");

    // 3. Open background tab
    let bg_id = service
        .open_tab(handle, "https://reddit.com".to_string(), true)
        .expect("open background tab should succeed");
    let tabs = service.get_tabs().expect("get_tabs should succeed");
    assert_eq!(tabs.len(), 2);
    let bg_tab = tabs.iter().find(|t| t.id == bg_id).expect("bg tab must exist");
    assert!(bg_tab.is_background);

    // 4. Close foreground tab
    service.close_tab(handle, id).expect("close_tab should succeed");
    let tabs = service.get_tabs().expect("get_tabs should succeed");
    assert_eq!(tabs.len(), 1);
    assert_eq!(tabs[0].id, bg_id);
}
