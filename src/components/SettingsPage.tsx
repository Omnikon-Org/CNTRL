import {
  Component,
  For,
  Show,
  createSignal,
  onMount,
} from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { askAi, getHfModels, getOpenRouterFreeModels, healthCheckAll, testIntentRouter } from "../stores/aiStore";
import type { ProviderHealth } from "../types";
import "./SettingsPage.css";
import { browserActions } from "../stores/browserStore";
import { AuditViewer } from "./AuditViewer";
import { PluginManager } from "./PluginManager";
const IconBot = () => (
  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <rect width="18" height="10" x="3" y="11" rx="2" />
    <circle cx="12" cy="5" r="2" />
    <path d="M12 7v4" />
    <line x1="8" x2="8" y1="16" y2="16" />
    <line x1="16" x2="16" y1="16" y2="16" />
  </svg>
);
const IconKey = () => (
  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <circle cx="7.5" cy="15.5" r="5.5" />
    <path d="m21 2-9.6 9.6" />
    <path d="m15.5 7.5 3 3L22 7l-3-3" />
  </svg>
);

const IconSparkles = () => (
  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="m12 3-1.912 5.813a2 2 0 0 1-1.275 1.275L3 12l5.813 1.912a2 2 0 0 1 1.275 1.275L12 21l1.912-5.813a2 2 0 0 1 1.275-1.275L21 12l-5.813-1.912a2 2 0 0 1-1.275-1.275L12 3Z" />
    <path d="M5 3v4" /><path d="M19 17v4" /><path d="M3 5h4" /><path d="M17 19h4" />
  </svg>
);
const IconBoxes = () => (
  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M2.97 12.92A2 2 0 0 0 2 14.63v3.24a2 2 0 0 0 .97 1.71l3 1.8a2 2 0 0 0 2.06 0L12 19v-5.5l-5-3-4.03 2.42Z" />
    <path d="m7 16.5-4.74-2.85" /><path d="m7 16.5 5-3" /><path d="M7 16.5v5.17" />
    <path d="M12 13.5V19l3.97 2.38a2 2 0 0 0 2.06 0l3-1.8a2 2 0 0 0 .97-1.71v-3.24a2 2 0 0 0-.97-1.71L17 10.5l-5 3Z" />
    <path d="m17 16.5-5-3" /><path d="m17 16.5 4.74-2.85" /><path d="M17 16.5v5.17" />
    <path d="M7.97 4.42A2 2 0 0 0 7 6.13v4.37l5 3 5-3V6.13a2 2 0 0 0-.97-1.71l-3-1.8a2 2 0 0 0-2.06 0l-3 1.8Z" />
    <path d="M12 8 7.26 5.15" /><path d="m12 8 4.74-2.85" /><path d="M12 13.5V8" />
  </svg>
);
const IconEye = () => (
  <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z" />
    <circle cx="12" cy="12" r="3" />
  </svg>
);
const IconEyeOff = () => (
  <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M9.88 9.88a3 3 0 1 0 4.24 4.24" />
    <path d="M10.73 5.08A10.43 10.43 0 0 1 12 5c7 0 10 7 10 7a13.16 13.16 0 0 1-1.67 2.68" />
    <path d="M6.61 6.61A13.526 13.526 0 0 0 2 12s3 7 10 7a9.74 9.74 0 0 0 5.39-1.61" />
    <line x1="2" x2="22" y1="2" y2="22" />
  </svg>
);
const IconLoader = () => (
  <svg class="sp-spin" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" aria-hidden="true">
    <path d="M21 12a9 9 0 1 1-6.219-8.56" />
  </svg>
);

type IntentScore = [string, number, string];

interface ProviderKeyEntry {
  id: string;
  label: string;
  placeholder: string;
}
const PROVIDER_KEYS: ProviderKeyEntry[] = [
  { id: "openrouter", label: "OpenRouter API Key", placeholder: "sk-or-v1-…" },
  { id: "gemini",     label: "Google Gemini API Key", placeholder: "AIza…" },
  { id: "groq",       label: "Groq API Key", placeholder: "gsk_…" },
  { id: "huggingface",label: "HuggingFace Token", placeholder: "hf_…" },
];

export const SettingsPage: Component = () => {
  const [keyInputs, setKeyInputs] = createSignal<Record<string, string>>({});
  const [showKeys, setShowKeys] = createSignal<Record<string, boolean>>({});
  const [keySaveStatus, setKeySaveStatus] = createSignal<Record<string, "idle" | "saving" | "saved" | "error">>({});
  
  const [ollamaUrl, setOllamaUrl] = createSignal("http://localhost:11434");
  const [ollamaModel, setOllamaModel] = createSignal("llama3");
  const [testPrompt, setTestPrompt] = createSignal("What is the capital of France?");
  const [testResponse, setTestResponse] = createSignal("");
  const [testIsError, setTestIsError] = createSignal(false);
  const [isTesting, setIsTesting] = createSignal(false);
  const [intentScores, setIntentScores] = createSignal<IntentScore[]>([]);
  const [isScoring, setIsScoring] = createSignal(false);
  const [userAgent, setUserAgent] = createSignal("");
  const [isLoadingBrowserConfig] = createSignal(true);
  
  const [hfModels, setHfModels] = createSignal<string[]>([]);
  const [orModels, setOrModels] = createSignal<string[]>([]);
  const [isLoadingModels, setIsLoadingModels] = createSignal(false);

  const [privacyEnabled, setPrivacyEnabled] = createSignal(false);

  const [providerHealth, setProviderHealth] = createSignal<ProviderHealth[]>([]);
  const [isCheckingHealth, setIsCheckingHealth] = createSignal(false);
  
  onMount(async () => {
    const statuses: Record<string, string> = {};
    for (const p of PROVIDER_KEYS) {
      try {
        statuses[p.id] = await invoke<string>("get_api_key_status", { provider: p.id });
      } catch {
        statuses[p.id] = "";
      }
    }
    setKeyInputs(statuses);
    setIsLoadingModels(true);
    const [hf, or_] = await Promise.allSettled([getHfModels(), getOpenRouterFreeModels()]);
    if (hf.status === "fulfilled") setHfModels(hf.value);
    if (or_.status === "fulfilled") setOrModels(or_.value);
    setIsLoadingModels(false);
    
    try {
      const pEnabled = await invoke<boolean>("is_privacy_mode_enabled");
      setPrivacyEnabled(pEnabled);
    } catch (err) {
      console.error("Failed to load privacy mode status:", err);
    }

    await runHealthCheck();
  });
  const handleTogglePrivacy = async (enabled: boolean) => {
    setPrivacyEnabled(enabled);
    try {
      await invoke("set_privacy_mode", { enabled });
    } catch (err) {
      console.error("Failed to set privacy mode:", err);
      setPrivacyEnabled(!enabled);
    }
  };
  const handleSaveBrowserConfig = async () => {
    try {
      await browserActions.updateBrowserConfig({
        user_agent: userAgent() || null,
      });
    } catch (err) {
      console.error("Failed to save browser config:", err);
    }
  };
  const handleSaveKey = async (providerId: string): Promise<void> => {
    const value = keyInputs()[providerId] ?? "";
    setKeySaveStatus((prev) => ({ ...prev, [providerId]: "saving" }));
    try {
      await invoke<void>("store_api_key", { provider: providerId, value });
      const status = await invoke<string>("get_api_key_status", { provider: providerId });
      setKeyInputs((prev) => ({ ...prev, [providerId]: status }));
      setKeySaveStatus((prev) => ({ ...prev, [providerId]: "saved" }));
      setTimeout(() => setKeySaveStatus((prev) => ({ ...prev, [providerId]: "idle" })), 2500);
    } catch (err) {
      console.error("Failed to save key:", err);
      setKeySaveStatus((prev) => ({ ...prev, [providerId]: "error" }));
    }
  };
  const handleDeleteKey = async (providerId: string): Promise<void> => {
    await invoke<void>("delete_api_key", { provider: providerId });
    setKeyInputs((prev) => ({ ...prev, [providerId]: "" }));
  };
  const handleTestAi = async (): Promise<void> => {
    setIsTesting(true);
    setTestResponse("");
    setTestIsError(false);
    try {
      const response = await askAi(testPrompt());
      setTestResponse(response);
    } catch (err: unknown) {
      setTestResponse(String(err));
      setTestIsError(true);
    } finally {
      setIsTesting(false);
    }
  };
  const handleTestIntents = async (): Promise<void> => {
    setIsScoring(true);
    const sampleIntents = [
      "Summarize this article for me.",
      "Write a complex React hook for debouncing.",
      "Analyze the logical flaws in this reasoning.",
      "Find me a recipe for chocolate cake.",
      "What is the weather like today?",
      "Can you run locally and offline?",
      "Private and secure offline mode",
      "Translate this text to Spanish.",
      "Reason through this math problem step by step.",
      "Write a local script to rename files.",
    ];
    try {
      const scores = await testIntentRouter(sampleIntents);
      setIntentScores(scores as unknown as IntentScore[]);
    } catch (err) {
      console.error(err);
    } finally {
      setIsScoring(false);
    }
  };
  const runHealthCheck = async (): Promise<void> => {
    setIsCheckingHealth(true);
    try {
      const results = await healthCheckAll();
      const health: ProviderHealth[] = Object.entries(results).map(([provider, healthy]) => ({
        provider,
        healthy,
      }));
      setProviderHealth(health);
    } catch (err) {
      console.error("Health check failed:", err);
    } finally {
      setIsCheckingHealth(false);
    }
  };
  return (
    <div class="sp-page">
      <div class="sp-content">
        <header class="sp-header">
          <div class="sp-header-icon"><IconBoxes /></div>
          <div>
            <h1 class="sp-title">CNTRL Settings</h1>
            <p class="sp-subtitle">Configure AI providers and inspect connection health</p>
          </div>
        </header>
        {}
        <section class="sp-card" aria-labelledby="advanced-heading">
          <div class="sp-card-header">
            <span class="sp-card-icon"><IconKey /></span>
            <h2 class="sp-card-title" id="advanced-heading">Advanced Settings</h2>
          </div>
          <div class="sp-field">
            <label class="sp-label" for="sp-user-agent">
              User Agent
            </label>
            <div class="sp-input-group">
              <input
                id="sp-user-agent"
                class="sp-input"
                type="text"
                placeholder="Leave empty to use default Chrome User Agent"
                value={userAgent()}
                disabled={isLoadingBrowserConfig()}
                onInput={(e) => setUserAgent(e.currentTarget.value)}
              />
              <button
                type="button"
                class="sp-btn sp-btn-secondary"
                onClick={() => void handleSaveBrowserConfig()}
                disabled={isLoadingBrowserConfig()}
              >
                Save
              </button>
            </div>
            <p class="sp-hint">This User Agent will be applied to newly opened browser tabs.</p>
          </div>
        </section>
        {}
        <section class="sp-card" aria-labelledby="privacy-heading">
          <div class="sp-card-header">
            <span class="sp-card-icon"><IconEye /></span>
            <h2 class="sp-card-title" id="privacy-heading">Privacy & Security</h2>
          </div>
          <div class="sp-field">
            <div class="privacy-toggle-row">
              <div>
                <label class="sp-label" style="margin-bottom: 0.25rem;">
                  Privacy Guard Mode
                </label>
                <p class="sp-hint">
                  When enabled, all remote AI calls (Tier 2/3) are blocked. Only local Ollama (Tier 1) can be used.
                </p>
              </div>
              <label class="privacy-switch">
                <input
                  type="checkbox"
                  checked={privacyEnabled()}
                  onChange={(e) => void handleTogglePrivacy(e.currentTarget.checked)}
                />
                <span class="privacy-slider"></span>
              </label>
            </div>
          </div>
          <div class="sp-field" style="margin-top: 1rem;">
            <label class="sp-label">Security Audit Log</label>
            <p class="sp-hint">
              A real-time, append-only record of AI interactions and keychain credential accesses.
            </p>
            <AuditViewer />
          </div>
        </section>
        {}
        <section class="sp-card" aria-labelledby="health-heading">
          <div class="sp-card-header">
            <span class="sp-card-icon"><IconSparkles /></span>
            <h2 class="sp-card-title" id="health-heading">Provider Health</h2>
          </div>
          <div class="sp-health-grid" role="list" aria-label="Provider health status">
            <Show when={isCheckingHealth()} fallback={
              <Show when={providerHealth().length > 0} fallback={
                <p class="sp-hint">No providers checked yet.</p>
              }>
                <For each={providerHealth()}>
                  {(entry) => (
                    <div class="sp-health-row" role="listitem">
                      <span
                        class={`status-dot ${entry.healthy ? "success" : "error"}`}
                        aria-label={entry.healthy ? "healthy" : "unreachable"}
                      />
                      <span class="sp-health-name">{entry.provider}</span>
                      <span class={`sp-health-label ${entry.healthy ? "sp-text-success" : "sp-text-danger"}`}>
                        {entry.healthy ? "OK" : "OFFLINE"}
                      </span>
                    </div>
                  )}
                </For>
              </Show>
            }>
              <div class="sp-status sp-status-processing">
                <IconLoader />
                <span>Checking providers…</span>
              </div>
            </Show>
          </div>
          <div class="sp-row">
            <button
              id="sp-health-check-btn"
              type="button"
              class="sp-btn sp-btn-secondary"
              onClick={() => void runHealthCheck()}
              disabled={isCheckingHealth()}
              aria-busy={isCheckingHealth()}
            >
              {isCheckingHealth() ? "Checking…" : "Refresh Health"}
            </button>
          </div>
        </section>
        {}
        <section class="sp-card" aria-labelledby="auth-heading">
          <div class="sp-card-header">
            <span class="sp-card-icon"><IconKey /></span>
            <h2 class="sp-card-title" id="auth-heading">API Keys</h2>
          </div>
          <p class="sp-hint" style="margin-bottom: 1rem;">
            Keys are stored in the OS keychain — never on disk or in any database.
          </p>
          <For each={PROVIDER_KEYS}>
            {(entry) => {
              const inputId = `sp-key-${entry.id}`;
              const status = () => keySaveStatus()[entry.id] ?? "idle";
              const value = () => keyInputs()[entry.id] ?? "";
              const visible = () => showKeys()[entry.id] ?? false;
              return (
                <div class="sp-field sp-key-field">
                  <label class="sp-label" for={inputId}>
                    {entry.label}
                  </label>
                  <div class="sp-input-group">
                    <input
                      id={inputId}
                      class="sp-input sp-input-key"
                      type={visible() ? "text" : "password"}
                      value={value()}
                      onInput={(e) =>
                        setKeyInputs((prev) => ({ ...prev, [entry.id]: e.currentTarget.value }))
                      }
                      placeholder={entry.placeholder}
                      autocomplete="off"
                      spellcheck={false}
                    />
                    <button
                      type="button"
                      class="sp-input-action"
                      onClick={() =>
                        setShowKeys((prev) => ({ ...prev, [entry.id]: !prev[entry.id] }))
                      }
                      title={visible() ? "Hide key" : "Show key"}
                      aria-label={visible() ? "Hide API key" : "Show API key"}
                    >
                      <Show when={visible()} fallback={<IconEye />}>
                        <IconEyeOff />
                      </Show>
                    </button>
                    <button
                      type="button"
                      class="sp-btn sp-btn-secondary"
                      onClick={() => void handleSaveKey(entry.id)}
                      disabled={status() === "saving" || !value()}
                      aria-busy={status() === "saving"}
                    >
                      {status() === "saving" ? "Saving…" : status() === "saved" ? "Saved ✓" : "Save"}
                    </button>
                    <Show when={value()}>
                      <button
                        type="button"
                        class="sp-btn sp-btn-danger"
                        onClick={() => void handleDeleteKey(entry.id)}
                        title="Remove key from keychain"
                        aria-label={`Remove ${entry.label} from keychain`}
                      >
                        Remove
                      </button>
                    </Show>
                  </div>
                  <Show when={status() === "error"}>
                    <p class="sp-hint sp-text-danger">Failed to save key. Check keychain access.</p>
                  </Show>
                </div>
              );
            }}
          </For>
        </section>
        {}
        <section class="sp-card" aria-labelledby="ollama-heading">
          <div class="sp-card-header">
            <span class="sp-card-icon"><IconBot /></span>
            <h2 class="sp-card-title" id="ollama-heading">Ollama (Local / Tier 1)</h2>
          </div>
          <div class="sp-field">
            <label class="sp-label" for="sp-ollama-url">Ollama API URL</label>
            <input
              id="sp-ollama-url"
              class="sp-input"
              type="text"
              value={ollamaUrl()}
              onInput={(e) => setOllamaUrl(e.currentTarget.value)}
              placeholder="http://localhost:11434"
            />
          </div>
          <div class="sp-field">
            <label class="sp-label" for="sp-ollama-model">Model</label>
            <input
              id="sp-ollama-model"
              class="sp-input"
              type="text"
              value={ollamaModel()}
              onInput={(e) => setOllamaModel(e.currentTarget.value)}
              placeholder="llama3"
            />
          </div>
        </section>
        {}
        <section class="sp-card" aria-labelledby="models-heading">
          <div class="sp-card-header">
            <span class="sp-card-icon"><IconBoxes /></span>
            <h2 class="sp-card-title" id="models-heading">Available Models</h2>
          </div>
          <Show when={isLoadingModels()} fallback={
            <div class="sp-model-cols">
              <div>
                <p class="sp-label" style="margin-bottom: 0.5rem;">OpenRouter Free Models ({orModels().length})</p>
                <ul class="sp-model-list" role="list" aria-label="OpenRouter free models">
                  <For each={orModels().slice(0, 8)}>
                    {(m) => <li class="sp-model-item" role="listitem">{m}</li>}
                  </For>
                  <Show when={orModels().length > 8}>
                    <li class="sp-model-item sp-text-secondary" role="listitem">+{orModels().length - 8} more…</li>
                  </Show>
                </ul>
              </div>
              <div>
                <p class="sp-label" style="margin-bottom: 0.5rem;">HuggingFace Models ({hfModels().length})</p>
                <ul class="sp-model-list" role="list" aria-label="HuggingFace models">
                  <For each={hfModels().slice(0, 8)}>
                    {(m) => <li class="sp-model-item" role="listitem">{m}</li>}
                  </For>
                  <Show when={hfModels().length > 8}>
                    <li class="sp-model-item sp-text-secondary" role="listitem">+{hfModels().length - 8} more…</li>
                  </Show>
                </ul>
              </div>
            </div>
          }>
            <div class="sp-status sp-status-processing">
              <IconLoader />
              <span>Loading model lists…</span>
            </div>
          </Show>
        </section>
        {}
        <section class="sp-card" aria-labelledby="test-heading">
          <div class="sp-card-header">
            <span class="sp-card-icon"><IconSparkles /></span>
            <h2 class="sp-card-title" id="test-heading">Test AI Connection</h2>
          </div>
          <div class="sp-field">
            <label class="sp-label" for="sp-test-prompt">Prompt</label>
            <input
              id="sp-test-prompt"
              class="sp-input"
              type="text"
              value={testPrompt()}
              onInput={(e) => setTestPrompt(e.currentTarget.value)}
              placeholder="Ask the AI a question…"
            />
          </div>
          <div class="sp-row">
            <button
              id="sp-test-ai-btn"
              type="button"
              class="sp-btn sp-btn-secondary"
              onClick={() => void handleTestAi()}
              disabled={isTesting()}
              aria-busy={isTesting()}
            >
              {isTesting() ? "Testing…" : "Test AI"}
            </button>
          </div>
          <Show when={testResponse()}>
            <div class={`sp-response${testIsError() ? " sp-response-error" : ""}`} role="status">
              <div class="sp-response-label">
                <span class={`status-indicator ${testIsError() ? "error" : "success"}`} />
                <span>{testIsError() ? "Error" : "Response"}</span>
              </div>
              <p class="sp-response-text">{testResponse()}</p>
            </div>
          </Show>
        </section>
        {}
        <section class="sp-card" aria-labelledby="router-heading">
          <div class="sp-card-header">
            <span class="sp-card-icon"><IconBoxes /></span>
            <h2 class="sp-card-title" id="router-heading">Intent Router</h2>
            <p class="sp-card-desc">Run 10 sample prompts through the complexity scorer.</p>
          </div>

          <div class="sp-row">
            <button
              id="sp-run-intent-test-btn"
              type="button"
              class="sp-btn sp-btn-secondary"
              onClick={() => void handleTestIntents()}
              disabled={isScoring()}
              aria-busy={isScoring()}
            >
              {isScoring() ? "Scoring…" : "Run Intent Test"}
            </button>
          </div>
          <Show when={intentScores().length > 0}>
            <ul class="sp-scores" role="list">
              <For each={intentScores()}>
                {([intent, score, tier]) => (
                  <li class="sp-score-item" role="listitem">
                    <span class={`sp-tier-badge sp-tier-${(tier ?? "").toLowerCase()}`}>
                      {tier} ({score})
                    </span>
                    <span class="sp-score-text">{intent}</span>
                  </li>
                )}
              </For>
            </ul>
          </Show>
          <AuditViewer />
        </section>
        
        <PluginManager />
      </div>
    </div>
  );
};