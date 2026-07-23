export type ModelTier = "Local" | "Freemium" | "Premium";
export interface ModelConfig {
  tier: ModelTier;
  openrouter_key: string | null;
  ollama_url: string;
  selected_model: string;
  gemini_key?: string | null | undefined;
  groq_key?: string | null | undefined;
  hf_token?: string | null | undefined;
}
export interface ProviderHealth {
  provider: string;
  healthy: boolean;
}
export type IntentRouterResult = [string, string][];