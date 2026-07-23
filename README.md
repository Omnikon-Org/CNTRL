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
- 🛠️ **[TROUBLESHOOT.md](./TROUBLESHOOT.md)** — Common installation, build, runtime, and dependency troubleshooting steps.

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

### Missing WebView dependencies

Refer to the official Tauri prerequisites documentation: <https://v2.tauri.app/start/prerequisites/>

### Additional notes

- Use the latest stable versions of Node.js and Rust where possible.
- If you hit dependency issues, delete `node_modules` and reinstall:
  ```bash
  npm install
  ```
- If problems persist, update your Rust toolchain:
  ```bash
  rustup update
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

## Documentation

Additional documentation is available in the `docs` directory.

- [Architecture](docs/ARCHITECTURE.md)
- [Roadmap](docs/ROADMAP.md)
- [Open Source Checklist](docs/OPEN_SOURCE_CHECKLIST.md)
- [Tauri Linux Troubleshooting Guide](docs/TAURI-LINUX.md)
- [Accessibility Guide](docs/ACCESSIBILITY.md)
- [Make a tiny UI change (beginner walk-through)](docs/UI_WALKTHROUGH.md)
## Contributing

We welcome contributions of all kinds — bug fixes, features, tests, and documentation improvements.

1. Fork the repo and clone your fork.
2. Create a branch off `main` following the naming conventions above (e.g. `feat/intent-scoring`).
3. Make your changes, write tests, and ensure all CI checks pass locally.
4. Open a Pull Request against **`main`** with a clear description.

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for the full contribution guide, code style requirements, and commit message conventions.

> **All pull requests must target `main`.** The `main` branch is protected – direct pushes are not allowed; every change goes through a reviewed PR that passes CI.

## Documentation

Additional documentation is available in the `docs` directory.

- [Architecture](docs/ARCHITECTURE.md)
- [Roadmap](docs/ROADMAP.md)
- [Open Source Checklist](docs/OPEN_SOURCE_CHECKLIST.md)
- [Tauri Linux Troubleshooting Guide](docs/TAURI-LINUX.md)
- [Make a tiny UI change (beginner walk-through)](docs/UI_WALKTHROUGH.md)
---

## 📜 License

[MIT License](./LICENSE) © 2026 CNTRL Browser Contributors.