# Changelog

All notable changes to the CNTRL Browser project will be documented in this file.

## [0.1.0] - 2026-07-19

### Added
- **Native Webview Engine**: Multi-tab native OS browser view (macOS WebKit, Windows WebView2, Linux WebKitGTK).
- **Playwright Fallback Engine**: Sandboxed fallback browser engine for complex layouts and compatibility rendering.
- **Secure Key Enclave**: 100% OS Keychain integration (macOS Keychain, Windows Credential Manager, Linux Secret Service).
- **Hybrid AI Router**: 3-tier routing supporting Ollama, Gemini, Groq, HuggingFace, and OpenAI-compatible endpoints.
- **Intent Planner & Executor**: Command bar (`Cmd+K`) natural language intent classification and step execution.
- **Encrypted Memory Engine**: Transactional SQLite persistence via `sqlx` and LanceDB vector store for semantic recall.
- **Background Agent & Scheduler**: `.vibe` macro format recorder, Tokio background runtime worker queue, and cron scheduler.
- **Design System & Theme**: Unified Mecha-Industrial visual tokens with light/dark mode toggle.
- **Documentation Suite**: Added `Important Documentation/` directory containing `PRD.md`, `Architecture.md`, `Rules.md`, `Design.md`, and `Memory.md`.

### Security
- Mandatory Privacy Mode lock blocking remote API calls when enabled.
- Cryptographically logged credential access audit trails.
- Wasmtime sandbox stub for third-party plugin isolation.
