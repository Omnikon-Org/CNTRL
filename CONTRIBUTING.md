# Contributing to CNTRL Browser

Thank you for your interest in contributing to CNTRL Browser! We welcome community contributions — bug fixes, new features, tests, and documentation improvements are all valued.

---

## Getting Started

### Prerequisites
To build and run CNTRL Browser locally, you will need:
- **Node.js**: Version 20 or higher.
- **Rust**: The stable Rust toolchain (including `cargo`, `rustc`, and `rustfmt`).
- **Tauri Prerequisites**: Setup instructions for Tauri v2 dependencies vary by operating system. Please follow the official [Tauri Prerequisite Guide](https://v2.tauri.app/start/prerequisites/).

### Local Setup

1. **Fork** the repository on GitHub, then clone your fork:
   ```bash
   git clone https://github.com/<your-username>/CNTRL.git
   cd CNTRL
   ```

2. Add the upstream remote so you can keep your fork in sync:
   ```bash
   git remote add upstream https://github.com/Omnikon-Org/CNTRL.git
   ```

3. Install frontend dependencies:
   ```bash
   npm install
   ```

4. Run the development server (starts Vite and launches the Tauri app):
   ```bash
   npm run tauri dev
   ```

---

## Branching Convention

CNTRL Browser uses a single stable integration branch. **All pull requests must target `main`.**

| Branch pattern | When to use |
|---|---|
| `feat/<short-description>` | New features or enhancements |
| `fix/<short-description>` | Bug fixes |
| `docs/<short-description>` | Documentation-only changes |
| `chore/<short-description>` | Tooling, dependency bumps, CI changes |
| `test/<short-description>` | Adding or improving tests |

**Workflow:**

```bash
# 1. Sync your fork with upstream main
git fetch upstream
git checkout main
git merge upstream/main

# 2. Create your branch off main
git checkout -b feat/your-feature-name

# 3. Make your changes, commit, push
git push origin feat/your-feature-name

# 4. Open a Pull Request on GitHub targeting main
```

> **Note:** The `main` branch is **protected**. Direct pushes are not permitted. All changes must go through a Pull Request that passes CI and receives at least one approving review.

---

## Development Guidelines

### Code Style & Quality

We use automated formatters and linters to keep the codebase clean and consistent:
- **Frontend**: Biome and ESLint are used for formatting and linting.
- **Backend (Rust)**: `cargo fmt` and `cargo clippy` are used.

Before committing, verify all checks pass locally:

```bash
# Frontend
npx tsc --noEmit
npm run lint
npx vitest run

# Backend (Rust)
cd src-tauri
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
```

### Commit Messages

We follow the **[Conventional Commits](https://www.conventionalcommits.org/)** specification. Your commit messages should be formatted as:

```
<type>: <short description>
```

**Common types:**

| Type | Use for |
|---|---|
| `feat` | A new feature |
| `fix` | A bug fix |
| `docs` | Documentation changes only |
| `chore` | Tooling, dependencies, or build changes |
| `test` | Adding or fixing tests |
| `refactor` | Code change that neither fixes a bug nor adds a feature |

**Examples:**
- `feat: add intent scoring logic`
- `fix: correct tab bound update on last tab close`
- `docs: update branching model in README`
- `chore: bump biome to v2`

---

## Pull Request Process

1. Ensure your branch is up to date with `upstream/main` before opening a PR.
2. Add or update tests for any code changes where applicable.
3. Verify all frontend and backend checks pass locally (see above).
4. Open a Pull Request against the **`main`** branch.
5. Fill in the PR template completely — describe the problem, your solution, and how you tested it.
6. A maintainer will review your PR. Address any requested changes promptly.
7. Once approved and CI passes, a maintainer will merge your PR.

---

## Reporting Issues

Use the GitHub Issue templates for:
- **Bug reports** — include steps to reproduce, expected vs. actual behavior, and your environment.
- **Feature requests** — describe the problem you're solving and your proposed solution.

---

## Code of Conduct

By participating in this project, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md). We are committed to a welcoming and respectful community for everyone.
