import type { CacheConfig, RequestConfig } from '../core/types';
export interface CacheStore {
    get<T>(key: string): T | null;
    set<T>(key: string, value: T, ttl?: number): void;
    has(key: string): boolean;
    delete(key: string): boolean;
    clear(): void;
    size(): number;
}
export declare class MemoryCacheStore implements CacheStore {
    private cache;
    private maxSize;
    private defaultTtl;
    constructor(config?: Partial<CacheConfig>);
    get<T>(key: string): T | null;
    set<T>(key: string, value: T, ttl?: number): void;
    has(key: string): boolean;
    delete(key: string): boolean;
    clear(): void;
    size(): number;
    private evictOldest;
}
export declare function createCacheStore(config?: Partial<CacheConfig>): CacheStore;
export declare function generateCacheKey(config: RequestConfig): string;
//# sourceMappingURL=cache.d.ts.map