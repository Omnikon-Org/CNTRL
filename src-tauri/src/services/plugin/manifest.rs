// services/plugin/manifest.rs
//
// ─────────────────────────────────────────────────────────────────────────────
// .vibe-plugin Format — Schema & Capabilities
// ─────────────────────────────────────────────────────────────────────────────
// A `.vibe-plugin` file is a zip archive containing:
// 1. `manifest.json` (Parsed into PluginManifest below)
// 2. `module.wasm` (The WebAssembly binary)
//
// SECURITY BOUNDARIES & CAPABILITIES:
// The CNTRL Plugin SDK is strictly unprivileged. Plugins CANNOT:
// - Access the local filesystem (except via explicit virtualized Memory FS if granted).
// - Execute arbitrary system shell commands.
// - Read the user's secure keychain or unredacted memories.
//
// Plugins CAN:
// - Intercept intent execution via `IntentExecution` hook (e.g. adding new intent handlers).
// - Perform isolated network requests via a host-provided `fetch` function (if NetworkAccess granted).
// ─────────────────────────────────────────────────────────────────────────────

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub entrypoint: String,
    pub permissions: Vec<PluginPermission>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PluginPermission {
    NetworkAccess,
    FileSystemAccess, // Only virtualized memory-fs
    IntentExecution,  // Allow registering custom intent handlers
}
