# CNTRL Browser

CNTRL Browser is an experimental desktop browser for intent-based autonomous browsing. It combines a Tauri v2 Rust backend with a SolidJS frontend, native child webviews, compatibility fallback rendering, and a configurable AI router for local and cloud model providers.

The project is currently an early-stage prototype. It already includes browser chrome, tab management, native webview navigation, fallback HTML rendering, AI model configuration, and intent-tier scoring. Natural-language command execution, persistent memory, background agents, and a plugin SDK are planned but not complete.

## Features

- Desktop browser shell built with Tauri v2, Rust, SolidJS, TypeScript, and Vite.
- Multi-tab browser UI with new tab, close tab, active tab switching, back, forward, reload, and URL navigation.
- Native webview engine using Tauri child webviews.
- Compatibility fallback mode for pages that do not load in the native child webview within the timeout window.
- Internal settings page at `vibe://settings`.
- AI router with three tiers:
  - Local: Ollama-compatible local models.
  - Freemium: OpenRouter free models.
  - Premium: OpenRouter or other compatible higher-capability models.
- OpenRouter key storage with masked UI display.
- Intent scoring helper that routes sample intents toward local, freemium, or premium model tiers.
- Frontend tests with Vitest and Solid Testing Library.
- Rust service tests and integration-test scaffolding.

## Tech Stack

- Tauri v2
- Rust 2021
- SolidJS
- TypeScript
- Vite
- Biome
- Vitest
- Reqwest
- Tokio
- OpenRouter API
- Ollama API

## Project Structure

```text
.
├── src/
│   ├── components/          # Browser chrome, URL bar, webview area, settings page
│   ├── stores/              # Solid stores for browser state and AI config
│   ├── styles/              # Design tokens
│   ├── test/                # Frontend test setup and smoke tests
│   ├── App.tsx              # App shell
│   └── index.tsx            # Frontend entrypoint
├── src-tauri/
│   ├── src/
│   │   ├── commands/        # Tauri commands exposed to the frontend
│   │   ├── services/        # Browser service, AI router, fallback service
│   │   ├── error.rs         # Shared error type
│   │   ├── lib.rs           # Tauri app setup
│   │   └── main.rs          # Native entrypoint
│   ├── capabilities/        # Tauri capability permissions
│   ├── icons/               # App icons
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
├── vite.config.ts
├── vitest.config.ts
└── biome.json
```

## Prerequisites

- Node.js 20 or newer.
- npm.
- Rust stable toolchain.
- Tauri v2 system prerequisites for your OS.
- Optional: Ollama for local model routing.
- Optional: OpenRouter API key for freemium or premium model routing.

See the official Tauri prerequisites guide for OS-specific packages: https://tauri.app/start/prerequisites/

## Getting Started

Install dependencies:

```bash
npm install
```

Start the Tauri development app:

```bash
npm run tauri dev
```

Run the frontend only:

```bash
npm run dev
```

Build the frontend:

```bash
npm run build
```

Build the Tauri app:

```bash
npm run tauri build
```

## AI Configuration

Open the settings page from the browser toolbar or navigate to:

```text
vibe://settings
```

Available tiers:

- `Local`: sends prompts to an Ollama server. The default Ollama URL is `http://localhost:11434`.
- `Freemium`: uses OpenRouter free models.
- `Premium`: uses the selected paid or higher-capability model through OpenRouter-compatible routing.

OpenRouter keys are stored by the desktop backend and displayed as a masked placeholder in the UI. Do not commit real API keys, local storage files, `.env` files, or app data generated during testing.

## Testing

Run frontend tests:

```bash
npm test
```

If the repository does not yet define an npm test script, use:

```bash
npx vitest run
```

Run Rust tests:

```bash
cd src-tauri
cargo test
```

Run formatting:

```bash
npx biome format . --write
```

## Current Status

Completed:

- Project scaffold.
- Tauri desktop shell.
- Browser chrome and tab state.
- Native child webview flow.
- Compatibility fallback architecture.
- AI configuration UI.
- Local, free, and premium model routing scaffolding.
- Intent-tier scoring helper.

In progress or planned:

- Natural-language command bar.
- Autonomous action execution.
- Memory engine.
- Security hardening.
- Background agents.
- Macro recorder.
- Plugin SDK.
- Production-ready release workflow.

## Roadmap

See [docs/ROADMAP.md](docs/ROADMAP.md).

## Architecture

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Contributing

Contributions are welcome once the repository is published. Please read [CONTRIBUTING.md](CONTRIBUTING.md) before opening issues or pull requests.

## Security

This project embeds browser surfaces, remote content, and AI-provider integrations. Please read [SECURITY.md](SECURITY.md) before reporting vulnerabilities or shipping public builds.

## License

MIT. See [LICENSE](LICENSE).
