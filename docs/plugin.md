# Plugin SDK

The Plugin SDK (Phase 7) defines how unprivileged WebAssembly modules can safely extend CNTRL.

## `.vibe-plugin` Format
A `.vibe-plugin` file is essentially a ZIP archive that includes:
- `manifest.json`: Configuration defining the plugin ID, name, entrypoint, and permissions.
- `module.wasm`: The compiled WebAssembly logic.

## Security Boundary
The WASM sandbox operates entirely without filesystem or system shell access.
Available Permissions:
- `IntentExecution`: Allows the plugin to intercept and execute custom intents.
- `NetworkAccess`: Grants access to the host's proxy `fetch` function for sandboxed network requests.
- `FileSystemAccess`: Grants access **only** to a virtualized memory-fs layer.
