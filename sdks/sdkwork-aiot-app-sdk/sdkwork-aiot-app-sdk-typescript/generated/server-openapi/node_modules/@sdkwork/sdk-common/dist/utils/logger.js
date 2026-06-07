//#region src/utils/logger.ts
var LOG_LEVELS = {
	debug: 0,
	info: 1,
	warn: 2,
	error: 3,
	silent: 4
};
var ConsoleLogger = class {
	level;
	prefix;
	timestamp;
	colors;
	constructor(config = {}) {
		this.level = config.level ?? "info";
		this.prefix = config.prefix ?? "[SDK]";
		this.timestamp = config.timestamp ?? true;
		this.colors = config.colors ?? true;
	}
	formatMessage(level, message) {
		const parts = [];
		if (this.timestamp) parts.push((/* @__PURE__ */ new Date()).toISOString());
		parts.push(this.prefix);
		parts.push(`[${level.toUpperCase()}]`);
		parts.push(message);
		return parts.join(" ");
	}
	getColorCode(level) {
		if (!this.colors) return "";
		return {
			debug: "\x1B[36m",
			info: "\x1B[32m",
			warn: "\x1B[33m",
			error: "\x1B[31m",
			silent: ""
		}[level];
	}
	getResetCode() {
		return this.colors ? "\x1B[0m" : "";
	}
	log(level, message, ...args) {
		if (LOG_LEVELS[level] < LOG_LEVELS[this.level]) return;
		const formattedMessage = this.formatMessage(level, message);
		const output = `${this.getColorCode(level)}${formattedMessage}${this.getResetCode()}`;
		switch (level) {
			case "debug":
				console.debug(output, ...args);
				break;
			case "info":
				console.info(output, ...args);
				break;
			case "warn":
				console.warn(output, ...args);
				break;
			case "error":
				console.error(output, ...args);
				break;
		}
	}
	debug(message, ...args) {
		this.log("debug", message, ...args);
	}
	info(message, ...args) {
		this.log("info", message, ...args);
	}
	warn(message, ...args) {
		this.log("warn", message, ...args);
	}
	error(message, ...args) {
		this.log("error", message, ...args);
	}
	setLevel(level) {
		this.level = level;
	}
};
var noopLogger = {
	debug: () => {},
	info: () => {},
	warn: () => {},
	error: () => {},
	log: () => {},
	setLevel: () => {}
};
function createLogger(config) {
	if (config?.level === "silent") return noopLogger;
	return new ConsoleLogger(config);
}
//#endregion
export { ConsoleLogger, createLogger, noopLogger };

//# sourceMappingURL=logger.js.map