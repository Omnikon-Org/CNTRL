# CNTRL Browser

**Intent-based autonomous browsing**

CNTRL Browser is a lightweight, AI-driven autonomous browser built with Tauri (Rust backend) and SolidJS (TypeScript frontend). It is designed to interpret natural language intents and execute them autonomously across the web.

## Architecture Overview
- **Runtime**: Tauri v2
- **Backend**: Rust (Business logic, SQLite memory, OS Keychain, AI router, OS webview fallback)
- **Frontend**: SolidJS + TypeScript (State: Zustand/Solid stores, Styling: CSS custom properties)
- **AI Tiers**:
  - Tier 1 (Local): Ollama
  - Tier 2 (Freemium): Gemini Flash, Groq, Hugging Face
  - Tier 3 (Precision): OpenAI-compatible endpoints (Claude, GPT-4o, etc.)

## 7-Phase Build Plan
- [x] **Phase 1**: Project Scaffold & CI Pipeline
- [x] **Phase 2**: Webview Engine & Browser Chrome (Native fallback architecture)
- [x] **Phase 3**: Hybrid Brain & Model Router (Ollama/OpenRouter/HF integration)
- [x] **Phase 4**: Intent Layer & Command Bar (Natural Language Actions)
- [ ] **Phase 5**: Memory Engine & Security Layer
- [ ] **Phase 6**: Background Agents & Macro Recorder
- [ ] **Phase 7**: Design System, Plugin SDK & OSS Release

## Getting Started

### Prerequisites
- Node.js 20+
- Rust stable toolchain
- Tauri v2 prerequisites (OS-specific webview dev headers)

### Run Locally
```bash
npm install
npm run tauri dev
```

## Contributing
Please see the CONTRIBUTING.md for details (coming in Phase 7).
All contributions must follow our conventional commits and pass all CI checks (linting, formatting, testing).
