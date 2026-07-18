# Background Agent

The Background Agent (Phase 6) allows recorded macros (`.vibe` files) to run headlessly.

## Architecture
- It leverages Tokio `async_runtime::spawn` to detach the execution from the main Tauri UI thread.
- `MacroScheduler` leverages `tokio-cron-scheduler` to emit trigger events based on parsed cron expressions.
- Native OS Notifications notify the user when a macro starts, completes, or fails in the background.
