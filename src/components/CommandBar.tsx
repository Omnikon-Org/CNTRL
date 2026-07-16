import { Component, createSignal, onCleanup, onMount, For, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { marked } from "marked";
import DOMPurify from "dompurify";
import "./CommandBar.css";
import { SparklesIcon } from "./Icons";

interface StepStatusEvent {
  step_index: number;
  total_steps: number;
  status: "Pending" | "Running" | "Done" | "Failed";
  result_markdown: string | null;
}

interface StepState {
  status: "Pending" | "Running" | "Done" | "Failed";
  result: string | null;
}

export const CommandBar: Component = () => {
  const [isOpen, setIsOpen] = createSignal(false);
  const [input, setInput] = createSignal("");
  const [steps, setSteps] = createSignal<StepState[]>([]);
  const [isProcessing, setIsProcessing] = createSignal(false);
  const [isPrivacyEnabled, setIsPrivacyEnabled] = createSignal(false);
  let inputRef: HTMLInputElement | undefined;
  let unlisten: UnlistenFn | undefined;

  const checkPrivacyMode = async () => {
    try {
      const enabled = await invoke<boolean>("is_privacy_mode_enabled");
      setIsPrivacyEnabled(enabled);
    } catch (err) {
      console.error("Failed to fetch privacy mode status:", err);
    }
  };

  onMount(async () => {
    const handleGlobalKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        setIsOpen(true);
        void checkPrivacyMode();
        // Focus is handled by an effect or small timeout because of the DOM render tick
        setTimeout(() => inputRef?.focus(), 50);
      } else if (e.key === "Escape" && isOpen()) {
        e.preventDefault();
        setIsOpen(false);
      }
    };
    window.addEventListener("keydown", handleGlobalKeyDown);
    void checkPrivacyMode();


    unlisten = await listen<StepStatusEvent>("intent://step-status", (event) => {
      const payload = event.payload;
      setSteps((prev) => {
        const newSteps = [...prev];
        // Ensure array is large enough
        while (newSteps.length <= payload.step_index) {
          newSteps.push({ status: "Pending", result: null });
        }
        newSteps[payload.step_index] = {
          status: payload.status,
          result: payload.result_markdown,
        };
        return newSteps;
      });

      if (payload.step_index === payload.total_steps - 1 && (payload.status === "Done" || payload.status === "Failed")) {
        setIsProcessing(false);
      }
    });

    onCleanup(() => {
      window.removeEventListener("keydown", handleGlobalKeyDown);
      if (unlisten) unlisten();
    });
  });

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    const query = input().trim();
    if (!query || isProcessing()) return;

    setIsProcessing(true);
    setSteps([]); // reset previous steps

    try {
      await invoke("submit_intent", { input: query });
    } catch (err) {
      console.error(err);
      setSteps([{ status: "Failed", result: String(err) }]);
      setIsProcessing(false);
    }
  };

  const renderMarkdown = (markdown: string) => {
    try {
      const html = marked(markdown) as string;
      return DOMPurify.sanitize(html);
    } catch {
      return DOMPurify.sanitize(markdown);
    }
  };

  return (
    <Show when={isOpen()}>
      <div class="cmd-bar-overlay" onClick={(e) => { if (e.target === e.currentTarget) setIsOpen(false); }}>
        <div class={`cmd-bar ${isPrivacyEnabled() ? "privacy-active" : ""}`}>
          <form class="cmd-bar-input-wrapper" onSubmit={handleSubmit}>
            <SparklesIcon />
            <input
              ref={inputRef}
              class="cmd-bar-input"
              type="text"
              placeholder="What do you want to do? (e.g. 'go to github', 'bitcoin price')"
              value={input()}
              onInput={(e) => setInput(e.currentTarget.value)}
              disabled={isProcessing()}
              autocomplete="off"
              spellcheck={false}
            />
            <Show when={isPrivacyEnabled()}>
              <span class="cmd-privacy-badge" title="Privacy mode active: remote AI blocked.">Privacy Active</span>
            </Show>
          </form>

          <Show when={steps().length > 0}>
            <div class="cmd-bar-results">
              <For each={steps()}>
                {(step, idx) => (
                  <div class="cmd-step">
                    <div class="cmd-step-header">
                      <span>Step {idx() + 1}</span>
                      <span class={`cmd-step-status ${step.status.toLowerCase()}`}>
                        {step.status}
                      </span>
                    </div>
                    <Show when={step.result}>
                      <div
                        class="cmd-step-result"
                        innerHTML={renderMarkdown(step.result!)}
                      />
                    </Show>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </div>
      </div>
    </Show>
  );
};
