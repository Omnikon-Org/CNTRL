/**
 * @module types/browser
 * Shared browser-domain TypeScript types that mirror the Rust `Tab` struct
 * and `BrowserState`. All frontend code must import from here — do not
 * redeclare these shapes inline.
 */

/**
 * Mirrors `src-tauri/src/services/browser.rs :: Tab`.
 * Every field must stay in sync with the Rust serde serialisation.
 */
export interface Tab {
  /** UUID v4 assigned by the Rust backend on tab creation. */
  readonly id: string;
  /** Current URL of the tab. May be `"about:blank"` or a `cntrl://` internal URL. */
  url: string;
  /** Page title, updated after the page loads. Defaults to `"New Tab"`. */
  title: string;
  /** URL of the favicon image, or undefined if not yet resolved. */
  favicon?: string | undefined;
  /**
   * When `true` the tab is managed in the background and never shown in the
   * tab bar. Background tabs can still be navigated and their DOM content
   * can be fetched via the fallback pathway.
   */
  is_background: boolean;
  /** ISO-8601 timestamp of when the tab was created (set by the Rust backend). */
  created_at: string;
  /**
   * When `true` the native webview failed to load the URL and the frontend
   * should render the sandboxed iframe fallback instead.
   */
  fallback_mode: boolean;
  /**
   * Becomes `true` after the page-load callback fires in the Rust backend.
   * Used to drive the loading indicator in `UrlBar`.
   */
  loaded: boolean;
}

/** Reactive browser state shape used by the SolidJS store. */
export interface BrowserState {
  tabs: Tab[];
  activeTabId: string | null;
}
