//! Services module — core business logic decoupled from Tauri IPC.
//!
//! Each submodule handles a distinct domain:
//! - [`ai`]       — AI provider trait, per-provider implementations, and router.
//! - [`browser`]  — Tab lifecycle and webview management.
//! - [`fallback`] — Playwright-based headless fallback for WebKit-hostile sites.
//! - [`keychain`] — OS-native secret storage (wraps the `keyring` crate).

pub mod ai;
pub mod browser;
pub mod fallback;
pub mod background;
pub mod keychain;
pub mod intent;
pub mod planner;
pub mod executor;
