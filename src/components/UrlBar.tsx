import { open } from "@tauri-apps/plugin-shell";
import { Component, createEffect, createSignal, onCleanup, onMount } from "solid-js";
import { browserActions, browserState } from "../stores/browserStore";
import "./UrlBar.css";

import { AlertIcon, BackIcon, ForwardIcon, LockIcon, ReloadIcon, SettingsIcon, StopIcon } from "./Icons";

export const UrlBar: Component = () => {
  const [inputUrl, setInputUrl] = createSignal("");
  let inputRef: HTMLInputElement | undefined;

  createEffect(() => {
    const activeTab = browserState.tabs.find((t) => t.id === browserState.activeTabId);
    if (activeTab) {
      setInputUrl(activeTab.url);
    } else {
      setInputUrl("");
    }
  });

  onMount(() => {
    const handler = (e: KeyboardEvent) => {
      const activeElement = document.activeElement;
      if (
        activeElement instanceof HTMLInputElement ||
        activeElement instanceof HTMLTextAreaElement
      ) {
        return;
      }

      if ((e.metaKey || e.ctrlKey) && e.key === "l") {
        e.preventDefault();
        inputRef?.focus();
        inputRef?.select();
        return;
      }

      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "r") {
        e.preventDefault();
        handleReload();
        return;
      }

      if ((e.metaKey && e.key === "[") || (e.altKey && e.key === "ArrowLeft")) {
        e.preventDefault();
        handleBack();
        return;
      }

      if ((e.metaKey && e.key === "]") || (e.altKey && e.key === "ArrowRight")) {
        e.preventDefault();
        handleForward();
        return;
      }
    }; 
    window.addEventListener("keydown", handler);
    onCleanup(() => window.removeEventListener("keydown", handler));
  });

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Enter") {
      let url = inputUrl().trim();

      if (!url) return;

      if (
        !url.startsWith("http://") &&
        !url.startsWith("https://") &&
        !url.startsWith("cntrl://")
      ) {
        url = `https://${url}`;
      }

      try {
        const parsed = new URL(url);
        parsed.hostname = parsed.hostname.toLowerCase();
        url = parsed.toString();
      } catch {
        // non-standard protocols like cntrl:// — leave url as-is
      }

      setInputUrl(url);

      if (browserState.activeTabId) {
        browserActions.navigate(browserState.activeTabId, url);
      } else {
        browserActions.openTab(url);
      }
    }
  };

  const handleOpenSettings = () => {
    if (browserState.activeTabId) {
      browserActions.navigate(browserState.activeTabId, "cntrl://settings");
    } else {
      browserActions.openTab("cntrl://settings");
    }
  };
  const handleOpenExternal = async () => {
    const url = activeTab()?.url;
    if (url) {
      await open(url);
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

  const isHttps = () => inputUrl().startsWith("https://");
  const isLoading = () => {
    const tab = browserState.tabs.find((t) => t.id === browserState.activeTabId);
    return tab && !tab.loaded && tab.url !== "about:blank" && !tab.url.startsWith("cntrl://");
  };

  const activeTab = () => browserState.tabs.find((t) => t.id === browserState.activeTabId);

  return (
    <div class="url-bar-container">
      <div class="nav-buttons">
        <button class="nav-btn" onClick={handleBack} title="Back" aria-label="Go back">
          <BackIcon />
        </button>
        <button class="nav-btn" onClick={handleForward} title="Forward" aria-label="Go forward">
          <ForwardIcon />
        </button>
        <button class="nav-btn" onClick={handleReload} title={isLoading() ? "Stop" : "Reload"} aria-label={isLoading() ? "Stop loading" : "Reload page"}>
          {isLoading() ? <StopIcon /> : <ReloadIcon />}
        </button>
      </div>
      <div class="url-bar">
        <span class="icon">{isHttps() ? <LockIcon /> : <AlertIcon />}</span>
        <input
          ref={inputRef}
          aria-label="Address bar"
          type="text"
          value={inputUrl() || ""}
          onInput={(e) => setInputUrl(e.target.value)}
          onKeyDown={handleKeyDown}
          onFocus={(e) => e.currentTarget.select()}
          class="url-input"
          placeholder="Enter URL or intent… (⌘L to focus)"
          autocomplete="off"
          spellcheck={false}
          autocorrect="off"
          autocapitalize="off"
        />
        {activeTab()?.fallback_mode && (
          <span class="fallback-badge">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="10"
              height="10"
              viewBox="0 0 24 24"
              fill="currentColor"
              stroke="none"
            >
              <path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z" />
            </svg>
            compat mode
          </span>
        )}
        <button class="nav-btn" onClick={handleOpenExternal} title="Open in External Browser">
          <span>Open</span>
        </button>
        <button class="settings-icon-btn" onClick={handleOpenSettings} title="Settings" aria-label="Open settings">
          <SettingsIcon />
        </button>
      </div>
    </div>
  );
};
