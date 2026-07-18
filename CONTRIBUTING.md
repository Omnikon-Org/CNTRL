# Contributing to CNTRL

First off, thank you for considering contributing to CNTRL!

## Development Setup

CNTRL is built using Tauri (Rust backend) and SolidJS (TypeScript frontend).

### Prerequisites
- Node.js (v18+)
- Rust (latest stable)
- Playwright (`npx playwright install`)

### Running Locally
1. `npm install`
2. `npm run dev`

### Project Architecture
- **src/**: SolidJS UI, structured into components and stores.
- **src-tauri/src/services/**: Core AI routing, execution, macro logic, memory.
- **src-tauri/src/commands/**: Tauri IPC boundary connecting frontend to backend.

## Pull Request Process
1. Ensure your code passes all linters (`npm run typecheck`, `cargo clippy`).
2. Update the README.md with details of changes to the interface, if applicable.
3. Your PR must pass the CI tests (run via GitHub Actions).
