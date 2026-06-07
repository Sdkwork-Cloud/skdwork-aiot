export interface AuthTokens {
    accessToken?: string;
    authToken?: string;
    refreshToken?: string;
    expiresIn?: number;
    expiresAt?: number;
    tokenType?: string;
    scope?: string;
}
export interface TokenManagerEvents {
    onTokenRefresh?: (tokens: AuthTokens) => void;
    onTokenExpired?: () => void;
    onTokenCleared?: () => void;
    onTokenSet?: (tokens: AuthTokens) => void;
    onTokenInvalid?: () => void;
}
export interface AuthTokenManager {
    getAccessToken(): string | undefined;
    getAuthToken(): string | undefined;
    getRefreshToken(): string | undefined;
    getTokens(): AuthTokens;
    setTokens(tokens: AuthTokens): void;
    setAccessToken(token: string): void;
    setAuthToken(token: string): void;
    setRefreshToken(token: string): void;
    clearTokens(): void;
    clearAuthToken(): void;
    clearAccessToken(): void;
    isExpired(): boolean;
    isValid(): boolean;
    hasToken(): boolean;
    hasAuthToken(): boolean;
    hasAccessToken(): boolean;
    willExpireIn(seconds: number): boolean;
}
export type AuthMode = 'apikey' | 'dual-token';
export interface AuthConfig {
    mode: AuthMode;
    apiKey?: string;
    accessToken?: string;
    authToken?: string;
    tokenManager?: AuthTokenManager;
}
export declare class DefaultAuthTokenManager implements AuthTokenManager {
    private tokens;
    private readonly events?;
    constructor(initialTokens?: AuthTokens, events?: TokenManagerEvents);
    getAccessToken(): string | undefined;
    getAuthToken(): string | undefined;
    getRefreshToken(): string | undefined;
    getTokens(): AuthTokens;
    setTokens(tokens: AuthTokens): void;
    setAccessToken(token: string): void;
    setAuthToken(token: string): void;
    setRefreshToken(token: string): void;
    clearTokens(): void;
    clearAuthToken(): void;
    clearAccessToken(): void;
    isExpired(): boolean;
    isValid(): boolean;
    hasToken(): boolean;
    hasAuthToken(): boolean;
    hasAccessToken(): boolean;
    willExpireIn(seconds: number): boolean;
}
export declare function createTokenManager(tokens?: AuthTokens, events?: TokenManagerEvents): AuthTokenManager;
export declare function buildAuthHeaders(authMode: AuthMode, apiKey?: string, tokenManager?: AuthTokenManager): Record<string, string>;
export interface OAuthConfig {
    clientId: string;
    redirectUri: string;
    scope?: string;
    state?: string;
}
export interface OAuthTokens extends AuthTokens {
    tokenType: string;
    scope?: string;
}
export declare function isTokenValid(manager: AuthTokenManager | undefined): boolean;
export declare function requiresRefresh(manager: AuthTokenManager | undefined, thresholdSeconds?: number): boolean;
//# sourceMappingURL=token-manager.d.ts.map