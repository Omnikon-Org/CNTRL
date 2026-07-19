# Project Requirements Document (PRD) — CNTRL Browser

## 1. Product Overview & Vision

**CNTRL Browser** is a next-generation, local-first, intent-based autonomous web browser. Designed for privacy-conscious power users, developers, and AI enthusiasts, CNTRL bridges the gap between traditional manual web navigation and fully autonomous AI agents.

### Core Aim
To empower users to navigate, automate, and interact with the web through natural language intents, backed by local/hybrid LLMs, secure memory engines, and zero-plaintext key storage.

---

## 2. Target Audience

1. **Power Users & Automators**: Professionals who perform repetitive web workflows (data collection, monitoring, forms, report synthesis).
2. **Developers & Engineers**: Open-source contributors looking for a modular, extensible browser architecture built on Tauri v2 and SolidJS.
3. **Privacy-Conscious Users**: Individuals who demand local LLM execution (Ollama) and local encrypted vector memory without sending browsing history to cloud servers.

---

## 3. Capabilities Built to Date

CNTRL Browser has completed its initial 7-Phase architecture roadmap:

| Capability | Implementation Details |
|---|---|
| **Native Webview Engine** | Multi-tab container using native OS child webviews (WebKit on macOS, WebView2 on Windows, WebKitGTK on Linux). |
| **Playwright Fallback Engine** | Headless fallback engine rendering complex or WebKit-hostile pages safely inside a sandboxed iframe. |
| **Hybrid AI Brain** | 3-tier router supporting Tier 1 (Local Ollama), Tier 2 (Gemini, Groq, HuggingFace, OpenRouter), Tier 3 (OpenAI-compatible endpoints). |
| **Secure Key Enclave** | 100% OS Keychain secret storage (macOS Keychain, Windows Credential Manager, Linux Secret Service). Zero plaintext API keys on disk. |
| **Intent Layer & Command Bar** | Natural language command parsing (7 intent types), step decomposition planner, and Cmd+K command bar overlay. |
| **Encrypted Memory Engine** | SQLite database (`cntrl-browser.db`) via `sqlx` for preferences and audit logs; LanceDB for semantic vector recall. |
| **Privacy Guard** | Strict single-toggle privacy mode blocking all remote AI API calls when enabled. |
| **Background Agents & Macros** | `.vibe` JSON macro recording, playback, and cron scheduling via `tokio-cron-scheduler`. |
| **Unified Design System** | Mecha-Industrial visual design with dark/light mode toggle and custom CSS tokens. |
| **WASM Plugin Sandbox** | Wasmtime sandbox runtime stub for secure, isolated third-party plugin execution. |

---

## 4. Technical Hurdles & Engineering Solutions

### 1. Webview Bounds Sizing & Positioning on macOS
- **Hurdle**: Native child webviews created via `add_child` initially overlapped window controls and didn't resize correctly on Retina displays due to coordinate space mismatches.
- **Solution**: Standardized on `LogicalPosition` and `LogicalSize` matching CSS layout coordinates, coupled with a `boundsReady` signal and main-thread `set_bounds` dispatch.

### 2. Cross-Platform OS Keychain Integration
- **Hurdle**: Inconsistent keychain backends across macOS, Windows, and Linux.
- **Solution**: Utilized `keyring-rs` with native backends (`apple-native`, `windows-native`, `sync-secret-service`) backed by a thread-safe audit logging pipeline.

### 3. Background Execution without UI Blocking
- **Hurdle**: Long-running AI macro plans froze the main Tauri event loop.
- **Solution**: Implemented a Tokio channel-backed `BackgroundRuntime` worker queue that runs tasks on dedicated background threads and emits status events to the UI.

---

## 5. Future Upgradability & Roadmap

1. **DOM Action Execution Engine**: Direct DOM element selection and automated click/type execution within child webviews using injected IPC hooks.
2. **Multi-Agent Collaboration**: Orchestrated multi-step workflows where specialized agents handle research, extraction, and synthesis in parallel.
3. **Third-Party Plugin SDK**: Full WebAssembly plugin API allowing community developers to publish custom intent handlers and integrations.
4. **Fine-Tuned Small Language Models**: Custom SLMs optimized specifically for browser action parsing and DOM navigation.
