export interface Tab {
  readonly id: string;
  
  url: string;
  title: string;
  favicon?: string | undefined;
  is_background: boolean;
  created_at: string;
  fallback_mode: boolean;
  loaded: boolean;
}
export interface BrowserState {
  tabs: Tab[];
  activeTabId: string | null;
}