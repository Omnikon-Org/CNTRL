/**
 * Browser state store using SolidJS store.
 * Manages tab state, navigation actions, and IPC calls to the Tauri backend.
 * @module stores/browserStore
 */
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { createStore } from "solid-js/store";
import { eventBus } from "../core/events";
import type { Tab } from "../types";

export type { Tab };

export interface BrowserConfig {
  user_agent: string | null;
}

export const [browserState, setBrowserState] = createStore({
  tabs: [] as Tab[],
  activeTabId: null as string | null,
});

// Sync tab state when backend updates tabs
void listen("tabs-updated", () => {
  void browserActions.fetchTabs();
});

const closedTabsStack: string[] = [];

export const browserActions = {
  /**
   * Fetch latest tab list from Rust service.
   */
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

  /**
   * Open a new tab with optional URL and background mode.
   */
  async openTab(url: string = "about:blank", isBackground: boolean = false): Promise<string> {
    const id: string = await invoke<string>("open_tab", { url, isBackground });
    await this.fetchTabs();
    if (!isBackground) {
      setBrowserState("activeTabId", id);
    }
    return id;
  },

  /**
   * Close a tab by ID.
   */
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

  /**
   * Reopen the last closed tab.
   */
  async reopenLastTab(): Promise<void> {
    const url = closedTabsStack.pop();
    if (url !== undefined) {
      await this.openTab(url);
    }
  },

  /**
   * Navigate active tab to a URL.
   */
  async navigate(id: string, url: string): Promise<void> {
    await invoke<void>("navigate", { id, url });
    await this.fetchTabs();
  },

  /**
   * Switch active tab.
   */
  async setActiveTab(id: string): Promise<void> {
    await invoke<void>("set_active_tab", { id });
    setBrowserState("activeTabId", id);
  },

  /**
   * Fetch fallback HTML for sandboxed rendering.
   */
  async fetchFallback(url: string): Promise<string> {
    return invoke<string>("fetch_fallback", { url });
  },

  /**
   * Navigate back in history.
   */
  async goBack(id: string): Promise<void> {
    await invoke<void>("go_back", { id });
  },

  /**
   * Navigate forward in history.
   */
  async goForward(id: string): Promise<void> {
    await invoke<void>("go_forward", { id });
  },

  /**
   * Reload current page.
   */
  async reload(id: string): Promise<void> {
    await invoke<void>("reload", { id });
  },

  /**
   * Get browser configuration.
   */
  async getBrowserConfig(): Promise<BrowserConfig> {
    return await invoke<BrowserConfig>("get_browser_config");
  },

  /**
   * Update browser configuration.
   */
  async updateBrowserConfig(config: BrowserConfig): Promise<void> {
    await invoke("update_browser_config", { config });
  },
};

// Event Bus Subscriptions
eventBus.on("TAB_OPEN_NEW", (payload: { url: string; isBackground?: boolean }) => {
  void browserActions.openTab(payload.url, payload.isBackground);
});

eventBus.on("TAB_CLOSE_ACTIVE", () => {
  if (browserState.activeTabId) {
    void browserActions.closeTab(browserState.activeTabId);
  }
});

eventBus.on("TAB_REOPEN_LAST", () => {
  void browserActions.reopenLastTab();
});
