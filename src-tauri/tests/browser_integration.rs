use cntrl_browser_lib::services::browser::BrowserService;

#[test]
fn test_browser_service_lifecycle() {
    let app = tauri::test::mock_app();
    let handle = app.handle();
    let service = BrowserService::new();

    // 1. Open tab
    let id = service
        .open_tab(handle, "https://example.com".to_string(), false)
        .unwrap();
    let tabs = service.get_tabs().unwrap();
    assert_eq!(tabs.len(), 1);
    assert_eq!(tabs[0].id, id);
    assert_eq!(tabs[0].url, "https://example.com");

    // 2. Navigate
    service
        .navigate(handle, id, "https://news.ycombinator.com".to_string())
        .unwrap();
    let tabs = service.get_tabs().unwrap();
    assert_eq!(tabs[0].url, "https://news.ycombinator.com");

    // 3. Open background tab
    let bg_id = service
        .open_tab(handle, "https://reddit.com".to_string(), true)
        .unwrap();
    let tabs = service.get_tabs().unwrap();
    assert_eq!(tabs.len(), 2);
    let bg_tab = tabs.iter().find(|t| t.id == bg_id).unwrap();
    assert!(bg_tab.is_background);

    // 4. Close tab
    service.close_tab(handle, id).unwrap();
    let tabs = service.get_tabs().unwrap();
    assert_eq!(tabs.len(), 1);
    assert_eq!(tabs[0].id, bg_id);
}
