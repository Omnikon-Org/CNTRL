# Implementation Plan: CI Workflow Gaps

## Overview

Three targeted file edits close the remaining gaps from issue #19:
1. Extend the branch trigger list in `ci.yml`.
2. Add inline contributor-guidance comments to `ci.yml`.
3. Add a CI section to `CONTRIBUTING.md`.

All tasks involve editing existing files only — no new files, dependencies, or infrastructure.

## Tasks

- [ ] 1. Add `phase-1-scaffold` to branch triggers in `ci.yml`
  - In `.github/workflows/ci.yml`, locate the `on.push.branches` list and add `"phase-1-scaffold"` as a second element: `[ "main", "phase-1-scaffold" ]`.
  - Apply the identical change to the `on.pull_request.branches` list.
  - Verify the file parses as valid YAML after the edit.
  - _Requirements: 1.1, 1.2, 1.3_

- [ ] 2. Add inline comments to `ci.yml`
  - Immediately above the first step (`uses: actions/checkout@v4`), insert a block of YAML comments that:
    - Describe the purpose of the `steps` section at a glance.
    - Tell contributors they can append new steps below the last existing step.
    - Show the `- name: / run:` pattern as a brief example.
  - After the last step (`Vitest Check`), insert a clearly marked comment block that says "Add new steps here" and shows a concrete example step (e.g., a Tauri debug build).
  - Confirm all original step names, `run` commands, `working-directory` fields, and `uses` directives are byte-for-byte unchanged.
  - _Requirements: 2.1, 2.2, 2.3_

- [ ] 3. Checkpoint — validate ci.yml
  - Ensure all tests pass, ask the user if questions arise.
  - Confirm `ci.yml` is valid YAML (no parse errors).
  - Confirm `on.push.branches` and `on.pull_request.branches` each contain exactly `["main", "phase-1-scaffold"]`.
  - Confirm all six original step names are still present.

- [ ] 4. Add CI section to `CONTRIBUTING.md`
  - Open `CONTRIBUTING.md` and locate the position between the "🧪 Verification Checks" section and the "🔀 Pull Request Process" section.
  - Insert a new section with the heading `## 🤖 CI / Continuous Integration` at that position.
  - The section body must include:
    - A sentence stating that CI runs on every PR and every push to `main` and `phase-1-scaffold`.
    - A table (or equivalent list) naming all six checks: Rust Clippy, Rust tests, Rust format check, TypeScript type check, ESLint check, Vitest tests — each paired with its exact command.
    - A fenced code block containing the six local shell commands a contributor can run to replicate CI.
    - A note that exit code 0 on all commands means the branch will pass CI.
  - Verify the five original sections (Prerequisites, Development Setup, Code Style & Guidelines, Verification Checks, Pull Request Process) are all still present and unmodified.
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_

- [ ] 5. Final checkpoint — Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.
  - Confirm `CONTRIBUTING.md` renders correctly (no broken Markdown syntax).
  - Confirm the new CI section appears between "Verification Checks" and "Pull Request Process".
  - Confirm all five original headings remain present in `CONTRIBUTING.md`.

## Notes

- No property-based tests apply to this feature — all acceptance criteria are structural checks on YAML and Markdown files.
- Each task references specific requirements from `requirements.md` for full traceability.
- The two checkpoint tasks serve as explicit review gates before and after the documentation change.
