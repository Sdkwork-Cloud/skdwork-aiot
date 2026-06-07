const require_types = require("../core/types.cjs");
const require_errors = require("../errors.cjs");
//#region src/utils/retry.ts
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
		default: delay = baseDelay;
	}
	return Math.min(delay, maxDelay);
}
function shouldRetry(error, attempt, config) {
	if (attempt >= config.maxRetries) return false;
	if (config.retryCondition) return config.retryCondition(error, attempt);
	return require_errors.isRetryableError(error);
}
async function withRetry(fn, config = {}) {
	const fullConfig = {
		...require_types.DEFAULT_RETRY_CONFIG,
		...config
	};
	let lastError;
	let attempt = 0;
	while (attempt <= fullConfig.maxRetries) try {
		return await fn();
	} catch (error) {
		lastError = error;
		attempt++;
		if (!shouldRetry(lastError, attempt, fullConfig)) throw lastError;
		await sleep(calculateDelay(attempt, fullConfig.retryDelay, fullConfig.retryBackoff, fullConfig.maxRetryDelay));
	}
	throw lastError;
}
function createRetryConfig(config) {
	return {
		...require_types.DEFAULT_RETRY_CONFIG,
		...config
	};
}
//#endregion
exports.calculateDelay = calculateDelay;
exports.createRetryConfig = createRetryConfig;
exports.shouldRetry = shouldRetry;
exports.sleep = sleep;
exports.withRetry = withRetry;

//# sourceMappingURL=retry.cjs.map