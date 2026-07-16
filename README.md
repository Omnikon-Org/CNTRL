# CNTRL Browser

**Intent-based autonomous browsing**

CNTRL Browser is a lightweight, AI-driven autonomous browser built with Tauri (Rust backend) and SolidJS (TypeScript frontend). It is designed to interpret natural language intents and execute them autonomously across the web.

## Architecture Overview
- **Runtime**: Tauri v2
- **Backend**: Rust (Business logic, SQLite memory, OS Keychain, AI router, OS webview fallback)
- **Frontend**: SolidJS + TypeScript (State: Solid stores, Styling: CSS custom properties)
- **AI Tiers**:
  - Tier 1 (Local): Ollama
  - Tier 2 (Freemium): Gemini Flash, Groq, Hugging Face
  - Tier 3 (Precision): OpenAI-compatible endpoints (Claude, GPT-4o, etc.)

## 7-Phase Build Plan
- [x] **Phase 1**: Project Scaffold & CI Pipeline
- [x] **Phase 2**: Webview Engine & Browser Chrome (Native fallback architecture)
- [x] **Phase 3**: Hybrid Brain & Model Router (Ollama/OpenRouter/HF integration)
- [ ] **Phase 4**: Intent Layer & Command Bar (Natural Language Actions)
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

## Branching Model

CNTRL Browser uses a straightforward OSS branching strategy:

| Branch | Purpose |
|---|---|
| `main` | **Stable integration branch.** All contributor PRs target here. Always in a passing CI state. |
| `phase-X-*` | Internal milestone branches used by core maintainers. Merged into `main` when a phase is complete. |
| `feat/<name>` | Feature branches opened by contributors. Branch from `main`, PR back to `main`. |
| `fix/<name>` | Bug fix or hotfix branches. Branch from `main`, PR back to `main`. |
| `docs/<name>` | Documentation-only changes. Branch from `main`, PR back to `main`. |

> **All pull requests must target `main`.** The `main` branch is protected — direct pushes are not allowed; every change goes through a reviewed PR that passes CI.

## Contributing

We welcome contributions of all kinds — bug fixes, features, tests, and documentation improvements.

1. Fork the repo and clone your fork.
2. Create a branch off `main` following the naming conventions above (e.g. `feat/intent-scoring`).
3. Make your changes, write tests, and ensure all CI checks pass locally.
4. Open a Pull Request against **`main`** with a clear description.

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for the full contribution guide, code style requirements, and commit message conventions.
## Continuous Integration (CI)

A CI workflow automatically runs on all Pull Requests targeting `main` and `phase-1-scaffold`. It ensures:
*   **Frontend Linting:** Validates TypeScript files using the project linter.
*   **Rust Check:** Verifies backend compilation using `cargo check`.

Please ensure your local environment builds and lints cleanly before submitting a PR!