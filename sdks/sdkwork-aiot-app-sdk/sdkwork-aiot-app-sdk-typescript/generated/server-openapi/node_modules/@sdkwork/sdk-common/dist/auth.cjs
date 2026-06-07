Object.defineProperty(exports, Symbol.toStringTag, { value: "Module" });
const require_token_manager = require("./auth/token-manager.cjs");
exports.DefaultAuthTokenManager = require_token_manager.DefaultAuthTokenManager;
exports.buildAuthHeaders = require_token_manager.buildAuthHeaders;
exports.createTokenManager = require_token_manager.createTokenManager;
exports.isTokenValid = require_token_manager.isTokenValid;
exports.requiresRefresh = require_token_manager.requiresRefresh;
