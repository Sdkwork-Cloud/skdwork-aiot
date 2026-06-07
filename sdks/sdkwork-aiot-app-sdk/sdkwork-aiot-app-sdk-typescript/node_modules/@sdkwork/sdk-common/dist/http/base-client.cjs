"use strict";
Object.defineProperty(exports, Symbol.toStringTag, { value: "Module" });
const types = require("../core/types.cjs");
const errors = require("../errors.cjs");
const tokenManager = require("../auth/token-manager.cjs");
const logger = require("../utils/logger.cjs");
const cache = require("../utils/cache.cjs");
const retry = require("../utils/retry.cjs");
class BaseHttpClient {
  config;
  authConfig;
  logger;
  cache;
  interceptors;
  tenantId;
  organizationId;
  platform;
  userId;
  constructor(config) {
    this.config = {
      baseUrl: config.baseUrl,
      timeout: config.timeout ?? types.DEFAULT_TIMEOUT,
      headers: config.headers ?? {},
      retry: {
        maxRetries: 3,
        retryDelay: 1e3,
        retryBackoff: "exponential",
        maxRetryDelay: 3e4,
        ...config.retry
      },
      cache: {
        enabled: false,
        ttl: 5 * 60 * 1e3,
        maxSize: 100,
        ...config.cache
      },
      logger: {
        level: "info",
        prefix: "[SDK]",
        timestamp: true,
        colors: true,
        ...config.logger
      }
    };
    this.logger = logger.createLogger(this.config.logger);
    this.cache = cache.createCacheStore(this.config.cache);
    this.interceptors = config.interceptors ?? {
      request: [],
      response: [],
      error: []
    };
    const authMode = this.determineAuthMode(config);
    this.authConfig = {
      authMode,
      apiKey: config.apiKey,
      tokenManager: config.tokenManager ?? new tokenManager.DefaultAuthTokenManager({
        accessToken: config.accessToken,
        authToken: config.authToken
      })
    };
  }
  determineAuthMode(config) {
    if (config.apiKey) {
      return "apikey";
    }
    return "dual-token";
  }
  getAuthMode() {
    return this.authConfig.authMode;
  }
  setAuthMode(mode) {
    this.authConfig.authMode = mode;
  }
  getTokenManager() {
    return this.authConfig.tokenManager;
  }
  setTokenManager(manager) {
    this.authConfig.tokenManager = manager;
  }
  setApiKey(apiKey) {
    this.authConfig.apiKey = apiKey;
    this.authConfig.authMode = "apikey";
    this.authConfig.tokenManager?.clearTokens();
  }
  setAuthToken(token) {
    this.authConfig.tokenManager?.setAuthToken(token);
    if (this.authConfig.authMode === "apikey") {
      this.authConfig.authMode = "dual-token";
      this.authConfig.apiKey = void 0;
    }
  }
  setAccessToken(token) {
    this.authConfig.tokenManager?.setAccessToken(token);
    if (this.authConfig.authMode === "apikey") {
      this.authConfig.authMode = "dual-token";
      this.authConfig.apiKey = void 0;
    }
  }
  setTenantId(tenantId) {
    this.tenantId = tenantId;
  }
  setOrganizationId(organizationId) {
    this.organizationId = organizationId;
  }
  setPlatform(platform) {
    this.platform = platform;
  }
  setUserId(userId) {
    this.userId = userId;
  }
  clearAuthToken() {
    this.authConfig.tokenManager?.clearTokens();
  }
  addRequestInterceptor(interceptor) {
    this.interceptors.request.push(interceptor);
    return () => {
      const index = this.interceptors.request.indexOf(interceptor);
      if (index > -1) {
        this.interceptors.request.splice(index, 1);
      }
    };
  }
  addResponseInterceptor(interceptor) {
    this.interceptors.response.push(interceptor);
    return () => {
      const index = this.interceptors.response.indexOf(interceptor);
      if (index > -1) {
        this.interceptors.response.splice(index, 1);
      }
    };
  }
  addErrorInterceptor(interceptor) {
    this.interceptors.error.push(interceptor);
    return () => {
      const index = this.interceptors.error.indexOf(interceptor);
      if (index > -1) {
        this.interceptors.error.splice(index, 1);
      }
    };
  }
  clearCache() {
    this.cache.clear();
  }
  getConfig() {
    return {
      baseUrl: this.config.baseUrl,
      timeout: this.config.timeout,
      authMode: this.authConfig.authMode,
      apiKey: this.authConfig.apiKey,
      accessToken: this.authConfig.tokenManager?.getAccessToken(),
      authToken: this.authConfig.tokenManager?.getAuthToken(),
      tenantId: this.tenantId,
      organizationId: this.organizationId,
      platform: this.platform,
      userId: this.userId
    };
  }
  isAuthenticated() {
    return this.authConfig.tokenManager?.isValid() ?? false;
  }
  buildBaseUrl(path, params) {
    const baseUrl = this.config.baseUrl.replace(/\/$/, "");
    const normalizedPath = path.startsWith("/") ? path : `/${path}`;
    let url = `${baseUrl}${normalizedPath}`;
    if (params) {
      const searchParams = new URLSearchParams();
      Object.entries(params).forEach(([key, value]) => {
        if (value !== void 0 && value !== null) {
          searchParams.append(key, String(value));
        }
      });
      const queryString = searchParams.toString();
      if (queryString) {
        url += `?${queryString}`;
      }
    }
    return url;
  }
  buildHeaders(config, skipAuth = false) {
    const headers = {
      "Content-Type": types.MIME_TYPES.JSON,
      ...this.config.headers,
      ...config.headers
    };
    if (!skipAuth && !config.skipAuth) {
      const authHeaders = tokenManager.buildAuthHeaders(
        this.authConfig.authMode,
        this.authConfig.apiKey,
        this.authConfig.tokenManager
      );
      Object.assign(headers, authHeaders);
    }
    if (this.tenantId) {
      headers["X-Tenant-Id"] = this.tenantId;
    }
    if (this.organizationId) {
      headers["X-Organization-Id"] = this.organizationId;
    }
    if (this.platform) {
      headers["X-Platform"] = this.platform;
    }
    if (this.userId !== void 0) {
      headers["X-User-Id"] = String(this.userId);
    }
    return headers;
  }
  serializeRequestBody(body, headers) {
    if (body === void 0 || body === null) {
      return void 0;
    }
    if (typeof FormData !== "undefined" && body instanceof FormData) {
      delete headers["Content-Type"];
      return body;
    }
    if (typeof URLSearchParams !== "undefined" && body instanceof URLSearchParams) {
      headers["Content-Type"] = "application/x-www-form-urlencoded;charset=UTF-8";
      return body.toString();
    }
    if (typeof Blob !== "undefined" && body instanceof Blob) {
      delete headers["Content-Type"];
      return body;
    }
    if (typeof ArrayBuffer !== "undefined") {
      if (body instanceof ArrayBuffer) {
        delete headers["Content-Type"];
        return body;
      }
      if (ArrayBuffer.isView(body)) {
        delete headers["Content-Type"];
        return body;
      }
    }
    if (typeof body === "string") {
      headers["Content-Type"] = headers["Content-Type"] || "text/plain;charset=UTF-8";
      return body;
    }
    return JSON.stringify(body);
  }
  async applyRequestInterceptors(config) {
    let processedConfig = config;
    for (const interceptor of this.interceptors.request) {
      processedConfig = await interceptor(processedConfig);
    }
    return processedConfig;
  }
  async applyResponseInterceptors(response, config) {
    let processedResponse = response;
    for (const interceptor of this.interceptors.response) {
      processedResponse = await interceptor(processedResponse, config);
    }
    return processedResponse;
  }
  async applyErrorInterceptors(error, config) {
    for (const interceptor of this.interceptors.error) {
      await interceptor(error, config);
    }
  }
  async handleErrorResponse(response, config) {
    let errorMessage = `HTTP ${response.status}: ${response.statusText}`;
    try {
      const result = await response.json();
      errorMessage = result.msg || result.message || errorMessage;
    } catch {
    }
    const error = errors.SdkError.fromHttpStatus(response.status, errorMessage);
    await this.applyErrorInterceptors(error, config);
    throw error;
  }
  async processResponse(response, config) {
    if (!response.ok) {
      await this.handleErrorResponse(response, config);
    }
    const contentType = response.headers.get("content-type");
    if (contentType?.includes(types.MIME_TYPES.JSON)) {
      const result = await response.json();
      if (!types.SUCCESS_CODES.includes(result.code) && !types.SUCCESS_CODES.includes(String(result.code))) {
        throw errors.SdkError.fromApiResult(result, response.status);
      }
      return result.data;
    }
    if (contentType?.includes("text/")) {
      return await response.text();
    }
    return await response.json();
  }
  async executeFetch(url, options) {
    const controller = new AbortController();
    let timedOut = false;
    const timeoutId = setTimeout(() => {
      timedOut = true;
      controller.abort();
    }, options.timeout);
    const abortHandler = () => controller.abort();
    if (options.signal) {
      if (options.signal.aborted) {
        controller.abort();
      } else {
        options.signal.addEventListener("abort", abortHandler, { once: true });
      }
    }
    try {
      this.logger.debug(`${options.method} ${url}`);
      const response = await fetch(url, {
        method: options.method,
        headers: options.headers,
        body: options.body,
        signal: controller.signal
      });
      return response;
    } catch (error) {
      if (error instanceof Error) {
        if (error.name === "AbortError") {
          if (timedOut) {
            throw new errors.TimeoutError(`Request timeout after ${options.timeout}ms`, options.timeout);
          }
          throw new errors.CancelledError("Request was cancelled");
        }
        throw new errors.NetworkError(error.message);
      }
      throw new errors.NetworkError("Unknown network error");
    } finally {
      clearTimeout(timeoutId);
      if (options.signal) {
        options.signal.removeEventListener("abort", abortHandler);
      }
    }
  }
  async execute(config) {
    const processedConfig = await this.applyRequestInterceptors(config);
    const url = this.buildBaseUrl(processedConfig.url, processedConfig.params);
    const headers = this.buildHeaders(processedConfig);
    const serializedBody = this.serializeRequestBody(processedConfig.body, headers);
    const response = await this.executeFetch(url, {
      method: processedConfig.method,
      headers,
      body: serializedBody,
      timeout: processedConfig.timeout ?? this.config.timeout,
      signal: processedConfig.signal
    });
    return this.processResponse(response, processedConfig);
  }
  async upload(path, options) {
    const formData = new FormData();
    formData.append(options.fieldName ?? "file", options.file);
    if (options.additionalData) {
      Object.entries(options.additionalData).forEach(([key, value]) => {
        formData.append(key, value);
      });
    }
    const config = {
      url: path,
      method: "POST",
      body: formData,
      skipAuth: false
    };
    const processedConfig = await this.applyRequestInterceptors(config);
    const url = this.buildBaseUrl(processedConfig.url, processedConfig.params);
    const headers = this.buildHeaders(processedConfig);
    delete headers["Content-Type"];
    const response = await this.executeFetch(url, {
      method: "POST",
      headers,
      body: formData,
      timeout: processedConfig.timeout ?? this.config.timeout,
      signal: processedConfig.signal
    });
    return this.processResponse(response, processedConfig);
  }
  async download(path, _options) {
    const config = {
      url: path,
      method: "GET",
      skipAuth: false
    };
    const processedConfig = await this.applyRequestInterceptors(config);
    const url = this.buildBaseUrl(processedConfig.url, processedConfig.params);
    const headers = this.buildHeaders(processedConfig);
    const response = await this.executeFetch(url, {
      method: "GET",
      headers,
      timeout: processedConfig.timeout ?? this.config.timeout,
      signal: processedConfig.signal
    });
    if (!response.ok) {
      await this.handleErrorResponse(response, processedConfig);
    }
    return response.blob();
  }
  async *stream(path, options) {
    const config = {
      url: path,
      method: options?.method ?? "POST",
      body: options?.body,
      headers: options?.headers,
      skipAuth: options?.skipAuth
    };
    const processedConfig = await this.applyRequestInterceptors(config);
    const url = this.buildBaseUrl(processedConfig.url, processedConfig.params);
    const headers = this.buildHeaders(processedConfig);
    const response = await this.executeFetch(url, {
      method: processedConfig.method,
      headers,
      body: processedConfig.body ? JSON.stringify(processedConfig.body) : void 0,
      timeout: processedConfig.timeout ?? this.config.timeout,
      signal: processedConfig.signal
    });
    if (!response.ok) {
      await this.handleErrorResponse(response, processedConfig);
    }
    const reader = response.body?.getReader();
    if (!reader) {
      throw new errors.NetworkError("No response body");
    }
    const decoder = new TextDecoder();
    let buffer = "";
    try {
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        buffer += decoder.decode(value, { stream: true });
        const lines = buffer.split("\n");
        buffer = lines.pop() ?? "";
        for (const line of lines) {
          const trimmedLine = line.trim();
          if (trimmedLine === "" || trimmedLine === "data: [DONE]") continue;
          if (trimmedLine.startsWith("data: ")) {
            yield trimmedLine.slice(6);
          } else {
            yield trimmedLine;
          }
        }
      }
    } finally {
      reader.releaseLock();
    }
  }
}
function createBaseHttpClient(config) {
  return new class extends BaseHttpClient {
    async request(path, options = {}) {
      const config2 = {
        url: path,
        method: options.method ?? "GET",
        headers: options.headers,
        params: options.params,
        body: options.body,
        timeout: options.timeout,
        signal: options.signal,
        skipAuth: options.skipAuth
      };
      return retry.withRetry(
        () => this.execute(config2),
        { ...this.config.retry, ...options.retry }
      );
    }
    async get(path, params) {
      return this.request(path, { method: "GET", params });
    }
    async post(path, body) {
      return this.request(path, { method: "POST", body });
    }
    async put(path, body) {
      return this.request(path, { method: "PUT", body });
    }
    async delete(path, body) {
      return this.request(path, { method: "DELETE", body });
    }
    async patch(path, body) {
      return this.request(path, { method: "PATCH", body });
    }
  }(config);
}
exports.BaseHttpClient = BaseHttpClient;
exports.createBaseHttpClient = createBaseHttpClient;
//# sourceMappingURL=base-client.cjs.map
