# Contributing to CNTRL Browser

Thank you for your interest in contributing to CNTRL Browser! We welcome community contributions to help build a lightweight, intent-based autonomous browser.

## Getting Started

### Prerequisites
To build and run CNTRL Browser locally, you will need:
- **Node.js**: Version 20 or higher.
- **Rust**: The stable Rust toolchain (including cargo, rustc, and rustfmt).
- **Tauri Prerequisites**: Setup instructions for Tauri v2 dependencies vary by operating system. Please follow the official [Tauri Prerequisite Guide](https://v2.tauri.app/start/prerequisites/).

### Local Setup

1. Fork and clone the repository:
   ```bash
   git clone https://github.com/<your-username>/CNTRL-Browser.git
   cd CNTRL-Browser
   ```
2. Install frontend dependencies:
   ```bash
   npm install
   ```
3. Run the development server (this starts Vite and launches the Tauri app):
   ```bash
   npm run tauri dev
   ```

## Development Guidelines

### Code Style & Quality
To keep the codebase clean and maintainable, we use automated formatters and linters:
- **Frontend**: Biome and ESLint are used for formatting and linting.
- **Backend (Rust)**: `cargo fmt` and `cargo clippy` are used.

Before committing, run:
```bash
# Frontend
npx tsc --noEmit
npx eslint . --max-warnings 0
npx vitest run

# Backend (Rust)
cd src-tauri
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
```

### Commit Messages
We follow the **Conventional Commits** specification. Please format your commit messages accordingly:
- `feat: add intent scoring logic`
- `fix: correct tab bound update logic`
- `docs: update setup steps in readme`

### Pull Request Process
1. Create a branch for your feature or fix (e.g., `feature/xyz` or `fix/abc`).
2. Add tests for your code if applicable.
3. Verify that all frontend and backend checks pass locally.
4. Push your branch and open a Pull Request against the `main` branch.
5. Provide a clear description of the problem solved and the implementation details in your PR description.
