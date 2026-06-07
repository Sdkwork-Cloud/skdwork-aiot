const require_types = require("../core/types.cjs");
//#region src/utils/cache.ts
var MemoryCacheStore = class {
	cache = /* @__PURE__ */ new Map();
	maxSize;
	defaultTtl;
	constructor(config = {}) {
		this.maxSize = config.maxSize ?? require_types.DEFAULT_CACHE_CONFIG.maxSize;
		this.defaultTtl = config.ttl ?? require_types.DEFAULT_CACHE_CONFIG.ttl;
	}
	get(key) {
		const entry = this.cache.get(key);
		if (!entry) return null;
		if (Date.now() > entry.expiresAt) {
			this.cache.delete(key);
			return null;
		}
		return entry.value;
	}
	set(key, value, ttl) {
		if (this.cache.size >= this.maxSize) this.evictOldest();
		const expiresAt = Date.now() + (ttl ?? this.defaultTtl);
		this.cache.set(key, {
			value,
			expiresAt
		});
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
		for (const [key, entry] of this.cache) if (entry.expiresAt < oldestTime) {
			oldestTime = entry.expiresAt;
			oldestKey = key;
		}
		if (oldestKey) this.cache.delete(oldestKey);
	}
};
function createCacheStore(config) {
	return new MemoryCacheStore(config);
}
function generateCacheKey(config) {
	return [
		config.method,
		config.url,
		JSON.stringify(config.params ?? {}),
		JSON.stringify(config.body ?? {})
	].join(":");
}
//#endregion
exports.MemoryCacheStore = MemoryCacheStore;
exports.createCacheStore = createCacheStore;
exports.generateCacheKey = generateCacheKey;

//# sourceMappingURL=cache.cjs.map