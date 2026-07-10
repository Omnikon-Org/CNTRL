/**
 * @module stores/browserStore
 * Reactive browser state managed by a SolidJS store.
 * Types are imported from `../types` — do not redeclare them here.
 */
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { createStore } from "solid-js/store";
import type { Tab } from "../types";

export type { Tab };

export interface BrowserConfig {
  user_agent: string | null;
}

export const [browserState, setBrowserState] = createStore({
  tabs: [] as Tab[],
  activeTabId: null as string | null,
});

listen("tabs-updated", () => {
  void browserActions.fetchTabs();
});

const closedTabsStack: string[] = [];

export const browserActions = {
  async fetchTabs(): Promise<void> {
    const tabs: Tab[] = await invoke<Tab[]>("get_tabs");
    setBrowserState("tabs", tabs);
    if (tabs.length > 0) {
      const activeExists = tabs.some((t) => t.id === browserState.activeTabId);
      if (!activeExists) {
        setBrowserState("activeTabId", tabs[tabs.length - 1]?.id ?? null);
      }
    } else {
      setBrowserState("activeTabId", null);
    }
  },

  async openTab(url: string = "about:blank", isBackground: boolean = false): Promise<string> {
    const id: string = await invoke<string>("open_tab", { url, isBackground });
    await this.fetchTabs();
    if (!isBackground) {
      setBrowserState("activeTabId", id);
    }
    return id;
  },

  async closeTab(id: string): Promise<void> {
    const tab = browserState.tabs.find((t) => t.id === id);
    if (tab !== undefined && tab.url !== "about:blank") {
      closedTabsStack.push(tab.url);
    }
    await invoke<void>("close_tab", { id });
    await this.fetchTabs();
    if (browserState.tabs.length === 0) {
      await this.openTab("about:blank");
    }
  },

  async reopenLastTab(): Promise<void> {
    const url = closedTabsStack.pop();
    if (url !== undefined) {
      await this.openTab(url);
    }
  },

  async navigate(id: string, url: string): Promise<void> {
    await invoke<void>("navigate", { id, url });
    await this.fetchTabs();
  },

  async setActiveTab(id: string): Promise<void> {
    await invoke<void>("set_active_tab", { id });
    setBrowserState("activeTabId", id);
  },

  async fetchFallback(url: string): Promise<string> {
    return invoke<string>("fetch_fallback", { url });
  },

  async goBack(id: string): Promise<void> {
    await invoke<void>("go_back", { id });
  },

  async goForward(id: string): Promise<void> {
    await invoke<void>("go_forward", { id });
  },

  async reload(id: string): Promise<void> {
    await invoke<void>("reload", { id });
  },
  async getBrowserConfig() {
    return await invoke<BrowserConfig>("get_browser_config");
  },

  async updateBrowserConfig(config: BrowserConfig) {
    await invoke("update_browser_config", { config });
  },
};
