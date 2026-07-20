import {
  Component,
  For,
  Show,
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  onMount,
} from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { marked } from "marked";
import DOMPurify from "dompurify";
import { SparklesIcon } from "./Icons";
import { macroState } from "../stores/macroStore";
import { commandHistoryStore, type Suggestion } from "../stores/commandHistoryStore";
import "./CommandBar.css";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export const CommandBar: Component = () => {
  // -- State -----------------------------------------------------------------
  const [isOpen, setIsOpen] = createSignal(false);
  const [input, setInput] = createSignal("");
  const [steps, setSteps] = createSignal<StepState[]>([]);
  const [isProcessing, setIsProcessing] = createSignal(false);
  const [isPrivacyEnabled, setIsPrivacyEnabled] = createSignal(false);

  /** Index of the currently highlighted suggestion (-1 = none / input has focus) */
  const [activeIndex, setActiveIndex] = createSignal(-1);

  let inputRef: HTMLInputElement | undefined;
  let listRef: HTMLUListElement | undefined;
  let unlisten: UnlistenFn | undefined;

  // -- Derived ---------------------------------------------------------------

  const suggestions = createMemo<Suggestion[]>(() =>
    // Only show suggestions when no results are displayed yet
    steps().length === 0 ? commandHistoryStore.getSuggestions(input()) : [],
  );

  // Reset active index whenever suggestions change
  createEffect(() => {
    suggestions(); // subscribe
    setActiveIndex(-1);
  });

  // Scroll highlighted item into view
  createEffect(() => {
    const idx = activeIndex();
    if (idx >= 0 && listRef) {
      const item = listRef.children[idx] as HTMLElement | undefined;
      item?.scrollIntoView({ block: "nearest" });
    }
  });

  // -- Privacy ---------------------------------------------------------------

  const checkPrivacyMode = async () => {
    try {
      const enabled = await invoke<boolean>("is_privacy_mode_enabled");
      setIsPrivacyEnabled(enabled);
    } catch (err) {
      console.error("Failed to fetch privacy mode status:", err);
    }
  };

  // -- Open / Close ----------------------------------------------------------

  const open = () => {
    setIsOpen(true);
    setInput("");
    setSteps([]);
    setActiveIndex(-1);
    void checkPrivacyMode();
    // Wait one tick for the DOM to mount before focusing
    setTimeout(() => inputRef?.focus(), 50);
  };

  const close = () => {
    setIsOpen(false);
    setInput("");
    setSteps([]);
    setActiveIndex(-1);
    setIsProcessing(false);
  };

  // -- Submit ----------------------------------------------------------------

  const submitCommand = async (query: string) => {
    const trimmed = query.trim();
    if (!trimmed || isProcessing()) return;

    setInput(trimmed);
    setIsProcessing(true);
    setSteps([]);
    commandHistoryStore.push(trimmed);

    try {
      await invoke("submit_intent", { input: trimmed });

      // Phase 6: capture intent for macro recording
      if (macroState.isRecording) {
        try {
          await invoke("capture_intent", { intent: trimmed });
        } catch (captureErr) {
          console.error("Failed to capture intent for macro:", captureErr);
        }
      }
    } catch (err) {
      console.error(err);
      setSteps([{ status: "Failed", result: String(err) }]);
      setIsProcessing(false);
    }
  };

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    await submitCommand(input());
  };

  // -- Keyboard navigation ---------------------------------------------------

  const handleInputKeyDown = (e: KeyboardEvent) => {
    const list = suggestions();

    switch (e.key) {
      case "ArrowDown": {
        e.preventDefault();
        if (list.length === 0) return;
        setActiveIndex((prev) => (prev + 1) % list.length);
        break;
      }
      case "ArrowUp": {
        e.preventDefault();
        if (list.length === 0) return;
        setActiveIndex((prev) => (prev <= 0 ? list.length - 1 : prev - 1));
        break;
      }
      case "Enter": {
        const idx = activeIndex();
        if (idx >= 0 && list[idx]) {
          e.preventDefault();
          void submitCommand(list[idx].label);
        }
        // If no suggestion is highlighted, let the form's onSubmit handle it
        break;
      }
      case "Tab": {
        // Auto-complete first suggestion into the input
        if (list.length > 0) {
          e.preventDefault();
          const pickIdx = activeIndex() >= 0 ? activeIndex() : 0;
          const pick = list[pickIdx];
          if (pick) {
            setInput(pick.label);
            setActiveIndex(-1);
          }
        }
        break;
      }
      case "Escape": {
        e.preventDefault();
        close();
        break;
      }
    }
  };

  // -- Event listeners -------------------------------------------------------

  onMount(async () => {
    const handleGlobalKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        if (isOpen()) {
          close();
        } else {
          open();
        }
      } else if (e.key === "Escape" && isOpen()) {
        e.preventDefault();
        close();
      }
    };
    window.addEventListener("keydown", handleGlobalKeyDown);

    void checkPrivacyMode();

    unlisten = await listen<StepStatusEvent>("intent://step-status", (event) => {
      const payload = event.payload;
      setSteps((prev) => {
        const newSteps = [...prev];
        while (newSteps.length <= payload.step_index) {
          newSteps.push({ status: "Pending", result: null });
        }
        newSteps[payload.step_index] = {
          status: payload.status,
          result: payload.result_markdown,
        };
        return newSteps;
      });

      const isLast = payload.step_index === payload.total_steps - 1;
      if (isLast && (payload.status === "Done" || payload.status === "Failed")) {
        setIsProcessing(false);
      }
    });

    onCleanup(() => {
      window.removeEventListener("keydown", handleGlobalKeyDown);
      if (unlisten) unlisten();
    });
  });

  // -- Helpers ---------------------------------------------------------------

  const renderMarkdown = (markdown: string): string => {
    try {
      return DOMPurify.sanitize(marked(markdown) as string);
    } catch {
      return DOMPurify.sanitize(markdown);
    }
  };

  const categoryLabel: Record<Suggestion["category"], string> = {
    history: "history",
    navigation: "nav",
    search: "search",
    system: "system",
    macro: "macro",
  };

  // -- Render ----------------------------------------------------------------

  return (
    <Show when={isOpen()}>
      {/* Backdrop */}
      <div
        class="cmd-overlay"
        role="dialog"
        aria-modal="true"
        aria-label="Command bar"
        onClick={(e) => {
          if (e.target === e.currentTarget) close();
        }}
      >
        {/* Modal panel */}
        <div class={`cmd-panel${isPrivacyEnabled() ? " privacy-active" : ""}`}>

          {/* Input row */}
          <form class="cmd-input-row" onSubmit={handleSubmit} autocomplete="off">
            <span class="cmd-icon" aria-hidden="true">
              <SparklesIcon />
            </span>
            <input
              ref={inputRef}
              id="cmd-bar-input"
              class="cmd-input"
              type="text"
              role="combobox"
              aria-autocomplete="list"
              aria-controls="cmd-suggestions"
              aria-expanded={suggestions().length > 0}
              aria-activedescendant={
                activeIndex() >= 0 ? `cmd-suggestion-${activeIndex()}` : undefined
              }
              placeholder="What do you want to do? (e.g. 'go to github', 'bitcoin price')"
              value={input()}
              onInput={(e) => setInput(e.currentTarget.value)}
              onKeyDown={handleInputKeyDown}
              disabled={isProcessing()}
              autocomplete="off"
              spellcheck={false}
            />
            <Show when={isPrivacyEnabled()}>
              <span class="cmd-privacy-badge" title="Privacy mode active: remote AI is blocked.">
                Privacy
              </span>
            </Show>
            <kbd class="cmd-hint" aria-hidden="true">ESC</kbd>
          </form>

          {/* Suggestions list (only shown when no results yet) */}
          <Show when={suggestions().length > 0 && steps().length === 0}>
            <ul
              ref={listRef}
              id="cmd-suggestions"
              class="cmd-suggestions"
              role="listbox"
              aria-label="Command suggestions"
            >
              <For each={suggestions()}>
                {(suggestion, idx) => (
                  <li
                    id={`cmd-suggestion-${idx()}`}
                    class={`cmd-suggestion-item${activeIndex() === idx() ? " active" : ""}`}
                    role="option"
                    aria-selected={activeIndex() === idx()}
                    onMouseEnter={() => setActiveIndex(idx())}
                    onMouseLeave={() => setActiveIndex(-1)}
                    onClick={() => void submitCommand(suggestion.label)}
                  >
                    <span class="cmd-suggestion-label">{suggestion.label}</span>
                    <span class="cmd-suggestion-meta">
                      {suggestion.subtitle && (
                        <span class="cmd-suggestion-subtitle">{suggestion.subtitle}</span>
                      )}
                      <span class={`cmd-suggestion-category cmd-category-${suggestion.category}`}>
                        {categoryLabel[suggestion.category]}
                      </span>
                    </span>
                  </li>
                )}
              </For>
            </ul>
          </Show>

          {/* Results / Steps */}
          <Show when={steps().length > 0}>
            <div class="cmd-results" role="log" aria-live="polite" aria-label="Command results">
              <For each={steps()}>
                {(step, idx) => (
                  <div class={`cmd-step cmd-step-${step.status.toLowerCase()}`}>
                    <div class="cmd-step-header">
                      <span class="cmd-step-number">Step {idx() + 1}</span>
                      <span class={`cmd-step-status ${step.status.toLowerCase()}`}>
                        {step.status === "Running" && (
                          <span class="cmd-spinner" aria-hidden="true" />
                        )}
                        {step.status}
                      </span>
                    </div>
                    <Show when={step.result}>
                      <div
                        class="cmd-step-result"
                        // biome-ignore lint/security/noDangerouslySetInnerHtml: sanitised by DOMPurify
                        innerHTML={renderMarkdown(step.result!)}
                      />
                    </Show>
                  </div>
                )}
              </For>
            </div>
          </Show>

          {/* Footer bar */}
          <div class="cmd-footer" aria-hidden="true">
            <span><kbd>↑↓</kbd> navigate</span>
            <span><kbd>↵</kbd> select</span>
            <span><kbd>Tab</kbd> complete</span>
            <span><kbd>Esc</kbd> close</span>
          </div>
        </div>
      </div>
    </Show>
  );
};
