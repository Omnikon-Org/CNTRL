# Troubleshooting Guide

This guide provides solutions to common issues encountered while setting up, building, and running **CNTRL Browser**. Follow the sections below to diagnose and resolve problems quickly.

---

# Installation Issues

## Node.js Version Mismatch

### Symptom

* `npm install` fails.
* Dependencies fail to install.
* Unexpected syntax or package errors.

### Cause

CNTRL Browser requires **Node.js 20 or later**.

### Fix

Verify your Node.js version:

```bash
node -v
```

If the version is below **20**, install the latest LTS release from:

https://nodejs.org/

Then reinstall dependencies:

```bash
npm install
```

---

## Rust Toolchain Not Found

### Symptom

Running `npm run tauri dev` reports:

```
cargo: command not found
```

or

```
Rust toolchain not found
```

### Cause

Rust is not installed or the environment variables are not configured.

### Fix

Install the stable Rust toolchain:

https://rustup.rs

Verify installation:

```bash
rustc --version
cargo --version
```

Restart the terminal after installation if necessary.

---

## Missing Tauri CLI

### Symptom

```
tauri: command not found
```

or

```
failed to execute tauri
```

### Cause

The Tauri CLI has not been installed.

### Fix

Install the Tauri CLI:

```bash
cargo install tauri-cli
```

Verify installation:

```bash
cargo tauri --version
```

---

## Missing Protobuf Compiler (`protoc`)

### Symptom

Compilation fails with errors mentioning:

```
protoc
```

or

```
Could not find protoc
```

### Cause

The Protobuf compiler is required for the LanceDB vector store but is not installed.

### Fix

Install `protoc` for your platform.

**macOS**

```bash
brew install protobuf
```

**Linux**

```bash
sudo apt install protobuf-compiler
```

**Windows**

```bash
choco install protoc
```

Verify installation:

```bash
protoc --version
```

---

# Build Failures

## `npm install` Fails

### Symptom

Package installation stops with dependency errors.

### Cause

* Unsupported Node.js version
* Interrupted installation
* Corrupted dependency cache

### Fix

Delete the existing dependencies:

```bash
rm -rf node_modules
```

Remove the lock file if needed:

```bash
rm package-lock.json
```

Install again:

```bash
npm install
```

---

## Cargo Build Errors

### Symptom

Rust compilation fails during `npm run tauri dev`.

### Cause

* Missing Rust dependencies
* Outdated toolchain
* Missing system requirements

### Fix

Update Rust:

```bash
rustup update
```

Clean the build:

```bash
cargo clean
```

Run again:

```bash
npm run tauri dev
```

---

# Runtime Errors

## Application Fails to Launch

### Symptom

The application exits immediately after starting.

### Cause

One or more required dependencies may be missing or the build may have failed.

### Fix

Run:

```bash
npm install
npm run tauri dev
```

Review the terminal logs for the first reported error instead of the final error message.

---

## Frontend Starts but Desktop Window Does Not Open

### Symptom

`npm run dev` works, but `npm run tauri dev` does not launch the desktop application.

### Cause

The Rust backend or Tauri CLI may not be configured correctly.

### Fix

Verify:

* Rust is installed.
* Tauri CLI is installed.
* All prerequisites listed in the README are satisfied.

---

# Dependency Problems

## Outdated Dependencies

### Symptom

Unexpected compilation or runtime issues after pulling recent changes.

### Cause

Local dependencies are out of sync with the repository.

### Fix

Update dependencies:

```bash
npm install
cargo update
```

If problems persist, perform a clean installation.

---

## Lockfile Conflicts

### Symptom

Dependency resolution fails after switching branches.

### Cause

The lockfile is inconsistent with the current branch.

### Fix

Reinstall project dependencies:

```bash
npm install
```

If necessary, delete `node_modules` and reinstall.

---

# Common Fixes

## Start From a Clean Environment

When troubleshooting persistent issues:

```bash
rm -rf node_modules
cargo clean
npm install
npm run tauri dev
```

---

## Verify Installed Versions

Check that the required tools are available:

```bash
node -v
npm -v
rustc --version
cargo --version
cargo tauri --version
protoc --version
```

Ensure they match the versions recommended in the project README.

---

## Review Documentation

Before opening an issue, consult:

* `README.md`
* `Important Documentation/Rules.md`
* `Important Documentation/Architecture.md`
* `CONTRIBUTING.md`

Many setup and development questions are answered there.

---

# Still Need Help?

If the issue persists:

1. Verify all prerequisites are installed.
2. Capture the complete terminal output.
3. Search existing GitHub issues for similar reports.
4. If no existing issue matches, open a new issue and include:

   * Operating system
   * Node.js version
   * Rust version
   * Tauri CLI version
   * `protoc` version
   * Complete error message
   * Steps to reproduce the problem

Providing detailed information will help maintainers diagnose and resolve the issue more efficiently.
