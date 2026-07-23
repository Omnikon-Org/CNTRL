# Memory & Secure Enclave

The Memory Engine (Phase 5) uses AES-256-GCM encryption for all persisted local data.

- **Short-Term Memory**: LanceDB vector database for RAG over recent habits and user actions.
- **Long-Term Memory**: SQLite for persistent relational facts and settings.
- **Keychain**: Native OS integration (macOS/Windows/Linux) via `keyring` crate to store API keys without leaving plain text on disk.
