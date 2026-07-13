# Changelog

All notable changes to the CNTRL Browser project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Phase 4: Intent Layer & Command Bar**
  - Built `services/intent` for natural language command parsing, supporting 7 core intents: `Navigation`, `Search`, `SystemCommand`, `AiQuery`, `MacroTrigger`, `SettingsAction`, and `UnknownFallback`.
  - Built `services/planner` to transform parsed intents into an ordered execution plan (`Step` array).
  - Built `services/executor` to run execution plans, featuring a live step-status feed via Tauri IPC events.
  - Added support for built-in system commands: `bitcoin_price` (via CoinGecko), `screenshot` (native macOS), and `mute` (native macOS).
  - Added `CommandBar.tsx`, a global Cmd+K triggered UI overlay to submit natural language intents and display live execution statuses.
  - Added `marked` and `dompurify` dependencies for safe and robust markdown rendering in the Command Bar.
  - Extracted SVGs from `UrlBar.tsx` into a new `Icons.tsx` component.
  - Added 21 intent integration tests in `tests/intent_integration.rs` covering all 7 intent types.

- **Phase 3: Hybrid Brain & Model Router**
  - Built a trait-based AI provider system (`services/ai/`).
  - Added support for Tier 1 (Ollama), Tier 2 (Gemini, Groq, HuggingFace, OpenRouter), and Tier 3 (OpenAI-compatible) AI models.
  - Created a complexity scorer to dynamically route queries to the most appropriate AI tier.
  - Implemented secure API key storage using the OS-native keychain.
  - Added a Settings UI with per-provider health checks and model selection.

- **Phase 2: Webview Engine & Browser Chrome**
  - Integrated native OS webviews per tab via `BrowserService`.
  - Added tab lifecycle management (open, close, navigate, back, forward, reload) and keyboard shortcuts.
  - Added Playwright-based headless fallback for WebKit-hostile websites, rendered in a sandboxed iframe.

- **Phase 1: Project Scaffold & CI Pipeline**
  - Initialized Tauri v2 + SolidJS + TypeScript monorepo.
  - Set up full CI pipeline with `cargo clippy`, `cargo test`, `tsc`, `eslint`, and `vitest`.
  - Configured `biome` for formatting and `thiserror` for global error handling in Rust.

### Changed
- Refactored `UrlBar.tsx` to streamline URL handling and remove inline SVGs.
- Fixed residual TypeScript warnings (`TS6133` in `SettingsPage.tsx`).
