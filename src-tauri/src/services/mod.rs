//! Services module — core business logic decoupled from Tauri IPC.
//!
//! Each submodule handles a distinct domain:
//! - [`ai`]       — AI provider trait, per-provider implementations, and router.
//! - [`browser`]  — Tab lifecycle and webview management.
//! - [`fallback`] — Playwright-based headless fallback for WebKit-hostile sites.
//! - [`keychain`] — OS-native secret storage (wraps the `keyring` crate).

pub mod ai;
pub mod background;
pub mod browser;
pub mod executor;
pub mod fallback;
pub mod intent;
pub mod keychain;
pub mod planner;
