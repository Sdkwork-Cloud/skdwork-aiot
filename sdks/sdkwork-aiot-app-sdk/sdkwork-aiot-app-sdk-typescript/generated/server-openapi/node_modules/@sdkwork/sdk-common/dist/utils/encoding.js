//#region src/utils/encoding.ts
var Encoding;
(function(_Encoding) {
	function base64Encode(input) {
		let bytes;
		if (typeof input === "string") bytes = new TextEncoder().encode(input);
		else bytes = input;
		const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
		let result = "";
		let i = 0;
		while (i < bytes.length) {
			const a = bytes[i++] ?? 0;
			const b = i < bytes.length ? bytes[i++] ?? 0 : 0;
			const c = i < bytes.length ? bytes[i++] ?? 0 : 0;
			const bitmap = a << 16 | b << 8 | c;
			result += chars[bitmap >> 18 & 63];
			result += chars[bitmap >> 12 & 63];
			result += i > bytes.length + 1 ? "=" : chars[bitmap >> 6 & 63];
			result += i > bytes.length ? "=" : chars[bitmap & 63];
		}
		return result;
	}
	_Encoding.base64Encode = base64Encode;
	function base64Decode(input) {
		const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
		input = input.replace(/[^A-Za-z0-9+/]/g, "");
		const len = input.length;
		let result = "";
		let i = 0;
		while (i < len) {
			const a = chars.indexOf(input[i++] ?? "");
			const b = chars.indexOf(input[i++] ?? "");
			const c = chars.indexOf(input[i++] ?? "");
			const d = chars.indexOf(input[i++] ?? "");
			const bitmap = a << 18 | b << 12 | c << 6 | d;
			result += String.fromCharCode(bitmap >> 16 & 255);
			if (c !== 64 && input[i - 2] !== "=") result += String.fromCharCode(bitmap >> 8 & 255);
			if (d !== 64 && input[i - 1] !== "=") result += String.fromCharCode(bitmap & 255);
		}
		return result;
	}
	_Encoding.base64Decode = base64Decode;
	function base64UrlEncode(input) {
		return base64Encode(input).replace(/\+/g, "-").replace(/\//g, "_").replace(/=/g, "");
	}
	_Encoding.base64UrlEncode = base64UrlEncode;
	function base64UrlDecode(input) {
		input = input.replace(/-/g, "+").replace(/_/g, "/");
		const pad = input.length % 4;
		if (pad) input += "=".repeat(4 - pad);
		return base64Decode(input);
	}
	_Encoding.base64UrlDecode = base64UrlDecode;
	function base64ToBytes(base64) {
		const binary = atob(base64);
		const bytes = new Uint8Array(binary.length);
		for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
		return bytes;
	}
	_Encoding.base64ToBytes = base64ToBytes;
	function bytesToBase64(bytes) {
		let binary = "";
		for (let i = 0; i < bytes.length; i++) binary += String.fromCharCode(bytes[i] ?? 0);
		return btoa(binary);
	}
	_Encoding.bytesToBase64 = bytesToBase64;
	function utf8Encode(input) {
		return new TextEncoder().encode(input);
	}
	_Encoding.utf8Encode = utf8Encode;
	function utf8Decode(input) {
		return new TextDecoder().decode(input);
	}
	_Encoding.utf8Decode = utf8Decode;
	function hexEncode(input) {
		const bytes = typeof input === "string" ? utf8Encode(input) : input;
		return Array.from(bytes).map((byte) => byte.toString(16).padStart(2, "0")).join("");
	}
	_Encoding.hexEncode = hexEncode;
	function hexDecode(input) {
		const bytes = new Uint8Array(input.length / 2);
		for (let i = 0; i < input.length; i += 2) bytes[i / 2] = parseInt(input.substr(i, 2), 16);
		return utf8Decode(bytes);
	}
	_Encoding.hexDecode = hexDecode;
	function hexToBytes(hex) {
		const bytes = new Uint8Array(hex.length / 2);
		for (let i = 0; i < hex.length; i += 2) bytes[i / 2] = parseInt(hex.substr(i, 2), 16);
		return bytes;
	}
	_Encoding.hexToBytes = hexToBytes;
	function bytesToHex(bytes) {
		return Array.from(bytes).map((byte) => byte.toString(16).padStart(2, "0")).join("");
	}
	_Encoding.bytesToHex = bytesToHex;
	function urlEncode(input) {
		return encodeURIComponent(input);
	}
	_Encoding.urlEncode = urlEncode;
	function urlDecode(input) {
		return decodeURIComponent(input);
	}
	_Encoding.urlDecode = urlDecode;
	function urlEncodeComponent(input) {
		return encodeURIComponent(input);
	}
	_Encoding.urlEncodeComponent = urlEncodeComponent;
	function urlDecodeComponent(input) {
		return decodeURIComponent(input);
	}
	_Encoding.urlDecodeComponent = urlDecodeComponent;
	function htmlEncode(input) {
		const htmlEntities = {
			"&": "&amp;",
			"<": "&lt;",
			">": "&gt;",
			"\"": "&quot;",
			"'": "&#39;",
			"/": "&#x2F;",
			"`": "&#x60;",
			"=": "&#x3D;"
		};
		return input.replace(/[&<>"'`=/]/g, (char) => htmlEntities[char] || char);
	}
	_Encoding.htmlEncode = htmlEncode;
	function htmlDecode(input) {
		const htmlEntities = {
			"&amp;": "&",
			"&lt;": "<",
			"&gt;": ">",
			"&quot;": "\"",
			"&#39;": "'",
			"&#x27;": "'",
			"&#x2F;": "/",
			"&#x60;": "`",
			"&#x3D;": "=",
			"&nbsp;": " "
		};
		return input.replace(/&[^;]+;/g, (entity) => htmlEntities[entity] || entity);
	}
	_Encoding.htmlDecode = htmlDecode;
	function jsonEncode(value, replacer, space) {
		return JSON.stringify(value, replacer, space);
	}
	_Encoding.jsonEncode = jsonEncode;
	function jsonDecode(input) {
		return JSON.parse(input);
	}
	_Encoding.jsonDecode = jsonDecode;
	function jsonEncodePretty(value, indent = 2) {
		return JSON.stringify(value, null, indent);
	}
	_Encoding.jsonEncodePretty = jsonEncodePretty;
	function tryJsonDecode(input, defaultValue) {
		try {
			return JSON.parse(input);
		} catch {
			return defaultValue;
		}
	}
	_Encoding.tryJsonDecode = tryJsonDecode;
	function isJson(input) {
		try {
			JSON.parse(input);
			return true;
		} catch {
			return false;
		}
	}
	_Encoding.isJson = isJson;
	function xmlEncode(input) {
		const xmlEntities = {
			"&": "&amp;",
			"<": "&lt;",
			">": "&gt;",
			"\"": "&quot;",
			"'": "&apos;"
		};
		return input.replace(/[&<>"']/g, (char) => xmlEntities[char] || char);
	}
	_Encoding.xmlEncode = xmlEncode;
	function xmlDecode(input) {
		const xmlEntities = {
			"&amp;": "&",
			"&lt;": "<",
			"&gt;": ">",
			"&quot;": "\"",
			"&apos;": "'"
		};
		return input.replace(/&[^;]+;/g, (entity) => xmlEntities[entity] || entity);
	}
	_Encoding.xmlDecode = xmlDecode;
	function escapeRegex(input) {
		return input.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
	}
	_Encoding.escapeRegex = escapeRegex;
	function escapeSql(input) {
		return input.replace(/[\0\x08\x09\x1a\n\r"'\\\%]/g, (char) => {
			return {
				"\0": "\\0",
				"\b": "\\b",
				"	": "\\t",
				"": "\\z",
				"\n": "\\n",
				"\r": "\\r",
				"\"": "\\\"",
				"'": "\\'",
				"\\": "\\\\",
				"%": "\\%"
			}[char] || char;
		});
	}
	_Encoding.escapeSql = escapeSql;
	function escapeShell(input) {
		return input.replace(/[^A-Za-z0-9_\-.,:\/@\n]/g, (char) => {
			if (char === "\n") return "'\\n'";
			return `\\${char}`;
		});
	}
	_Encoding.escapeShell = escapeShell;
	function escapeCString(input) {
		return input.replace(/[\\"'\n\r\t\b\f\v\0]/g, (char) => {
			return {
				"\\": "\\\\",
				"\"": "\\\"",
				"'": "\\'",
				"\n": "\\n",
				"\r": "\\r",
				"	": "\\t",
				"\b": "\\b",
				"\f": "\\f",
				"\v": "\\v",
				"\0": "\\0"
			}[char] || char;
		});
	}
	_Encoding.escapeCString = escapeCString;
	function unescapeCString(input) {
		return input.replace(/\\([\\\"'nrtbfv0])/g, (_, char) => {
			return {
				"\\": "\\",
				"\"": "\"",
				"'": "'",
				"n": "\n",
				"r": "\r",
				"t": "	",
				"b": "\b",
				"f": "\f",
				"v": "\v",
				"0": "\0"
			}[char] || char;
		});
	}
	_Encoding.unescapeCString = unescapeCString;
	function camelToSnake(input) {
		return input.replace(/[A-Z]/g, (letter) => `_${letter.toLowerCase()}`);
	}
	_Encoding.camelToSnake = camelToSnake;
	function snakeToCamel(input) {
		return input.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase());
	}
	_Encoding.snakeToCamel = snakeToCamel;
	function camelToKebab(input) {
		return input.replace(/[A-Z]/g, (letter) => `-${letter.toLowerCase()}`);
	}
	_Encoding.camelToKebab = camelToKebab;
	function kebabToCamel(input) {
		return input.replace(/-([a-z])/g, (_, letter) => letter.toUpperCase());
	}
	_Encoding.kebabToCamel = kebabToCamel;
	function camelToPascal(input) {
		return input.charAt(0).toUpperCase() + input.slice(1);
	}
	_Encoding.camelToPascal = camelToPascal;
	function pascalToCamel(input) {
		return input.charAt(0).toLowerCase() + input.slice(1);
	}
	_Encoding.pascalToCamel = pascalToCamel;
	function pascalToSnake(input) {
		return camelToSnake(input);
	}
	_Encoding.pascalToSnake = pascalToSnake;
	function snakeToPascal(input) {
		return camelToPascal(snakeToCamel(input));
	}
	_Encoding.snakeToPascal = snakeToPascal;
	function pascalToKebab(input) {
		return camelToKebab(input);
	}
	_Encoding.pascalToKebab = pascalToKebab;
	function kebabToPascal(input) {
		return camelToPascal(kebabToCamel(input));
	}
	_Encoding.kebabToPascal = kebabToPascal;
	function toSnakeCase(input) {
		return input.replace(/([a-z])([A-Z])/g, "$1_$2").replace(/[-\s]+/g, "_").toLowerCase();
	}
	_Encoding.toSnakeCase = toSnakeCase;
	function toKebabCase(input) {
		return input.replace(/([a-z])([A-Z])/g, "$1-$2").replace(/[_\s]+/g, "-").toLowerCase();
	}
	_Encoding.toKebabCase = toKebabCase;
	function toCamelCase(input) {
		return input.replace(/[-_\s]+(.)?/g, (_, char) => char ? char.toUpperCase() : "").replace(/^(.)/, (char) => char.toLowerCase());
	}
	_Encoding.toCamelCase = toCamelCase;
	function toPascalCase(input) {
		const camel = toCamelCase(input);
		return camel.charAt(0).toUpperCase() + camel.slice(1);
	}
	_Encoding.toPascalCase = toPascalCase;
	function toConstantCase(input) {
		return toSnakeCase(input).toUpperCase();
	}
	_Encoding.toConstantCase = toConstantCase;
	function toSentenceCase(input) {
		return input.charAt(0).toUpperCase() + input.slice(1).toLowerCase();
	}
	_Encoding.toSentenceCase = toSentenceCase;
	function toTitleCase(input) {
		return input.replace(/\b\w/g, (char) => char.toUpperCase());
	}
	_Encoding.toTitleCase = toTitleCase;
	function toCapitalCase(input) {
		return input.replace(/[-_\s]+(.)?/g, (_, char) => char ? ` ${char.toUpperCase()}` : "").trim();
	}
	_Encoding.toCapitalCase = toCapitalCase;
	function toDotCase(input) {
		return input.replace(/([a-z])([A-Z])/g, "$1.$2").replace(/[-_\s]+/g, ".").toLowerCase();
	}
	_Encoding.toDotCase = toDotCase;
	function toPathCase(input) {
		return input.replace(/([a-z])([A-Z])/g, "$1/$2").replace(/[-_\s]+/g, "/").toLowerCase();
	}
	_Encoding.toPathCase = toPathCase;
	function rot13(input) {
		return input.replace(/[a-zA-Z]/g, (char) => {
			const start = char <= "Z" ? 65 : 97;
			return String.fromCharCode((char.charCodeAt(0) - start + 13) % 26 + start);
		});
	}
	_Encoding.rot13 = rot13;
	function caesarCipher(input, shift) {
		return input.replace(/[a-zA-Z]/g, (char) => {
			const start = char <= "Z" ? 65 : 97;
			const shifted = ((char.charCodeAt(0) - start + shift) % 26 + 26) % 26;
			return String.fromCharCode(shifted + start);
		});
	}
	_Encoding.caesarCipher = caesarCipher;
	function caesarDecipher(input, shift) {
		return caesarCipher(input, -shift);
	}
	_Encoding.caesarDecipher = caesarDecipher;
	function xorEncode(input, key) {
		const inputBytes = utf8Encode(input);
		const keyBytes = utf8Encode(key);
		const result = new Uint8Array(inputBytes.length);
		for (let i = 0; i < inputBytes.length; i++) result[i] = (inputBytes[i] ?? 0) ^ (keyBytes[i % keyBytes.length] ?? 0);
		return bytesToHex(result);
	}
	_Encoding.xorEncode = xorEncode;
	function xorDecode(input, key) {
		const inputBytes = hexToBytes(input);
		const keyBytes = utf8Encode(key);
		const result = new Uint8Array(inputBytes.length);
		for (let i = 0; i < inputBytes.length; i++) result[i] = (inputBytes[i] ?? 0) ^ (keyBytes[i % keyBytes.length] ?? 0);
		return utf8Decode(result);
	}
	_Encoding.xorDecode = xorDecode;
	function charCodeEncode(input) {
		return Array.from(input).map((char) => char.charCodeAt(0));
	}
	_Encoding.charCodeEncode = charCodeEncode;
	function charCodeDecode(codes) {
		return String.fromCharCode(...codes);
	}
	_Encoding.charCodeDecode = charCodeDecode;
	function binaryEncode(input) {
		return Array.from(input).map((char) => char.charCodeAt(0).toString(2).padStart(8, "0")).join(" ");
	}
	_Encoding.binaryEncode = binaryEncode;
	function binaryDecode(input) {
		return input.split(/\s+/).map((byte) => String.fromCharCode(parseInt(byte, 2))).join("");
	}
	_Encoding.binaryDecode = binaryDecode;
	function octalEncode(input) {
		return Array.from(input).map((char) => char.charCodeAt(0).toString(8).padStart(3, "0")).join(" ");
	}
	_Encoding.octalEncode = octalEncode;
	function octalDecode(input) {
		return input.split(/\s+/).map((byte) => String.fromCharCode(parseInt(byte, 8))).join("");
	}
	_Encoding.octalDecode = octalDecode;
	function decimalEncode(input) {
		return Array.from(input).map((char) => char.charCodeAt(0).toString(10)).join(" ");
	}
	_Encoding.decimalEncode = decimalEncode;
	function decimalDecode(input) {
		return input.split(/\s+/).map((code) => String.fromCharCode(parseInt(code, 10))).join("");
	}
	_Encoding.decimalDecode = decimalDecode;
	function punycodeEncode(input) {
		const prefix = "xn--";
		if (input.startsWith(prefix)) return input;
		const asciiPart = input.replace(/[^\x00-\x7F]/g, "");
		const nonAsciiPart = input.replace(/[\x00-\x7F]/g, "");
		if (!nonAsciiPart) return input;
		return prefix + asciiPart + "-" + nonAsciiPart.split("").map((c) => c.charCodeAt(0).toString(36)).join("");
	}
	_Encoding.punycodeEncode = punycodeEncode;
	function slugify(input) {
		return input.toLowerCase().trim().replace(/[^\w\s-]/g, "").replace(/[\s_-]+/g, "-").replace(/^-+|-+$/g, "");
	}
	_Encoding.slugify = slugify;
	function unslugify(input) {
		return input.replace(/-/g, " ").replace(/\b\w/g, (char) => char.toUpperCase());
	}
	_Encoding.unslugify = unslugify;
	function queryStringEncode(params) {
		return Object.entries(params).filter(([, value]) => value !== void 0 && value !== null).map(([key, value]) => {
			if (Array.isArray(value)) return value.map((v) => `${urlEncode(key)}=${urlEncode(String(v))}`).join("&");
			return `${urlEncode(key)}=${urlEncode(String(value))}`;
		}).join("&");
	}
	_Encoding.queryStringEncode = queryStringEncode;
	function queryStringDecode(query) {
		const result = {};
		if (!query) return result;
		query = query.replace(/^[?#]/, "");
		for (const pair of query.split("&")) {
			const parts = pair.split("=");
			const key = parts[0];
			const value = parts[1];
			if (!key) continue;
			const decodedKey = urlDecode(key);
			const decodedValue = value ? urlDecode(value) : "";
			if (result[decodedKey]) if (Array.isArray(result[decodedKey])) result[decodedKey].push(decodedValue);
			else result[decodedKey] = [result[decodedKey], decodedValue];
			else result[decodedKey] = decodedValue;
		}
		return result;
	}
	_Encoding.queryStringDecode = queryStringDecode;
	function formDataEncode(data) {
		return Object.entries(data).filter(([, value]) => value !== void 0 && value !== null).map(([key, value]) => `${urlEncode(key)}=${urlEncode(String(value))}`).join("&");
	}
	_Encoding.formDataEncode = formDataEncode;
	function mimeTypeToExtension(mimeType) {
		return {
			"application/json": "json",
			"application/xml": "xml",
			"application/pdf": "pdf",
			"application/zip": "zip",
			"application/gzip": "gz",
			"application/x-tar": "tar",
			"application/x-rar-compressed": "rar",
			"application/x-7z-compressed": "7z",
			"application/vnd.ms-excel": "xls",
			"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet": "xlsx",
			"application/vnd.ms-powerpoint": "ppt",
			"application/vnd.openxmlformats-officedocument.presentationml.presentation": "pptx",
			"application/msword": "doc",
			"application/vnd.openxmlformats-officedocument.wordprocessingml.document": "docx",
			"text/plain": "txt",
			"text/html": "html",
			"text/css": "css",
			"text/javascript": "js",
			"text/csv": "csv",
			"text/xml": "xml",
			"image/jpeg": "jpg",
			"image/png": "png",
			"image/gif": "gif",
			"image/svg+xml": "svg",
			"image/webp": "webp",
			"image/bmp": "bmp",
			"image/tiff": "tiff",
			"image/x-icon": "ico",
			"audio/mpeg": "mp3",
			"audio/wav": "wav",
			"audio/ogg": "ogg",
			"audio/aac": "aac",
			"video/mp4": "mp4",
			"video/mpeg": "mpeg",
			"video/webm": "webm",
			"video/ogg": "ogv",
			"video/x-msvideo": "avi",
			"video/quicktime": "mov"
		}[mimeType.toLowerCase()] || "";
	}
	_Encoding.mimeTypeToExtension = mimeTypeToExtension;
	function extensionToMimeType(extension) {
		return {
			"json": "application/json",
			"xml": "application/xml",
			"pdf": "application/pdf",
			"zip": "application/zip",
			"gz": "application/gzip",
			"tar": "application/x-tar",
			"rar": "application/x-rar-compressed",
			"7z": "application/x-7z-compressed",
			"xls": "application/vnd.ms-excel",
			"xlsx": "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
			"ppt": "application/vnd.ms-powerpoint",
			"pptx": "application/vnd.openxmlformats-officedocument.presentationml.presentation",
			"doc": "application/msword",
			"docx": "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
			"txt": "text/plain",
			"html": "text/html",
			"htm": "text/html",
			"css": "text/css",
			"js": "text/javascript",
			"csv": "text/csv",
			"jpg": "image/jpeg",
			"jpeg": "image/jpeg",
			"png": "image/png",
			"gif": "image/gif",
			"svg": "image/svg+xml",
			"webp": "image/webp",
			"bmp": "image/bmp",
			"tiff": "image/tiff",
			"tif": "image/tiff",
			"ico": "image/x-icon",
			"mp3": "audio/mpeg",
			"wav": "audio/wav",
			"ogg": "audio/ogg",
			"aac": "audio/aac",
			"mp4": "video/mp4",
			"mpeg": "video/mpeg",
			"mpg": "video/mpeg",
			"webm": "video/webm",
			"ogv": "video/ogg",
			"avi": "video/x-msvideo",
			"mov": "video/quicktime"
		}[extension.toLowerCase().replace(/^\./, "")] || "application/octet-stream";
	}
	_Encoding.extensionToMimeType = extensionToMimeType;
	function charsetEncode(input, _charset) {
		return new TextEncoder().encode(input);
	}
	_Encoding.charsetEncode = charsetEncode;
	function charsetDecode(input, charset) {
		return new TextDecoder(charset).decode(input);
	}
	_Encoding.charsetDecode = charsetDecode;
	function stripBom(input) {
		if (input.charCodeAt(0) === 65279) return input.slice(1);
		return input;
	}
	_Encoding.stripBom = stripBom;
	function addBom(input, bom = "utf-8") {
		return {
			"utf-8": "﻿",
			"utf-16le": "￾",
			"utf-16be": "﻿"
		}[bom] + input;
	}
	_Encoding.addBom = addBom;
	function normalizeEncoding(input, fromEncoding, toEncoding) {
		return charsetDecode(charsetEncode(input, fromEncoding), toEncoding);
	}
	_Encoding.normalizeEncoding = normalizeEncoding;
	function isValidBase64(input) {
		if (!input || input.length % 4 !== 0) return false;
		return /^[A-Za-z0-9+/]*={0,2}$/.test(input);
	}
	_Encoding.isValidBase64 = isValidBase64;
	function isValidHex(input) {
		return /^[0-9a-fA-F]*$/.test(input) && input.length % 2 === 0;
	}
	_Encoding.isValidHex = isValidHex;
	function isValidUrl(input) {
		try {
			new URL(input);
			return true;
		} catch {
			return false;
		}
	}
	_Encoding.isValidUrl = isValidUrl;
	function isValidEmail(input) {
		return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(input);
	}
	_Encoding.isValidEmail = isValidEmail;
	function detectEncoding(input) {
		if (input.charCodeAt(0) === 65279) return "utf-8-bom";
		if (input.charCodeAt(0) === 65534) return "utf-16le";
		if (input.charCodeAt(0) === 65279 && input.charCodeAt(1) === 0) return "utf-16be";
		if (/[\u4e00-\u9fa5]/.test(input)) return "utf-8";
		return "ascii";
	}
	_Encoding.detectEncoding = detectEncoding;
})(Encoding || (Encoding = {}));
Encoding.base64Encode;
Encoding.base64Decode;
Encoding.base64UrlEncode;
Encoding.base64UrlDecode;
Encoding.utf8Encode;
Encoding.utf8Decode;
Encoding.hexEncode;
Encoding.hexDecode;
Encoding.urlEncode;
Encoding.urlDecode;
Encoding.htmlEncode;
Encoding.htmlDecode;
Encoding.jsonEncode;
Encoding.jsonDecode;
Encoding.xmlEncode;
Encoding.xmlDecode;
Encoding.escapeRegex;
Encoding.escapeSql;
Encoding.escapeShell;
Encoding.queryStringEncode;
Encoding.queryStringDecode;
Encoding.slugify;
Encoding.unslugify;
//#endregion
export { Encoding };

//# sourceMappingURL=encoding.js.map