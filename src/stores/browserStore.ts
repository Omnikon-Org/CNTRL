import { createStore } from 'solid-js/store';
import { invoke } from '@tauri-apps/api/core';

export interface Tab {
  id: string;
  url: string;
  title: string;
  favicon?: string;
  is_background: boolean;
  created_at: string;
  fallback_mode: boolean;
}

export interface BrowserState {
  tabs: Tab[];
  activeTabId: string | null;
  bounds: { x: number; y: number; width: number; height: number };
}

export const [browserState, setBrowserState] = createStore<BrowserState>({
  tabs: [],
  activeTabId: null,
  bounds: { x: 0, y: 0, width: 800, height: 600 },
});

import { listen } from '@tauri-apps/api/event';
void listen('tabs-updated', () => {
  void browserActions.fetchTabs();
});

export const browserActions = {
  async fetchTabs() {
    const tabs: Tab[] = await invoke('get_tabs');
    setBrowserState('tabs', tabs);
    const activeTabId: string | null = await invoke('get_active_tab_id');
    if (activeTabId) {
      setBrowserState('activeTabId', activeTabId);
    } else if (tabs.length > 0) {
      const activeExists = tabs.some(t => t.id === browserState.activeTabId);
      if (!activeExists) {
        setBrowserState('activeTabId', tabs[tabs.length - 1]?.id || null);
      }
    } else {
      setBrowserState('activeTabId', null);
    }
  },

  async openTab(url: string = 'about:blank', isBackground: boolean = false) {
    const { x, y, width, height } = browserState.bounds;
    const id: string = await invoke('open_tab', { url, isBackground, x, y, width, height });
    await this.fetchTabs();
    if (!isBackground) {
      await this.setActiveTab(id);
    }
    return id;
  },

  async closeTab(id: string) {
    await invoke('close_tab', { id });
    await this.fetchTabs();
    if (browserState.tabs.length === 0) {
      await this.openTab('about:blank');
    }
  },

  async navigate(id: string, url: string) {
    await invoke('navigate', { id, url });
    await this.fetchTabs();
  },

  async setActiveTab(id: string) {
    await invoke('set_active_tab', { id });
    setBrowserState('activeTabId', id);
  },

  async fetchFallback(url: string) {
    return await invoke<string>('fetch_fallback', { url });
  },
  
  async goBack(id: string) {
    await invoke('go_back', { id });
  },

  async goForward(id: string) {
    await invoke('go_forward', { id });
  },

  async reload(id: string) {
    await invoke('reload', { id });
  }
};
