//#region src/core/types.ts
var DEFAULT_RETRY_CONFIG = {
	maxRetries: 3,
	retryDelay: 1e3,
	retryBackoff: "exponential",
	maxRetryDelay: 3e4
};
var DEFAULT_CACHE_CONFIG = {
	enabled: false,
	ttl: 300 * 1e3,
	maxSize: 100
};
var DEFAULT_LOGGER_CONFIG = {
	level: "info",
	prefix: "[SDK]",
	timestamp: true,
	colors: true
};
var DEFAULT_TIMEOUT = 3e4;
var SUCCESS_CODES = [
	0,
	200,
	2e3,
	"0",
	"200",
	"2000"
];
var HTTP_STATUS = {
	OK: 200,
	CREATED: 201,
	NO_CONTENT: 204,
	BAD_REQUEST: 400,
	UNAUTHORIZED: 401,
	FORBIDDEN: 403,
	NOT_FOUND: 404,
	METHOD_NOT_ALLOWED: 405,
	CONFLICT: 409,
	UNPROCESSABLE_ENTITY: 422,
	TOO_MANY_REQUESTS: 429,
	INTERNAL_SERVER_ERROR: 500,
	BAD_GATEWAY: 502,
	SERVICE_UNAVAILABLE: 503,
	GATEWAY_TIMEOUT: 504
};
var MIME_TYPES = {
	JSON: "application/json",
	FORM_DATA: "multipart/form-data",
	URL_ENCODED: "application/x-www-form-urlencoded",
	OCTET_STREAM: "application/octet-stream",
	TEXT_PLAIN: "text/plain",
	TEXT_HTML: "text/html"
};
//#endregion
exports.DEFAULT_CACHE_CONFIG = DEFAULT_CACHE_CONFIG;
exports.DEFAULT_LOGGER_CONFIG = DEFAULT_LOGGER_CONFIG;
exports.DEFAULT_RETRY_CONFIG = DEFAULT_RETRY_CONFIG;
exports.DEFAULT_TIMEOUT = DEFAULT_TIMEOUT;
exports.HTTP_STATUS = HTTP_STATUS;
exports.MIME_TYPES = MIME_TYPES;
exports.SUCCESS_CODES = SUCCESS_CODES;

//# sourceMappingURL=types.cjs.map