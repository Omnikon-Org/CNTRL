import { createStore } from "solid-js/store";
import { invoke } from "@tauri-apps/api/core";

export type ModelTier = "Local" | "Freemium" | "Premium";

export interface ModelConfig {
    tier: ModelTier;
    openrouter_key: string | null;
    ollama_url: string;
    selected_model: string;
}

export const [aiState, setAiState] = createStore<ModelConfig>({
    tier: "Freemium",
    openrouter_key: null,
    ollama_url: "http://localhost:11434",
    selected_model: "meta-llama/llama-3-8b-instruct:free",
});

export const initAiStore = async () => {
    try {
        const config: ModelConfig = await invoke("get_ai_config");
        setAiState(config);
    } catch (error) {
        console.error("Failed to load AI config:", error);
    }
};

export const updateAiConfig = async (newConfig: ModelConfig) => {
    try {
        await invoke("update_ai_config", { config: newConfig });
        setAiState(newConfig);
    } catch (error) {
        console.error("Failed to update AI config:", error);
    }
};

export const askAi = async (prompt: string): Promise<string> => {
    try {
        return await invoke("ask_ai", { prompt });
    } catch (error) {
        console.error("AI Request Failed:", error);
        throw error;
    }
};

export const getHfModels = async (): Promise<string[]> => {
    try {
        return await invoke("get_hf_models");
    } catch (error) {
        console.error("Failed to fetch HF models:", error);
        return [];
    }
};

export const getOpenRouterFreeModels = async (): Promise<string[]> => {
    try {
        return await invoke("get_openrouter_free_models");
    } catch (error) {
        console.error("Failed to fetch OpenRouter free models:", error);
        return [];
    }
};

export const testIntentRouter = async (intents: string[]): Promise<[string, string][]> => {
    try {
        return await invoke("test_intent_router", { intents });
    } catch (error) {
        console.error("Failed to test intent router:", error);
        return [];
    }
};
