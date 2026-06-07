import type { ApiResult } from '../core/types';
export type ErrorCode = 'UNKNOWN' | 'NETWORK_ERROR' | 'TIMEOUT' | 'CANCELLED' | 'UNAUTHORIZED' | 'FORBIDDEN' | 'NOT_FOUND' | 'VALIDATION_ERROR' | 'RATE_LIMIT' | 'SERVER_ERROR' | 'TOKEN_EXPIRED' | 'TOKEN_INVALID' | 'BUSINESS_ERROR' | 'CONFLICT' | 'SERVICE_UNAVAILABLE' | 'BAD_GATEWAY' | 'GATEWAY_TIMEOUT';
export interface ErrorDetail {
    field?: string;
    message?: string;
    value?: unknown;
    code?: string;
    constraint?: string;
}
export interface ErrorOptions {
    cause?: Error;
    details?: ErrorDetail[];
    traceId?: string;
    metadata?: Record<string, unknown>;
}
export declare class SdkError extends Error {
    readonly code: ErrorCode;
    readonly httpStatus?: number;
    readonly details?: ErrorDetail[];
    readonly timestamp: number;
    readonly traceId?: string;
    readonly metadata?: Record<string, unknown>;
    constructor(message: string, code?: ErrorCode, httpStatus?: number, options?: ErrorOptions);
    static fromApiResult(result: ApiResult, httpStatus?: number): SdkError;
    static fromHttpStatus(status: number, message?: string): SdkError;
    toJSON(): Record<string, unknown>;
    toString(): string;
    isRetryable(): boolean;
    isAuthError(): boolean;
    isNetworkError(): boolean;
    isClientError(): boolean;
    isServerError(): boolean;
}
export declare class NetworkError extends SdkError {
    constructor(message?: string, options?: ErrorOptions);
}
export declare class TimeoutError extends SdkError {
    readonly timeout?: number;
    constructor(message?: string, timeout?: number, options?: ErrorOptions);
    toJSON(): Record<string, unknown>;
}
export declare class CancelledError extends SdkError {
    constructor(message?: string, options?: ErrorOptions);
}
export declare class AuthenticationError extends SdkError {
    constructor(message?: string, options?: ErrorOptions);
}
export declare class TokenExpiredError extends AuthenticationError {
    constructor(message?: string, options?: ErrorOptions);
}
export declare class TokenInvalidError extends AuthenticationError {
    constructor(message?: string, options?: ErrorOptions);
}
export declare class ForbiddenError extends SdkError {
    constructor(message?: string, options?: ErrorOptions);
}
export declare class NotFoundError extends SdkError {
    constructor(message?: string, options?: ErrorOptions);
}
export declare class ValidationError extends SdkError {
    constructor(message?: string, details?: ErrorDetail[], options?: ErrorOptions);
}
export declare class ConflictError extends SdkError {
    constructor(message?: string, options?: ErrorOptions);
}
export declare class MethodNotAllowedError extends SdkError {
    readonly allowedMethods?: string[];
    constructor(message?: string, allowedMethods?: string[], options?: ErrorOptions);
}
export declare class RateLimitError extends SdkError {
    readonly retryAfter?: number;
    constructor(message?: string, retryAfter?: number, options?: ErrorOptions);
    toJSON(): Record<string, unknown>;
}
export declare class ServerError extends SdkError {
    constructor(message?: string, httpStatus?: number, options?: ErrorOptions);
}
export declare class BadGatewayError extends ServerError {
    constructor(message?: string, options?: ErrorOptions);
}
export declare class ServiceUnavailableError extends ServerError {
    constructor(message?: string, options?: ErrorOptions);
}
export declare class GatewayTimeoutError extends ServerError {
    constructor(message?: string, options?: ErrorOptions);
}
export declare class BusinessError extends SdkError {
    readonly businessCode?: string | number;
    readonly data?: unknown;
    constructor(message: string, code?: string | number, data?: unknown, options?: ErrorOptions);
    toJSON(): Record<string, unknown>;
}
export declare function isSdkError(error: unknown): error is SdkError;
export declare function isNetworkError(error: unknown): error is NetworkError;
export declare function isTimeoutError(error: unknown): error is TimeoutError;
export declare function isCancelledError(error: unknown): error is CancelledError;
export declare function isAuthError(error: unknown): error is AuthenticationError;
export declare function isValidationError(error: unknown): error is ValidationError;
export declare function isRateLimitError(error: unknown): error is RateLimitError;
export declare function isServerError(error: unknown): error is ServerError;
export declare function isBusinessError(error: unknown): error is BusinessError;
export declare function isRetryableError(error: unknown): boolean;
//# sourceMappingURL=index.d.ts.map