/**
 * @module test/types.test
 * Unit tests for shared TypeScript type definitions in `src/types/`.
 * Verifies that the type shapes match what the Rust backend serialises.
 */
import { describe, expect, it } from "vitest";
import type { Tab, BrowserState } from "../types/browser";
import type { ModelConfig, ModelTier, ProviderHealth } from "../types/ai";

describe("Tab type shape", () => {
  it("accepts a valid fully-specified Tab object", () => {
    const tab: Tab = {
      id: "550e8400-e29b-41d4-a716-446655440000",
      url: "https://example.com",
      title: "Example Domain",
      favicon: undefined,
      is_background: false,
      created_at: new Date().toISOString(),
      fallback_mode: false,
      loaded: true,
    };
    expect(tab.id).toBeTruthy();
    expect(tab.url).toBe("https://example.com");
    expect(tab.is_background).toBe(false);
    expect(tab.fallback_mode).toBe(false);
    expect(tab.loaded).toBe(true);
  });

  it("accepts a background tab with optional favicon omitted", () => {
    const tab: Tab = {
      id: "550e8400-e29b-41d4-a716-446655440001",
      url: "https://background.example.com",
      title: "Background Tab",
      is_background: true,
      created_at: new Date().toISOString(),
      fallback_mode: false,
      loaded: false,
    };
    expect(tab.is_background).toBe(true);
    expect(tab.favicon).toBeUndefined();
  });
});

describe("BrowserState type shape", () => {
  it("accepts a valid BrowserState with tabs and activeTabId", () => {
    const state: BrowserState = {
      tabs: [],
      activeTabId: null,
    };
    expect(state.tabs).toEqual([]);
    expect(state.activeTabId).toBeNull();
  });
});

describe("ModelTier type", () => {
  it("accepts all three tier values", () => {
    const tiers: ModelTier[] = ["Local", "Freemium", "Premium"];
    expect(tiers).toHaveLength(3);
  });
});

describe("ModelConfig type shape", () => {
  it("accepts a minimal config with required fields", () => {
    const config: ModelConfig = {
      tier: "Freemium",
      openrouter_key: null,
      ollama_url: "http://localhost:11434",
      selected_model: "meta-llama/llama-3-8b-instruct:free",
    };
    expect(config.tier).toBe("Freemium");
    expect(config.openrouter_key).toBeNull();
  });

  it("accepts a config with optional keys set", () => {
    const config: ModelConfig = {
      tier: "Freemium",
      openrouter_key: "***stored***",
      ollama_url: "http://localhost:11434",
      selected_model: "meta-llama/llama-3-8b-instruct:free",
      gemini_key: "***stored***",
      groq_key: null,
      hf_token: undefined,
    };
    expect(config.gemini_key).toBe("***stored***");
    expect(config.groq_key).toBeNull();
    expect(config.hf_token).toBeUndefined();
  });
});

describe("ProviderHealth type shape", () => {
  it("accepts a healthy provider entry", () => {
    const health: ProviderHealth = {
      provider: "Ollama",
      healthy: true,
    };
    expect(health.provider).toBe("Ollama");
    expect(health.healthy).toBe(true);
  });

  it("accepts an unhealthy provider entry", () => {
    const health: ProviderHealth = {
      provider: "Gemini",
      healthy: false,
    };
    expect(health.healthy).toBe(false);
  });
});
