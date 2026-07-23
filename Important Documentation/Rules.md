# Architectural & AI Development Rules — CNTRL Browser

These rules govern all past, present, and future development on CNTRL Browser. Both human contributors and AI coding assistants MUST strictly follow these rules without exception.

---

## 1. Core Principles

1. **Simplest Correct Solution**: Prefer standard library primitives and framework built-ins. Avoid heavy wrapper abstractions unless required.
2. **Zero Plaintext Secrets**: NEVER write API keys, passwords, or authentication tokens to flat files, JSON configs, or logs. Always use the `keychain` service module.
3. **No Silent Failures**: All error paths in Rust and TypeScript must either surface to the user via UI/notifications or be logged explicitly with contextual error types (`CntrlError`).
4. **Decoupled Event Bus**: UI components must interact through state stores (`stores/`) or the decoupled `eventBus` (`core/events.ts`). Never pollute UI components with raw backend logic.

---

## 2. Technology & Library Constraints

### Allowed Stack
- **Frontend**: SolidJS, TypeScript (strict), Vanilla CSS (custom properties).
- **Backend**: Rust 2021, Tauri v2, Tokio, SQLite (`sqlx`), LanceDB, Wasmtime, Keyring.

### Explicitly Forbidden
- **No Tailwind CSS**: Use Vanilla CSS custom property design tokens in `src/styles/tokens.css`.
- **No Direct `cat` / Shell Mutation Files**: Use dedicated tool APIs (`write_to_file`, `replace_file_content`).
- **No Arbitrary Any Types**: TypeScript `no-explicit-any` must remain strictly enforced.

---

## 3. Rust Code Standards

- **Error Handling**: Use `thiserror` in `error.rs`. Derive `#[from]` for third-party errors (`std::io::Error`, `sqlx::Error`).
- **UI Thread Safety**: Any Webview layout or native window manipulation MUST run inside `app.run_on_main_thread(...)`.
- **Async Safety**: Avoid blocking the main thread. Long operations must use `tokio::spawn` or the `BackgroundRuntime`.

---

## 4. Frontend & Component Rules

- **Component Placement**: All UI components belong in `src/components/`. State stores belong in `src/stores/`. Event interfaces belong in `src/core/`.
- **Type Cleanliness**: Import common types from `src/types/` (e.g., `import type { Tab } from '../types'`). Do not duplicate type interfaces across components.
- **Accessibility**: All interactive buttons must have `aria-label` or visible text. Inputs must have clear labels/placeholders.
