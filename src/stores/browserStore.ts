import { invoke } from "@tauri-apps/api/core";
import { createStore } from "solid-js/store";

export interface Tab {
  id: string;
  url: string;
  title: string;
  favicon?: string;
  is_background: boolean;
  created_at: string;
  fallback_mode: boolean;
  loaded: boolean;
}

export const [browserState, setBrowserState] = createStore({
  tabs: [] as Tab[],
  activeTabId: null as string | null,
});

import { listen } from "@tauri-apps/api/event";

listen("tabs-updated", () => {
  browserActions.fetchTabs();
});

const closedTabsStack: string[] = [];

export const browserActions = {
  async fetchTabs() {
    const tabs: Tab[] = await invoke("get_tabs");
    setBrowserState("tabs", tabs);
    if (tabs.length > 0) {
      const activeExists = tabs.some((t) => t.id === browserState.activeTabId);
      if (!activeExists) {
        setBrowserState("activeTabId", tabs[tabs.length - 1]?.id || null);
      }
    } else {
      setBrowserState("activeTabId", null);
    }
  },

  async openTab(url: string = "about:blank", isBackground: boolean = false) {
    const id: string = await invoke("open_tab", { url, isBackground });
    await this.fetchTabs();
    if (!isBackground) {
      setBrowserState("activeTabId", id);
    }
    return id;
  },

  async closeTab(id: string) {
    const tab = browserState.tabs.find((t) => t.id === id);
    if (tab && tab.url !== "about:blank") {
      closedTabsStack.push(tab.url);
    }
    await invoke("close_tab", { id });
    await this.fetchTabs();
    if (browserState.tabs.length === 0) {
      await this.openTab("about:blank");
    }
  },

  async reopenLastTab() {
    const url = closedTabsStack.pop();
    if (url) {
      await this.openTab(url);
    }
  },

  async navigate(id: string, url: string) {
    await invoke("navigate", { id, url });
    await this.fetchTabs();
  },

  async setActiveTab(id: string) {
    await invoke("set_active_tab", { id });
    setBrowserState("activeTabId", id);
  },

  async fetchFallback(url: string) {
    return await invoke<string>("fetch_fallback", { url });
  },

  async goBack(id: string) {
    await invoke("go_back", { id });
  },

  async goForward(id: string) {
    await invoke("go_forward", { id });
  },

  async reload(id: string) {
    await invoke("reload", { id });
  },
};
