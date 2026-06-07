import type { HttpClientConfig, RequestConfig, RequestOptions, QueryParams, HttpHeaders, Interceptors, UploadOptions, DownloadOptions } from '../core/types';
import type { AuthTokenManager, AuthMode } from '../auth';
import { type Logger } from '../utils/logger';
import { type CacheStore } from '../utils/cache';
export interface HttpClientOptions extends HttpClientConfig {
    apiKey?: string;
    accessToken?: string;
    authToken?: string;
    tokenManager?: AuthTokenManager;
}
export interface HttpClientAuthConfig {
    authMode: AuthMode;
    apiKey?: string;
    tokenManager?: AuthTokenManager;
}
export interface RequestExecutor {
    execute<T>(config: RequestConfig): Promise<T>;
}
export interface ResponseProcessor {
    process<T>(response: Response, config: RequestConfig): Promise<T>;
}
export interface UrlBuilder {
    build(path: string, params?: QueryParams): string;
}
export interface HeaderBuilder {
    build(config: RequestConfig, skipAuth?: boolean): HttpHeaders;
}
export declare abstract class BaseHttpClient implements RequestExecutor {
    protected config: Required<Omit<HttpClientConfig, 'interceptors'>> & {
        baseUrl: string;
    };
    protected authConfig: HttpClientAuthConfig;
    protected logger: Logger;
    protected cache: CacheStore;
    protected interceptors: Interceptors;
    protected tenantId?: string;
    protected organizationId?: string;
    protected platform?: string;
    protected userId?: string | number;
    constructor(config: HttpClientOptions);
    protected determineAuthMode(config: HttpClientOptions): AuthMode;
    getAuthMode(): AuthMode;
    setAuthMode(mode: AuthMode): void;
    getTokenManager(): AuthTokenManager | undefined;
    setTokenManager(manager: AuthTokenManager): void;
    setApiKey(apiKey: string): void;
    setAuthToken(token: string): void;
    setAccessToken(token: string): void;
    setTenantId(tenantId: string): void;
    setOrganizationId(organizationId: string): void;
    setPlatform(platform: string): void;
    setUserId(userId: string | number): void;
    clearAuthToken(): void;
    addRequestInterceptor(interceptor: (config: RequestConfig) => RequestConfig | Promise<RequestConfig>): () => void;
    addResponseInterceptor(interceptor: (response: unknown, config: RequestConfig) => unknown | Promise<unknown>): () => void;
    addErrorInterceptor(interceptor: (error: Error, config: RequestConfig) => void | Promise<void>): () => void;
    clearCache(): void;
    getConfig(): {
        baseUrl: string;
        timeout: number;
        authMode: AuthMode;
        apiKey: string | undefined;
        accessToken: string | undefined;
        authToken: string | undefined;
        tenantId: string | undefined;
        organizationId: string | undefined;
        platform: string | undefined;
        userId: string | number | undefined;
    };
    isAuthenticated(): boolean;
    protected buildBaseUrl(path: string, params?: QueryParams): string;
    protected buildHeaders(config: RequestConfig, skipAuth?: boolean): HttpHeaders;
    protected serializeRequestBody(body: unknown, headers: HttpHeaders): BodyInit | string | undefined;
    protected applyRequestInterceptors(config: RequestConfig): Promise<RequestConfig>;
    protected applyResponseInterceptors<T>(response: T, config: RequestConfig): Promise<T>;
    protected applyErrorInterceptors(error: Error, config: RequestConfig): Promise<void>;
    protected handleErrorResponse(response: Response, config: RequestConfig): Promise<never>;
    protected processResponse<T>(response: Response, config: RequestConfig): Promise<T>;
    protected executeFetch(url: string, options: {
        method: string;
        headers: HttpHeaders;
        body?: string | BodyInit | null;
        timeout: number;
        signal?: AbortSignal;
    }): Promise<Response>;
    execute<T>(config: RequestConfig): Promise<T>;
    abstract request<T>(path: string, options?: RequestOptions): Promise<T>;
    abstract get<T>(path: string, params?: QueryParams): Promise<T>;
    abstract post<T>(path: string, body?: unknown): Promise<T>;
    abstract put<T>(path: string, body?: unknown): Promise<T>;
    abstract delete<T>(path: string, body?: unknown): Promise<T>;
    abstract patch<T>(path: string, body?: unknown): Promise<T>;
    upload<T>(path: string, options: UploadOptions): Promise<T>;
    download(path: string, _options?: DownloadOptions): Promise<Blob>;
    stream(path: string, options?: RequestOptions): AsyncIterable<string>;
}
export declare function createBaseHttpClient(config: HttpClientOptions): BaseHttpClient;
//# sourceMappingURL=base-client.d.ts.map