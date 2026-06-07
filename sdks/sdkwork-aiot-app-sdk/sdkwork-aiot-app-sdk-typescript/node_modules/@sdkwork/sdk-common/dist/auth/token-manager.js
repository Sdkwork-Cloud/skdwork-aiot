class DefaultAuthTokenManager {
  tokens = {};
  events;
  constructor(initialTokens, events) {
    if (initialTokens) {
      this.tokens = { ...initialTokens };
      if (initialTokens.expiresIn && !initialTokens.expiresAt) {
        this.tokens.expiresAt = Date.now() + initialTokens.expiresIn * 1e3;
      }
    }
    this.events = events;
  }
  getAccessToken() {
    return this.tokens.accessToken;
  }
  getAuthToken() {
    return this.tokens.authToken;
  }
  getRefreshToken() {
    return this.tokens.refreshToken;
  }
  getTokens() {
    return { ...this.tokens };
  }
  setTokens(tokens) {
    this.tokens = { ...tokens };
    if (tokens.expiresIn && !tokens.expiresAt) {
      this.tokens.expiresAt = Date.now() + tokens.expiresIn * 1e3;
    }
    this.events?.onTokenSet?.(this.tokens);
  }
  setAccessToken(token) {
    this.tokens.accessToken = token;
    this.events?.onTokenSet?.(this.tokens);
  }
  setAuthToken(token) {
    this.tokens.authToken = token;
    this.events?.onTokenSet?.(this.tokens);
  }
  setRefreshToken(token) {
    this.tokens.refreshToken = token;
  }
  clearTokens() {
    this.tokens = {};
    this.events?.onTokenCleared?.();
  }
  clearAuthToken() {
    this.tokens.authToken = void 0;
  }
  clearAccessToken() {
    this.tokens.accessToken = void 0;
  }
  isExpired() {
    if (!this.tokens.expiresAt) {
      return false;
    }
    const expired = Date.now() >= this.tokens.expiresAt;
    if (expired) {
      this.events?.onTokenExpired?.();
    }
    return expired;
  }
  isValid() {
    return this.hasToken() && !this.isExpired();
  }
  hasToken() {
    return !!(this.tokens.accessToken || this.tokens.authToken);
  }
  hasAuthToken() {
    return !!this.tokens.authToken;
  }
  hasAccessToken() {
    return !!this.tokens.accessToken;
  }
  willExpireIn(seconds) {
    if (!this.tokens.expiresAt) {
      return false;
    }
    return Date.now() + seconds * 1e3 >= this.tokens.expiresAt;
  }
}
function createTokenManager(tokens, events) {
  return new DefaultAuthTokenManager(tokens, events);
}
function buildAuthHeaders(authMode, apiKey, tokenManager) {
  const headers = {};
  if (authMode === "apikey") {
    if (apiKey) {
      headers["Authorization"] = `Bearer ${apiKey}`;
    }
  } else if (authMode === "dual-token") {
    if (tokenManager) {
      const accessToken = tokenManager.getAccessToken();
      const authToken = tokenManager.getAuthToken();
      if (accessToken) {
        headers["Access-Token"] = accessToken;
      }
      if (authToken) {
        headers["Authorization"] = `Bearer ${authToken}`;
      }
    }
  }
  return headers;
}
function isTokenValid(manager) {
  return manager?.isValid() ?? false;
}
function requiresRefresh(manager, thresholdSeconds = 300) {
  if (!manager) return false;
  return manager.willExpireIn(thresholdSeconds);
}
export {
  DefaultAuthTokenManager,
  buildAuthHeaders,
  createTokenManager,
  isTokenValid,
  requiresRefresh
};
//# sourceMappingURL=token-manager.js.map
