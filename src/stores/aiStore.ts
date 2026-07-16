import { invoke } from "@tauri-apps/api/core";
import { createStore } from "solid-js/store";
import type { IntentRouterResult, ModelConfig, ModelTier } from "../types";
export type { ModelConfig, ModelTier, IntentRouterResult };
export const [aiState, setAiState] = createStore<ModelConfig>({
  tier: "Freemium",
  openrouter_key: null,
  ollama_url: "http://localhost:11434",
  selected_model: "meta-llama/llama-3-8b-instruct:free",
});
export const initAiStore = async (): Promise<void> => {
  
};
export const askAi = async (prompt: string): Promise<string> => {
  return invoke<string>("ask_ai", { prompt });
};
export const getHfModels = async (): Promise<string[]> => {
  try {
    return await invoke<string[]>("get_hf_models");
  } catch (error) {
    console.error("Failed to fetch HF models:", error);
    return [];
  }
};
export const getOpenRouterFreeModels = async (): Promise<string[]> => {
  try {
    return await invoke<string[]>("get_openrouter_free_models");
  } catch (error) {
    console.error("Failed to fetch OpenRouter free models:", error);
    return [];
  }
};
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
export const healthCheckAll = async (): Promise<Record<string, boolean>> => {
  try {
    return await invoke<Record<string, boolean>>("health_check_all");
  } catch (error) {
    console.error("Failed to run health checks:", error);
    return {};
  }
};