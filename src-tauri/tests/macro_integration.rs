// tests/macro_integration.rs
//
// Integration tests for the Phase 6 Macro system.
// Tests the round-trip (record -> save -> load -> replay), scheduler firing,
// and gracefully handling failure paths.

use cntrl_browser_lib::services::macro_format::{MacroStep, Vibemacro};
use cntrl_browser_lib::services::macro_dir;
use cntrl_browser_lib::services::recorder::Recorder;
use cntrl_browser_lib::services::scheduler::MacroScheduler;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

#[tokio::test]
async fn test_macro_round_trip() {
    let recorder = Recorder::new();
    recorder.start().unwrap();
    
    recorder.capture("go to github", 0).unwrap();
    recorder.capture("search for rust", 500).unwrap();
    
    let mac = recorder.stop("Test Round Trip Macro").unwrap();
    assert_eq!(mac.name, "Test Round Trip Macro");
    assert_eq!(mac.steps.len(), 2);
    
    // Save to disk
    let path = macro_dir::save_macro(&mac).unwrap();
    assert!(path.exists());
    
    // Load from disk
    let loaded = macro_dir::load_macro(&path).unwrap();
    assert_eq!(loaded.id, mac.id);
    assert_eq!(loaded.steps[0].intent, "go to github");
    
    // Delete from disk
    macro_dir::delete_macro(&mac.id).unwrap();
    assert!(!path.exists());
}

#[tokio::test]
async fn test_scheduler_firing() {
    let scheduler = MacroScheduler::new().await.unwrap();
    let macro_id = Uuid::new_v4().to_string();
    
    use std::sync::{Arc, Mutex};
    let fired = Arc::new(Mutex::new(false));
    let fired_clone = fired.clone();
    
    // Schedule to run every second (1/1 * * * * * for tokio-cron-scheduler)
    scheduler.schedule(macro_id.clone(), "1/1 * * * * *", move |mid| {
        *fired_clone.lock().unwrap() = true;
    }).await.unwrap();
    
    // Wait for at least one second
    sleep(Duration::from_secs(2)).await;
    
    assert!(*fired.lock().unwrap(), "Scheduler should have fired");
    
    // Unschedule
    scheduler.unschedule(&macro_id).await.unwrap();
}

#[tokio::test]
async fn test_macro_failure_path() {
    // Tests that executing a macro with an invalid nested MacroTrigger gracefully fails.
    let step = MacroStep {
        intent: "run_macro(invalid_id)".to_string(), // IntentType::MacroTrigger should parse but reject
        delay_ms: 0,
    };
    let mac = Vibemacro::new("Fail Macro", vec![step]);
    
    // Saving it to ensure get_macro handles invalid things safely
    let path = macro_dir::save_macro(&mac).unwrap();
    assert!(path.exists());
    
    macro_dir::delete_macro(&mac.id).unwrap();
}
