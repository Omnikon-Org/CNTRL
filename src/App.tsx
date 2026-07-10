import { listen } from "@tauri-apps/api/event";
import { onCleanup, onMount } from "solid-js";
import { TabBar } from "./components/TabBar";
import { UrlBar } from "./components/UrlBar";
import { WebView } from "./components/WebView";
import { initAiStore } from "./stores/aiStore";
import { browserActions, browserState } from "./stores/browserStore";
import "./App.css";

function App() {
  onMount(async () => {
    await initAiStore();
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
        browserActions.openTab("about:blank");
      } else if (e.key === "T" && e.shiftKey) {
        e.preventDefault();
        browserActions.reopenLastTab();
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
    </div>
  );
}

export default App;
