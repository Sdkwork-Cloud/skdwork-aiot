import { HTTP_STATUS } from "./core/types.js";
class SdkError extends Error {
  code;
  httpStatus;
  details;
  timestamp;
  traceId;
  metadata;
  constructor(message, code = "UNKNOWN", httpStatus, options) {
    super(message, { cause: options?.cause });
    this.name = this.constructor.name;
    this.code = code;
    this.httpStatus = httpStatus;
    this.details = options?.details;
    this.timestamp = Date.now();
    this.traceId = options?.traceId;
    this.metadata = options?.metadata;
    Object.setPrototypeOf(this, new.target.prototype);
  }
  static fromApiResult(result, httpStatus) {
    const code = String(result.code);
    const message = result.msg || result.message || "Unknown error";
    switch (code) {
      case "400":
      case "4000":
        return new ValidationError(message);
      case "401":
      case "4010":
        return new AuthenticationError(message);
      case "403":
      case "4030":
        return new ForbiddenError(message);
      case "404":
      case "4040":
        return new NotFoundError(message);
      case "409":
      case "4090":
        return new ConflictError(message);
      case "429":
      case "4290":
        return new RateLimitError(message);
      default:
        if (code.startsWith("5")) {
          return new ServerError(message, httpStatus ?? HTTP_STATUS.INTERNAL_SERVER_ERROR);
        }
        return new BusinessError(message, result.code, result.data);
    }
  }
  static fromHttpStatus(status, message) {
    const defaultMessage = message ?? `HTTP Error ${status}`;
    switch (status) {
      case HTTP_STATUS.BAD_REQUEST:
      case HTTP_STATUS.UNPROCESSABLE_ENTITY:
        return new ValidationError(defaultMessage);
      case HTTP_STATUS.UNAUTHORIZED:
        return new AuthenticationError(defaultMessage);
      case HTTP_STATUS.FORBIDDEN:
        return new ForbiddenError(defaultMessage);
      case HTTP_STATUS.NOT_FOUND:
        return new NotFoundError(defaultMessage);
      case HTTP_STATUS.METHOD_NOT_ALLOWED:
        return new ValidationError(defaultMessage);
      case HTTP_STATUS.CONFLICT:
        return new ConflictError(defaultMessage);
      case HTTP_STATUS.TOO_MANY_REQUESTS:
        return new RateLimitError(defaultMessage);
      case HTTP_STATUS.INTERNAL_SERVER_ERROR:
        return new ServerError(defaultMessage, status);
      case HTTP_STATUS.BAD_GATEWAY:
        return new BadGatewayError(defaultMessage);
      case HTTP_STATUS.SERVICE_UNAVAILABLE:
        return new ServiceUnavailableError(defaultMessage);
      case HTTP_STATUS.GATEWAY_TIMEOUT:
        return new GatewayTimeoutError(defaultMessage);
      default:
        if (status >= 500) {
          return new ServerError(defaultMessage, status);
        }
        return new NetworkError(defaultMessage);
    }
  }
  toJSON() {
    return {
      name: this.name,
      message: this.message,
      code: this.code,
      httpStatus: this.httpStatus,
      details: this.details,
      timestamp: this.timestamp,
      traceId: this.traceId,
      metadata: this.metadata
    };
  }
  toString() {
    return `${this.name}: ${this.message} (code: ${this.code})`;
  }
  isRetryable() {
    return isRetryableError(this);
  }
  isAuthError() {
    return this.code === "UNAUTHORIZED" || this.code === "TOKEN_EXPIRED" || this.code === "TOKEN_INVALID";
  }
  isNetworkError() {
    return this.code === "NETWORK_ERROR" || this.code === "TIMEOUT";
  }
  isClientError() {
    return this.httpStatus !== void 0 && this.httpStatus >= 400 && this.httpStatus < 500;
  }
  isServerError() {
    return this.httpStatus !== void 0 && this.httpStatus >= 500;
  }
}
class NetworkError extends SdkError {
  constructor(message = "Network error", options) {
    super(message, "NETWORK_ERROR", void 0, options);
  }
}
class TimeoutError extends SdkError {
  timeout;
  constructor(message = "Request timeout", timeout, options) {
    super(message, "TIMEOUT", void 0, options);
    this.timeout = timeout;
  }
  toJSON() {
    return { ...super.toJSON(), timeout: this.timeout };
  }
}
class CancelledError extends SdkError {
  constructor(message = "Request cancelled", options) {
    super(message, "CANCELLED", void 0, options);
  }
}
class AuthenticationError extends SdkError {
  constructor(message = "Authentication failed", options) {
    super(message, "UNAUTHORIZED", HTTP_STATUS.UNAUTHORIZED, options);
  }
}
class TokenExpiredError extends AuthenticationError {
  constructor(message = "Token expired", options) {
    super(message, options);
    this.code = "TOKEN_EXPIRED";
  }
}
class TokenInvalidError extends AuthenticationError {
  constructor(message = "Invalid token", options) {
    super(message, options);
    this.code = "TOKEN_INVALID";
  }
}
class ForbiddenError extends SdkError {
  constructor(message = "Access forbidden", options) {
    super(message, "FORBIDDEN", HTTP_STATUS.FORBIDDEN, options);
  }
}
class NotFoundError extends SdkError {
  constructor(message = "Resource not found", options) {
    super(message, "NOT_FOUND", HTTP_STATUS.NOT_FOUND, options);
  }
}
class ValidationError extends SdkError {
  constructor(message = "Validation error", details, options) {
    super(message, "VALIDATION_ERROR", HTTP_STATUS.BAD_REQUEST, { ...options, details });
  }
}
class ConflictError extends SdkError {
  constructor(message = "Resource conflict", options) {
    super(message, "CONFLICT", HTTP_STATUS.CONFLICT, options);
  }
}
class MethodNotAllowedError extends SdkError {
  allowedMethods;
  constructor(message = "Method not allowed", allowedMethods, options) {
    super(message, "VALIDATION_ERROR", HTTP_STATUS.METHOD_NOT_ALLOWED, options);
    this.allowedMethods = allowedMethods;
  }
}
class RateLimitError extends SdkError {
  retryAfter;
  constructor(message = "Rate limit exceeded", retryAfter, options) {
    super(message, "RATE_LIMIT", HTTP_STATUS.TOO_MANY_REQUESTS, options);
    this.retryAfter = retryAfter;
  }
  toJSON() {
    return { ...super.toJSON(), retryAfter: this.retryAfter };
  }
}
class ServerError extends SdkError {
  constructor(message = "Server error", httpStatus = HTTP_STATUS.INTERNAL_SERVER_ERROR, options) {
    super(message, "SERVER_ERROR", httpStatus, options);
  }
}
class BadGatewayError extends ServerError {
  constructor(message = "Bad gateway", options) {
    super(message, HTTP_STATUS.BAD_GATEWAY, options);
    this.code = "BAD_GATEWAY";
  }
}
class ServiceUnavailableError extends ServerError {
  constructor(message = "Service unavailable", options) {
    super(message, HTTP_STATUS.SERVICE_UNAVAILABLE, options);
    this.code = "SERVICE_UNAVAILABLE";
  }
}
class GatewayTimeoutError extends ServerError {
  constructor(message = "Gateway timeout", options) {
    super(message, HTTP_STATUS.GATEWAY_TIMEOUT, options);
    this.code = "GATEWAY_TIMEOUT";
  }
}
class BusinessError extends SdkError {
  businessCode;
  data;
  constructor(message, code, data, options) {
    super(message, "BUSINESS_ERROR", void 0, options);
    this.businessCode = code;
    this.data = data;
  }
  toJSON() {
    return { ...super.toJSON(), businessCode: this.businessCode, data: this.data };
  }
}
function isSdkError(error) {
  return error instanceof SdkError;
}
function isNetworkError(error) {
  return error instanceof NetworkError;
}
function isTimeoutError(error) {
  return error instanceof TimeoutError;
}
function isCancelledError(error) {
  return error instanceof CancelledError;
}
function isAuthError(error) {
  return error instanceof AuthenticationError;
}
function isValidationError(error) {
  return error instanceof ValidationError;
}
function isRateLimitError(error) {
  return error instanceof RateLimitError;
}
function isServerError(error) {
  return error instanceof ServerError;
}
function isBusinessError(error) {
  return error instanceof BusinessError;
}
function isRetryableError(error) {
  if (!(error instanceof SdkError)) return false;
  return error instanceof NetworkError || error instanceof TimeoutError || error instanceof ServerError || error instanceof RateLimitError || error instanceof BadGatewayError || error instanceof ServiceUnavailableError || error instanceof GatewayTimeoutError;
}
export {
  AuthenticationError,
  BadGatewayError,
  BusinessError,
  CancelledError,
  ConflictError,
  ForbiddenError,
  GatewayTimeoutError,
  MethodNotAllowedError,
  NetworkError,
  NotFoundError,
  RateLimitError,
  SdkError,
  ServerError,
  ServiceUnavailableError,
  TimeoutError,
  TokenExpiredError,
  TokenInvalidError,
  ValidationError,
  isAuthError,
  isBusinessError,
  isCancelledError,
  isNetworkError,
  isRateLimitError,
  isRetryableError,
  isSdkError,
  isServerError,
  isTimeoutError,
  isValidationError
};
//# sourceMappingURL=errors.js.map
