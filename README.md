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

## Installing Prerequisites

Before running the project locally, ensure all required tools are installed.

### Install Node.js(20+)

Download and install the latest LTS version of Node.js (version 20 or later):

https://nodejs.org/

Verify the installation:

```bash
node -v
npm -v
```

### 2. Install Rust (Stable Toolchain)

Install the Rust stable toolchain using Rustup.

#### Linux / macOS

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### Windows

Download Rustup from:

https://rustup.rs/

After installation, verify:

```bash
rustc --version
cargo --version
```

---

### 3. Install Tauri CLI

After installing Node.js and Rust, install the Tauri CLI:

```bash
cargo install tauri-cli
```

Verify the installation:

```bash
cargo tauri --version
```

---

## Running the Project

Follow these steps to start the application locally.

### 1. Clone the repository

```bash
git clone https://github.com/Omnikon-Org/CNTRL.git
cd CNTRL
```

### 2. Install project dependencies

```bash
npm install
```

### 3. Launch the development application

```bash
npm run tauri dev
```

---
## Troubleshooting

### Node.js version is too old

Ensure you are using Node.js 20 or later.

```bash
node -v
```

---

### Cargo command not found

If Cargo is not recognized, restart your terminal after installing Rust.

Verify the installation:

```bash
cargo --version
```

---

### Tauri CLI not found

Install or reinstall the Tauri CLI:

```bash
cargo install tauri-cli
```

Verify:

```bash
cargo tauri --version
```

---

### Missing WebView dependencies

Tauri requires platform-specific WebView dependencies.

Refer to the official Tauri prerequisites documentation:

https://v2.tauri.app/start/prerequisites/

---

### Additional Notes

- Use the latest stable versions of Node.js and Rust whenever possible.
- If you encounter dependency issues, delete the `node_modules` folder and run:

```bash
npm install
```

- If problems persist, ensure your Rust toolchain is up to date:

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
## Contributing

We welcome contributions of all kinds — bug fixes, features, tests, and documentation improvements.

1. Fork the repo and clone your fork.
2. Create a branch off `main` following the naming conventions above (e.g. `feat/intent-scoring`).
3. Make your changes, write tests, and ensure all CI checks pass locally.
4. Open a Pull Request against **`main`** with a clear description.

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for the full contribution guide, code style requirements, and commit message conventions.
