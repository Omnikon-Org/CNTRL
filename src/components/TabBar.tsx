import { Component, For, createSignal, onMount } from 'solid-js';
import { platform } from '@tauri-apps/plugin-os';
import { browserState, browserActions } from '../stores/browserStore';
import { WindowControls } from './WindowControls';
import './TabBar.css';

export const TabBar: Component = () => {
  const [isMacOS, setIsMacOS] = createSignal(false);
  const [isWindows, setIsWindows] = createSignal(false);

  onMount(async () => {
    const p = await platform();
    setIsMacOS(p === 'macos');
    setIsWindows(p === 'windows');
  });

  const handleNewTab = () => {
    browserActions.openTab('about:blank');
  };

  return (
    <div 
      class="tab-bar" 
      data-tauri-drag-region 
      style={isMacOS() ? { 'padding-left': '72px' } : {}}
    >
      <For each={browserState.tabs.filter(t => !t.is_background)}>
        {(tab) => (
          <div
            class={`tab ${browserState.activeTabId === tab.id ? 'active' : ''}`}
            onClick={() => browserActions.setActiveTab(tab.id)}
            title={tab.title}
          >
            <div class="tab-content">
              {tab.favicon && (
                <img src={tab.favicon} class="favicon" alt="" />
              )}
              <span class="title">{tab.title}</span>
            </div>
            <button
              class="close-btn"
              aria-label={`Close tab: ${tab.title}`}
              onClick={(e) => {
                e.stopPropagation();
                browserActions.closeTab(tab.id);
              }}
            >
              ×
            </button>
          </div>
        )}
      </For>
      <button class="new-tab-btn" onClick={handleNewTab} title="New Tab" aria-label="Open new tab">+</button>
      {isWindows() && (
        <div style="margin-left: auto; display: flex; align-items: center;">
          <WindowControls />
        </div>
      )}
    </div>
  );
};
