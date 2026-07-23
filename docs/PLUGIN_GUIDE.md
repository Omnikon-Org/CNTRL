# WASM Plugin Guide — CNTRL Browser

## Overview

CNTRL Browser features a WebAssembly (WASM) extension sandbox powered by **Wasmtime**. Third-party plugins execute in an isolated environment with zero raw filesystem or unmonitored network access.

---

## Plugin Manifest Specification (`vibe-plugin.json`)

Every CNTRL plugin package must contain a `vibe-plugin.json` manifest:

```json
{
  "name": "crypto-price-tracker",
  "version": "1.0.0",
  "description": "Fetches current cryptocurrency prices and feeds them to the Intent Router",
  "author": "CNTRL Community",
  "entrypoint": "plugin.wasm",
  "permissions": [
    "network:api.coingecko.com",
    "intent:read"
  ]
}
```

---

## Host ABI Functions

Plugins communicate with the CNTRL Rust host via C-compatible ABI exports:

```rust
// Exported by CNTRL Host into WASM Sandbox
extern "C" {
    fn cntrl_log(level: u32, ptr: *const u8, len: u32);
    fn cntrl_fetch(url_ptr: *const u8, url_len: u32, out_ptr: *mut u8);
    fn cntrl_emit_intent(intent_ptr: *const u8, intent_len: u32);
}
```

---

## Building a Plugin

1. Write your plugin in Rust, C, AssemblyScript, or Zig targeting `wasm32-wasip1`.
2. Compile to `.wasm`:
   ```bash
   cargo build --target wasm32-wasip1 --release
   ```
3. Bundle `vibe-plugin.json` and `plugin.wasm` into a `.zip` archive.
4. Load via CNTRL Browser Settings → Plugin Manager.
