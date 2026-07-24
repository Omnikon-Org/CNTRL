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

## 🚀 Running the Project Locally

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

#### Protobuf Compiler (`protoc`)
Required for LanceDB vector store:
- **macOS**: `brew install protobuf`
- **Linux**: `sudo apt install protobuf-compiler`
- **Windows**: `choco install protoc` or download binary from GitHub releases.

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
git clone https://github.com/Omnikon-Org/CNTRL.git
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
    - Check whether SAC is on under: **Settings → Privacy & security → Windows Security → App & browser control → Smart App Control**. If it is, you can turn it off.
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

## 🌿 Branching Model

CNTRL Browser follows a clean single-branch open-source workflow:

| Branch | Purpose |
|---|---|
| `main` | **Stable integration branch.** All contributor PRs target here. Always in a passing CI state. |
| `feat/<name>` | Feature branches opened by contributors. Branch from `main`, PR back to `main`. |
| `fix/<name>` | Bug fix or hotfix branches. Branch from `main`, PR back to `main`. |
| `docs/<name>` | Documentation-only changes. Branch from `main`, PR back to `main`. |

> **All pull requests must target `main`.** The `main` branch is protected — direct pushes are not allowed; every change goes through a reviewed PR that passes CI.

---

## 📖 Documentation

Additional documentation is available in the `docs` directory:

- [Architecture](docs/ARCHITECTURE.md)
- [Roadmap](docs/ROADMAP.md)
- [Open Source Checklist](docs/OPEN_SOURCE_CHECKLIST.md)
- [Tauri Linux Troubleshooting Guide](docs/TAURI-LINUX.md)
- [Make a tiny UI change (beginner walk-through)](docs/UI_WALKTHROUGH.md)

---

## 🤝 Contributing

We welcome contributions of all kinds — bug fixes, features, tests, and documentation improvements.

1. Fork the repo and clone your fork.
2. Create a branch off `main` following the naming conventions above (e.g. `feat/intent-scoring`).
3. Make your changes, write tests, and ensure all CI checks pass locally.
4. Open a Pull Request against **`main`** with a clear description.

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for the full contribution guide, code style requirements, and commit message conventions.

---

## 📜 License

[MIT License](./LICENSE) © 2026 CNTRL Browser Contributors.
