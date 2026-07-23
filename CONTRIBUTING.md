# Contributing to CNTRL Browser

Thank you for contributing to CNTRL Browser!

## 🛠️ Prerequisites

- **Node.js**: v20 or higher
- **Rust**: Latest stable toolchain (`rustup update`)
- **Protobuf Compiler**: `protoc` (macOS: `brew install protobuf`, Linux: `sudo apt install protobuf-compiler`)

## 💻 Development Setup

1. **Install dependencies**:
   ```bash
   npm install
   ```
2. **Run dev server**:
   ```bash
   npm run tauri dev
   ```

## 📐 Code Style & Guidelines

- Read **[`Important Documentation/Rules.md`](./Important%20Documentation/Rules.md)** before writing code.
- **Frontend**: Use Vanilla CSS custom property tokens in `src/styles/tokens.css`. Do NOT add Tailwind CSS.
- **TypeScript**: Strict types (`no-explicit-any`). Interfaces imported from `src/types/`.
- **Rust**: Modular services under `src-tauri/src/services/`. Explicit error handling via `CntrlError`.

## 🧪 Verification Checks

Before submitting a Pull Request, run:
```bash
# Rust type checking & lints
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml

# TypeScript type checking
npx tsc --noEmit
```

## 🔀 Pull Request Process

1. Fork the repo and create your feature branch off `main` (e.g., `feat/custom-prompt-template`).
2. Target **`main`** for all Pull Requests.
3. Include a clear description of changes in the PR template.
