"use strict";
Object.defineProperty(exports, Symbol.toStringTag, { value: "Module" });
const tokenManager = require("./auth/token-manager.cjs");
exports.DefaultAuthTokenManager = tokenManager.DefaultAuthTokenManager;
exports.buildAuthHeaders = tokenManager.buildAuthHeaders;
exports.createTokenManager = tokenManager.createTokenManager;
exports.isTokenValid = tokenManager.isTokenValid;
exports.requiresRefresh = tokenManager.requiresRefresh;
//# sourceMappingURL=auth.cjs.map
