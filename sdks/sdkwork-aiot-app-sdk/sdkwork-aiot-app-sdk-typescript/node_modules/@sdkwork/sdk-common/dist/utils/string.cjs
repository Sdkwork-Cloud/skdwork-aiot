"use strict";
Object.defineProperty(exports, Symbol.toStringTag, { value: "Module" });
const EMPTY_STRING = "";
const SPACE = " ";
const DASH = "-";
const UNDERSCORE = "_";
const DOT = ".";
const SLASH = "/";
const BACKSLASH = "\\";
const NEWLINE = "\n";
const CARRIAGE_RETURN = "\r";
const TAB = "	";
exports.StringUtils = void 0;
((StringUtils2) => {
  function isEmpty(value) {
    return value === null || value === void 0 || value === "";
  }
  StringUtils2.isEmpty = isEmpty;
  function isNotEmpty(value) {
    return !isEmpty(value);
  }
  StringUtils2.isNotEmpty = isNotEmpty;
  function isBlank(value) {
    if (isEmpty(value)) return true;
    if (typeof value !== "string") return false;
    return value.trim().length === 0;
  }
  StringUtils2.isBlank = isBlank;
  function isNotBlank(value) {
    return !isBlank(value);
  }
  StringUtils2.isNotBlank = isNotBlank;
  function trim(value) {
    return value?.trim() ?? "";
  }
  StringUtils2.trim = trim;
  function trimStart(value) {
    return value?.trimStart() ?? "";
  }
  StringUtils2.trimStart = trimStart;
  function trimEnd(value) {
    return value?.trimEnd() ?? "";
  }
  StringUtils2.trimEnd = trimEnd;
  function toLowerCase(value) {
    return value?.toLowerCase() ?? "";
  }
  StringUtils2.toLowerCase = toLowerCase;
  function toUpperCase(value) {
    return value?.toUpperCase() ?? "";
  }
  StringUtils2.toUpperCase = toUpperCase;
  function capitalize(value) {
    if (isEmpty(value)) return "";
    return value.charAt(0).toUpperCase() + value.slice(1).toLowerCase();
  }
  StringUtils2.capitalize = capitalize;
  function capitalizeWords(value) {
    if (isEmpty(value)) return "";
    return value.split(/\s+/).map(capitalize).join(" ");
  }
  StringUtils2.capitalizeWords = capitalizeWords;
  function camelCase(value) {
    if (isEmpty(value)) return "";
    return value.replace(/[-_\s]+(.)?/g, (_, char) => char ? char.toUpperCase() : "").replace(/^(.)/, (char) => char.toLowerCase());
  }
  StringUtils2.camelCase = camelCase;
  function pascalCase(value) {
    if (isEmpty(value)) return "";
    const camel = camelCase(value);
    return camel.charAt(0).toUpperCase() + camel.slice(1);
  }
  StringUtils2.pascalCase = pascalCase;
  function kebabCase(value) {
    if (isEmpty(value)) return "";
    return value.replace(/([a-z])([A-Z])/g, "$1-$2").replace(/[\s_]+/g, "-").toLowerCase();
  }
  StringUtils2.kebabCase = kebabCase;
  function snakeCase(value) {
    if (isEmpty(value)) return "";
    return value.replace(/([a-z])([A-Z])/g, "$1_$2").replace(/[\s-]+/g, "_").toLowerCase();
  }
  StringUtils2.snakeCase = snakeCase;
  function constantCase(value) {
    return snakeCase(value).toUpperCase();
  }
  StringUtils2.constantCase = constantCase;
  function truncate(value, length, suffix = "...") {
    if (isEmpty(value) || value.length <= length) return value ?? "";
    return value.slice(0, length - suffix.length) + suffix;
  }
  StringUtils2.truncate = truncate;
  function truncateWords(value, wordCount2, suffix = "...") {
    if (isEmpty(value)) return "";
    const words2 = value.split(/\s+/);
    if (words2.length <= wordCount2) return value;
    return words2.slice(0, wordCount2).join(" ") + suffix;
  }
  StringUtils2.truncateWords = truncateWords;
  function padStart(value, length, padChar = " ") {
    return value?.padStart(length, padChar) ?? "";
  }
  StringUtils2.padStart = padStart;
  function padEnd(value, length, padChar = " ") {
    return value?.padEnd(length, padChar) ?? "";
  }
  StringUtils2.padEnd = padEnd;
  function repeat(value, count) {
    if (isEmpty(value) || count <= 0) return "";
    return value.repeat(count);
  }
  StringUtils2.repeat = repeat;
  function reverse(value) {
    if (isEmpty(value)) return "";
    return value.split("").reverse().join("");
  }
  StringUtils2.reverse = reverse;
  function startsWith(value, prefix) {
    return value?.startsWith(prefix) ?? false;
  }
  StringUtils2.startsWith = startsWith;
  function endsWith(value, suffix) {
    return value?.endsWith(suffix) ?? false;
  }
  StringUtils2.endsWith = endsWith;
  function contains(value, search) {
    return value?.includes(search) ?? false;
  }
  StringUtils2.contains = contains;
  function containsIgnoreCase(value, search) {
    return value?.toLowerCase().includes(search.toLowerCase()) ?? false;
  }
  StringUtils2.containsIgnoreCase = containsIgnoreCase;
  function indexOf(value, search) {
    return value?.indexOf(search) ?? -1;
  }
  StringUtils2.indexOf = indexOf;
  function lastIndexOf(value, search) {
    return value?.lastIndexOf(search) ?? -1;
  }
  StringUtils2.lastIndexOf = lastIndexOf;
  function substring(value, start, end) {
    if (isEmpty(value)) return "";
    return end !== void 0 ? value.slice(start, end) : value.slice(start);
  }
  StringUtils2.substring = substring;
  function slice(value, start, end) {
    return substring(value, start, end);
  }
  StringUtils2.slice = slice;
  function split(value, separator, limit) {
    if (isEmpty(value)) return [];
    return value.split(separator, limit);
  }
  StringUtils2.split = split;
  function join(values, separator = "") {
    return values?.join(separator) ?? "";
  }
  StringUtils2.join = join;
  function replace(value, search, replacement) {
    return value?.replace(search, replacement) ?? "";
  }
  StringUtils2.replace = replace;
  function replaceAll(value, search, replacement) {
    return value?.replaceAll(search, replacement) ?? "";
  }
  StringUtils2.replaceAll = replaceAll;
  function remove(value, search) {
    return value?.replace(search, "") ?? "";
  }
  StringUtils2.remove = remove;
  function removeAll(value, search) {
    const regex = typeof search === "string" ? new RegExp(search, "g") : new RegExp(search.source, `${search.flags}g`);
    return value?.replace(regex, "") ?? "";
  }
  StringUtils2.removeAll = removeAll;
  function countOccurrences(value, search) {
    if (isEmpty(value) || isEmpty(search)) return 0;
    return (value.match(new RegExp(escapeRegex(search), "g")) || []).length;
  }
  StringUtils2.countOccurrences = countOccurrences;
  function escapeHtml(value) {
    const htmlEntities = {
      "&": "&amp;",
      "<": "&lt;",
      ">": "&gt;",
      '"': "&quot;",
      "'": "&#39;"
    };
    return value?.replace(/[&<>"']/g, (char) => htmlEntities[char] || char) ?? "";
  }
  StringUtils2.escapeHtml = escapeHtml;
  function unescapeHtml(value) {
    const htmlEntities = {
      "&amp;": "&",
      "&lt;": "<",
      "&gt;": ">",
      "&quot;": '"',
      "&#39;": "'",
      "&#x27;": "'",
      "&apos;": "'"
    };
    return value?.replace(/&(?:amp|lt|gt|quot|#39|#x27|apos);/g, (entity) => htmlEntities[entity] || entity) ?? "";
  }
  StringUtils2.unescapeHtml = unescapeHtml;
  function escapeRegex(value) {
    return value?.replace(/[.*+?^${}()|[\]\\]/g, "\\$&") ?? "";
  }
  StringUtils2.escapeRegex = escapeRegex;
  function isNumeric(value) {
    if (isEmpty(value)) return false;
    return !isNaN(Number(value)) && !isNaN(parseFloat(value));
  }
  StringUtils2.isNumeric = isNumeric;
  function isAlpha(value) {
    if (isEmpty(value)) return false;
    return /^[a-zA-Z]+$/.test(value);
  }
  StringUtils2.isAlpha = isAlpha;
  function isAlphanumeric(value) {
    if (isEmpty(value)) return false;
    return /^[a-zA-Z0-9]+$/.test(value);
  }
  StringUtils2.isAlphanumeric = isAlphanumeric;
  function isHex(value) {
    if (isEmpty(value)) return false;
    return /^[0-9a-fA-F]+$/.test(value);
  }
  StringUtils2.isHex = isHex;
  function isUuid(value) {
    if (isEmpty(value)) return false;
    return /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(value);
  }
  StringUtils2.isUuid = isUuid;
  function isEmail(value) {
    if (isEmpty(value)) return false;
    return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value);
  }
  StringUtils2.isEmail = isEmail;
  function isUrl(value) {
    if (isEmpty(value)) return false;
    try {
      new URL(value);
      return true;
    } catch {
      return false;
    }
  }
  StringUtils2.isUrl = isUrl;
  function isPhoneNumber(value) {
    if (isEmpty(value)) return false;
    return /^\+?[\d\s-()]{10,}$/.test(value);
  }
  StringUtils2.isPhoneNumber = isPhoneNumber;
  function mask(value, start, end, maskChar = "*") {
    if (isEmpty(value)) return "";
    const actualStart = Math.max(0, start);
    const actualEnd = Math.min(value.length, end);
    if (actualStart >= actualEnd) return value;
    const masked = maskChar.repeat(actualEnd - actualStart);
    return value.slice(0, actualStart) + masked + value.slice(actualEnd);
  }
  StringUtils2.mask = mask;
  function maskEmail(value) {
    if (!isEmail(value)) return value;
    const parts = value.split("@");
    const localPart = parts[0];
    const domain = parts[1];
    if (!localPart || !domain) return value;
    const maskedLocal = mask(localPart, 2, localPart.length - 2);
    return `${maskedLocal}@${domain}`;
  }
  StringUtils2.maskEmail = maskEmail;
  function maskPhone(value) {
    if (isEmpty(value)) return value;
    const digits = value.replace(/\D/g, "");
    if (digits.length < 7) return value;
    return mask(digits, 3, digits.length - 4);
  }
  StringUtils2.maskPhone = maskPhone;
  function maskCreditCard(value) {
    if (isEmpty(value)) return value;
    const digits = value.replace(/\D/g, "");
    if (digits.length < 8) return value;
    return mask(digits, 4, digits.length - 4);
  }
  StringUtils2.maskCreditCard = maskCreditCard;
  function formatNumber(value, options) {
    const num = typeof value === "string" ? parseFloat(value) : value;
    if (isNaN(num)) return "";
    return num.toLocaleString(void 0, options);
  }
  StringUtils2.formatNumber = formatNumber;
  function formatCurrency(value, currency = "USD", locale) {
    const num = typeof value === "string" ? parseFloat(value) : value;
    if (isNaN(num)) return "";
    return num.toLocaleString(locale, { style: "currency", currency });
  }
  StringUtils2.formatCurrency = formatCurrency;
  function formatPercentage(value, decimals = 0) {
    const num = typeof value === "string" ? parseFloat(value) : value;
    if (isNaN(num)) return "";
    return `${(num * 100).toFixed(decimals)}%`;
  }
  StringUtils2.formatPercentage = formatPercentage;
  function formatBytes(bytes, decimals = 2) {
    if (bytes === 0) return "0 Bytes";
    const k = 1024;
    const sizes = ["Bytes", "KB", "MB", "GB", "TB", "PB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(decimals))} ${sizes[i]}`;
  }
  StringUtils2.formatBytes = formatBytes;
  function random(length = 16, charset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789") {
    let result = "";
    for (let i = 0; i < length; i++) {
      result += charset.charAt(Math.floor(Math.random() * charset.length));
    }
    return result;
  }
  StringUtils2.random = random;
  function uuid() {
    return "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace(/[xy]/g, (c) => {
      const r = Math.random() * 16 | 0;
      const v = c === "x" ? r : r & 3 | 8;
      return v.toString(16);
    });
  }
  StringUtils2.uuid = uuid;
  function slugify(value) {
    return value?.toLowerCase().trim().replace(/[^\w\s-]/g, "").replace(/[\s_-]+/g, "-").replace(/^-+|-+$/g, "") ?? "";
  }
  StringUtils2.slugify = slugify;
  function unslugify(value) {
    return value?.replace(/-/g, " ").replace(/\b\w/g, (char) => char.toUpperCase()) ?? "";
  }
  StringUtils2.unslugify = unslugify;
  function levenshteinDistance(a, b) {
    const matrix = [];
    for (let i = 0; i <= b.length; i++) {
      matrix[i] = [i];
    }
    for (let j = 0; j <= a.length; j++) {
      if (matrix[0]) matrix[0][j] = j;
    }
    for (let i = 1; i <= b.length; i++) {
      for (let j = 1; j <= a.length; j++) {
        if (b.charAt(i - 1) === a.charAt(j - 1)) {
          matrix[i][j] = matrix[i - 1][j - 1];
        } else {
          matrix[i][j] = Math.min(
            matrix[i - 1][j - 1] + 1,
            matrix[i][j - 1] + 1,
            matrix[i - 1][j] + 1
          );
        }
      }
    }
    return matrix[b.length][a.length];
  }
  StringUtils2.levenshteinDistance = levenshteinDistance;
  function similarity(a, b) {
    if (isEmpty(a) && isEmpty(b)) return 1;
    if (isEmpty(a) || isEmpty(b)) return 0;
    const distance = levenshteinDistance(a, b);
    const maxLength = Math.max(a.length, b.length);
    return 1 - distance / maxLength;
  }
  StringUtils2.similarity = similarity;
  function fuzzyMatch(text, pattern, threshold = 0.6) {
    return similarity(text, pattern) >= threshold;
  }
  StringUtils2.fuzzyMatch = fuzzyMatch;
  function equals(a, b, ignoreCase = false) {
    if (ignoreCase) {
      return a?.toLowerCase() === b?.toLowerCase();
    }
    return a === b;
  }
  StringUtils2.equals = equals;
  function equalsIgnoreCase(a, b) {
    return equals(a, b, true);
  }
  StringUtils2.equalsIgnoreCase = equalsIgnoreCase;
  function wordCount(value) {
    if (isEmpty(value)) return 0;
    return value.trim().split(/\s+/).filter(Boolean).length;
  }
  StringUtils2.wordCount = wordCount;
  function characterCount(value, includeSpaces = true) {
    if (isEmpty(value)) return 0;
    return includeSpaces ? value.length : value.replace(/\s/g, "").length;
  }
  StringUtils2.characterCount = characterCount;
  function lineCount(value) {
    if (isEmpty(value)) return 0;
    return value.split(/\r?\n/).length;
  }
  StringUtils2.lineCount = lineCount;
  function splitLines(value) {
    if (isEmpty(value)) return [];
    return value.split(/\r?\n/);
  }
  StringUtils2.splitLines = splitLines;
  function words(value) {
    if (isEmpty(value)) return [];
    return value.trim().split(/\s+/).filter(Boolean);
  }
  StringUtils2.words = words;
  function charAt(value, index) {
    return value?.charAt(index) ?? "";
  }
  StringUtils2.charAt = charAt;
  function charCodeAt(value, index) {
    return value?.charCodeAt(index) ?? NaN;
  }
  StringUtils2.charCodeAt = charCodeAt;
  function fromCharCode(...codes) {
    return String.fromCharCode(...codes);
  }
  StringUtils2.fromCharCode = fromCharCode;
  function insert(value, index, insertValue) {
    if (isEmpty(value)) return insertValue;
    return value.slice(0, index) + insertValue + value.slice(index);
  }
  StringUtils2.insert = insert;
  function swapCase(value) {
    return value?.replace(/[a-zA-Z]/g, (char) => {
      return char === char.toUpperCase() ? char.toLowerCase() : char.toUpperCase();
    }) ?? "";
  }
  StringUtils2.swapCase = swapCase;
  function surround(value, wrapper) {
    return `${wrapper}${value}${wrapper}`;
  }
  StringUtils2.surround = surround;
  function quote(value, quoteChar = '"') {
    return `${quoteChar}${value}${quoteChar}`;
  }
  StringUtils2.quote = quote;
  function unquote(value) {
    if (isEmpty(value)) return "";
    if (value.startsWith('"') && value.endsWith('"') || value.startsWith("'") && value.endsWith("'") || value.startsWith("`") && value.endsWith("`")) {
      return value.slice(1, -1);
    }
    return value;
  }
  StringUtils2.unquote = unquote;
  function wrap(value, prefix, suffix = prefix) {
    return `${prefix}${value}${suffix}`;
  }
  StringUtils2.wrap = wrap;
  function unwrap(value, prefix, suffix = prefix) {
    if (isEmpty(value)) return "";
    if (value.startsWith(prefix) && value.endsWith(suffix)) {
      return value.slice(prefix.length, -suffix.length);
    }
    return value;
  }
  StringUtils2.unwrap = unwrap;
  function template(templateStr, values) {
    return templateStr?.replace(/\{\{(\w+)\}\}/g, (_, key) => String(values[key] ?? "")) ?? "";
  }
  StringUtils2.template = template;
  function interpolate(templateStr, values) {
    return template(templateStr, values);
  }
  StringUtils2.interpolate = interpolate;
  function dedent(value) {
    const lines = value.split("\n");
    const minIndent = Math.min(
      ...lines.filter((line) => line.trim().length > 0).map((line) => line.match(/^\s*/)?.[0].length ?? 0)
    );
    return lines.map((line) => line.slice(minIndent)).join("\n");
  }
  StringUtils2.dedent = dedent;
  function indent(value, spaces = 2) {
    const indentation = " ".repeat(spaces);
    return value.split("\n").map((line) => indentation + line).join("\n");
  }
  StringUtils2.indent = indent;
  function center(value, width, padChar = " ") {
    if (isEmpty(value) || value.length >= width) return value ?? "";
    const padding = width - value.length;
    const leftPad = Math.floor(padding / 2);
    const rightPad = padding - leftPad;
    return padChar.repeat(leftPad) + value + padChar.repeat(rightPad);
  }
  StringUtils2.center = center;
  function alignLeft(value, width, padChar = " ") {
    return padEnd(value, width, padChar);
  }
  StringUtils2.alignLeft = alignLeft;
  function alignRight(value, width, padChar = " ") {
    return padStart(value, width, padChar);
  }
  StringUtils2.alignRight = alignRight;
  function alignCenter(value, width, padChar = " ") {
    return center(value, width, padChar);
  }
  StringUtils2.alignCenter = alignCenter;
  function toBoolean(value) {
    const truthy = ["true", "1", "yes", "on", "y"];
    return truthy.includes(value?.toLowerCase().trim());
  }
  StringUtils2.toBoolean = toBoolean;
  function toNumber(value, defaultValue = 0) {
    const num = parseFloat(value);
    return isNaN(num) ? defaultValue : num;
  }
  StringUtils2.toNumber = toNumber;
  function toArray(value, separator = ",") {
    return split(value, separator);
  }
  StringUtils2.toArray = toArray;
  function hashCode(value) {
    let hash = 0;
    for (let i = 0; i < value.length; i++) {
      const char = value.charCodeAt(i);
      hash = (hash << 5) - hash + char;
      hash = hash & hash;
    }
    return hash;
  }
  StringUtils2.hashCode = hashCode;
  function isPalindrome(value) {
    const cleaned = value.toLowerCase().replace(/[^a-z0-9]/g, "");
    return cleaned === cleaned.split("").reverse().join("");
  }
  StringUtils2.isPalindrome = isPalindrome;
  function isAnagram(a, b) {
    const normalize = (s) => s.toLowerCase().replace(/[^a-z0-9]/g, "").split("").sort().join("");
    return normalize(a) === normalize(b);
  }
  StringUtils2.isAnagram = isAnagram;
  function reverseWords(value) {
    return value?.split(/\s+/).reverse().join(" ") ?? "";
  }
  StringUtils2.reverseWords = reverseWords;
  function sortCharacters(value) {
    return value?.split("").sort().join("") ?? "";
  }
  StringUtils2.sortCharacters = sortCharacters;
  function uniqueCharacters(value) {
    return [...new Set(value)].join("");
  }
  StringUtils2.uniqueCharacters = uniqueCharacters;
  function removeDuplicates(value) {
    return value?.split("").filter((char, index, arr) => arr.indexOf(char) === index).join("") ?? "";
  }
  StringUtils2.removeDuplicates = removeDuplicates;
  function longestCommonSubstring(a, b) {
    if (isEmpty(a) || isEmpty(b)) return "";
    const matrix = Array(a.length + 1).fill(null).map(() => Array(b.length + 1).fill(0));
    let maxLength = 0;
    let endIndex = 0;
    for (let i = 1; i <= a.length; i++) {
      for (let j = 1; j <= b.length; j++) {
        if (a[i - 1] === b[j - 1]) {
          matrix[i][j] = matrix[i - 1][j - 1] + 1;
          if (matrix[i][j] > maxLength) {
            maxLength = matrix[i][j];
            endIndex = i;
          }
        }
      }
    }
    return a.slice(endIndex - maxLength, endIndex);
  }
  StringUtils2.longestCommonSubstring = longestCommonSubstring;
  function longestCommonPrefix(strings) {
    if (strings.length === 0) return "";
    if (strings.length === 1) return strings[0] ?? "";
    const sorted = [...strings].sort();
    const first = sorted[0] ?? "";
    const last = sorted[sorted.length - 1] ?? "";
    let i = 0;
    while (i < first.length && first[i] === last[i]) {
      i++;
    }
    return first.slice(0, i);
  }
  StringUtils2.longestCommonPrefix = longestCommonPrefix;
  function longestCommonSuffix(strings) {
    const reversed = strings.map((s) => s?.split("").reverse().join("") ?? "");
    return longestCommonPrefix(reversed).split("").reverse().join("");
  }
  StringUtils2.longestCommonSuffix = longestCommonSuffix;
  function truncateMiddle(value, maxLength, separator = "...") {
    if (isEmpty(value) || value.length <= maxLength) return value ?? "";
    const sepLen = separator.length;
    const charsToShow = maxLength - sepLen;
    const frontChars = Math.ceil(charsToShow / 2);
    const backChars = Math.floor(charsToShow / 2);
    return value.slice(0, frontChars) + separator + value.slice(-backChars);
  }
  StringUtils2.truncateMiddle = truncateMiddle;
  function ellipsis(value, maxLength) {
    return truncate(value, maxLength, "...");
  }
  StringUtils2.ellipsis = ellipsis;
  function ellipsisMiddle(value, maxLength) {
    return truncateMiddle(value, maxLength, "...");
  }
  StringUtils2.ellipsisMiddle = ellipsisMiddle;
  function pad(value, length, padChar = " ") {
    return center(value, length, padChar);
  }
  StringUtils2.pad = pad;
  function padCenter(value, length, padChar = " ") {
    return center(value, length, padChar);
  }
  StringUtils2.padCenter = padCenter;
  function isAscii(value) {
    return /^[\x00-\x7F]*$/.test(value);
  }
  StringUtils2.isAscii = isAscii;
  function isLowerCase(value) {
    return value === value.toLowerCase();
  }
  StringUtils2.isLowerCase = isLowerCase;
  function isUpperCase(value) {
    return value === value.toUpperCase();
  }
  StringUtils2.isUpperCase = isUpperCase;
  function isCapitalized(value) {
    return value.charAt(0) === value.charAt(0).toUpperCase();
  }
  StringUtils2.isCapitalized = isCapitalized;
  function swapPrefix(value, oldPrefix, newPrefix) {
    if (value.startsWith(oldPrefix)) {
      return newPrefix + value.slice(oldPrefix.length);
    }
    return value;
  }
  StringUtils2.swapPrefix = swapPrefix;
  function swapSuffix(value, oldSuffix, newSuffix) {
    if (value.endsWith(oldSuffix)) {
      return value.slice(0, -oldSuffix.length) + newSuffix;
    }
    return value;
  }
  StringUtils2.swapSuffix = swapSuffix;
  function ensurePrefix(value, prefix) {
    return value.startsWith(prefix) ? value : prefix + value;
  }
  StringUtils2.ensurePrefix = ensurePrefix;
  function ensureSuffix(value, suffix) {
    return value.endsWith(suffix) ? value : value + suffix;
  }
  StringUtils2.ensureSuffix = ensureSuffix;
  function removePrefix(value, prefix) {
    return value.startsWith(prefix) ? value.slice(prefix.length) : value;
  }
  StringUtils2.removePrefix = removePrefix;
  function removeSuffix(value, suffix) {
    return value.endsWith(suffix) ? value.slice(0, -suffix.length) : value;
  }
  StringUtils2.removeSuffix = removeSuffix;
  function take(value, n) {
    return value?.slice(0, n) ?? "";
  }
  StringUtils2.take = take;
  function takeRight(value, n) {
    return value?.slice(-n) ?? "";
  }
  StringUtils2.takeRight = takeRight;
  function takeWhile(value, predicate) {
    let result = "";
    for (const char of value ?? "") {
      if (!predicate(char)) break;
      result += char;
    }
    return result;
  }
  StringUtils2.takeWhile = takeWhile;
  function takeRightWhile(value, predicate) {
    let result = "";
    for (let i = (value?.length ?? 0) - 1; i >= 0; i--) {
      const char = value?.charAt(i) ?? "";
      if (!predicate(char)) break;
      result = char + result;
    }
    return result;
  }
  StringUtils2.takeRightWhile = takeRightWhile;
  function drop(value, n) {
    return value?.slice(n) ?? "";
  }
  StringUtils2.drop = drop;
  function dropRight(value, n) {
    return value?.slice(0, -n) ?? "";
  }
  StringUtils2.dropRight = dropRight;
  function dropWhile(value, predicate) {
    let i = 0;
    for (const char of value ?? "") {
      if (!predicate(char)) break;
      i++;
    }
    return value?.slice(i) ?? "";
  }
  StringUtils2.dropWhile = dropWhile;
  function dropRightWhile(value, predicate) {
    let i = (value?.length ?? 0) - 1;
    while (i >= 0 && predicate(value?.charAt(i) ?? "")) {
      i--;
    }
    return value?.slice(0, i + 1) ?? "";
  }
  StringUtils2.dropRightWhile = dropRightWhile;
  function countLines(value) {
    return lineCount(value);
  }
  StringUtils2.countLines = countLines;
  function getLine(value, lineNumber) {
    const lines = splitLines(value);
    return lines[lineNumber] ?? "";
  }
  StringUtils2.getLine = getLine;
  function getLines(value) {
    return splitLines(value);
  }
  StringUtils2.getLines = getLines;
  function isSingleLine(value) {
    return !value?.includes("\n");
  }
  StringUtils2.isSingleLine = isSingleLine;
  function isMultiLine(value) {
    return value?.includes("\n") ?? false;
  }
  StringUtils2.isMultiLine = isMultiLine;
  function normalizeLineEndings(value, lineEnding = "\n") {
    return value?.replace(/\r\n|\r|\n/g, lineEnding) ?? "";
  }
  StringUtils2.normalizeLineEndings = normalizeLineEndings;
  function toCamelCase(value) {
    return camelCase(value);
  }
  StringUtils2.toCamelCase = toCamelCase;
  function toKebabCase(value) {
    return kebabCase(value);
  }
  StringUtils2.toKebabCase = toKebabCase;
  function toSnakeCase(value) {
    return snakeCase(value);
  }
  StringUtils2.toSnakeCase = toSnakeCase;
  function toPascalCase(value) {
    return pascalCase(value);
  }
  StringUtils2.toPascalCase = toPascalCase;
  function toConstantCase(value) {
    return constantCase(value);
  }
  StringUtils2.toConstantCase = toConstantCase;
  function toSentenceCase(value) {
    if (isEmpty(value)) return "";
    return value.charAt(0).toUpperCase() + value.slice(1).toLowerCase();
  }
  StringUtils2.toSentenceCase = toSentenceCase;
  function toTitleCase(value) {
    return capitalizeWords(value);
  }
  StringUtils2.toTitleCase = toTitleCase;
  function toCapitalCase(value) {
    return capitalizeWords(value);
  }
  StringUtils2.toCapitalCase = toCapitalCase;
  function toDotCase(value) {
    return value?.replace(/([a-z])([A-Z])/g, "$1.$2").replace(/[-_\s]+/g, ".").toLowerCase() ?? "";
  }
  StringUtils2.toDotCase = toDotCase;
  function toPathCase(value) {
    return value?.replace(/([a-z])([A-Z])/g, "$1/$2").replace(/[-_\s]+/g, "/").toLowerCase() ?? "";
  }
  StringUtils2.toPathCase = toPathCase;
  function stripTags(value) {
    return value?.replace(/<[^>]*>/g, "") ?? "";
  }
  StringUtils2.stripTags = stripTags;
  function stripNumbers(value) {
    return value?.replace(/\d+/g, "") ?? "";
  }
  StringUtils2.stripNumbers = stripNumbers;
  function stripWhitespace(value) {
    return value?.replace(/\s+/g, "") ?? "";
  }
  StringUtils2.stripWhitespace = stripWhitespace;
  function stripPunctuation(value) {
    return value?.replace(/[^\w\s]/g, "") ?? "";
  }
  StringUtils2.stripPunctuation = stripPunctuation;
  function normalizeWhitespace(value) {
    return value?.replace(/\s+/g, " ").trim() ?? "";
  }
  StringUtils2.normalizeWhitespace = normalizeWhitespace;
  function includesAll(value, searches) {
    return searches.every((search) => value?.includes(search) ?? false);
  }
  StringUtils2.includesAll = includesAll;
  function includesAny(value, searches) {
    return searches.some((search) => value?.includes(search) ?? false);
  }
  StringUtils2.includesAny = includesAny;
})(exports.StringUtils || (exports.StringUtils = {}));
exports.BACKSLASH = BACKSLASH;
exports.CARRIAGE_RETURN = CARRIAGE_RETURN;
exports.DASH = DASH;
exports.DOT = DOT;
exports.EMPTY_STRING = EMPTY_STRING;
exports.NEWLINE = NEWLINE;
exports.SLASH = SLASH;
exports.SPACE = SPACE;
exports.TAB = TAB;
exports.UNDERSCORE = UNDERSCORE;
//# sourceMappingURL=string.cjs.map
