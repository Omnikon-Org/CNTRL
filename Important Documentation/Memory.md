# Development Memory & State Tracker — CNTRL Browser

> **Purpose**: This living context file maintains current state, architecture decisions, invariants, and progress checkpoints for human maintainers and AI assistants. It prevents token waste and eliminates context loss across chat sessions.

---

## 1. Project Invariants & Decisions

- **Single Branch Model**: `main` is the sole development and release branch. No lingering `phase-X-*` branches.
- **Database Name**: SQLite database file is `cntrl-browser.db` located in app data directory.
- **Secret Storage**: All API keys stored exclusively via OS Keychain (`services/keychain.rs`).
- **Events & State**: Decoupled EventBus in `src/core/events.ts` handles UI-wide commands (`TAB_OPEN_NEW`, `TAB_CLOSE_ACTIVE`, `TAB_REOPEN_LAST`).
- **Window Management**: Component `src/components/WindowControls.tsx` handles OS window minimize/maximize/close.

---

## 2. Completed Phase Map

- ✅ **Phase 1**: Tauri v2 + SolidJS scaffold, Clippy, strict TypeScript.
- ✅ **Phase 2**: Native OS child webviews + Playwright sandboxed iframe fallback.
- ✅ **Phase 3**: Hybrid AI router (Tier 1 Ollama, Tier 2 Gemini/Groq/HF, Tier 3 OpenAI-compat) + OS Keychain enclave.
- ✅ **Phase 4**: Natural language intent parser, planner, executor, and Cmd+K command bar overlay.
- ✅ **Phase 5**: SQLite memory engine (`sqlx`), LanceDB semantic vector recall, privacy guard, audit log.
- ✅ **Phase 6**: `.vibe` macro format, visual macro recorder, background Tokio runtime, cron scheduler, OS notifications.
- ✅ **Phase 7**: Mecha-Industrial design system, light/dark theme toggle, Wasmtime plugin sandbox stub.

---

## 3. Directory Quick Map for AI Assistants

| Task Domain | Canonical File Path |
|---|---|
| IPC Boundary / Setup | `src-tauri/src/lib.rs` |
| Native Webview Service | `src-tauri/src/services/browser.rs` |
| AI Model Routing | `src-tauri/src/services/ai/router.rs` |
| Intent Classification | `src-tauri/src/services/intent/mod.rs` |
| Memory & DB | `src-tauri/src/services/memory/db.rs` |
| Frontend Root | `src/App.tsx` |
| State Stores | `src/stores/browserStore.ts`, `src/stores/aiStore.ts`, `src/stores/macroStore.ts` |
| CSS Tokens | `src/styles/tokens.css` |
| Document Hub | `Important Documentation/` |

---

## 4. Verification Checkpoint Status

- `cargo check`: **PASSED** (0 warnings)
- `tsc --noEmit`: **PASSED**
- Remote Git Branches: **CONSOLIDATED** (`main` only)
