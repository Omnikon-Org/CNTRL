# Architecture

CNTRL Browser is split into a SolidJS frontend and a Tauri v2 Rust backend. The frontend owns browser chrome and user interaction. The backend owns native webviews, tab lifecycle operations, fallback fetching, and AI-provider routing.

## Runtime Flow

```text
User
  -> SolidJS UI
  -> Solid stores
  -> Tauri invoke commands
  -> Rust command handlers
  -> Rust services
  -> Native child webviews, fallback fetcher, or AI providers
```

## Frontend

The frontend lives in `src/`.

Important areas:

- `src/App.tsx`: initializes AI config, fetches tabs, and opens the initial tab.
- `src/components/TabBar.tsx`: visible tab list and new-tab controls.
- `src/components/UrlBar.tsx`: URL input, navigation controls, and settings entrypoint.
- `src/components/WebView.tsx`: main browser viewport, internal settings page, fallback iframe rendering, and child-webview bounds sync.
- `src/components/SettingsPage.tsx`: AI tier, provider key, model selection, AI test, and intent router test UI.
- `src/stores/browserStore.ts`: browser state and Tauri command wrappers.
- `src/stores/aiStore.ts`: AI config state and Tauri command wrappers.

## Backend

The backend lives in `src-tauri/src/`.

Important areas:

- `lib.rs`: configures plugins, initializes services, and registers Tauri commands.
- `commands/browser.rs`: exposes tab and navigation commands.
- `commands/ai.rs`: exposes AI config and model commands.
- `services/browser.rs`: manages tab state and Tauri child webviews.
- `services/ai_router.rs`: stores model config and routes requests to Ollama or OpenRouter-compatible APIs.
- `services/fallback.rs`: fetches fallback HTML for compatibility mode.
- `error.rs`: shared error type.

## Browser Service

`BrowserService` stores tab state in an `Arc<RwLock<BrowserState>>`.

Each tab contains:

- UUID.
- URL.
- Title.
- Background-tab flag.
- Creation timestamp.
- Fallback-mode flag.
- Loaded flag.

When a tab opens, the backend creates a Tauri child webview using the label `tab-{uuid}`. Inactive tabs are hidden. Active tabs are shown unless they are internal pages or in fallback mode.

Navigation starts a timeout. If a page does not report loaded within the timeout window, the tab enters compatibility mode and the native child webview is hidden.

## Fallback Rendering

When a tab enters fallback mode, the frontend calls `fetch_fallback` and renders the returned HTML inside an iframe with:

```text
sandbox="allow-scripts allow-forms"
```

This is useful as a compatibility path, but it must be treated as security-sensitive. Public releases should document and test the fallback model carefully.

## AI Router

`AiRouter` stores a `ModelConfig` containing:

- Selected tier.
- OpenRouter key placeholder.
- Ollama URL.
- Selected model.

Routing behavior:

- `Local` sends prompts to Ollama `/api/generate`.
- `Freemium` sends prompts to OpenRouter chat completions.
- `Premium` currently also sends prompts through OpenRouter chat completions.

The intent scoring helper is rule-based:

- Privacy, offline, and local intents route to `Local`.
- Code, analysis, complex, and reasoning intents route to `Premium`.
- Other intents route to `Freemium`.

## Internal Pages

The current internal page is:

```text
cntrl://settings
```

The URL bar navigates to this internal route, and `WebView.tsx` renders the settings component instead of showing a native child webview.

## Events

The backend emits:

```text
tabs-updated
```

The frontend listens for this event and refreshes tab state.

## Security Notes

The app currently disables or relaxes some CSP behavior in `src-tauri/tauri.conf.json`. This is acceptable for early prototyping, but public releases should define stricter security controls before inviting broad usage.

High-priority hardening areas:

- Tauri CSP.
- Tauri capabilities.
- Fallback HTML sanitization and iframe sandbox policy.
- Webview navigation restrictions.
- Provider key storage.
- Future autonomous action permissions.

## Which file should I edit?

The table below provides a quick reference for common development tasks.

| Task                                    | File / Directory                      |
| --------------------------------------- | ------------------------------------- |
| Change the main application layout      | `src/App.tsx`                         |
| Modify the URL bar                      | `src/components/UrlBar.tsx`           |
| Update browser tabs                     | `src/components/TabBar.tsx`           |
| Change the embedded webview behavior    | `src/components/WebView.tsx`          |
| Update the Settings page                | `src/components/SettingsPage.tsx`     |
| Modify browser state management         | `src/stores/browserStore.ts`          |
| Update AI configuration state           | `src/stores/aiStore.ts`               |
| Add or modify a Tauri command           | `src-tauri/src/commands/`             |
| Change backend browser services         | `src-tauri/src/services/browser.rs`   |
| Modify AI routing logic                 | `src-tauri/src/services/ai_router.rs` |
| Update Tauri configuration              | `src-tauri/tauri.conf.json`           |
| Change frontend dependencies or scripts | `package.json`                        |
| Update Rust dependencies                | `src-tauri/Cargo.toml`                |
| Replace application icons               | `src-tauri/icons/`                    |
| Modify global design tokens             | `src/styles/tokens.css`               |

## Common Development Commands

```bash
# Install frontend dependencies
npm install

# Start the development application
npm run tauri dev

# Run frontend tests
npm test

# Format Rust code
cargo fmt

# Run Rust tests
cargo test
```

## Repository Layout

```
CNTRL/
├── src/                  # SolidJS frontend
│   ├── components/       # UI components
│   ├── stores/           # Application state
│   ├── styles/           # Shared styles
│   └── assets/           # Frontend assets
│
├── src-tauri/            # Rust backend
│   ├── src/
│   │   ├── commands/     # Tauri commands
│   │   ├── services/     # Backend services
│   │   ├── lib.rs        # Backend initialization
│   │   └── main.rs       # Application entry point
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── public/               # Static frontend assets
├── package.json
└── docs/

```
