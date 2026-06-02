import { Component, createSignal, onMount, createEffect } from "solid-js";
import { aiState, updateAiConfig, ModelTier, askAi, getHfModels, getOpenRouterFreeModels, testIntentRouter } from "../stores/aiStore";
import "./SettingsPage.css";

export const SettingsPage: Component = () => {
  const [tier, setTier] = createSignal<ModelTier>(aiState.tier);
  const [openRouterKey, setOpenRouterKey] = createSignal(aiState.openrouter_key || "");
  const [ollamaUrl, setOllamaUrl] = createSignal(aiState.ollama_url);
  const [selectedModel, setSelectedModel] = createSignal(aiState.selected_model);
  const [saveStatus, setSaveStatus] = createSignal("");

  const [testPrompt, setTestPrompt] = createSignal("What is the capital of France?");
  const [testResponse, setTestResponse] = createSignal("");
  const [isTesting, setIsTesting] = createSignal(false);

  const [models, setModels] = createSignal<string[]>([]);
  const [isLoadingModels, setIsLoadingModels] = createSignal(false);

  const [intentScores, setIntentScores] = createSignal<[string, string][]>([]);
  const [isScoring, setIsScoring] = createSignal(false);

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
    setSaveStatus("Saving...");
    await updateAiConfig({
      tier: tier(),
      openrouter_key: openRouterKey() || null,
      ollama_url: ollamaUrl(),
      selected_model: selectedModel(),
    });
    setSaveStatus("Saved successfully!");
    setTimeout(() => setSaveStatus(""), 2000);
  };

  const handleTestAi = async () => {
    setIsTesting(true);
    setTestResponse("");
    try {
      const response = await askAi(testPrompt());
      setTestResponse(response);
    } catch (err: any) {
      setTestResponse(`Error: ${err}`);
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
      "Write a local script to rename files."
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

  return (
    <div class="settings-page">
      <div class="settings-container">
        <h1>CNTRL Settings</h1>
        <form onSubmit={handleSave}>
          <div class="form-group">
            <label>AI Tier</label>
            <select
              value={tier()}
              onInput={(e) => setTier(e.currentTarget.value as ModelTier)}
            >
              <option value="Local">Tier 1: Local (Ollama)</option>
              <option value="Freemium">Tier 2: Freemium (OpenRouter Free)</option>
              <option value="Premium">Tier 3: Premium (HF/OpenRouter Pro)</option>
            </select>
          </div>

          <div class="form-group">
            <label>Model</label>
            {tier() === "Local" ? (
              <input
                type="text"
                value={selectedModel()}
                onInput={(e) => setSelectedModel(e.currentTarget.value)}
                placeholder="llama3"
              />
            ) : (
              <select
                value={selectedModel()}
                onInput={(e) => setSelectedModel(e.currentTarget.value)}
                disabled={isLoadingModels()}
              >
                {isLoadingModels() && <option>Loading models...</option>}
                {models().map((m) => (
                  <option value={m}>{m}</option>
                ))}
              </select>
            )}
          </div>

          {tier() === "Local" && (
            <div class="form-group">
              <label>Ollama API URL</label>
              <input
                type="text"
                value={ollamaUrl()}
                onInput={(e) => setOllamaUrl(e.currentTarget.value)}
                placeholder="http://localhost:11434"
              />
            </div>
          )}

          {(tier() === "Freemium" || tier() === "Premium") && (
            <div class="form-group">
              <label>OpenRouter API Key</label>
              <input
                type="password"
                value={openRouterKey()}
                onInput={(e) => setOpenRouterKey(e.currentTarget.value)}
                placeholder="sk-or-..."
              />
            </div>
          )}

          <div class="actions">
            <button type="submit" class="save-btn">Save Settings</button>
            {saveStatus() && <span class="status-msg">{saveStatus()}</span>}
          </div>
        </form>

        <hr class="divider" />
        
        <div class="test-ai-section">
          <h2>Test AI Connection</h2>
          <div class="form-group">
            <input
              type="text"
              value={testPrompt()}
              onInput={(e) => setTestPrompt(e.currentTarget.value)}
              placeholder="Ask the AI a question..."
            />
          </div>
          <button class="test-btn" onClick={handleTestAi} disabled={isTesting()}>
            {isTesting() ? "Testing..." : "Test AI"}
          </button>
          {testResponse() && (
            <div class="response-box">
              <p><strong>Response:</strong></p>
              <p>{testResponse()}</p>
            </div>
          )}
        </div>

        <hr class="divider" />

        <div class="test-ai-section">
          <h2>Test Intent Router</h2>
          <p class="description">Run 10 sample intents through the AI router scoring logic.</p>
          <button class="test-btn" onClick={handleTestIntents} disabled={isScoring()}>
            {isScoring() ? "Scoring..." : "Run Intent Test"}
          </button>
          {intentScores().length > 0 && (
            <div class="response-box scores-box">
              <ul>
                {intentScores().map(([intent, score]) => (
                  <li>
                    <strong>{score}:</strong> {intent}
                  </li>
                ))}
              </ul>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
