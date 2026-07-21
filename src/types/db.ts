/**
 * @module types/db
 * Database entity types for SQLite persistence.
 * Mirrors Rust data models in `src-tauri/src/services/db.rs`.
 */

export interface HistoryEntry {
  id: number;
  url: string;
  title?: string | null;
  visited_at: number;
}

export interface BookmarkEntry {
  id: number;
  url: string;
  title?: string | null;
  created_at: number;
}

export interface TabSessionEntry {
  id: number;
  url: string;
  position: number;
  is_active: boolean;
  title?: string | null;
}

export interface TabSessionInput {
  url: string;
  position: number;
  is_active: boolean;
  title?: string | null;
}
