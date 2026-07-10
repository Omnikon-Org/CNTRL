# Tauri Linux Troubleshooting Guide

This guide covers the most common Linux dependency issues encountered while building Tauri applications. Since Linux distributions do not bundle all required development libraries by default, installing the correct packages is necessary before running or building the project.

---

## Why Linux Needs Extra Dependencies

Unlike Windows and macOS, Linux requires several native development libraries for Tauri applications.

Tauri relies on:

- GTK 3 for the native application window
- WebKitGTK for rendering the frontend
- Rust toolchain for compiling the backend
- `pkg-config` to locate installed system libraries

If any of these dependencies are missing, the build may fail with linker or package detection errors.

---

## Ubuntu / Debian

Install the required development dependencies:

```bash
sudo apt update
sudo apt install -y build-essential libgtk-3-dev libwebkit2gtk-4.0-dev curl pkg-config
```

---

## Fedora

Install the required development dependencies:

```bash
sudo dnf install -y @development-tools gtk3-devel webkit2gtk3-devel curl pkgconfig
```

---

## Arch Linux

Install the required development dependencies:

```bash
sudo pacman -S --needed base-devel gtk3 webkit2gtk curl pkgconf
```

---

## WSL Notes

If you are building Tauri inside Windows Subsystem for Linux (WSL):

- Install the required GTK and WebKitGTK packages.
- Ensure Rust and Cargo are installed.
- GUI applications require WSLg (Windows 11) or an X Server.
- Restart the terminal after installing dependencies.

---

## Common Errors and Fixes

### Error: Missing `webkit2gtk`

Example:

```text
Package 'webkit2gtk-4.0' not found
```

**Solution**

Install the WebKitGTK development package for your Linux distribution using the commands above.

---

### Error: GTK Runtime Not Found

Example:

```text
Failed to initialize GTK
```

**Solution**

Verify that GTK 3 development libraries are installed correctly and available through `pkg-config`.

---

### Error: Cargo Build Failed

Example:

```text
cargo build failed
```

**Solution**

- Verify that Rust is installed correctly.
- Ensure Cargo is available in your PATH.
- Confirm all required Linux development libraries are installed.
- Re-run the build after installing missing dependencies.

---

## Quick Verification

Check GTK installation:

```bash
pkg-config --modversion gtk+-3.0
```

Check WebKitGTK installation:

```bash
pkg-config --modversion webkit2gtk-4.0
```

Verify linked libraries:

```bash
ldd src-tauri/target/debug/*
```

Verify Rust installation:

```bash
rustc --version
cargo --version
```

---

## Additional Resources

- https://tauri.app
- https://v2.tauri.app/start/prerequisites/
- https://www.rust-lang.org/tools/install