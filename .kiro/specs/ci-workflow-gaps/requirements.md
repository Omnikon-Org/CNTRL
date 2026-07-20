# Requirements Document

## Introduction

The CNTRL Browser project has an existing `ci.yml` GitHub Actions workflow that runs lint and test checks on the `main` branch. Three gaps remain from the original issue (#19) that need to be addressed:

1. The workflow does not trigger on the `phase-1-scaffold` branch.
2. The workflow file contains no inline comments guiding contributors on how to extend it.
3. `CONTRIBUTING.md` has no CI section explaining what CI does, when it runs, or how to replicate its checks locally.

This feature closes those three gaps through targeted edits to `.github/workflows/ci.yml` and `CONTRIBUTING.md`.

## Glossary

- **CI_Workflow**: The GitHub Actions workflow defined in `.github/workflows/ci.yml`.
- **CONTRIBUTING_Doc**: The contributor guide located at `CONTRIBUTING.md` in the repository root.
- **Branch_Trigger**: The `on.push.branches` and `on.pull_request.branches` lists in `ci.yml` that determine which branches activate the workflow.
- **Inline_Comment**: A YAML line comment (prefixed with `#`) placed inside `ci.yml` to explain the purpose of a section or how to extend it.
- **CI_Section**: A dedicated Markdown section in `CONTRIBUTING_Doc` describing CI behaviour and local equivalents.

## Requirements

### Requirement 1: Branch Trigger Coverage

**User Story:** As a contributor working on the `phase-1-scaffold` branch, I want CI to run on my pushes and pull requests, so that I receive the same automated feedback as contributors targeting `main`.

#### Acceptance Criteria

1. WHEN a push is made to the `phase-1-scaffold` branch, THE CI_Workflow SHALL execute the `test-and-lint` job.
2. WHEN a pull request targets the `phase-1-scaffold` branch, THE CI_Workflow SHALL execute the `test-and-lint` job.
3. THE CI_Workflow SHALL continue to execute the `test-and-lint` job on pushes to `main` and on pull requests targeting `main`.

---

### Requirement 2: Inline Contributor Guidance in ci.yml

**User Story:** As a new contributor, I want inline comments in `ci.yml` explaining where and how to add new steps, so that I can extend the workflow without needing to consult external documentation.

#### Acceptance Criteria

1. THE CI_Workflow file SHALL contain at least one Inline_Comment explaining that additional steps (such as build or integration tests) can be appended after the existing steps.
2. THE CI_Workflow file SHALL contain at least one Inline_Comment identifying the location within the `steps` list where a new step should be added.
3. WHEN the CI_Workflow file is modified to add the comments, THE existing step names, commands, and job structure SHALL remain unchanged.

---

### Requirement 3: CI Documentation in CONTRIBUTING.md

**User Story:** As a contributor, I want a CI section in `CONTRIBUTING.md` explaining what CI checks run, when they run, and how to reproduce them locally, so that I can understand and meet CI requirements before opening a pull request.

#### Acceptance Criteria

1. THE CONTRIBUTING_Doc SHALL contain a CI_Section with a heading titled "CI / Continuous Integration".
2. THE CI_Section SHALL list each check that CI runs: Rust Clippy, Rust tests, Rust format check, TypeScript type check, ESLint check, and Vitest tests.
3. THE CI_Section SHALL state that CI runs automatically on every pull request and on every push to `main` and `phase-1-scaffold`.
4. THE CI_Section SHALL provide the exact local shell commands a contributor can run to reproduce each CI check.
5. WHEN `CONTRIBUTING.md` is modified to add the CI_Section, THE existing sections (Prerequisites, Development Setup, Code Style, Verification Checks, Pull Request Process) SHALL remain unchanged.
