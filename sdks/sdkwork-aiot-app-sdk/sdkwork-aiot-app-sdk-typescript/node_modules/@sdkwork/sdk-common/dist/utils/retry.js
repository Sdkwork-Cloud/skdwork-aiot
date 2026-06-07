import { DEFAULT_RETRY_CONFIG } from "../core/types.js";
import { isRetryableError } from "../errors.js";
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
  return isRetryableError(error);
}
async function withRetry(fn, config = {}) {
  const fullConfig = {
    ...DEFAULT_RETRY_CONFIG,
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
    ...DEFAULT_RETRY_CONFIG,
    ...config
  };
}
export {
  DEFAULT_RETRY_CONFIG,
  calculateDelay,
  createRetryConfig,
  shouldRetry,
  sleep,
  withRetry
};
//# sourceMappingURL=retry.js.map
