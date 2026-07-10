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

[#running-the-project-locally](#running-the-project-locally)

### Prerequisites

[#prerequisites](#prerequisites)

- **Node.js 20+** — <https://nodejs.org/>
- **Rust (stable toolchain)** — <https://rustup.rs>
- **Tauri v2 CLI** — installed via Cargo (see below)
- **OS-specific Tauri v2 system dependencies** — see the [Installing OS dependencies](#installing-os-dependencies) section below

### Installing OS dependencies

[#installing-os-dependencies](#installing-os-dependencies)

#### macOS
```bash
xcode-select --install
```

#### Linux (Debian/Ubuntu)
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev \
build-essential \
curl \
wget \
file \
libxdo-dev \
libssl-dev \
libayatana-appindicator3-dev \
librsvg2-dev
```

#### Windows

Tauri v2 needs a working C++ linker. You have two toolchain options:

- **MSVC (recommended, default on Windows):** install the [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) and select the **"Desktop development with C++"** workload. This provides `link.exe`, which the Rust compiler needs to produce the final binary.
- **GNU (alternative):** if you'd rather avoid Visual Studio, use the GNU toolchain instead:
  ```bash
  rustup toolchain install stable-x86_64-pc-windows-gnu
  rustup default stable-x86_64-pc-windows-gnu
  ```
  This requires MSYS2/MinGW-w64 on your `PATH`.

Also make sure [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) is installed (it ships by default on up-to-date Windows 11 systems).

### Install the Tauri CLI

[#install-the-tauri-cli](#install-the-tauri-cli)
```bash
cargo install tauri-cli
```

Verify:
```bash
cargo tauri --version
```

### 1. Clone the repository

[#1-clone-the-repository](#1-clone-the-repository)
```bash
git clone https://github.com/Omnikon-Org/CNTRL.git
cd CNTRL
```

### 2. Install JavaScript dependencies

[#2-install-javascript-dependencies](#2-install-javascript-dependencies)

The repo ships with a `package-lock.json`, so use npm:
```bash
npm install
```

### 3. Run the app

[#3-run-the-app](#3-run-the-app)

**Full app (Tauri + SolidJS frontend, native window):**
```bash
npm run tauri dev
```

The first run will take noticeably longer, since Cargo has to compile the Rust backend from scratch.

**Frontend only (skip the Rust/Tauri build):**

If you just want to work on the SolidJS UI and don't need the native shell:
```bash
npm run dev
```

### Environment tested

[#environment-tested](#environment-tested)

- **OS:** Windows 11 Home
- **Node.js:** v24.13.1
- **Rust:** 1.96.1 (stable-x86_64-pc-windows-gnu)

---

## Troubleshooting

[#troubleshooting](#troubleshooting)

### Node.js version is too old

Ensure you're on Node.js 20 or later:
```bash
node -v
```

### Cargo command not found

Restart your terminal after installing Rust so `PATH` picks up Cargo.
```bash
cargo --version
```

### Tauri CLI not found
```bash
cargo install tauri-cli
cargo tauri --version
```

### `link.exe` not found / MSVC linker error (Windows)

This means the Visual Studio Build Tools' C++ workload isn't installed, or the install is incomplete. Reopen the Visual Studio Installer, confirm **"Desktop development with C++"** is checked, then restart your terminal. If you'd rather not install Visual Studio at all, switch to the GNU toolchain instead (see [Installing OS dependencies → Windows](#installing-os-dependencies) above).

### Windows Smart App Control (SAC) blocks the app — `os error 4551`

Windows Smart App Control blocks unsigned local binaries by default — this includes the debug binary that `npm run tauri dev` builds locally, since it isn't code-signed. If the app fails to launch with this error:

- Check whether SAC is on: **Settings → Privacy & security → Windows Security → App & browser control → Smart App Control**. If it's on, you can turn it off — note that once disabled, SAC can only be re-enabled via a clean Windows reinstall, so treat this as a one-way decision.
- Alternatively, develop inside a VM or a machine without SAC enabled, or sign the binary if you have a code-signing certificate available.

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