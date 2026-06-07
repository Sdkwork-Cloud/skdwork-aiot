//#region src/utils/string.ts
var EMPTY_STRING = "";
var SPACE = " ";
var DASH = "-";
var UNDERSCORE = "_";
var DOT = ".";
var SLASH = "/";
var BACKSLASH = "\\";
var NEWLINE = "\n";
var CARRIAGE_RETURN = "\r";
var TAB = "	";
var StringUtils;
(function(_StringUtils) {
	function isEmpty(value) {
		return value === null || value === void 0 || value === "";
	}
	_StringUtils.isEmpty = isEmpty;
	function isNotEmpty(value) {
		return !isEmpty(value);
	}
	_StringUtils.isNotEmpty = isNotEmpty;
	function isBlank(value) {
		if (isEmpty(value)) return true;
		if (typeof value !== "string") return false;
		return value.trim().length === 0;
	}
	_StringUtils.isBlank = isBlank;
	function isNotBlank(value) {
		return !isBlank(value);
	}
	_StringUtils.isNotBlank = isNotBlank;
	function trim(value) {
		return value?.trim() ?? "";
	}
	_StringUtils.trim = trim;
	function trimStart(value) {
		return value?.trimStart() ?? "";
	}
	_StringUtils.trimStart = trimStart;
	function trimEnd(value) {
		return value?.trimEnd() ?? "";
	}
	_StringUtils.trimEnd = trimEnd;
	function toLowerCase(value) {
		return value?.toLowerCase() ?? "";
	}
	_StringUtils.toLowerCase = toLowerCase;
	function toUpperCase(value) {
		return value?.toUpperCase() ?? "";
	}
	_StringUtils.toUpperCase = toUpperCase;
	function capitalize(value) {
		if (isEmpty(value)) return "";
		return value.charAt(0).toUpperCase() + value.slice(1).toLowerCase();
	}
	_StringUtils.capitalize = capitalize;
	function capitalizeWords(value) {
		if (isEmpty(value)) return "";
		return value.split(/\s+/).map(capitalize).join(" ");
	}
	_StringUtils.capitalizeWords = capitalizeWords;
	function camelCase(value) {
		if (isEmpty(value)) return "";
		return value.replace(/[-_\s]+(.)?/g, (_, char) => char ? char.toUpperCase() : "").replace(/^(.)/, (char) => char.toLowerCase());
	}
	_StringUtils.camelCase = camelCase;
	function pascalCase(value) {
		if (isEmpty(value)) return "";
		const camel = camelCase(value);
		return camel.charAt(0).toUpperCase() + camel.slice(1);
	}
	_StringUtils.pascalCase = pascalCase;
	function kebabCase(value) {
		if (isEmpty(value)) return "";
		return value.replace(/([a-z])([A-Z])/g, "$1-$2").replace(/[\s_]+/g, "-").toLowerCase();
	}
	_StringUtils.kebabCase = kebabCase;
	function snakeCase(value) {
		if (isEmpty(value)) return "";
		return value.replace(/([a-z])([A-Z])/g, "$1_$2").replace(/[\s-]+/g, "_").toLowerCase();
	}
	_StringUtils.snakeCase = snakeCase;
	function constantCase(value) {
		return snakeCase(value).toUpperCase();
	}
	_StringUtils.constantCase = constantCase;
	function truncate(value, length, suffix = "...") {
		if (isEmpty(value) || value.length <= length) return value ?? "";
		return value.slice(0, length - suffix.length) + suffix;
	}
	_StringUtils.truncate = truncate;
	function truncateWords(value, wordCount, suffix = "...") {
		if (isEmpty(value)) return "";
		const words = value.split(/\s+/);
		if (words.length <= wordCount) return value;
		return words.slice(0, wordCount).join(" ") + suffix;
	}
	_StringUtils.truncateWords = truncateWords;
	function padStart(value, length, padChar = " ") {
		return value?.padStart(length, padChar) ?? "";
	}
	_StringUtils.padStart = padStart;
	function padEnd(value, length, padChar = " ") {
		return value?.padEnd(length, padChar) ?? "";
	}
	_StringUtils.padEnd = padEnd;
	function repeat(value, count) {
		if (isEmpty(value) || count <= 0) return "";
		return value.repeat(count);
	}
	_StringUtils.repeat = repeat;
	function reverse(value) {
		if (isEmpty(value)) return "";
		return value.split("").reverse().join("");
	}
	_StringUtils.reverse = reverse;
	function startsWith(value, prefix) {
		return value?.startsWith(prefix) ?? false;
	}
	_StringUtils.startsWith = startsWith;
	function endsWith(value, suffix) {
		return value?.endsWith(suffix) ?? false;
	}
	_StringUtils.endsWith = endsWith;
	function contains(value, search) {
		return value?.includes(search) ?? false;
	}
	_StringUtils.contains = contains;
	function containsIgnoreCase(value, search) {
		return value?.toLowerCase().includes(search.toLowerCase()) ?? false;
	}
	_StringUtils.containsIgnoreCase = containsIgnoreCase;
	function indexOf(value, search) {
		return value?.indexOf(search) ?? -1;
	}
	_StringUtils.indexOf = indexOf;
	function lastIndexOf(value, search) {
		return value?.lastIndexOf(search) ?? -1;
	}
	_StringUtils.lastIndexOf = lastIndexOf;
	function substring(value, start, end) {
		if (isEmpty(value)) return "";
		return end !== void 0 ? value.slice(start, end) : value.slice(start);
	}
	_StringUtils.substring = substring;
	function slice(value, start, end) {
		return substring(value, start, end);
	}
	_StringUtils.slice = slice;
	function split(value, separator, limit) {
		if (isEmpty(value)) return [];
		return value.split(separator, limit);
	}
	_StringUtils.split = split;
	function join(values, separator = "") {
		return values?.join(separator) ?? "";
	}
	_StringUtils.join = join;
	function replace(value, search, replacement) {
		return value?.replace(search, replacement) ?? "";
	}
	_StringUtils.replace = replace;
	function replaceAll(value, search, replacement) {
		return value?.replaceAll(search, replacement) ?? "";
	}
	_StringUtils.replaceAll = replaceAll;
	function remove(value, search) {
		return value?.replace(search, "") ?? "";
	}
	_StringUtils.remove = remove;
	function removeAll(value, search) {
		const regex = typeof search === "string" ? new RegExp(search, "g") : new RegExp(search.source, `${search.flags}g`);
		return value?.replace(regex, "") ?? "";
	}
	_StringUtils.removeAll = removeAll;
	function countOccurrences(value, search) {
		if (isEmpty(value) || isEmpty(search)) return 0;
		return (value.match(new RegExp(escapeRegex(search), "g")) || []).length;
	}
	_StringUtils.countOccurrences = countOccurrences;
	function escapeHtml(value) {
		const htmlEntities = {
			"&": "&amp;",
			"<": "&lt;",
			">": "&gt;",
			"\"": "&quot;",
			"'": "&#39;"
		};
		return value?.replace(/[&<>"']/g, (char) => htmlEntities[char] || char) ?? "";
	}
	_StringUtils.escapeHtml = escapeHtml;
	function unescapeHtml(value) {
		const htmlEntities = {
			"&amp;": "&",
			"&lt;": "<",
			"&gt;": ">",
			"&quot;": "\"",
			"&#39;": "'",
			"&#x27;": "'",
			"&apos;": "'"
		};
		return value?.replace(/&(?:amp|lt|gt|quot|#39|#x27|apos);/g, (entity) => htmlEntities[entity] || entity) ?? "";
	}
	_StringUtils.unescapeHtml = unescapeHtml;
	function escapeRegex(value) {
		return value?.replace(/[.*+?^${}()|[\]\\]/g, "\\$&") ?? "";
	}
	_StringUtils.escapeRegex = escapeRegex;
	function isNumeric(value) {
		if (isEmpty(value)) return false;
		return !isNaN(Number(value)) && !isNaN(parseFloat(value));
	}
	_StringUtils.isNumeric = isNumeric;
	function isAlpha(value) {
		if (isEmpty(value)) return false;
		return /^[a-zA-Z]+$/.test(value);
	}
	_StringUtils.isAlpha = isAlpha;
	function isAlphanumeric(value) {
		if (isEmpty(value)) return false;
		return /^[a-zA-Z0-9]+$/.test(value);
	}
	_StringUtils.isAlphanumeric = isAlphanumeric;
	function isHex(value) {
		if (isEmpty(value)) return false;
		return /^[0-9a-fA-F]+$/.test(value);
	}
	_StringUtils.isHex = isHex;
	function isUuid(value) {
		if (isEmpty(value)) return false;
		return /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(value);
	}
	_StringUtils.isUuid = isUuid;
	function isEmail(value) {
		if (isEmpty(value)) return false;
		return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value);
	}
	_StringUtils.isEmail = isEmail;
	function isUrl(value) {
		if (isEmpty(value)) return false;
		try {
			new URL(value);
			return true;
		} catch {
			return false;
		}
	}
	_StringUtils.isUrl = isUrl;
	function isPhoneNumber(value) {
		if (isEmpty(value)) return false;
		return /^\+?[\d\s-()]{10,}$/.test(value);
	}
	_StringUtils.isPhoneNumber = isPhoneNumber;
	function mask(value, start, end, maskChar = "*") {
		if (isEmpty(value)) return "";
		const actualStart = Math.max(0, start);
		const actualEnd = Math.min(value.length, end);
		if (actualStart >= actualEnd) return value;
		const masked = maskChar.repeat(actualEnd - actualStart);
		return value.slice(0, actualStart) + masked + value.slice(actualEnd);
	}
	_StringUtils.mask = mask;
	function maskEmail(value) {
		if (!isEmail(value)) return value;
		const parts = value.split("@");
		const localPart = parts[0];
		const domain = parts[1];
		if (!localPart || !domain) return value;
		return `${mask(localPart, 2, localPart.length - 2)}@${domain}`;
	}
	_StringUtils.maskEmail = maskEmail;
	function maskPhone(value) {
		if (isEmpty(value)) return value;
		const digits = value.replace(/\D/g, "");
		if (digits.length < 7) return value;
		return mask(digits, 3, digits.length - 4);
	}
	_StringUtils.maskPhone = maskPhone;
	function maskCreditCard(value) {
		if (isEmpty(value)) return value;
		const digits = value.replace(/\D/g, "");
		if (digits.length < 8) return value;
		return mask(digits, 4, digits.length - 4);
	}
	_StringUtils.maskCreditCard = maskCreditCard;
	function formatNumber(value, options) {
		const num = typeof value === "string" ? parseFloat(value) : value;
		if (isNaN(num)) return "";
		return num.toLocaleString(void 0, options);
	}
	_StringUtils.formatNumber = formatNumber;
	function formatCurrency(value, currency = "USD", locale) {
		const num = typeof value === "string" ? parseFloat(value) : value;
		if (isNaN(num)) return "";
		return num.toLocaleString(locale, {
			style: "currency",
			currency
		});
	}
	_StringUtils.formatCurrency = formatCurrency;
	function formatPercentage(value, decimals = 0) {
		const num = typeof value === "string" ? parseFloat(value) : value;
		if (isNaN(num)) return "";
		return `${(num * 100).toFixed(decimals)}%`;
	}
	_StringUtils.formatPercentage = formatPercentage;
	function formatBytes(bytes, decimals = 2) {
		if (bytes === 0) return "0 Bytes";
		const k = 1024;
		const sizes = [
			"Bytes",
			"KB",
			"MB",
			"GB",
			"TB",
			"PB"
		];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return `${parseFloat((bytes / Math.pow(k, i)).toFixed(decimals))} ${sizes[i]}`;
	}
	_StringUtils.formatBytes = formatBytes;
	function random(length = 16, charset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789") {
		let result = "";
		for (let i = 0; i < length; i++) result += charset.charAt(Math.floor(Math.random() * charset.length));
		return result;
	}
	_StringUtils.random = random;
	function uuid() {
		return "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace(/[xy]/g, (c) => {
			const r = Math.random() * 16 | 0;
			return (c === "x" ? r : r & 3 | 8).toString(16);
		});
	}
	_StringUtils.uuid = uuid;
	function slugify(value) {
		return value?.toLowerCase().trim().replace(/[^\w\s-]/g, "").replace(/[\s_-]+/g, "-").replace(/^-+|-+$/g, "") ?? "";
	}
	_StringUtils.slugify = slugify;
	function unslugify(value) {
		return value?.replace(/-/g, " ").replace(/\b\w/g, (char) => char.toUpperCase()) ?? "";
	}
	_StringUtils.unslugify = unslugify;
	function levenshteinDistance(a, b) {
		const matrix = [];
		for (let i = 0; i <= b.length; i++) matrix[i] = [i];
		for (let j = 0; j <= a.length; j++) if (matrix[0]) matrix[0][j] = j;
		for (let i = 1; i <= b.length; i++) for (let j = 1; j <= a.length; j++) if (b.charAt(i - 1) === a.charAt(j - 1)) matrix[i][j] = matrix[i - 1][j - 1];
		else matrix[i][j] = Math.min(matrix[i - 1][j - 1] + 1, matrix[i][j - 1] + 1, matrix[i - 1][j] + 1);
		return matrix[b.length][a.length];
	}
	_StringUtils.levenshteinDistance = levenshteinDistance;
	function similarity(a, b) {
		if (isEmpty(a) && isEmpty(b)) return 1;
		if (isEmpty(a) || isEmpty(b)) return 0;
		return 1 - levenshteinDistance(a, b) / Math.max(a.length, b.length);
	}
	_StringUtils.similarity = similarity;
	function fuzzyMatch(text, pattern, threshold = .6) {
		return similarity(text, pattern) >= threshold;
	}
	_StringUtils.fuzzyMatch = fuzzyMatch;
	function equals(a, b, ignoreCase = false) {
		if (ignoreCase) return a?.toLowerCase() === b?.toLowerCase();
		return a === b;
	}
	_StringUtils.equals = equals;
	function equalsIgnoreCase(a, b) {
		return equals(a, b, true);
	}
	_StringUtils.equalsIgnoreCase = equalsIgnoreCase;
	function wordCount(value) {
		if (isEmpty(value)) return 0;
		return value.trim().split(/\s+/).filter(Boolean).length;
	}
	_StringUtils.wordCount = wordCount;
	function characterCount(value, includeSpaces = true) {
		if (isEmpty(value)) return 0;
		return includeSpaces ? value.length : value.replace(/\s/g, "").length;
	}
	_StringUtils.characterCount = characterCount;
	function lineCount(value) {
		if (isEmpty(value)) return 0;
		return value.split(/\r?\n/).length;
	}
	_StringUtils.lineCount = lineCount;
	function splitLines(value) {
		if (isEmpty(value)) return [];
		return value.split(/\r?\n/);
	}
	_StringUtils.splitLines = splitLines;
	function words(value) {
		if (isEmpty(value)) return [];
		return value.trim().split(/\s+/).filter(Boolean);
	}
	_StringUtils.words = words;
	function charAt(value, index) {
		return value?.charAt(index) ?? "";
	}
	_StringUtils.charAt = charAt;
	function charCodeAt(value, index) {
		return value?.charCodeAt(index) ?? NaN;
	}
	_StringUtils.charCodeAt = charCodeAt;
	function fromCharCode(...codes) {
		return String.fromCharCode(...codes);
	}
	_StringUtils.fromCharCode = fromCharCode;
	function insert(value, index, insertValue) {
		if (isEmpty(value)) return insertValue;
		return value.slice(0, index) + insertValue + value.slice(index);
	}
	_StringUtils.insert = insert;
	function swapCase(value) {
		return value?.replace(/[a-zA-Z]/g, (char) => {
			return char === char.toUpperCase() ? char.toLowerCase() : char.toUpperCase();
		}) ?? "";
	}
	_StringUtils.swapCase = swapCase;
	function surround(value, wrapper) {
		return `${wrapper}${value}${wrapper}`;
	}
	_StringUtils.surround = surround;
	function quote(value, quoteChar = "\"") {
		return `${quoteChar}${value}${quoteChar}`;
	}
	_StringUtils.quote = quote;
	function unquote(value) {
		if (isEmpty(value)) return "";
		if (value.startsWith("\"") && value.endsWith("\"") || value.startsWith("'") && value.endsWith("'") || value.startsWith("`") && value.endsWith("`")) return value.slice(1, -1);
		return value;
	}
	_StringUtils.unquote = unquote;
	function wrap(value, prefix, suffix = prefix) {
		return `${prefix}${value}${suffix}`;
	}
	_StringUtils.wrap = wrap;
	function unwrap(value, prefix, suffix = prefix) {
		if (isEmpty(value)) return "";
		if (value.startsWith(prefix) && value.endsWith(suffix)) return value.slice(prefix.length, -suffix.length);
		return value;
	}
	_StringUtils.unwrap = unwrap;
	function template(templateStr, values) {
		return templateStr?.replace(/\{\{(\w+)\}\}/g, (_, key) => String(values[key] ?? "")) ?? "";
	}
	_StringUtils.template = template;
	function interpolate(templateStr, values) {
		return template(templateStr, values);
	}
	_StringUtils.interpolate = interpolate;
	function dedent(value) {
		const lines = value.split("\n");
		const minIndent = Math.min(...lines.filter((line) => line.trim().length > 0).map((line) => line.match(/^\s*/)?.[0].length ?? 0));
		return lines.map((line) => line.slice(minIndent)).join("\n");
	}
	_StringUtils.dedent = dedent;
	function indent(value, spaces = 2) {
		const indentation = " ".repeat(spaces);
		return value.split("\n").map((line) => indentation + line).join("\n");
	}
	_StringUtils.indent = indent;
	function center(value, width, padChar = " ") {
		if (isEmpty(value) || value.length >= width) return value ?? "";
		const padding = width - value.length;
		const leftPad = Math.floor(padding / 2);
		const rightPad = padding - leftPad;
		return padChar.repeat(leftPad) + value + padChar.repeat(rightPad);
	}
	_StringUtils.center = center;
	function alignLeft(value, width, padChar = " ") {
		return padEnd(value, width, padChar);
	}
	_StringUtils.alignLeft = alignLeft;
	function alignRight(value, width, padChar = " ") {
		return padStart(value, width, padChar);
	}
	_StringUtils.alignRight = alignRight;
	function alignCenter(value, width, padChar = " ") {
		return center(value, width, padChar);
	}
	_StringUtils.alignCenter = alignCenter;
	function toBoolean(value) {
		return [
			"true",
			"1",
			"yes",
			"on",
			"y"
		].includes(value?.toLowerCase().trim());
	}
	_StringUtils.toBoolean = toBoolean;
	function toNumber(value, defaultValue = 0) {
		const num = parseFloat(value);
		return isNaN(num) ? defaultValue : num;
	}
	_StringUtils.toNumber = toNumber;
	function toArray(value, separator = ",") {
		return split(value, separator);
	}
	_StringUtils.toArray = toArray;
	function hashCode(value) {
		let hash = 0;
		for (let i = 0; i < value.length; i++) {
			const char = value.charCodeAt(i);
			hash = (hash << 5) - hash + char;
			hash = hash & hash;
		}
		return hash;
	}
	_StringUtils.hashCode = hashCode;
	function isPalindrome(value) {
		const cleaned = value.toLowerCase().replace(/[^a-z0-9]/g, "");
		return cleaned === cleaned.split("").reverse().join("");
	}
	_StringUtils.isPalindrome = isPalindrome;
	function isAnagram(a, b) {
		const normalize = (s) => s.toLowerCase().replace(/[^a-z0-9]/g, "").split("").sort().join("");
		return normalize(a) === normalize(b);
	}
	_StringUtils.isAnagram = isAnagram;
	function reverseWords(value) {
		return value?.split(/\s+/).reverse().join(" ") ?? "";
	}
	_StringUtils.reverseWords = reverseWords;
	function sortCharacters(value) {
		return value?.split("").sort().join("") ?? "";
	}
	_StringUtils.sortCharacters = sortCharacters;
	function uniqueCharacters(value) {
		return [...new Set(value)].join("");
	}
	_StringUtils.uniqueCharacters = uniqueCharacters;
	function removeDuplicates(value) {
		return value?.split("").filter((char, index, arr) => arr.indexOf(char) === index).join("") ?? "";
	}
	_StringUtils.removeDuplicates = removeDuplicates;
	function longestCommonSubstring(a, b) {
		if (isEmpty(a) || isEmpty(b)) return "";
		const matrix = Array(a.length + 1).fill(null).map(() => Array(b.length + 1).fill(0));
		let maxLength = 0;
		let endIndex = 0;
		for (let i = 1; i <= a.length; i++) for (let j = 1; j <= b.length; j++) if (a[i - 1] === b[j - 1]) {
			matrix[i][j] = matrix[i - 1][j - 1] + 1;
			if (matrix[i][j] > maxLength) {
				maxLength = matrix[i][j];
				endIndex = i;
			}
		}
		return a.slice(endIndex - maxLength, endIndex);
	}
	_StringUtils.longestCommonSubstring = longestCommonSubstring;
	function longestCommonPrefix(strings) {
		if (strings.length === 0) return "";
		if (strings.length === 1) return strings[0] ?? "";
		const sorted = [...strings].sort();
		const first = sorted[0] ?? "";
		const last = sorted[sorted.length - 1] ?? "";
		let i = 0;
		while (i < first.length && first[i] === last[i]) i++;
		return first.slice(0, i);
	}
	_StringUtils.longestCommonPrefix = longestCommonPrefix;
	function longestCommonSuffix(strings) {
		return longestCommonPrefix(strings.map((s) => s?.split("").reverse().join("") ?? "")).split("").reverse().join("");
	}
	_StringUtils.longestCommonSuffix = longestCommonSuffix;
	function truncateMiddle(value, maxLength, separator = "...") {
		if (isEmpty(value) || value.length <= maxLength) return value ?? "";
		const charsToShow = maxLength - separator.length;
		const frontChars = Math.ceil(charsToShow / 2);
		const backChars = Math.floor(charsToShow / 2);
		return value.slice(0, frontChars) + separator + value.slice(-backChars);
	}
	_StringUtils.truncateMiddle = truncateMiddle;
	function ellipsis(value, maxLength) {
		return truncate(value, maxLength, "...");
	}
	_StringUtils.ellipsis = ellipsis;
	function ellipsisMiddle(value, maxLength) {
		return truncateMiddle(value, maxLength, "...");
	}
	_StringUtils.ellipsisMiddle = ellipsisMiddle;
	function pad(value, length, padChar = " ") {
		return center(value, length, padChar);
	}
	_StringUtils.pad = pad;
	function padCenter(value, length, padChar = " ") {
		return center(value, length, padChar);
	}
	_StringUtils.padCenter = padCenter;
	function isAscii(value) {
		return /^[\x00-\x7F]*$/.test(value);
	}
	_StringUtils.isAscii = isAscii;
	function isLowerCase(value) {
		return value === value.toLowerCase();
	}
	_StringUtils.isLowerCase = isLowerCase;
	function isUpperCase(value) {
		return value === value.toUpperCase();
	}
	_StringUtils.isUpperCase = isUpperCase;
	function isCapitalized(value) {
		return value.charAt(0) === value.charAt(0).toUpperCase();
	}
	_StringUtils.isCapitalized = isCapitalized;
	function swapPrefix(value, oldPrefix, newPrefix) {
		if (value.startsWith(oldPrefix)) return newPrefix + value.slice(oldPrefix.length);
		return value;
	}
	_StringUtils.swapPrefix = swapPrefix;
	function swapSuffix(value, oldSuffix, newSuffix) {
		if (value.endsWith(oldSuffix)) return value.slice(0, -oldSuffix.length) + newSuffix;
		return value;
	}
	_StringUtils.swapSuffix = swapSuffix;
	function ensurePrefix(value, prefix) {
		return value.startsWith(prefix) ? value : prefix + value;
	}
	_StringUtils.ensurePrefix = ensurePrefix;
	function ensureSuffix(value, suffix) {
		return value.endsWith(suffix) ? value : value + suffix;
	}
	_StringUtils.ensureSuffix = ensureSuffix;
	function removePrefix(value, prefix) {
		return value.startsWith(prefix) ? value.slice(prefix.length) : value;
	}
	_StringUtils.removePrefix = removePrefix;
	function removeSuffix(value, suffix) {
		return value.endsWith(suffix) ? value.slice(0, -suffix.length) : value;
	}
	_StringUtils.removeSuffix = removeSuffix;
	function take(value, n) {
		return value?.slice(0, n) ?? "";
	}
	_StringUtils.take = take;
	function takeRight(value, n) {
		return value?.slice(-n) ?? "";
	}
	_StringUtils.takeRight = takeRight;
	function takeWhile(value, predicate) {
		let result = "";
		for (const char of value ?? "") {
			if (!predicate(char)) break;
			result += char;
		}
		return result;
	}
	_StringUtils.takeWhile = takeWhile;
	function takeRightWhile(value, predicate) {
		let result = "";
		for (let i = (value?.length ?? 0) - 1; i >= 0; i--) {
			const char = value?.charAt(i) ?? "";
			if (!predicate(char)) break;
			result = char + result;
		}
		return result;
	}
	_StringUtils.takeRightWhile = takeRightWhile;
	function drop(value, n) {
		return value?.slice(n) ?? "";
	}
	_StringUtils.drop = drop;
	function dropRight(value, n) {
		return value?.slice(0, -n) ?? "";
	}
	_StringUtils.dropRight = dropRight;
	function dropWhile(value, predicate) {
		let i = 0;
		for (const char of value ?? "") {
			if (!predicate(char)) break;
			i++;
		}
		return value?.slice(i) ?? "";
	}
	_StringUtils.dropWhile = dropWhile;
	function dropRightWhile(value, predicate) {
		let i = (value?.length ?? 0) - 1;
		while (i >= 0 && predicate(value?.charAt(i) ?? "")) i--;
		return value?.slice(0, i + 1) ?? "";
	}
	_StringUtils.dropRightWhile = dropRightWhile;
	function countLines(value) {
		return lineCount(value);
	}
	_StringUtils.countLines = countLines;
	function getLine(value, lineNumber) {
		return splitLines(value)[lineNumber] ?? "";
	}
	_StringUtils.getLine = getLine;
	function getLines(value) {
		return splitLines(value);
	}
	_StringUtils.getLines = getLines;
	function isSingleLine(value) {
		return !value?.includes("\n");
	}
	_StringUtils.isSingleLine = isSingleLine;
	function isMultiLine(value) {
		return value?.includes("\n") ?? false;
	}
	_StringUtils.isMultiLine = isMultiLine;
	function normalizeLineEndings(value, lineEnding = "\n") {
		return value?.replace(/\r\n|\r|\n/g, lineEnding) ?? "";
	}
	_StringUtils.normalizeLineEndings = normalizeLineEndings;
	function toCamelCase(value) {
		return camelCase(value);
	}
	_StringUtils.toCamelCase = toCamelCase;
	function toKebabCase(value) {
		return kebabCase(value);
	}
	_StringUtils.toKebabCase = toKebabCase;
	function toSnakeCase(value) {
		return snakeCase(value);
	}
	_StringUtils.toSnakeCase = toSnakeCase;
	function toPascalCase(value) {
		return pascalCase(value);
	}
	_StringUtils.toPascalCase = toPascalCase;
	function toConstantCase(value) {
		return constantCase(value);
	}
	_StringUtils.toConstantCase = toConstantCase;
	function toSentenceCase(value) {
		if (isEmpty(value)) return "";
		return value.charAt(0).toUpperCase() + value.slice(1).toLowerCase();
	}
	_StringUtils.toSentenceCase = toSentenceCase;
	function toTitleCase(value) {
		return capitalizeWords(value);
	}
	_StringUtils.toTitleCase = toTitleCase;
	function toCapitalCase(value) {
		return capitalizeWords(value);
	}
	_StringUtils.toCapitalCase = toCapitalCase;
	function toDotCase(value) {
		return value?.replace(/([a-z])([A-Z])/g, "$1.$2").replace(/[-_\s]+/g, ".").toLowerCase() ?? "";
	}
	_StringUtils.toDotCase = toDotCase;
	function toPathCase(value) {
		return value?.replace(/([a-z])([A-Z])/g, "$1/$2").replace(/[-_\s]+/g, "/").toLowerCase() ?? "";
	}
	_StringUtils.toPathCase = toPathCase;
	function stripTags(value) {
		return value?.replace(/<[^>]*>/g, "") ?? "";
	}
	_StringUtils.stripTags = stripTags;
	function stripNumbers(value) {
		return value?.replace(/\d+/g, "") ?? "";
	}
	_StringUtils.stripNumbers = stripNumbers;
	function stripWhitespace(value) {
		return value?.replace(/\s+/g, "") ?? "";
	}
	_StringUtils.stripWhitespace = stripWhitespace;
	function stripPunctuation(value) {
		return value?.replace(/[^\w\s]/g, "") ?? "";
	}
	_StringUtils.stripPunctuation = stripPunctuation;
	function normalizeWhitespace(value) {
		return value?.replace(/\s+/g, " ").trim() ?? "";
	}
	_StringUtils.normalizeWhitespace = normalizeWhitespace;
	function includesAll(value, searches) {
		return searches.every((search) => value?.includes(search) ?? false);
	}
	_StringUtils.includesAll = includesAll;
	function includesAny(value, searches) {
		return searches.some((search) => value?.includes(search) ?? false);
	}
	_StringUtils.includesAny = includesAny;
})(StringUtils || (StringUtils = {}));
//#endregion
export { BACKSLASH, CARRIAGE_RETURN, DASH, DOT, EMPTY_STRING, NEWLINE, SLASH, SPACE, StringUtils, TAB, UNDERSCORE };

//# sourceMappingURL=string.js.map