import { DEFAULT_CACHE_CONFIG } from "../core/types.js";
class MemoryCacheStore {
  cache = /* @__PURE__ */ new Map();
  maxSize;
  defaultTtl;
  constructor(config = {}) {
    this.maxSize = config.maxSize ?? DEFAULT_CACHE_CONFIG.maxSize;
    this.defaultTtl = config.ttl ?? DEFAULT_CACHE_CONFIG.ttl;
  }
  get(key) {
    const entry = this.cache.get(key);
    if (!entry) {
      return null;
    }
    if (Date.now() > entry.expiresAt) {
      this.cache.delete(key);
      return null;
    }
    return entry.value;
  }
  set(key, value, ttl) {
    if (this.cache.size >= this.maxSize) {
      this.evictOldest();
    }
    const expiresAt = Date.now() + (ttl ?? this.defaultTtl);
    this.cache.set(key, { value, expiresAt });
  }
  has(key) {
    const entry = this.cache.get(key);
    if (!entry) return false;
    if (Date.now() > entry.expiresAt) {
      this.cache.delete(key);
      return false;
    }
    return true;
  }
  delete(key) {
    return this.cache.delete(key);
  }
  clear() {
    this.cache.clear();
  }
  size() {
    return this.cache.size;
  }
  evictOldest() {
    let oldestKey = null;
    let oldestTime = Infinity;
    for (const [key, entry] of this.cache) {
      if (entry.expiresAt < oldestTime) {
        oldestTime = entry.expiresAt;
        oldestKey = key;
      }
    }
    if (oldestKey) {
      this.cache.delete(oldestKey);
    }
  }
}
function createCacheStore(config) {
  return new MemoryCacheStore(config);
}
function generateCacheKey(config) {
  const parts = [
    config.method,
    config.url,
    JSON.stringify(config.params ?? {}),
    JSON.stringify(config.body ?? {})
  ];
  return parts.join(":");
}
export {
  MemoryCacheStore,
  createCacheStore,
  generateCacheKey
};
//# sourceMappingURL=cache.js.map
