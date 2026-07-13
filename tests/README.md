# Integration Tests

This directory contains cross-boundary integration tests that span both the
Rust backend and the TypeScript frontend. Tests that live entirely within a
single crate belong in `src-tauri/tests/`; tests here exercise the full IPC
stack or scenarios that require both sides to be running.

## Structure (planned)

```
tests/
├── README.md               ← this file
├── ipc/                    ← Tauri command round-trip tests (requires tauri-driver)
│   └── browser_commands.rs
└── e2e/                    ← End-to-end UI tests via Playwright WebDriver
    └── navigation.spec.ts
```

## Running

> IPC and e2e tests require the full Tauri runtime to be running.
> They are intentionally excluded from the standard `cargo test` run.

```bash
# Start the app in test mode first
npm run tauri dev

# Then in a separate terminal:
npx playwright test tests/e2e
```

## Status

| Suite               | Phase | Status       |
|---------------------|-------|--------------|
| `ipc/`              | 2     | Planned      |
| `e2e/navigation`    | 2     | Planned      |
| `e2e/command-bar`   | 4     | Not started  |
