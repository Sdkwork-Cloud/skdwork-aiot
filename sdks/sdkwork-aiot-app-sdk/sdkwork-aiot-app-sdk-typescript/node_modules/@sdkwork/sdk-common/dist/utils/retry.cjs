"use strict";
Object.defineProperty(exports, Symbol.toStringTag, { value: "Module" });
const types = require("../core/types.cjs");
const errors = require("../errors.cjs");
function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
function calculateDelay(attempt, baseDelay, backoff, maxDelay) {
  let delay;
  switch (backoff) {
    case "fixed":
      delay = baseDelay;
      break;
    case "linear":
      delay = baseDelay * attempt;
      break;
    case "exponential":
      delay = baseDelay * Math.pow(2, attempt - 1);
      break;
    default:
      delay = baseDelay;
  }
  return Math.min(delay, maxDelay);
}
function shouldRetry(error, attempt, config) {
  if (attempt >= config.maxRetries) {
    return false;
  }
  if (config.retryCondition) {
    return config.retryCondition(error, attempt);
  }
  return errors.isRetryableError(error);
}
async function withRetry(fn, config = {}) {
  const fullConfig = {
    ...types.DEFAULT_RETRY_CONFIG,
    ...config
  };
  let lastError;
  let attempt = 0;
  while (attempt <= fullConfig.maxRetries) {
    try {
      return await fn();
    } catch (error) {
      lastError = error;
      attempt++;
      if (!shouldRetry(lastError, attempt, fullConfig)) {
        throw lastError;
      }
      const delay = calculateDelay(
        attempt,
        fullConfig.retryDelay,
        fullConfig.retryBackoff,
        fullConfig.maxRetryDelay
      );
      await sleep(delay);
    }
  }
  throw lastError;
}
function createRetryConfig(config) {
  return {
    ...types.DEFAULT_RETRY_CONFIG,
    ...config
  };
}
exports.DEFAULT_RETRY_CONFIG = types.DEFAULT_RETRY_CONFIG;
exports.calculateDelay = calculateDelay;
exports.createRetryConfig = createRetryConfig;
exports.shouldRetry = shouldRetry;
exports.sleep = sleep;
exports.withRetry = withRetry;
//# sourceMappingURL=retry.cjs.map
