/**
 * @module test/dbTypes.test
 * Unit tests for Database shared TypeScript type definitions and store helper functions.
 */
import { describe, expect, it } from "vitest";
import type { BookmarkEntry, HistoryEntry, TabSessionEntry, TabSessionInput } from "../types/db";

describe("Database Types shape", () => {
  it("validates HistoryEntry shape", () => {
    const entry: HistoryEntry = {
      id: 1,
      url: "https://google.com",
      title: "Google",
      visited_at: 1700000000,
    };
    expect(entry.id).toBe(1);
    expect(entry.url).toBe("https://google.com");
    expect(entry.title).toBe("Google");
    expect(entry.visited_at).toBe(1700000000);
  });

  it("validates BookmarkEntry shape", () => {
    const bm: BookmarkEntry = {
      id: 10,
      url: "https://rust-lang.org",
      title: "Rust Language",
      created_at: 1700000050,
    };
    expect(bm.id).toBe(10);
    expect(bm.url).toBe("https://rust-lang.org");
    expect(bm.title).toBe("Rust Language");
  });

  it("validates TabSessionEntry and TabSessionInput shapes", () => {
    const sessionInput: TabSessionInput = {
      url: "https://news.ycombinator.com",
      position: 0,
      is_active: true,
      title: "Hacker News",
    };
    const sessionEntry: TabSessionEntry = {
      id: 100,
      ...sessionInput,
    };
    expect(sessionEntry.id).toBe(100);
    expect(sessionEntry.is_active).toBe(true);
    expect(sessionEntry.position).toBe(0);
  });
});
