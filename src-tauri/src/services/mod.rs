//! Services module — core business logic decoupled from Tauri IPC.
//!
//! Each submodule handles a distinct domain:
//! - [`ai`]       — AI provider trait, per-provider implementations, and router.
//! - [`browser`]  — Tab lifecycle and webview management.
//! - [`fallback`] — Playwright-based headless fallback for WebKit-hostile sites.
//! - [`keychain`] — OS-native secret storage (wraps the `keyring` crate).
//! - [`memory`]   — SQLite-backed memory: DB pool, habits, recall (Phase 5).
//! - [`privacy`]  — Privacy mode guard: blocks remote AI calls when enabled (Phase 5).
//! - [`audit`]    — Append-only audit log for AI calls and credential access (Phase 5).

pub mod ai;
pub mod audit;
pub mod background;
pub mod browser;
pub mod executor;
pub mod fallback;
pub mod intent;
pub mod keychain;
pub mod memory;
pub mod planner;
pub mod privacy;
