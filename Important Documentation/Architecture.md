# Architecture & System Design — CNTRL Browser

## 1. System Overview

CNTRL Browser is built on a hybrid architecture combining a high-performance **Rust backend** (powered by Tauri v2) and a reactive **SolidJS frontend** (TypeScript).

```text
┌──────────────────────────────────────────────────────────────────┐
│                      SolidJS UI (Frontend)                       │
│  TabBar │ UrlBar │ WebView │ CommandBar │ MacroLibrary │ Settings │
└────────────────────────────────┬─────────────────────────────────┘
                                 │ Tauri IPC / EventBus
┌────────────────────────────────▼─────────────────────────────────┐
│                       Tauri v2 Rust Backend                      │
│                                                                  │
│  ┌─────────────────┐   ┌──────────────────┐   ┌───────────────┐ │
│  │ BrowserService  │   │  AI Model Router │   │ Memory Engine │ │
│  │ (Child Webviews)│   │ (Ollama/Cloud)   │   │ (SQLite/Lance)│ │
│  └─────────────────┘   └──────────────────┘   └───────────────┘ │
│  ┌─────────────────┐   ┌──────────────────┐   ┌───────────────┐ │
│  │ Intent Planner  │   │ Background Agent │   │ Keychain &    │ │
│  │ & Executor      │   │ Queue (Tokio)    │   │ Privacy Guard │ │
│  └─────────────────┘   └──────────────────┘   └───────────────┘ │
└──────────────────────────────────────────────────────────────────┘
```

---

## 2. Technical Stack

| Layer | Technology | Rationale |
|---|---|---|
| **Desktop Shell** | Tauri v2 | Lightweight binary size, native performance, low memory footprint. |
| **Backend Language** | Rust 2021 | Memory safety, speed, strict type correctness. |
| **Frontend Framework** | SolidJS + TypeScript | Fine-grained reactivity, zero virtual DOM overhead, small bundle. |
| **Relational DB** | SQLite (`sqlx`) | Embedded, serverless, transactional persistence for preferences & logs. |
| **Vector DB** | LanceDB | Embedded columnar vector store for semantic recall. |
| **Plugin Sandbox** | Wasmtime | Safe, WebAssembly-isolated plugin execution environment. |
| **Styling** | Vanilla CSS + Tokens | Zero framework bloat, maximum design flexibility. |

---

## 3. Directory Structure

```text
CNTRL/
├── Important Documentation/  # Core architectural & PRD reference docs
│   ├── PRD.md
│   ├── Architecture.md
│   ├── Rules.md
│   ├── Design.md
│   └── Memory.md
├── src/                      # SolidJS Frontend
│   ├── components/           # UI Components (TabBar, UrlBar, WebView, etc.)
│   ├── core/                 # Decoupled EventBus & utilities
│   ├── stores/               # SolidJS Stores (browserStore, aiStore, macroStore)
│   ├── styles/               # CSS Tokens & Global Theme
│   ├── types/                # TypeScript Interface Definitions
│   └── App.tsx               # Root Shell Component
├── src-tauri/                # Rust Backend
│   ├── src/
│   │   ├── commands/         # Tauri IPC Command Handlers
│   │   ├── services/         # Core Logic Services (browser, ai, memory, etc.)
│   │   │   ├── ai/           # LLM Providers (Ollama, Gemini, Groq, HF, OpenAI)
│   │   │   ├── background/   # Tokio Background Worker Queue
│   │   │   ├── intent/       # Natural Language Classification
│   │   │   ├── memory/       # SQLite & LanceDB Services
│   │   │   └── plugin/       # WASM Runtime Sandbox
│   │   ├── error.rs          # Shared CntrlError Enum
│   │   └── lib.rs            # Application Setup & Handler Registration
│   ├── Cargo.toml
│   └── tauri.conf.json
└── docs/                     # Secondary Developer Guides
```

---

## 4. Key Execution Flows

### 4.1 Tab Lifecycle & Webview Management
1. User requests a new tab via UI or `Cmd+T`.
2. `browserStore.ts` calls Rust `open_tab` command.
3. `BrowserService` creates a native OS child webview attached to the main window (`tab-{uuid}`).
4. `WebView.tsx` measures container bounds and dispatches `update_tab_bounds` to Rust.
5. Rust applies `LogicalPosition` and `LogicalSize` on the main thread via `app.run_on_main_thread`.

### 4.2 Intent Pipeline Execution
1. User types natural language query into `CommandBar.tsx` (`Cmd+K`).
2. Query is submitted to `submit_intent` Tauri command.
3. `IntentClassifier` categorizes query into 1 of 7 intent types.
4. `Planner` generates an ordered plan of atomic execution steps.
5. `Executor` runs steps (Navigation, Extraction, Summarization) via selected AI Provider while emitting step events to the UI.

### 4.3 Key & Privacy Handling
1. API Keys are stored directly into the OS Keychain via `keychain::store_secret`.
2. Secrets are masked in UI and never written to flat files.
3. When `PrivacyGuard` is enabled, any remote AI request (`Freemium` or `Premium`) returns an error before network calls are made.
