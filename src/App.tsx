import { listen } from "@tauri-apps/api/event";
import { onCleanup, onMount, createSignal, createEffect, Show } from "solid-js";
import { TabBar } from "./components/TabBar";
import { UrlBar } from "./components/UrlBar";
import { WebView } from "./components/WebView";
import { initAiStore } from "./stores/aiStore";
import { browserActions, browserState } from "./stores/browserStore";
import { CommandBar } from "./components/CommandBar";
import { eventBus } from "./core/events";
import { MacroLibrary } from "./components/MacroLibrary";
import { macroState, macroActions } from "./stores/macroStore";
import "./App.css";

function App() {
  const [showMacroLibrary, setShowMacroLibrary] = createSignal(false);
  const [theme, setTheme] = createSignal<"dark" | "light">("dark");

  createEffect(() => {
    document.documentElement.setAttribute("data-theme", theme());
  });

  onMount(async () => {
    await initAiStore();
    await macroActions.init();
    await browserActions.fetchTabs();
    if (browserState.tabs.length === 0) {
      await browserActions.openTab("https://google.com");
    }

    const unlistenCmdW = await listen<null>("cmd-w", () => {
      if (browserState.activeTabId) {
        browserActions.closeTab(browserState.activeTabId);
      }
    });

    const handler = (e: KeyboardEvent) => {
      if (!(e.metaKey || e.ctrlKey)) return;

      if (e.key === "t" && !e.shiftKey) {
        e.preventDefault();
        eventBus.emit("TAB_OPEN", "about:blank");
      } else if (e.key === "w") {
        e.preventDefault();
        eventBus.emit("TAB_CLOSE_ACTIVE");
        browserActions.openTab("about:blank");
      } else if (e.key === "T" && e.shiftKey) {
        e.preventDefault();
        browserActions.reopenLastTab();
      } else if (e.key === "m") {
        e.preventDefault();
        setShowMacroLibrary((prev) => !prev);
      } else if (e.key === "L" && e.shiftKey) {
        // Cmd+Shift+L to toggle light/dark theme
        e.preventDefault();
        setTheme((prev) => (prev === "dark" ? "light" : "dark"));
      }
    };

    window.addEventListener("keydown", handler);
    onCleanup(() => {
      unlistenCmdW();
      window.removeEventListener("keydown", handler);
    });
  });

  return (
    <div class="app-container">
      <TabBar />
      <UrlBar />
      <WebView />
      <CommandBar />

      <Show when={macroState.isRecording}>
        <div class="macro-recording-badge" onClick={() => setShowMacroLibrary(true)} style="cursor: pointer; pointer-events: auto;">
          <span class="recording-dot">●</span> RECORDING MACRO
        </div>
      </Show>

      <Show when={showMacroLibrary()}>
        <MacroLibrary onClose={() => setShowMacroLibrary(false)} />
      </Show>
    </div>
  );
}

export default App;
