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

## Running the project locally

This guide walks you through setting up and running the CNTRL browser locally for development.

### 1. Prerequisites

Ensure you have the following prerequisites installed on your system:

#### Node.js (LTS, Node 18+ or 20+)
- **Download**: [nodejs.org](https://nodejs.org/)
- **Verify**: `node -v` and `npm -v`

#### Rust (Stable toolchain)
- **macOS / Linux installation**:
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- **Windows installation**: Download and run the installer from [rustup.rs](https://rustup.rs/).
- **Verify**: `rustc --version` and `cargo --version`

#### Tauri v2 Prerequisites (OS-specific WebView libraries)
Tauri relies on native platform WebViews, which require additional development libraries:
- **macOS**: Install Xcode Command Line Tools:
  ```bash
  xcode-select --install
  ```
- **Linux**: Install development headers (e.g., `webkit2gtk`, `gtk3`, `appindicator` packages). For more detail, check the [Tauri Linux Troubleshooting Guide](docs/TAURI-LINUX.md) or follow the official [Tauri Prerequisites Guide](https://v2.tauri.app/start/prerequisites/).
- **Windows**: Install the **Visual Studio Build Tools** (Microsoft C++ Build Tools) with the **"C++ build tools"** workload selected in the installer. This is required for linking the native Rust binary.

#### Tauri CLI (Optional, but useful)
You can install the Tauri CLI toolchain globally via cargo:
```bash
cargo install tauri-cli
```
- **Verify**: `cargo tauri --version`

---

### 2. Clone the Repository

Clone the project repository and navigate into the folder:

```bash
git clone https://github.com/Demon-Die/CNTRL.git
cd CNTRL
```

---

### 3. Install JavaScript Dependencies

This project uses `npm` as its primary package manager (as verified by the `package-lock.json` lockfile). Run the following command in the project root:

```bash
npm install
```

---

### 4. Running the Development Application

Depending on what you are working on, you can run the app in one of two modes:

#### Option A: Full Desktop Application (Recommended)
This runs the full Tauri desktop environment, compiling the Rust backend and launching the native container:

```bash
npm run tauri dev
```

#### Option B: Frontend Web-Only Mode
If you are only editing UI components/styling and do not need any Tauri native API functionality, you can run the standalone SolidJS/Vite development server:

```bash
npm run dev
```
Open `http://localhost:1420/` in your browser.

---

### Gotchas & Common Issues

* **Windows Linker Error (`link.exe not found`)**
  * **Symptom**: Compilation fails with `error: linker 'link.exe' not found`.
  * **Fix**: Ensure Visual Studio Build Tools (or Visual Studio 2017+) is installed with the **"Desktop development with C++"** workload selected.
* **Tauri APIs in Standard Web Browsers**
  * **Symptom**: When running in Frontend Web-Only Mode (`npm run dev`), the browser console shows errors like `TypeError: Cannot read properties of undefined (reading 'invoke')`.
  * **Fix**: This is expected because Tauri's bridge APIs are only available within the native desktop window. To use features relying on Tauri native APIs, start the app with `npm run tauri dev`.
* **Cargo/Rust Command Not Found**
  * **Symptom**: `cargo` or `rustc` is not recognized.
  * **Fix**: Restart your terminal or system after installing Rust to ensure PATH variables are updated.

---

### Tested & Verified Environment

The setup instructions and commands have been successfully tested under the following environment:
- **Operating System**: Windows 11 Home (64-bit)
- **Node.js**: v24.13.1
- **npm**: 11.8.0
- **Rust (rustc/cargo)**: 1.96.1
- **Vite**: v6.4.2


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

> **All pull requests must target `main`.** The `main` branch is protected – direct pushes are not allowed; every change goes through a reviewed PR that passes CI.

## Documentation

Additional documentation is available in the `docs` directory.

- [Architecture](docs/ARCHITECTURE.md)
- [Roadmap](docs/ROADMAP.md)
- [Open Source Checklist](docs/OPEN_SOURCE_CHECKLIST.md)
- [Tauri Linux Troubleshooting Guide](docs/TAURI-LINUX.md)

## Contributing

We welcome contributions of all kinds – bug fixes, features, tests, and documentation improvements.