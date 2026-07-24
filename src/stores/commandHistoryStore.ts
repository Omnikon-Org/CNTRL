/**
 * Command history store for the CommandBar.
 * Persists recent commands to localStorage and provides fuzzy search
 * against both history and built-in local command suggestions.
 */

import { createStore } from "solid-js/store";

const STORAGE_KEY = "cntrl:command-history";
const MAX_HISTORY = 50;

export interface Suggestion {
  /** Display label */
  label: string;
  /** Subtitle / description shown below the label */
  subtitle?: string;
  /** Category badge shown on the right */
  category: "history" | "navigation" | "search" | "system" | "macro";
}

// ---------- Built-in static suggestions ----------

const STATIC_SUGGESTIONS: Suggestion[] = [
  // Navigation shortcuts
  { label: "go to github.com", subtitle: "Navigate to GitHub", category: "navigation" },
  { label: "go to google.com", subtitle: "Navigate to Google", category: "navigation" },
  { label: "go to settings", subtitle: "Open CNTRL settings", category: "navigation" },
  { label: "open settings", subtitle: "Open CNTRL settings", category: "navigation" },
  // Search
  { label: "search for ...", subtitle: "Search on Google", category: "search" },
  { label: "google ...", subtitle: "Search on Google", category: "search" },
  // System commands
  { label: "bitcoin price", subtitle: "Fetch live BTC/USD price", category: "system" },
  { label: "take screenshot", subtitle: "Capture screen to clipboard", category: "system" },
  { label: "mute volume", subtitle: "Mute system audio", category: "system" },
  // Macro
  { label: "run macro ...", subtitle: "Execute a saved macro", category: "macro" },
  { label: "trigger macro ...", subtitle: "Execute a saved macro", category: "macro" },
];

// ---------- Fuzzy scoring ----------

/**
 * Returns a relevance score (higher = better match) for a query against a target string.
 * Uses a simple character-subsequence approach so "gthb" matches "go to github.com".
 */
export function fuzzyScore(query: string, target: string): number {
  if (!query) return 0;
  const q = query.toLowerCase();
  const t = target.toLowerCase();

  // Exact prefix match is best
  if (t.startsWith(q)) return 100 + (100 - t.length);

  // Substring match
  if (t.includes(q)) return 60 + (100 - t.length);

  // Character subsequence match
  let qi = 0;
  let bonus = 0;
  for (let ti = 0; ti < t.length && qi < q.length; ti++) {
    if (t[ti] === q[qi]) {
      qi++;
      // Reward consecutive matches
      bonus += ti === 0 || t[ti - 1] === " " ? 3 : 1;
    }
  }
  if (qi === q.length) return bonus;

  return 0;
}

// ---------- Store ----------

interface CommandHistoryState {
  history: string[];
}

function loadHistory(): string[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) return JSON.parse(raw) as string[];
  } catch {
    // ignore
  }
  return [];
}

function saveHistory(history: string[]): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(history));
  } catch {
    // ignore
  }
}

const [historyState, setHistoryState] = createStore<CommandHistoryState>({
  history: loadHistory(),
});

export const commandHistoryStore = {
  get history() {
    return historyState.history;
  },

  /** Record a submitted command at the front of the history list. */
  push(command: string): void {
    const trimmed = command.trim();
    if (!trimmed) return;
    const deduped = historyState.history.filter((h) => h !== trimmed);
    const updated = [trimmed, ...deduped].slice(0, MAX_HISTORY);
    setHistoryState("history", updated);
    saveHistory(updated);
  },

  /** Clear all saved history. */
  clear(): void {
    setHistoryState("history", []);
    saveHistory([]);
  },

  /**
   * Return ordered suggestions for a query.
   * When query is empty, shows the most recent history entries.
   * When query is present, fuzzy-matches against both history and static suggestions.
   */
  getSuggestions(query: string, limit = 8): Suggestion[] {
    const trimmed = query.trim();

    if (!trimmed) {
      // Show recent history when bar is empty
      return historyState.history.slice(0, limit).map((h) => ({
        label: h,
        category: "history" as const,
      }));
    }

    const scored: Array<{ suggestion: Suggestion; score: number }> = [];

    // Score history entries
    for (const h of historyState.history) {
      const score = fuzzyScore(trimmed, h);
      if (score > 0) {
        scored.push({ suggestion: { label: h, category: "history" }, score: score + 10 }); // +10 bias for personal history
      }
    }

    // Score static suggestions
    for (const s of STATIC_SUGGESTIONS) {
      const score = fuzzyScore(trimmed, s.label);
      if (score > 0) {
        scored.push({ suggestion: s, score });
      }
    }

    // Deduplicate (history wins over static for same label)
    const seen = new Set<string>();
    const unique: Array<{ suggestion: Suggestion; score: number }> = [];
    for (const item of scored.sort((a, b) => b.score - a.score)) {
      if (!seen.has(item.suggestion.label)) {
        seen.add(item.suggestion.label);
        unique.push(item);
      }
    }

    return unique.slice(0, limit).map((i) => i.suggestion);
  },
};
