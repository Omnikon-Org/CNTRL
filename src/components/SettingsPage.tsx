import { Component, createEffect, createSignal, onMount, Show } from "solid-js";
import {
  aiState,
  askAi,
  getHfModels,
  getOpenRouterFreeModels,
  ModelTier,
  testIntentRouter,
  updateAiConfig,
} from "../stores/aiStore";
import "./SettingsPage.css";

const IconBot = () => (
  <svg
    width="16"
    height="16"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
  >
    <rect width="18" height="10" x="3" y="11" rx="2" />
    <circle cx="12" cy="5" r="2" />
    <path d="M12 7v4" />
    <line x1="8" x2="8" y1="16" y2="16" />
    <line x1="16" x2="16" y1="16" y2="16" />
  </svg>
);

const IconKey = () => (
  <svg
    width="16"
    height="16"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
  >
    <circle cx="7.5" cy="15.5" r="5.5" />
    <path d="m21 2-9.6 9.6" />
    <path d="m15.5 7.5 3 3L22 7l-3-3" />
  </svg>
);

const IconSparkles = () => (
  <svg
    width="16"
    height="16"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
  >
    <path d="m12 3-1.912 5.813a2 2 0 0 1-1.275 1.275L3 12l5.813 1.912a2 2 0 0 1 1.275 1.275L12 21l1.912-5.813a2 2 0 0 1 1.275-1.275L21 12l-5.813-1.912a2 2 0 0 1-1.275-1.275L12 3Z" />
    <path d="M5 3v4" />
    <path d="M19 17v4" />
    <path d="M3 5h4" />
    <path d="M17 19h4" />
  </svg>
);

const IconCheckCircle = () => (
  <svg
    width="15"
    height="15"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2.5"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
  >
    <circle cx="12" cy="12" r="10" />
    <path d="m9 12 2 2 4-4" />
  </svg>
);

const IconEye = () => (
  <svg
    width="15"
    height="15"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
  >
    <path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z" />
    <circle cx="12" cy="12" r="3" />
  </svg>
);

const IconEyeOff = () => (
  <svg
    width="15"
    height="15"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
  >
    <path d="M9.88 9.88a3 3 0 1 0 4.24 4.24" />
    <path d="M10.73 5.08A10.43 10.43 0 0 1 12 5c7 0 10 7 10 7a13.16 13.16 0 0 1-1.67 2.68" />
    <path d="M6.61 6.61A13.526 13.526 0 0 0 2 12s3 7 10 7a9.74 9.74 0 0 0 5.39-1.61" />
    <line x1="2" x2="22" y1="2" y2="22" />
  </svg>
);

const IconCopy = () => (
  <svg
    width="14"
    height="14"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
  >
    <rect width="14" height="14" x="8" y="8" rx="2" ry="2" />
    <path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" />
  </svg>
);

const IconBoxes = () => (
  <svg
    width="16"
    height="16"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
  >
    <path d="M2.97 12.92A2 2 0 0 0 2 14.63v3.24a2 2 0 0 0 .97 1.71l3 1.8a2 2 0 0 0 2.06 0L12 19v-5.5l-5-3-4.03 2.42Z" />
    <path d="m7 16.5-4.74-2.85" />
    <path d="m7 16.5 5-3" />
    <path d="M7 16.5v5.17" />
    <path d="M12 13.5V19l3.97 2.38a2 2 0 0 0 2.06 0l3-1.8a2 2 0 0 0 .97-1.71v-3.24a2 2 0 0 0-.97-1.71L17 10.5l-5 3Z" />
    <path d="m17 16.5-5-3" />
    <path d="m17 16.5 4.74-2.85" />
    <path d="M17 16.5v5.17" />
    <path d="M7.97 4.42A2 2 0 0 0 7 6.13v4.37l5 3 5-3V6.13a2 2 0 0 0-.97-1.71l-3-1.8a2 2 0 0 0-2.06 0l-3 1.8Z" />
    <path d="M12 8 7.26 5.15" />
    <path d="m12 8 4.74-2.85" />
    <path d="M12 13.5V8" />
  </svg>
);

const IconLoader = () => (
  <svg
    class="sp-spin"
    width="14"
    height="14"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2.5"
    stroke-linecap="round"
    aria-hidden="true"
  >
    <path d="M21 12a9 9 0 1 1-6.219-8.56" />
  </svg>
);

export const SettingsPage: Component = () => {
  const [tier, setTier] = createSignal<ModelTier>(aiState.tier);
  const [openRouterKey, setOpenRouterKey] = createSignal(aiState.openrouter_key || "");
  const [ollamaUrl, setOllamaUrl] = createSignal(aiState.ollama_url);
  const [selectedModel, setSelectedModel] = createSignal(aiState.selected_model);
  const [saveStatus, setSaveStatus] = createSignal<"idle" | "saving" | "saved" | "error">("idle");

  const [testPrompt, setTestPrompt] = createSignal("What is the capital of France?");
  const [testResponse, setTestResponse] = createSignal("");
  const [testIsError, setTestIsError] = createSignal(false);
  const [isTesting, setIsTesting] = createSignal(false);

  const [models, setModels] = createSignal<string[]>([]);
  const [isLoadingModels, setIsLoadingModels] = createSignal(false);

  const [intentScores, setIntentScores] = createSignal<[string, string][]>([]);
  const [isScoring, setIsScoring] = createSignal(false);

  const [showKey, setShowKey] = createSignal(false);
  const [copied, setCopied] = createSignal(false);

  onMount(() => {
    setTier(aiState.tier);
    setOpenRouterKey(aiState.openrouter_key || "");
    setOllamaUrl(aiState.ollama_url);
    setSelectedModel(aiState.selected_model);
  });

  createEffect(() => {
    const currentTier = tier();
    if (currentTier === "Freemium") {
      setIsLoadingModels(true);
      getOpenRouterFreeModels().then((res) => {
        setModels(res);
        const firstModel = res[0];
        if (firstModel !== undefined && !res.includes(selectedModel())) {
          setSelectedModel(firstModel);
        }
        setIsLoadingModels(false);
      });
    } else if (currentTier === "Premium") {
      setIsLoadingModels(true);
      getHfModels().then((res) => {
        setModels(res);
        const firstModel = res[0];
        if (firstModel !== undefined && !res.includes(selectedModel())) {
          setSelectedModel(firstModel);
        }
        setIsLoadingModels(false);
      });
    } else {
      setModels([]);
      setSelectedModel("llama3");
    }
  });

  const handleSave = async (e: Event) => {
    e.preventDefault();
    setSaveStatus("saving");
    await updateAiConfig({
      tier: tier(),
      openrouter_key: openRouterKey() || null,
      ollama_url: ollamaUrl(),
      selected_model: selectedModel(),
    });
    setSaveStatus("saved");
    setTimeout(() => setSaveStatus("idle"), 2500);
  };

  const handleTestAi = async () => {
    setIsTesting(true);
    setTestResponse("");
    setTestIsError(false);
    try {
      const response = await askAi(testPrompt());
      setTestResponse(response);
      setTestIsError(false);
    } catch (err: any) {
      setTestResponse(`${err}`);
      setTestIsError(true);
    } finally {
      setIsTesting(false);
    }
  };

  const handleTestIntents = async () => {
    setIsScoring(true);
    const sampleIntents = [
      "Summarize this article for me.",
      "Write a complex react hook for debouncing.",
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
      setIntentScores(scores);
    } catch (err) {
      console.error(err);
    } finally {
      setIsScoring(false);
    }
  };

  const handleCopyKey = async () => {
    const key = openRouterKey();
    if (!key) return;
    await navigator.clipboard.writeText(key);
    setCopied(true);
    setTimeout(() => setCopied(false), 1800);
  };

  return (
    <div class="sp-page">
      <div class="sp-content">
        <header class="sp-header">
          <div class="sp-header-icon">
            <IconBoxes />
          </div>
          <div>
            <h1 class="sp-title">CNTRL Settings</h1>
            <p class="sp-subtitle">Configure your AI model and connection preferences</p>
          </div>
        </header>

        <form onSubmit={handleSave}>
          <section class="sp-card">
            <div class="sp-card-header">
              <span class="sp-card-icon">
                <IconBot />
              </span>
              <h2 class="sp-card-title">AI Configuration</h2>
            </div>

            <div class="sp-field">
              <label class="sp-label" for="sp-tier">
                AI Tier
              </label>
              <div class="sp-select-wrap">
                <select
                  id="sp-tier"
                  class="sp-select"
                  value={tier()}
                  onInput={(e) => setTier(e.currentTarget.value as ModelTier)}
                >
                  <option value="Local">Tier 1 — Local (Ollama)</option>
                  <option value="Freemium">Tier 2 — Freemium (OpenRouter Free)</option>
                  <option value="Premium">Tier 3 — Premium (HF / OpenRouter Pro)</option>
                </select>
                <span class="sp-select-caret" aria-hidden="true">
                  <svg
                    width="12"
                    height="12"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2.5"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  >
                    <path d="m6 9 6 6 6-6" />
                  </svg>
                </span>
              </div>
            </div>

            <div class="sp-field">
              <label class="sp-label" for="sp-model">
                Model
              </label>
              <Show
                when={tier() !== "Local"}
                fallback={
                  <input
                    id="sp-model"
                    class="sp-input"
                    type="text"
                    value={selectedModel()}
                    onInput={(e) => setSelectedModel(e.currentTarget.value)}
                    placeholder="e.g. llama3"
                  />
                }
              >
                <div class="sp-select-wrap">
                  <select
                    id="sp-model"
                    class="sp-select"
                    value={selectedModel()}
                    onInput={(e) => setSelectedModel(e.currentTarget.value)}
                    disabled={isLoadingModels()}
                  >
                    <Show when={isLoadingModels()}>
                      <option>Loading available models…</option>
                    </Show>
                    <Show when={!isLoadingModels() && models().length === 0}>
                      <option>No models found</option>
                    </Show>
                    {models().map((m) => (
                      <option value={m}>{m}</option>
                    ))}
                  </select>
                  <span class="sp-select-caret" aria-hidden="true">
                    <Show when={!isLoadingModels()} fallback={<IconLoader />}>
                      <svg
                        width="12"
                        height="12"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2.5"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                      >
                        <path d="m6 9 6 6 6-6" />
                      </svg>
                    </Show>
                  </span>
                </div>
              </Show>
            </div>

            <Show when={tier() === "Local"}>
              <div class="sp-field">
                <label class="sp-label" for="sp-ollama-url">
                  Ollama API URL
                </label>
                <input
                  id="sp-ollama-url"
                  class="sp-input"
                  type="text"
                  value={ollamaUrl()}
                  onInput={(e) => setOllamaUrl(e.currentTarget.value)}
                  placeholder="http://localhost:11434"
                />
              </div>
            </Show>
          </section>

          <Show when={tier() === "Freemium" || tier() === "Premium"}>
            <section class="sp-card">
              <div class="sp-card-header">
                <span class="sp-card-icon">
                  <IconKey />
                </span>
                <h2 class="sp-card-title">Authentication</h2>
              </div>

              <div class="sp-field">
                <label class="sp-label" for="sp-api-key">
                  OpenRouter API Key
                </label>
                <div class="sp-input-group">
                  <input
                    id="sp-api-key"
                    class="sp-input sp-input-key"
                    type={showKey() ? "text" : "password"}
                    value={openRouterKey()}
                    onInput={(e) => setOpenRouterKey(e.currentTarget.value)}
                    placeholder="sk-or-v1-…"
                    autocomplete="off"
                    spellcheck={false}
                  />
                  <button
                    type="button"
                    class="sp-input-action"
                    onClick={() => setShowKey((v) => !v)}
                    title={showKey() ? "Hide key" : "Show key"}
                    aria-label={showKey() ? "Hide API key" : "Show API key"}
                  >
                    <Show when={showKey()} fallback={<IconEye />}>
                      <IconEyeOff />
                    </Show>
                  </button>
                  <button
                    type="button"
                    class={`sp-input-action${copied() ? " sp-input-action--copied" : ""}`}
                    onClick={handleCopyKey}
                    disabled={!openRouterKey()}
                    title="Copy key"
                    aria-label="Copy API key to clipboard"
                  >
                    <Show when={!copied()} fallback={<IconCheckCircle />}>
                      <IconCopy />
                    </Show>
                  </button>
                </div>
                <p class="sp-hint">Stored securely on disk and never transmitted in plaintext.</p>
              </div>
            </section>
          </Show>

          <section class="sp-card sp-card-actions">
            <button
              type="submit"
              class="sp-btn sp-btn-primary"
              disabled={saveStatus() === "saving"}
            >
              <Show
                when={saveStatus() === "saving"}
                fallback={
                  <Show when={saveStatus() === "saved"} fallback={<span>Save Settings</span>}>
                    <span>Saved</span>
                  </Show>
                }
              >
                <span>Saving…</span>
              </Show>
            </button>
            <Show when={saveStatus() === "saved"}>
              <span class="sp-status sp-status-success">
                <span class="status-indicator success"></span>
                <span>Settings saved successfully</span>
              </span>
            </Show>
          </section>
        </form>

        <section class="sp-card">
          <div class="sp-card-header">
            <span class="sp-card-icon">
              <IconSparkles />
            </span>
            <h2 class="sp-card-title">Test AI Connection</h2>
          </div>

          <div class="sp-field">
            <label class="sp-label" for="sp-test-prompt">
              Prompt
            </label>
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
              type="button"
              class="sp-btn sp-btn-secondary"
              onClick={handleTestAi}
              disabled={isTesting()}
            >
              <Show when={isTesting()} fallback={<span>Test AI</span>}>
                <span>Testing…</span>
              </Show>
            </button>
            <Show when={isTesting()}>
              <span class="sp-status sp-status-processing">
                <span class="status-indicator processing"></span>
                <span>Testing AI…</span>
              </span>
            </Show>
          </div>

          <Show when={testResponse()}>
            <div class={`sp-response${testIsError() ? " sp-response-error" : ""}`}>
              <div class="sp-response-label">
                <Show
                  when={!testIsError()}
                  fallback={
                    <>
                      <span class="status-indicator error"></span>
                      <span>Error</span>
                    </>
                  }
                >
                  <span class="status-indicator success"></span>
                  <span>Response</span>
                </Show>
              </div>
              <p class="sp-response-text">{testResponse()}</p>
            </div>
          </Show>
        </section>

        <section class="sp-card">
          <div class="sp-card-header">
            <span class="sp-card-icon">
              <IconBoxes />
            </span>
            <h2 class="sp-card-title">Intent Router</h2>
            <p class="sp-card-desc">Run 10 sample prompts through the AI router scoring logic.</p>
          </div>

          <div class="sp-row">
            <button
              type="button"
              class="sp-btn sp-btn-secondary"
              onClick={handleTestIntents}
              disabled={isScoring()}
            >
              <Show when={isScoring()} fallback={<span>Run Intent Test</span>}>
                <span>Scoring…</span>
              </Show>
            </button>
            <Show when={isScoring()}>
              <span class="sp-status sp-status-processing">
                <span class="status-indicator processing"></span>
                <span>Scoring…</span>
              </span>
            </Show>
          </div>

          <Show when={intentScores().length > 0}>
            <ul class="sp-scores" role="list">
              {intentScores().map(([intent, score]) => (
                <li class="sp-score-item">
                  <span class={`sp-tier-badge sp-tier-${score.toLowerCase()}`}>{score}</span>
                  <span class="sp-score-text">{intent}</span>
                </li>
              ))}
            </ul>
          </Show>
        </section>
      </div>
    </div>
  );
};
