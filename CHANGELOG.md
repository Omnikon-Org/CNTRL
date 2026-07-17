# Changelog

All notable changes to the CNTRL (Vibe Browser) project will be documented in this file.

## [0.1.0] - 2026-07-16

### Added
- **Phase 1: Foundation:** Project scaffolded with Tauri (Rust) and SolidJS. Added SQLite/LanceDB setup, core system schemas, and strict TypeScript/Clippy linters.
- **Phase 2: Headless Browser Core:** Implemented an integrated Playwright-driven headless browser instance, capable of executing commands securely from the Tauri backend while avoiding unsafe iframe usage. Added sandboxed Chromium fallback for complex layouts.
- **Phase 3: Secure Enclave & Memory:** Implemented AES-256-GCM encryption for all local databases. Stored API keys in the native OS Keychain (macOS/Windows/Linux). Created short-term and long-term memory retrieval systems (Habits, Fact Extraction).
- **Phase 4: Intent Router & Planner:** Built a dynamic AI router supporting Local (Ollama), OpenRouter, Groq, and Hugging Face. Added `Planner` to decompose high-level intents into atomic steps.
- **Phase 5: Executor & Audit Logging:** Implemented `Executor` to sequentially perform browser actions or system commands. Added full cryptographically-signed audit logging to trace all AI actions, with a user-facing Audit Viewer.
- **Phase 6: Macros & Background Agents:** Introduced the `.vibe` macro format, allowing users to record, save, and schedule intent sequences via a visual recorder. Implemented a persistent background worker for running tasks without blocking the UI.
- **Phase 7: Polish & Distribution:** Added Light Mode and a Unified Design System using CSS variables. Introduced a WebAssembly (Wasmtime) Sandbox stub for future safe plugin execution.

### Security
- Remote execution is completely disabled in Privacy Mode.
- All stored memories are encrypted on disk.
- Audit logs maintain immutable records of AI behavior.
