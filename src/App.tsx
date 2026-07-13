import { onCleanup, onMount } from "solid-js";
import { TabBar } from "./components/TabBar";
import { UrlBar } from "./components/UrlBar";
import { WebView } from "./components/WebView";
import { initAiStore } from "./stores/aiStore";
import { browserActions, browserState } from "./stores/browserStore";
import { CommandBar } from "./components/CommandBar";
import { eventBus } from "./core/events";
import "./App.css";

function App() {
  onMount(async () => {
    await initAiStore();
    await browserActions.fetchTabs();
    if (browserState.tabs.length === 0) {
      await browserActions.openTab("https://google.com");
    }

    const handler = (e: KeyboardEvent) => {
      if (!(e.metaKey || e.ctrlKey)) return;

      if (e.key === "t") {
        e.preventDefault();
        eventBus.emit("TAB_OPEN", "about:blank");
      } else if (e.key === "w") {
        e.preventDefault();
        eventBus.emit("TAB_CLOSE_ACTIVE");
      }
    };

    window.addEventListener("keydown", handler);
    onCleanup(() => window.removeEventListener("keydown", handler));
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
