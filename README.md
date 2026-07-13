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

| Phase | Description | Status |
|-------|-------------|--------|
| **Phase 1** | Project Scaffold & CI Pipeline | ✅ Complete |
| **Phase 2** | Webview Engine & Browser Chrome | ✅ Complete |
| **Phase 3** | Hybrid Brain & Model Router | ✅ Complete |
| **Phase 4** | Intent Layer & Command Bar | ✅ Complete |
| **Phase 5** | Memory Engine & Security Layer | 🔲 Not started |
| **Phase 6** | Background Agents & Macro Recorder | 🔲 Not started |
| **Phase 7** | Design System, Plugin SDK & OSS Release | 🔲 Not started |

### Phase 1 — Project Scaffold & CI Pipeline
Tauri v2 + SolidJS + TypeScript monorepo. Full CI pipeline: Clippy, rustfmt, `cargo test`, `tsc --noEmit`, ESLint, Vitest. Global error types via `thiserror`. Biome formatter. Strict TypeScript with `noUncheckedIndexedAccess` and `exactOptionalPropertyTypes`.

### Phase 2 — Webview Engine & Browser Chrome
Native OS webview per tab. `BrowserService` managing tab lifecycle (open, close, navigate, back, forward, reload). Tab bar with Cmd+T / Cmd+W / Cmd+Shift+T. URL bar with HTTPS lock icon and HTTP warning indicator. Playwright-based headless fallback for WebKit-hostile sites, rendered in a sandboxed iframe.

### Phase 3 — Hybrid Brain & Model Router
Trait-based provider system with per-provider files under `services/ai/`. Tier 1 (Ollama), Tier 2 (Gemini, Groq, HuggingFace, OpenRouter), Tier 3 (generic OpenAI-compatible). Complexity scorer (0–10 int → tier mapping). All API keys stored in the OS keychain — zero plaintext secrets on disk. Settings UI with per-provider health indicators. OpenRouter free-model filter. HuggingFace model list and inference.

### Phase 4 — Intent Layer & Command Bar
Natural language command classification into 7 intent types. Multi-step task planner and executor. Cmd+K command bar overlay with live step feed.

### Phase 5 — Memory Engine & Security Layer *(planned)*
SQLite via `sqlx` for task history and habits. LanceDB semantic recall. OS keychain audit log. Privacy mode blocking remote AI calls.

### Phase 6 — Background Agents & Macro Recorder *(planned)*
Tokio background job queue. `.vibe` macro file format. Cron scheduling. OS notifications. Import/export and visual schedule picker.

### Phase 7 — Design System, Plugin SDK & OSS Release *(planned)*
Full Mecha-Industrial design token application. Light mode toggle. WASM plugin sandbox. OSS documentation, example macros, release pipeline.

---

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

#### Platform-Specific WebView Dependencies
Tauri depends on the operating system's native webview library. Follow the official setup guide for your OS:
- **macOS**: System default is sufficient, but ensure Xcode Command Line Tools are installed:
  ```bash
  xcode-select --install
  ```
- **Linux (Debian/Ubuntu)**: Install development headers:
  ```bash
  sudo apt update
  sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
  ```
- **Windows**: Tauri needs a working C++ linker. You have two toolchain options:
  - **MSVC (Recommended / Default on Windows)**: Install the [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) and select the **"Desktop development with C++"** workload. This provides `link.exe`, which the Rust compiler needs to produce the final binary.
  - **GNU (Alternative)**: If you would rather avoid Visual Studio, use the GNU toolchain instead:
    ```bash
    rustup toolchain install stable-x86_64-pc-windows-gnu
    rustup default stable-x86_64-pc-windows-gnu
    ```
    This requires MSYS2/MinGW-w64 on your `PATH`.
  - Also ensure that [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) is installed (installed by default on modern Windows 11).

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
*(The first run will take noticeably longer as Cargo compiles the Rust backend from scratch.)*

#### Option B: Standalone Frontend Web-Only Mode
If you are only editing UI components/styling and do not need any Tauri native API functionality, you can run the standalone SolidJS/Vite development server:

```bash
npm run dev
```
Open `http://localhost:1420/` in your browser.

---

### Troubleshooting & Gotchas

* **Windows Linker Error (`link.exe not found`)**
  - **Symptom**: Compilation fails with `error: linker 'link.exe' not found`.
  - **Fix**: Ensure Visual Studio Build Tools (or Visual Studio 2017+) is installed with the **"Desktop development with C++"** workload selected. If you do not want to install Visual Studio, switch to the GNU toolchain instead (see instructions under Windows prerequisites).
* **Windows Smart App Control (SAC) blocks the app — `os error 4551`**
  - **Symptom**: The app fails to launch with this error on Windows.
  - **Fix**: Windows Smart App Control blocks unsigned local debug binaries by default. 
    - Check whether SAC is on under: **Settings → Privacy & security → Windows Security → App & browser control → Smart App Control**. If it is, you can turn it off (note: this is a one-way decision on Windows unless you do a clean reinstall).
    - Alternatively, develop inside a virtual machine, on a machine without SAC enabled, or sign the binary if you have a certificate.
* **Tauri APIs in Standard Web Browsers**
  - **Symptom**: Standalone mode (`npm run dev`) console shows `TypeError: Cannot read properties of undefined (reading 'invoke')`.
  - **Fix**: This is expected because Tauri's bridge APIs are only available within the native desktop window wrapper. Use `npm run tauri dev` if you need features relying on native APIs.
* **Cargo/Rust Command Not Found**
  - **Symptom**: `cargo` or `rustc` is not recognized.
  - **Fix**: Restart your terminal or system after installing Rust to ensure PATH variables are updated.
* **Missing WebView dependencies**
  - **Fix**: Refer to the official Tauri prerequisites documentation: <https://v2.tauri.app/start/prerequisites/>

---

### Tested & Verified Environments

The local environment setup has been verified under the following environments:

**MSVC Windows Setup (Tested by Coding Assistant):**
- **Operating System**: Windows 11 Home (64-bit)
- **Node.js**: v24.13.1
- **npm**: 11.8.0
- **Rust (rustc/cargo)**: 1.96.1
- **Vite**: v6.4.2

**GNU Windows Setup (Tested by Community):**
- **Operating System**: Windows 11 Home
- **Node.js**: v24.13.1
- **Rust**: 1.96.1 (stable-x86_64-pc-windows-gnu)

---

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

## Contributing

We welcome contributions of all kinds — bug fixes, features, tests, and documentation improvements.

1. Fork the repo and clone your fork.
2. Create a branch off `main` following the naming conventions above (e.g. `feat/intent-scoring`).
3. Make your changes, write tests, and ensure all CI checks pass locally.
4. Open a Pull Request against **`main`** with a clear description.

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for the full contribution guide, code style requirements, and commit message conventions.

> **All pull requests must target `main`.** The `main` branch is protected – direct pushes are not allowed; every change goes through a reviewed PR that passes CI.