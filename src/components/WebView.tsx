import { Component, createEffect, createSignal, onMount } from 'solid-js';
import { browserState, browserActions } from '../stores/browserStore';
import './WebView.css';

export const WebView: Component = () => {
  const [htmlContent, setHtmlContent] = createSignal('');
  const [isLoading, setIsLoading] = createSignal(false);
  const [error, setError] = createSignal('');

  createEffect(() => {
    const activeTab = browserState.tabs.find(t => t.id === browserState.activeTabId);
    if (!activeTab || activeTab.url === 'about:blank') {
      setHtmlContent('');
      return;
    }

    // Try iframe first. If it fails due to CSP/X-Frame-Options, we trigger fallback.
    // In a real Tauri v2 app with tauri-plugin-webview, we would use the Webview window API natively.
    // For this prototype, we simulate webview loading, and if we encounter a problem (simulated here via cross-origin),
    // we use the headless playwright fallback.
    
    setIsLoading(true);
    setError('');

    // Native iframe cross-origin requests often fail to load modern sites (like google.com, youtube.com)
    // We immediately trigger the fallback engine to fetch HTML content securely without CORS restrictions.
    browserActions.fetchFallback(activeTab.url).then(html => {
      setHtmlContent(html);
      setIsLoading(false);
    }).catch(err => {
      console.error(err);
      setError(`Failed to load ${activeTab.url}`);
      setIsLoading(false);
    });
  });

  return (
    <div class="webview-container">
      {isLoading() && <div class="loading">Loading...</div>}
      {error() && <div class="error">{error()}</div>}
      {!isLoading() && !error() && htmlContent() && (
        <iframe
          class="sandbox-frame"
          srcdoc={htmlContent()}
          sandbox="allow-scripts allow-forms"
        ></iframe>
      )}
      {!isLoading() && !error() && !htmlContent() && (
        <div class="empty-state">
          <h1>VIBE BROWSER</h1>
          <p>Intent-based autonomous browsing</p>
        </div>
      )}
    </div>
  );
};
