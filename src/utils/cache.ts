/**
 * @module utils/cache
 * Generic Caching Interface and In-Memory LRU Implementation
 */

export interface ICache<K, V> {
  get(key: K): V | undefined;
  set(key: K, value: V, ttlMs?: number): void;
  invalidate(key: K): void;
  clear(): void;
}

interface CacheEntry<V> {
  value: V;
  expiry: number | null;
}

export class LRUCache<K, V> implements ICache<K, V> {
  private capacity: number;
  private cache: Map<K, CacheEntry<V>>;
  private defaultTtlMs: number | null;

  constructor(capacity: number, defaultTtlMs: number | null = null) {
    if (capacity <= 0) {
      throw new Error("Capacity must be greater than 0");
    }
    this.capacity = capacity;
    this.cache = new Map();
    this.defaultTtlMs = defaultTtlMs;
  }

  get(key: K): V | undefined {
    if (!this.cache.has(key)) {
      return undefined;
    }

    const entry = this.cache.get(key)!;
    
    // Check TTL
    if (entry.expiry !== null && Date.now() > entry.expiry) {
      this.cache.delete(key);
      return undefined;
    }

    // Refresh position for LRU
    this.cache.delete(key);
    this.cache.set(key, entry);

    return entry.value;
  }

  set(key: K, value: V, ttlMs?: number): void {
    const finalTtl = ttlMs ?? this.defaultTtlMs;
    const expiry = finalTtl ? Date.now() + finalTtl : null;

    if (this.cache.has(key)) {
      this.cache.delete(key);
    } else if (this.cache.size >= this.capacity) {
      // Evict least recently used (first item in Map)
      const firstKey = this.cache.keys().next().value;
      if (firstKey !== undefined) {
        this.cache.delete(firstKey);
      }
    }

    this.cache.set(key, { value, expiry });
  }

  invalidate(key: K): void {
    this.cache.delete(key);
  }

  clear(): void {
    this.cache.clear();
  }
}
