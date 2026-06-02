import { Component, createEffect, createSignal, onMount, onCleanup } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';
import { browserState, browserActions } from '../stores/browserStore';
import { SettingsPage } from './SettingsPage';
import './WebView.css';

export const WebView: Component = () => {
  const [htmlContent, setHtmlContent] = createSignal('');
  const [isLoading, setIsLoading] = createSignal(false);
  const [error, setError] = createSignal('');
  // eslint-disable-next-line no-unassigned-vars
  let containerRef: HTMLDivElement | undefined;
  let resizeObserver: ResizeObserver | undefined;

  const updateBounds = () => {
    if (containerRef) {
      requestAnimationFrame(() => {
        if (!containerRef) return;
        const rect = containerRef.getBoundingClientRect();
        if (rect.width === 0 || rect.height === 0) return;
        
        invoke('update_tab_bounds', {
          x: rect.x,
          y: rect.y,
          width: rect.width,
          height: rect.height,
        }).catch(console.error);
      });
    }
  };

  onMount(() => {
    if (containerRef) {
      resizeObserver = new ResizeObserver(() => {
        updateBounds();
      });
      resizeObserver.observe(containerRef);
      updateBounds();
    }
    window.addEventListener('resize', updateBounds);
  });

  onCleanup(() => {
    if (resizeObserver) resizeObserver.disconnect();
    window.removeEventListener('resize', updateBounds);
  });

  createEffect(() => {
    const activeTab = browserState.tabs.find(t => t.id === browserState.activeTabId);
    updateBounds(); // Sync bounds when tab changes

    if (!activeTab || activeTab.url === 'about:blank') {
      setHtmlContent('');
      return;
    }

    if (activeTab.fallback_mode) {
      setIsLoading(true);
      setError('');
      browserActions.fetchFallback(activeTab.url).then(html => {
        setHtmlContent(html);
        setIsLoading(false);
      }).catch(err => {
        console.error(err);
        setError(`Failed to load ${activeTab.url}`);
        setIsLoading(false);
      });
    } else {
      setHtmlContent('');
    }
  });

  const activeTab = () => browserState.tabs.find(t => t.id === browserState.activeTabId);

  return (
    <div class="webview-container" ref={containerRef}>
      {activeTab()?.url === 'cntrl://settings' && <SettingsPage />}
      
      {activeTab()?.fallback_mode && activeTab()?.url !== 'cntrl://settings' && (
        <>
          {isLoading() && <div class="loading">Loading compatibility mode...</div>}
          {error() && <div class="error">{error()}</div>}
          {!isLoading() && !error() && htmlContent() && (
            <iframe
              class="sandbox-frame"
              srcdoc={htmlContent()}
              sandbox="allow-scripts allow-forms"
            ></iframe>
          )}
        </>
      )}
      {!activeTab()?.fallback_mode && !activeTab() && (
        <div class="empty-state">
          <h1>CNTRL BROWSER</h1>
          <p>Intent-based autonomous browsing</p>
        </div>
      )}
    </div>
  );
};
