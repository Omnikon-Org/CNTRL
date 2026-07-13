import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { LRUCache } from "./cache";

describe("LRUCache", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("should store and retrieve values", () => {
    const cache = new LRUCache<string, string>(3);
    cache.set("a", "1");
    expect(cache.get("a")).toBe("1");
  });

  it("should evict least recently used item when capacity is exceeded", () => {
    const cache = new LRUCache<string, string>(2);
    cache.set("a", "1");
    cache.set("b", "2");
    cache.set("c", "3"); // 'a' should be evicted

    expect(cache.get("a")).toBeUndefined();
    expect(cache.get("b")).toBe("2");
    expect(cache.get("c")).toBe("3");
  });

  it("should update LRU status on get", () => {
    const cache = new LRUCache<string, string>(2);
    cache.set("a", "1");
    cache.set("b", "2");
    
    // access 'a', making it recently used
    cache.get("a");
    
    // 'b' is now least recently used
    cache.set("c", "3"); 

    expect(cache.get("b")).toBeUndefined();
    expect(cache.get("a")).toBe("1");
    expect(cache.get("c")).toBe("3");
  });

  it("should respect TTL expiration", () => {
    const cache = new LRUCache<string, string>(5, 1000); // 1 sec TTL
    cache.set("a", "1");
    
    expect(cache.get("a")).toBe("1");
    
    // Advance time by 1.1 seconds
    vi.advanceTimersByTime(1100);
    
    expect(cache.get("a")).toBeUndefined();
  });

  it("should override default TTL if specified in set", () => {
    const cache = new LRUCache<string, string>(5, 1000); // 1 sec default
    cache.set("a", "1", 5000); // 5 sec specific TTL
    
    vi.advanceTimersByTime(2000);
    expect(cache.get("a")).toBe("1"); // Still valid
    
    vi.advanceTimersByTime(4000);
    expect(cache.get("a")).toBeUndefined(); // Now expired
  });

  it("should invalidate keys correctly", () => {
    const cache = new LRUCache<string, string>(5);
    cache.set("a", "1");
    expect(cache.get("a")).toBe("1");
    
    cache.invalidate("a");
    expect(cache.get("a")).toBeUndefined();
  });
  
  it("should clear the cache correctly", () => {
    const cache = new LRUCache<string, string>(5);
    cache.set("a", "1");
    cache.set("b", "2");
    cache.clear();
    
    expect(cache.get("a")).toBeUndefined();
    expect(cache.get("b")).toBeUndefined();
  });
});
