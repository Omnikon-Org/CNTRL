//! Integration tests for the Phase 5 memory and security layer.

use cntrl_browser_lib::services::audit::{get_recent_entries, log_ai_call, log_credential_access};
use cntrl_browser_lib::services::memory::db::open_in_memory;
use cntrl_browser_lib::services::memory::habits::{find_preferred_service, record_outcome};
use cntrl_browser_lib::services::memory::recall::{find_relevant_context, save_task};
use cntrl_browser_lib::services::privacy::PrivacyGuard;

#[tokio::test]
async fn test_db_init_and_tables() {
    let db = open_in_memory().await.expect("DB must open");

    // Verify habits
    record_outcome(&db, "navigate", "mail", "gmail.com")
        .await
        .unwrap();
    let preference = find_preferred_service(&db, "mail").await.unwrap().unwrap();
    assert_eq!(preference.preferred_service, "gmail.com");
}

#[tokio::test]
async fn test_privacy_mode_guard() {
    let db = open_in_memory().await.expect("DB must open");
    let guard = PrivacyGuard::load(&db).await.unwrap();

    assert!(!guard.is_enabled());
    assert!(guard.check_tier("Freemium").is_ok());

    guard.enable(&db).await.unwrap();
    assert!(guard.is_enabled());
    assert!(guard.check_tier("Freemium").is_err());
    assert!(guard.check_tier("Local").is_ok());
}

#[tokio::test]
async fn test_recall_context() {
    let db = open_in_memory().await.expect("DB must open");

    save_task(
        &db,
        "task-123",
        "check the weather forecast",
        "search",
        "{}",
        "done",
        Some("weather is sunny"),
    )
    .await
    .unwrap();

    let context = find_relevant_context(&db, "weather forecast", 5)
        .await
        .unwrap();
    assert_eq!(context.len(), 1);
    assert_eq!(context[0].intent_raw, "check the weather forecast");
}

#[tokio::test]
async fn test_audit_logs() {
    let db = open_in_memory().await.expect("DB must open");

    log_ai_call(&db, "test search", "Local", "Ollama", 150, Some(50), true)
        .await
        .unwrap();
    log_credential_access(&db, "cntrl-browser", "openrouter_key", "read")
        .await
        .unwrap();

    let entries = get_recent_entries(&db, 10).await.unwrap();
    assert_eq!(entries.len(), 2);

    assert_eq!(entries[0].entry_type, "credential_access");
    assert_eq!(entries[1].entry_type, "ai_call");
}
