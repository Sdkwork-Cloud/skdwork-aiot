import type { RetryConfig } from '../core/types';
import { DEFAULT_RETRY_CONFIG } from '../core/types';
export { DEFAULT_RETRY_CONFIG };
export declare function sleep(ms: number): Promise<void>;
export declare function calculateDelay(attempt: number, baseDelay: number, backoff: RetryConfig['retryBackoff'], maxDelay: number): number;
export declare function shouldRetry(error: Error, attempt: number, config: RetryConfig): boolean;
export declare function withRetry<T>(fn: () => Promise<T>, config?: Partial<RetryConfig>): Promise<T>;
export declare function createRetryConfig(config?: Partial<RetryConfig>): RetryConfig;
//# sourceMappingURL=retry.d.ts.map