import { Component, createEffect, createSignal } from 'solid-js';
import { browserState, browserActions } from '../stores/browserStore';
import './UrlBar.css';

const BackIcon = () => (
  <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m15 18-6-6 6-6"/></svg>
);

const ForwardIcon = () => (
  <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m9 18 6-6-6-6"/></svg>
);

const ReloadIcon = () => (
  <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8"/><path d="M21 3v5h-5"/></svg>
);

const SettingsIcon = () => (
  <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
);

export const UrlBar: Component = () => {
  const [inputUrl, setInputUrl] = createSignal('');

  // Sync with active tab
  createEffect(() => {
    const activeTab = browserState.tabs.find(t => t.id === browserState.activeTabId);
    if (activeTab) {
      setInputUrl(activeTab.url);
    } else {
      setInputUrl('');
    }
  });

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Enter') {
      let url = inputUrl();
      if (!url.startsWith('http://') && !url.startsWith('https://') && !url.startsWith('vibe://')) {
        url = `https://${url}`;
      }
      
      if (browserState.activeTabId) {
        browserActions.navigate(browserState.activeTabId, url);
      } else {
        browserActions.openTab(url);
      }
    }
  };

  const handleOpenSettings = () => {
    if (browserState.activeTabId) {
      browserActions.navigate(browserState.activeTabId, 'vibe://settings');
    } else {
      browserActions.openTab('vibe://settings');
    }
  };

  const handleBack = () => {
    if (browserState.activeTabId) browserActions.goBack(browserState.activeTabId);
  };

  const handleForward = () => {
    if (browserState.activeTabId) browserActions.goForward(browserState.activeTabId);
  };

  const handleReload = () => {
    if (browserState.activeTabId) browserActions.reload(browserState.activeTabId);
  };

  const isHttps = () => inputUrl().startsWith('https://');

  const activeTab = () => browserState.tabs.find(t => t.id === browserState.activeTabId);

  return (
    <div class="url-bar-container">
      <div class="nav-buttons">
        <button class="nav-btn" onClick={handleBack} title="Back">
          <BackIcon />
        </button>
        <button class="nav-btn" onClick={handleForward} title="Forward">
          <ForwardIcon />
        </button>
        <button class="nav-btn" onClick={handleReload} title="Reload">
          <ReloadIcon />
        </button>
      </div>
      <div class="url-bar">
        <span class="icon">{isHttps() ? '🔒' : '⚠️'}</span>
        <input
          type="text"
          value={inputUrl()}
          onInput={(e) => setInputUrl(e.target.value)}
          onKeyDown={handleKeyDown}
          class="url-input"
          placeholder="Enter URL or search"
        />
        {activeTab()?.fallback_mode && (
          <span class="fallback-badge" style="color: var(--color-accent); font-size: 0.8rem; margin-left: 8px; font-weight: bold; white-space: nowrap;">
            ⚡ compat mode
          </span>
        )}
        <button class="settings-icon-btn" onClick={handleOpenSettings} title="Settings">
          <SettingsIcon />
        </button>
      </div>
    </div>
  );
};
