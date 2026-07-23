/**
 * @module stores/dbStore
 * Store for managing SQLite persisted bookmarks and history.
 */
import { invoke } from "@tauri-apps/api/core";
import { createStore } from "solid-js/store";
import type { BookmarkEntry, HistoryEntry } from "../types";

export const [dbState, setDbState] = createStore({
  history: [] as HistoryEntry[],
  bookmarks: [] as BookmarkEntry[],
  isLoadingHistory: false,
  isLoadingBookmarks: false,
});

export const dbActions = {
  async fetchHistory(limit: number = 100, offset: number = 0): Promise<HistoryEntry[]> {
    setDbState("isLoadingHistory", true);
    try {
      const entries = await invoke<HistoryEntry[]>("db_get_history", { limit, offset });
      setDbState("history", entries);
      return entries;
    } finally {
      setDbState("isLoadingHistory", false);
    }
  },

  async addHistoryEntry(url: string, title?: string): Promise<HistoryEntry> {
    const entry = await invoke<HistoryEntry>("db_add_history_entry", { url, title: title ?? null });
    setDbState("history", (prev) => [entry, ...prev]);
    return entry;
  },

  async fetchBookmarks(): Promise<BookmarkEntry[]> {
    setDbState("isLoadingBookmarks", true);
    try {
      const entries = await invoke<BookmarkEntry[]>("db_get_bookmarks");
      setDbState("bookmarks", entries);
      return entries;
    } finally {
      setDbState("isLoadingBookmarks", false);
    }
  },

  async addBookmark(url: string, title?: string): Promise<BookmarkEntry> {
    const entry = await invoke<BookmarkEntry>("db_add_bookmark", { url, title: title ?? null });
    await this.fetchBookmarks();
    return entry;
  },

  async removeBookmark(id: number): Promise<void> {
    await invoke<void>("db_remove_bookmark", { id });
    setDbState("bookmarks", (prev) => prev.filter((b) => b.id !== id));
  },
};
