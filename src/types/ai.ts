/**
 * @module types/ai
 * Shared AI-domain TypeScript types that mirror the Rust `ModelConfig`,
 * `ModelTier`, and provider-related structs returned via Tauri IPC.
 * All frontend AI code must import types from here.
 */

/**
 * Mirrors `src-tauri/src/services/ai/mod.rs :: ModelTier`.
 * - `"Local"`    → Tier 1: Ollama running on localhost
 * - `"Freemium"` → Tier 2: OpenRouter free models, Gemini free tier, Groq free tier
 * - `"Premium"`  → Tier 3: Paid OpenAI-compatible endpoints (Claude, GPT-4o, etc.)
 */
export type ModelTier = "Local" | "Freemium" | "Premium";

/**
 * Mirrors `src-tauri/src/services/ai/mod.rs :: ModelConfig`.
 * This shape is serialised by Tauri and returned from `get_ai_config`.
 * Secrets (API keys) are stored in the OS keychain; only a masked
 * sentinel is carried here so the UI can show "key is set" state.
 */
export interface ModelConfig {
  /** Which tier drives the current router default. */
  tier: ModelTier;
  /**
   * Masked sentinel `"sk-or-***"` if a key is stored in the keychain,
   * otherwise `null`. Never contains the real key.
   */
  openrouter_key: string | null;
  /** Base URL of the local Ollama instance. Defaults to `http://localhost:11434`. */
  ollama_url: string;
  /** ID of the model to use for the selected tier/provider. */
  selected_model: string;
  /** Optional Gemini API key sentinel. */
  gemini_key?: string | null | undefined;
  /** Optional Groq API key sentinel. */
  groq_key?: string | null | undefined;
  /** Optional HuggingFace token sentinel. */
  hf_token?: string | null | undefined;
}

/**
 * Describes the health/availability of a single provider as returned by
 * `health_check_all`.
 */
export interface ProviderHealth {
  /** Provider identifier — matches the keys returned by `health_check_all`. */
  provider: string;
  /** `true` if the provider answered a health ping within the timeout. */
  healthy: boolean;
}

/**
 * Result type returned by the `test_intent_router` command.
 * Each tuple is `[intent_text, tier_string]`.
 */
export type IntentRouterResult = [string, string][];
