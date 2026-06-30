/**
 * Tab bar component for managing browser tabs.
 * @module TabBar
 */
import { Component, For, createSignal, onMount } from 'solid-js';
import { platform } from '@tauri-apps/plugin-os';
import { browserState, browserActions } from '../stores/browserStore';
import { WindowControls } from './WindowControls';
import './TabBar.css';

export const TabBar: Component = () => {
  const [isMacOS, setIsMacOS] = createSignal(navigator.userAgent.includes('Mac OS'));
  const [isWindows, setIsWindows] = createSignal(navigator.userAgent.includes('Win'));
  const [closingTabId, setClosingTabId] = createSignal<string | null>(null);

  onMount(() => {
    // Relying on userAgent for initial render, then verify with platform()
    void platform().then((p) => {
      setIsMacOS(p === 'macos');
      setIsWindows(p === 'windows');
    }).catch(console.error);
  });

  const handleNewTab = () => {
    void browserActions.openTab('about:blank');
  };

  return (
    <div 
      class="tab-bar" 
      data-tauri-drag-region 
      style={isMacOS() ? { 'padding-left': '80px' } : {}}
    >
      <For each={browserState.tabs.filter(t => !t.is_background)}>
        {(tab) => (
          <div
            class={`tab ${browserState.activeTabId === tab.id ? 'active' : ''} ${closingTabId() === tab.id ? 'closing' : ''}`}
            onClick={() => void browserActions.setActiveTab(tab.id)}
          >
            <div class="tab-content">
              {tab.favicon && (
                <img src={tab.favicon} class="favicon" alt="" />
              )}
              <span class="title">{tab.title}</span>
            </div>
            <button
              class="close-btn"
              onClick={(e) => {
                e.stopPropagation();
                setClosingTabId(tab.id);
                setTimeout(() => {
                  void browserActions.closeTab(tab.id);
                  if (closingTabId() === tab.id) setClosingTabId(null);
                }, 200);
              }}
            >
              ×
            </button>
          </div>
        )}
      </For>
      <button class="new-tab-btn" onClick={handleNewTab} title="New Tab">+</button>
      {isWindows() && (
        <div style="margin-left: auto; display: flex; align-items: center;">
          <WindowControls />
        </div>
      )}
    </div>
  );
};
