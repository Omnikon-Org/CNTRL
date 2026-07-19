# CNTRL Browser

**Intent-based autonomous browsing**

CNTRL Browser is a lightweight, local-first autonomous browser built with Tauri v2 (Rust backend) and SolidJS (TypeScript frontend). It interprets natural language intents, automates complex web workflows, and prioritizes user privacy with local LLMs and zero plaintext key storage.

---

## 📚 Important Documentation

Detailed architectural, PRD, and development documentation can be found in the [`Important Documentation/`](./Important%20Documentation/) directory:

- 📄 **[PRD.md](./Important%20Documentation/PRD.md)** — Project Requirements, capabilities, target audience, hurdles, and roadmap.
- 🏗️ **[Architecture.md](./Important%20Documentation/Architecture.md)** — System architecture, flow diagrams, technical stack, and IPC model.
- ⚖️ **[Rules.md](./Important%20Documentation/Rules.md)** — Core engineering guidelines, AI rules, and code constraints.
- 🎨 **[Design.md](./Important%20Documentation/Design.md)** — Mecha-Industrial design tokens, typography, dark/light themes.
- 🧠 **[Memory.md](./Important%20Documentation/Memory.md)** — Living development state tracker and project invariants.

---

## ✨ Features

- 🌐 **Native Webview Engine**: Lightweight native browser tabs on macOS (WebKit), Windows (WebView2), and Linux (WebKitGTK).
- 🛡️ **Playwright Fallback**: Sandboxed fallback engine for complex or WebKit-hostile web pages.
- 🧠 **3-Tier AI Router**: Seamless execution across Tier 1 (Ollama), Tier 2 (Gemini, Groq, HF), and Tier 3 (OpenAI-compatible endpoints).
- 🔐 **OS Keychain Enclave**: 100% encrypted credential storage in macOS Keychain, Windows Credential Manager, and Linux Secret Service.
- 💬 **Intent Command Bar**: Natural language command parser and step executor overlay (`Cmd+K`).
- 📼 **Macro Recorder & Scheduler**: Record visual browser action macros (`.vibe` format) and run them on cron schedules.
- 🔒 **Privacy Guard**: One-click total privacy lock blocking all remote AI API calls.
- 🧩 **WASM Plugin Sandbox**: Embedded Wasmtime runtime for secure third-party extension isolation.

---

## 🚀 Running Locally

### Prerequisites

- **Node.js 20+** — <https://nodejs.org/>
- **Rust (stable toolchain)** — <https://rustup.rs>
- **Protobuf compiler (`protoc`)** — required for LanceDB vector store
  - macOS: `brew install protobuf`
  - Linux: `sudo apt install protobuf-compiler`
  - Windows: `choco install protoc`
- **Tauri v2 CLI** — `cargo install tauri-cli`

### Installation & Run

```bash
# 1. Clone repository
git clone https://github.com/Omnikon-Org/CNTRL.git
cd CNTRL

# 2. Install dependencies
npm install

# 3. Run full desktop application
npm run tauri dev

# Alternatively, run frontend only (UI iteration)
npm run dev
```

---

## 🌿 Branching Model

CNTRL Browser follows a clean single-branch open-source workflow:

- **`main`**: The sole production branch. All pull requests target `main`.
- **`feat/<name>`**: Feature branches opened by contributors.
- **`fix/<name>`**: Bug fix or patch branches.

---

## 📜 License

[MIT License](./LICENSE) © 2026 CNTRL Browser Contributors.