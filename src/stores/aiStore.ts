/**
 * @module stores/aiStore
 * Reactive AI configuration state managed by a SolidJS store.
 * Types are imported from `../types` — do not redeclare them here.
 */
import { invoke } from "@tauri-apps/api/core";
import { createStore } from "solid-js/store";
import { LRUCache } from "../utils/cache";
import type { IntentRouterResult, ModelConfig, ModelTier } from "../types";

export type { ModelConfig, ModelTier, IntentRouterResult };

const modelCache = new LRUCache<string, string[]>(10, 5 * 60 * 1000); // 5 minutes TTL

export const [aiState, setAiState] = createStore<ModelConfig>({
  tier: "Freemium",
  openrouter_key: null,
  ollama_url: "http://localhost:11434",
  selected_model: "meta-llama/llama-3-8b-instruct:free",
});

/**
 * Initialises the AI store. Config is now provider-specific (each provider's
 * key lives in the OS keychain); this function is a no-op kept for call-site
 * compatibility.
 */
export const initAiStore = async (): Promise<void> => {
  // Keys are fetched per-provider from the OS keychain via get_api_key_status.
  // Nothing to load into the global store.
};

/**
 * Sends a one-shot completion request through the active AI provider.
 * @param prompt - The user's prompt text.
 * @returns The model's response string.
 */
export const askAi = async (prompt: string): Promise<string> => {
  return invoke<string>("ask_ai", { prompt });
};

/**
 * Fetches the list of popular HuggingFace text-generation models.
 * @returns Array of model ID strings.
 */
export const getHfModels = async (): Promise<string[]> => {
  const cached = modelCache.get("hf_models");
  if (cached) return cached;

  try {
    const models = await invoke<string[]>("get_hf_models");
    modelCache.set("hf_models", models);
    return models;
  } catch (error) {
    console.error("Failed to fetch HF models:", error);
    return [];
  }
};

/**
 * Fetches the list of free models available on OpenRouter.
 * @returns Array of model ID strings.
 */
export const getOpenRouterFreeModels = async (): Promise<string[]> => {
  const cached = modelCache.get("openrouter_models");
  if (cached) return cached;

  try {
    const models = await invoke<string[]>("get_openrouter_free_models");
    modelCache.set("openrouter_models", models);
    return models;
  } catch (error) {
    console.error("Failed to fetch OpenRouter free models:", error);
    return [];
  }
};

/**
 * Tests the complexity-scoring router against an array of sample intents.
 * @param intents - Array of natural-language strings to score.
 * @returns Array of `[intent, score, tier_string]` tuples.
 */
export const testIntentRouter = async (
  intents: string[]
): Promise<[string, number, string][]> => {
  try {
    return await invoke<[string, number, string][]>("test_intent_router", { intents });
  } catch (error) {
    console.error("Failed to test intent router:", error);
    return [];
  }
};

/**
 * Performs a health check on all configured AI providers.
 * @returns Record mapping provider names to boolean health status.
 */
export const healthCheckAll = async (): Promise<Record<string, boolean>> => {
  try {
    return await invoke<Record<string, boolean>>("health_check_all");
  } catch (error) {
    console.error("Failed to run health checks:", error);
    return {};
  }
};
