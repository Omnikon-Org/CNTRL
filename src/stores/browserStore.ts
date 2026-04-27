import { createStore } from 'solid-js/store';
import { invoke } from '@tauri-apps/api/core';

export interface Tab {
  id: string;
  url: string;
  title: string;
  is_background: boolean;
  created_at: string;
  fallback_mode: boolean;
}

export const [browserState, setBrowserState] = createStore({
  tabs: [] as Tab[],
  activeTabId: null as string | null,
});

import { listen } from '@tauri-apps/api/event';
listen('tabs-updated', () => {
  browserActions.fetchTabs();
});

export const browserActions = {
  async fetchTabs() {
    const tabs: Tab[] = await invoke('get_tabs');
    setBrowserState('tabs', tabs);
    if (!browserState.activeTabId && tabs.length > 0) {
      setBrowserState('activeTabId', tabs[0].id);
    }
  },

  async openTab(url: string = 'about:blank', isBackground: boolean = false) {
    const id: string = await invoke('open_tab', { url, isBackground });
    await this.fetchTabs();
    if (!isBackground) {
      setBrowserState('activeTabId', id);
    }
    return id;
  },

  async closeTab(id: string) {
    await invoke('close_tab', { id });
    await this.fetchTabs();
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
