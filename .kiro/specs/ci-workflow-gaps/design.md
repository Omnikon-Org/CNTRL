# Design Document: CI Workflow Gaps

## Overview

This design closes three gaps left open after the initial `ci.yml` merge (issue #19):

1. **Branch trigger** — add `phase-1-scaffold` to the `on.push.branches` and `on.pull_request.branches` lists so the workflow fires on that branch too.
2. **Inline comments** — annotate `ci.yml` with YAML comments that tell contributors where and how to add new steps.
3. **CI documentation** — insert a "CI / Continuous Integration" section into `CONTRIBUTING.md` covering what runs, when it runs, and how to replicate it locally.

All changes are purely textual edits to two existing files. No new source code, dependencies, or infrastructure is introduced.

---

## Architecture

No architectural changes are required. The feature operates entirely at the repository-metadata layer:

```
Repository root
├── .github/
│   └── workflows/
│       └── ci.yml          ← edit: branch triggers + inline comments
└── CONTRIBUTING.md         ← edit: add CI section
```

The GitHub Actions runtime interprets `ci.yml` declaratively; the runner topology (ubuntu-latest, the job matrix, caching) is unchanged.

---

## Components and Interfaces

### Component 1: `ci.yml` — Branch Triggers

**Location**: `.github/workflows/ci.yml`, `on` block (lines 3–7).

**Current state**:
```yaml
on:
  push:
    branches: [ "main" ]
  pull_request:
    branches: [ "main" ]
```

**Target state**:
```yaml
on:
  push:
    branches: [ "main", "phase-1-scaffold" ]
  pull_request:
    branches: [ "main", "phase-1-scaffold" ]
```

**Interface contract**: GitHub Actions evaluates the `branches` list as an ordered set of glob patterns. Adding a literal branch name has no side-effects on existing triggers.

---

### Component 2: `ci.yml` — Inline Comments

**Location**: `.github/workflows/ci.yml`, `steps` block.

Two comment zones are required:

1. **Top-of-steps comment** — immediately above the first step, explaining the purpose of the job and inviting extension.
2. **End-of-steps comment** — after the last step (`Vitest Check`), marking the point where new steps should be inserted and giving a concrete example.

Example placement:

```yaml
    steps:
    # ─── Core checks ────────────────────────────────────────────────────────────
    # To add a new step (e.g., a build or integration-test step), append it below
    # the last existing step, following the same `- name: / run:` pattern.
    - uses: actions/checkout@v4
    ...
    - name: Vitest Check
      run: npx vitest run

    # ─── Add new steps here ──────────────────────────────────────────────────────
    # Example:
    #   - name: Build Tauri App
    #     run: npm run tauri build -- --debug
```

**Constraint**: Every existing step name, `run` command, `working-directory`, and `uses` directive must remain byte-for-byte identical after this edit.

---

### Component 3: `CONTRIBUTING.md` — CI Section

**Location**: `CONTRIBUTING.md`.

**Insertion point**: After the existing "🧪 Verification Checks" section and before the "🔀 Pull Request Process" section. This placement is logical because a contributor reading the verification steps will naturally want to know how those same steps are enforced automatically.

**Section content requirements** (from Requirements 3.1–3.4):

| Item | Content |
|---|---|
| Heading | `## 🤖 CI / Continuous Integration` |
| When CI runs | Every push to `main` and `phase-1-scaffold`; every PR targeting those branches |
| Checks listed | Rust Clippy, Rust tests, Rust format check, TypeScript type check, ESLint check, Vitest tests |
| Local commands | Exact commands to reproduce each check |

**Target section**:

```markdown
## 🤖 CI / Continuous Integration

CI runs automatically via GitHub Actions on every **pull request** and every **push to `main` or `phase-1-scaffold`**.

### What CI checks

| Check | Tool |
|---|---|
| Rust linting | `cargo clippy --all-targets --all-features -- -D warnings` |
| Rust tests | `cargo test --all` |
| Rust formatting | `cargo fmt --check` |
| TypeScript types | `npx tsc --noEmit` |
| ESLint | `npx eslint . --max-warnings 0` |
| Frontend tests | `npx vitest run` |

### Running CI checks locally

Run the following from the repository root to replicate every CI check:

```bash
# Rust checks (run from src-tauri/)
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo fmt --check

# Frontend checks (run from repo root)
npx tsc --noEmit
npx eslint . --max-warnings 0
npx vitest run
```

If all commands exit with code 0, your branch will pass CI.
```

**Constraint**: The five existing sections (Prerequisites, Development Setup, Code Style & Guidelines, Verification Checks, Pull Request Process) must remain present and unmodified.

---

## Data Models

No data models are introduced. The only data artefacts are:

- A YAML scalar list (`branches`) in `ci.yml` — extended with one string element per trigger type.
- A Markdown string (the new CI section) — inserted into `CONTRIBUTING.md`.

---

## Error Handling

| Scenario | Mitigation |
|---|---|
| YAML parse error after edit | Validate `ci.yml` with `yamllint` or GitHub's workflow linter before merging |
| Accidental removal of existing steps | Review the diff to confirm all original step blocks are present |
| Markdown rendering issues | Preview `CONTRIBUTING.md` in a Markdown renderer before merging |
| Branch name typo in trigger | Cross-check the exact branch name against `git branch -a` output |

---

## Testing Strategy

Property-based testing does **not** apply to this feature. All acceptance criteria are structural/configuration checks on two files — they verify presence or absence of specific textual content in a YAML file and a Markdown file. The input space does not vary meaningfully; running 100 iterations would not surface additional defects. Appropriate test strategies are:

### Smoke tests (manual or CI-based)

These checks should be performed before merging the PR:

1. **Branch trigger check** (Requirements 1.1–1.3)
   - Parse `ci.yml` and assert `on.push.branches` equals `["main", "phase-1-scaffold"]`.
   - Assert `on.pull_request.branches` equals `["main", "phase-1-scaffold"]`.

2. **Inline comment check** (Requirements 2.1–2.2)
   - Assert the `steps` block contains at least two YAML comment lines (`#`) that reference adding or extending steps.

3. **Existing steps preservation** (Requirement 2.3)
   - Assert all six original step names (`Rust Clippy`, `Rust Tests`, `Rust Format`, `TypeScript Check`, `ESLint Check`, `Vitest Check`) remain present.

4. **CI section heading** (Requirement 3.1)
   - Assert `CONTRIBUTING.md` contains the heading `CI / Continuous Integration`.

5. **CI section content** (Requirements 3.2–3.4)
   - Assert the section names all six checks.
   - Assert the section mentions `pull request`, `main`, and `phase-1-scaffold`.
   - Assert the section contains code blocks with the six local commands.

6. **Existing CONTRIBUTING sections preserved** (Requirement 3.5)
   - Assert all five original headings remain present in `CONTRIBUTING.md`.

### Manual review checklist

- [ ] `ci.yml` YAML is valid (no syntax errors).
- [ ] Workflow is readable and comments are clear to a first-time contributor.
- [ ] `CONTRIBUTING.md` renders correctly in GitHub's Markdown viewer.
- [ ] New CI section sits between "Verification Checks" and "Pull Request Process".
