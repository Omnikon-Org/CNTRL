# AI Router

The AI Router (Phase 4) abstracts multi-provider API calls. It natively integrates with:
- **Ollama**: For local, offline privacy-first inference.
- **Groq**: For ultra-low latency cloud requests.
- **OpenRouter**: For diverse model proxying.
- **HuggingFace**: For specialized model endpoints.

## Fallback Logic
If a provider fails or the network is unreachable, the router seamlessly falls back to Ollama if Privacy Mode is enabled, or alerts the user otherwise.
