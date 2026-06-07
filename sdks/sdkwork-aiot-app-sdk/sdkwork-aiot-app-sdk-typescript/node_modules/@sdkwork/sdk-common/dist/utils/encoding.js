var Encoding;
((Encoding2) => {
  function base64Encode2(input) {
    let bytes;
    if (typeof input === "string") {
      bytes = new TextEncoder().encode(input);
    } else {
      bytes = input;
    }
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
  Encoding2.base64Encode = base64Encode2;
  function base64Decode2(input) {
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
      if (c !== 64 && input[i - 2] !== "=") {
        result += String.fromCharCode(bitmap >> 8 & 255);
      }
      if (d !== 64 && input[i - 1] !== "=") {
        result += String.fromCharCode(bitmap & 255);
      }
    }
    return result;
  }
  Encoding2.base64Decode = base64Decode2;
  function base64UrlEncode2(input) {
    return base64Encode2(input).replace(/\+/g, "-").replace(/\//g, "_").replace(/=/g, "");
  }
  Encoding2.base64UrlEncode = base64UrlEncode2;
  function base64UrlDecode2(input) {
    input = input.replace(/-/g, "+").replace(/_/g, "/");
    const pad = input.length % 4;
    if (pad) {
      input += "=".repeat(4 - pad);
    }
    return base64Decode2(input);
  }
  Encoding2.base64UrlDecode = base64UrlDecode2;
  function base64ToBytes(base64) {
    const binary = atob(base64);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i);
    }
    return bytes;
  }
  Encoding2.base64ToBytes = base64ToBytes;
  function bytesToBase64(bytes) {
    let binary = "";
    for (let i = 0; i < bytes.length; i++) {
      binary += String.fromCharCode(bytes[i] ?? 0);
    }
    return btoa(binary);
  }
  Encoding2.bytesToBase64 = bytesToBase64;
  function utf8Encode2(input) {
    return new TextEncoder().encode(input);
  }
  Encoding2.utf8Encode = utf8Encode2;
  function utf8Decode2(input) {
    return new TextDecoder().decode(input);
  }
  Encoding2.utf8Decode = utf8Decode2;
  function hexEncode2(input) {
    const bytes = typeof input === "string" ? utf8Encode2(input) : input;
    return Array.from(bytes).map((byte) => byte.toString(16).padStart(2, "0")).join("");
  }
  Encoding2.hexEncode = hexEncode2;
  function hexDecode2(input) {
    const bytes = new Uint8Array(input.length / 2);
    for (let i = 0; i < input.length; i += 2) {
      bytes[i / 2] = parseInt(input.substr(i, 2), 16);
    }
    return utf8Decode2(bytes);
  }
  Encoding2.hexDecode = hexDecode2;
  function hexToBytes(hex) {
    const bytes = new Uint8Array(hex.length / 2);
    for (let i = 0; i < hex.length; i += 2) {
      bytes[i / 2] = parseInt(hex.substr(i, 2), 16);
    }
    return bytes;
  }
  Encoding2.hexToBytes = hexToBytes;
  function bytesToHex(bytes) {
    return Array.from(bytes).map((byte) => byte.toString(16).padStart(2, "0")).join("");
  }
  Encoding2.bytesToHex = bytesToHex;
  function urlEncode2(input) {
    return encodeURIComponent(input);
  }
  Encoding2.urlEncode = urlEncode2;
  function urlDecode2(input) {
    return decodeURIComponent(input);
  }
  Encoding2.urlDecode = urlDecode2;
  function urlEncodeComponent(input) {
    return encodeURIComponent(input);
  }
  Encoding2.urlEncodeComponent = urlEncodeComponent;
  function urlDecodeComponent(input) {
    return decodeURIComponent(input);
  }
  Encoding2.urlDecodeComponent = urlDecodeComponent;
  function htmlEncode2(input) {
    const htmlEntities = {
      "&": "&amp;",
      "<": "&lt;",
      ">": "&gt;",
      '"': "&quot;",
      "'": "&#39;",
      "/": "&#x2F;",
      "`": "&#x60;",
      "=": "&#x3D;"
    };
    return input.replace(/[&<>"'`=/]/g, (char) => htmlEntities[char] || char);
  }
  Encoding2.htmlEncode = htmlEncode2;
  function htmlDecode2(input) {
    const htmlEntities = {
      "&amp;": "&",
      "&lt;": "<",
      "&gt;": ">",
      "&quot;": '"',
      "&#39;": "'",
      "&#x27;": "'",
      "&#x2F;": "/",
      "&#x60;": "`",
      "&#x3D;": "=",
      "&nbsp;": " "
    };
    return input.replace(/&[^;]+;/g, (entity) => htmlEntities[entity] || entity);
  }
  Encoding2.htmlDecode = htmlDecode2;
  function jsonEncode2(value, replacer, space) {
    return JSON.stringify(value, replacer, space);
  }
  Encoding2.jsonEncode = jsonEncode2;
  function jsonDecode2(input) {
    return JSON.parse(input);
  }
  Encoding2.jsonDecode = jsonDecode2;
  function jsonEncodePretty(value, indent = 2) {
    return JSON.stringify(value, null, indent);
  }
  Encoding2.jsonEncodePretty = jsonEncodePretty;
  function tryJsonDecode(input, defaultValue) {
    try {
      return JSON.parse(input);
    } catch {
      return defaultValue;
    }
  }
  Encoding2.tryJsonDecode = tryJsonDecode;
  function isJson(input) {
    try {
      JSON.parse(input);
      return true;
    } catch {
      return false;
    }
  }
  Encoding2.isJson = isJson;
  function xmlEncode2(input) {
    const xmlEntities = {
      "&": "&amp;",
      "<": "&lt;",
      ">": "&gt;",
      '"': "&quot;",
      "'": "&apos;"
    };
    return input.replace(/[&<>"']/g, (char) => xmlEntities[char] || char);
  }
  Encoding2.xmlEncode = xmlEncode2;
  function xmlDecode2(input) {
    const xmlEntities = {
      "&amp;": "&",
      "&lt;": "<",
      "&gt;": ">",
      "&quot;": '"',
      "&apos;": "'"
    };
    return input.replace(/&[^;]+;/g, (entity) => xmlEntities[entity] || entity);
  }
  Encoding2.xmlDecode = xmlDecode2;
  function escapeRegex2(input) {
    return input.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  }
  Encoding2.escapeRegex = escapeRegex2;
  function escapeSql2(input) {
    return input.replace(/[\0\x08\x09\x1a\n\r"'\\\%]/g, (char) => {
      const sqlEscapes = {
        "\0": "\\0",
        "\b": "\\b",
        "	": "\\t",
        "": "\\z",
        "\n": "\\n",
        "\r": "\\r",
        '"': '\\"',
        "'": "\\'",
        "\\": "\\\\",
        "%": "\\%"
      };
      return sqlEscapes[char] || char;
    });
  }
  Encoding2.escapeSql = escapeSql2;
  function escapeShell2(input) {
    return input.replace(/[^A-Za-z0-9_\-.,:\/@\n]/g, (char) => {
      if (char === "\n") {
        return "'\\n'";
      }
      return `\\${char}`;
    });
  }
  Encoding2.escapeShell = escapeShell2;
  function escapeCString(input) {
    return input.replace(/[\\"'\n\r\t\b\f\v\0]/g, (char) => {
      const cEscapes = {
        "\\": "\\\\",
        '"': '\\"',
        "'": "\\'",
        "\n": "\\n",
        "\r": "\\r",
        "	": "\\t",
        "\b": "\\b",
        "\f": "\\f",
        "\v": "\\v",
        "\0": "\\0"
      };
      return cEscapes[char] || char;
    });
  }
  Encoding2.escapeCString = escapeCString;
  function unescapeCString(input) {
    return input.replace(/\\([\\\"'nrtbfv0])/g, (_, char) => {
      const cUnescapes = {
        "\\": "\\",
        '"': '"',
        "'": "'",
        "n": "\n",
        "r": "\r",
        "t": "	",
        "b": "\b",
        "f": "\f",
        "v": "\v",
        "0": "\0"
      };
      return cUnescapes[char] || char;
    });
  }
  Encoding2.unescapeCString = unescapeCString;
  function camelToSnake(input) {
    return input.replace(/[A-Z]/g, (letter) => `_${letter.toLowerCase()}`);
  }
  Encoding2.camelToSnake = camelToSnake;
  function snakeToCamel(input) {
    return input.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase());
  }
  Encoding2.snakeToCamel = snakeToCamel;
  function camelToKebab(input) {
    return input.replace(/[A-Z]/g, (letter) => `-${letter.toLowerCase()}`);
  }
  Encoding2.camelToKebab = camelToKebab;
  function kebabToCamel(input) {
    return input.replace(/-([a-z])/g, (_, letter) => letter.toUpperCase());
  }
  Encoding2.kebabToCamel = kebabToCamel;
  function camelToPascal(input) {
    return input.charAt(0).toUpperCase() + input.slice(1);
  }
  Encoding2.camelToPascal = camelToPascal;
  function pascalToCamel(input) {
    return input.charAt(0).toLowerCase() + input.slice(1);
  }
  Encoding2.pascalToCamel = pascalToCamel;
  function pascalToSnake(input) {
    return camelToSnake(input);
  }
  Encoding2.pascalToSnake = pascalToSnake;
  function snakeToPascal(input) {
    return camelToPascal(snakeToCamel(input));
  }
  Encoding2.snakeToPascal = snakeToPascal;
  function pascalToKebab(input) {
    return camelToKebab(input);
  }
  Encoding2.pascalToKebab = pascalToKebab;
  function kebabToPascal(input) {
    return camelToPascal(kebabToCamel(input));
  }
  Encoding2.kebabToPascal = kebabToPascal;
  function toSnakeCase(input) {
    return input.replace(/([a-z])([A-Z])/g, "$1_$2").replace(/[-\s]+/g, "_").toLowerCase();
  }
  Encoding2.toSnakeCase = toSnakeCase;
  function toKebabCase(input) {
    return input.replace(/([a-z])([A-Z])/g, "$1-$2").replace(/[_\s]+/g, "-").toLowerCase();
  }
  Encoding2.toKebabCase = toKebabCase;
  function toCamelCase(input) {
    return input.replace(/[-_\s]+(.)?/g, (_, char) => char ? char.toUpperCase() : "").replace(/^(.)/, (char) => char.toLowerCase());
  }
  Encoding2.toCamelCase = toCamelCase;
  function toPascalCase(input) {
    const camel = toCamelCase(input);
    return camel.charAt(0).toUpperCase() + camel.slice(1);
  }
  Encoding2.toPascalCase = toPascalCase;
  function toConstantCase(input) {
    return toSnakeCase(input).toUpperCase();
  }
  Encoding2.toConstantCase = toConstantCase;
  function toSentenceCase(input) {
    return input.charAt(0).toUpperCase() + input.slice(1).toLowerCase();
  }
  Encoding2.toSentenceCase = toSentenceCase;
  function toTitleCase(input) {
    return input.replace(/\b\w/g, (char) => char.toUpperCase());
  }
  Encoding2.toTitleCase = toTitleCase;
  function toCapitalCase(input) {
    return input.replace(/[-_\s]+(.)?/g, (_, char) => char ? ` ${char.toUpperCase()}` : "").trim();
  }
  Encoding2.toCapitalCase = toCapitalCase;
  function toDotCase(input) {
    return input.replace(/([a-z])([A-Z])/g, "$1.$2").replace(/[-_\s]+/g, ".").toLowerCase();
  }
  Encoding2.toDotCase = toDotCase;
  function toPathCase(input) {
    return input.replace(/([a-z])([A-Z])/g, "$1/$2").replace(/[-_\s]+/g, "/").toLowerCase();
  }
  Encoding2.toPathCase = toPathCase;
  function rot13(input) {
    return input.replace(/[a-zA-Z]/g, (char) => {
      const start = char <= "Z" ? 65 : 97;
      return String.fromCharCode((char.charCodeAt(0) - start + 13) % 26 + start);
    });
  }
  Encoding2.rot13 = rot13;
  function caesarCipher(input, shift) {
    return input.replace(/[a-zA-Z]/g, (char) => {
      const start = char <= "Z" ? 65 : 97;
      const shifted = ((char.charCodeAt(0) - start + shift) % 26 + 26) % 26;
      return String.fromCharCode(shifted + start);
    });
  }
  Encoding2.caesarCipher = caesarCipher;
  function caesarDecipher(input, shift) {
    return caesarCipher(input, -shift);
  }
  Encoding2.caesarDecipher = caesarDecipher;
  function xorEncode(input, key) {
    const inputBytes = utf8Encode2(input);
    const keyBytes = utf8Encode2(key);
    const result = new Uint8Array(inputBytes.length);
    for (let i = 0; i < inputBytes.length; i++) {
      result[i] = (inputBytes[i] ?? 0) ^ (keyBytes[i % keyBytes.length] ?? 0);
    }
    return bytesToHex(result);
  }
  Encoding2.xorEncode = xorEncode;
  function xorDecode(input, key) {
    const inputBytes = hexToBytes(input);
    const keyBytes = utf8Encode2(key);
    const result = new Uint8Array(inputBytes.length);
    for (let i = 0; i < inputBytes.length; i++) {
      result[i] = (inputBytes[i] ?? 0) ^ (keyBytes[i % keyBytes.length] ?? 0);
    }
    return utf8Decode2(result);
  }
  Encoding2.xorDecode = xorDecode;
  function charCodeEncode(input) {
    return Array.from(input).map((char) => char.charCodeAt(0));
  }
  Encoding2.charCodeEncode = charCodeEncode;
  function charCodeDecode(codes) {
    return String.fromCharCode(...codes);
  }
  Encoding2.charCodeDecode = charCodeDecode;
  function binaryEncode(input) {
    return Array.from(input).map((char) => char.charCodeAt(0).toString(2).padStart(8, "0")).join(" ");
  }
  Encoding2.binaryEncode = binaryEncode;
  function binaryDecode(input) {
    return input.split(/\s+/).map((byte) => String.fromCharCode(parseInt(byte, 2))).join("");
  }
  Encoding2.binaryDecode = binaryDecode;
  function octalEncode(input) {
    return Array.from(input).map((char) => char.charCodeAt(0).toString(8).padStart(3, "0")).join(" ");
  }
  Encoding2.octalEncode = octalEncode;
  function octalDecode(input) {
    return input.split(/\s+/).map((byte) => String.fromCharCode(parseInt(byte, 8))).join("");
  }
  Encoding2.octalDecode = octalDecode;
  function decimalEncode(input) {
    return Array.from(input).map((char) => char.charCodeAt(0).toString(10)).join(" ");
  }
  Encoding2.decimalEncode = decimalEncode;
  function decimalDecode(input) {
    return input.split(/\s+/).map((code) => String.fromCharCode(parseInt(code, 10))).join("");
  }
  Encoding2.decimalDecode = decimalDecode;
  function punycodeEncode(input) {
    const prefix = "xn--";
    if (input.startsWith(prefix)) {
      return input;
    }
    const asciiPart = input.replace(/[^\x00-\x7F]/g, "");
    const nonAsciiPart = input.replace(/[\x00-\x7F]/g, "");
    if (!nonAsciiPart) {
      return input;
    }
    return prefix + asciiPart + "-" + nonAsciiPart.split("").map((c) => c.charCodeAt(0).toString(36)).join("");
  }
  Encoding2.punycodeEncode = punycodeEncode;
  function slugify2(input) {
    return input.toLowerCase().trim().replace(/[^\w\s-]/g, "").replace(/[\s_-]+/g, "-").replace(/^-+|-+$/g, "");
  }
  Encoding2.slugify = slugify2;
  function unslugify2(input) {
    return input.replace(/-/g, " ").replace(/\b\w/g, (char) => char.toUpperCase());
  }
  Encoding2.unslugify = unslugify2;
  function queryStringEncode2(params) {
    return Object.entries(params).filter(([, value]) => value !== void 0 && value !== null).map(([key, value]) => {
      if (Array.isArray(value)) {
        return value.map((v) => `${urlEncode2(key)}=${urlEncode2(String(v))}`).join("&");
      }
      return `${urlEncode2(key)}=${urlEncode2(String(value))}`;
    }).join("&");
  }
  Encoding2.queryStringEncode = queryStringEncode2;
  function queryStringDecode2(query) {
    const result = {};
    if (!query) {
      return result;
    }
    query = query.replace(/^[?#]/, "");
    for (const pair of query.split("&")) {
      const parts = pair.split("=");
      const key = parts[0];
      const value = parts[1];
      if (!key) continue;
      const decodedKey = urlDecode2(key);
      const decodedValue = value ? urlDecode2(value) : "";
      if (result[decodedKey]) {
        if (Array.isArray(result[decodedKey])) {
          result[decodedKey].push(decodedValue);
        } else {
          result[decodedKey] = [result[decodedKey], decodedValue];
        }
      } else {
        result[decodedKey] = decodedValue;
      }
    }
    return result;
  }
  Encoding2.queryStringDecode = queryStringDecode2;
  function formDataEncode(data) {
    return Object.entries(data).filter(([, value]) => value !== void 0 && value !== null).map(([key, value]) => `${urlEncode2(key)}=${urlEncode2(String(value))}`).join("&");
  }
  Encoding2.formDataEncode = formDataEncode;
  function mimeTypeToExtension(mimeType) {
    const mimeMap = {
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
    };
    return mimeMap[mimeType.toLowerCase()] || "";
  }
  Encoding2.mimeTypeToExtension = mimeTypeToExtension;
  function extensionToMimeType(extension) {
    const extMap = {
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
    };
    return extMap[extension.toLowerCase().replace(/^\./, "")] || "application/octet-stream";
  }
  Encoding2.extensionToMimeType = extensionToMimeType;
  function charsetEncode(input, _charset) {
    const encoder = new TextEncoder();
    return encoder.encode(input);
  }
  Encoding2.charsetEncode = charsetEncode;
  function charsetDecode(input, charset) {
    const decoder = new TextDecoder(charset);
    return decoder.decode(input);
  }
  Encoding2.charsetDecode = charsetDecode;
  function stripBom(input) {
    if (input.charCodeAt(0) === 65279) {
      return input.slice(1);
    }
    return input;
  }
  Encoding2.stripBom = stripBom;
  function addBom(input, bom = "utf-8") {
    const boms = {
      "utf-8": "\uFEFF",
      "utf-16le": "￾",
      "utf-16be": "\uFEFF"
    };
    return boms[bom] + input;
  }
  Encoding2.addBom = addBom;
  function normalizeEncoding(input, fromEncoding, toEncoding) {
    const bytes = charsetEncode(input);
    return charsetDecode(bytes, toEncoding);
  }
  Encoding2.normalizeEncoding = normalizeEncoding;
  function isValidBase64(input) {
    if (!input || input.length % 4 !== 0) {
      return false;
    }
    const base64Regex = /^[A-Za-z0-9+/]*={0,2}$/;
    return base64Regex.test(input);
  }
  Encoding2.isValidBase64 = isValidBase64;
  function isValidHex(input) {
    return /^[0-9a-fA-F]*$/.test(input) && input.length % 2 === 0;
  }
  Encoding2.isValidHex = isValidHex;
  function isValidUrl(input) {
    try {
      new URL(input);
      return true;
    } catch {
      return false;
    }
  }
  Encoding2.isValidUrl = isValidUrl;
  function isValidEmail(input) {
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return emailRegex.test(input);
  }
  Encoding2.isValidEmail = isValidEmail;
  function detectEncoding(input) {
    if (input.charCodeAt(0) === 65279) {
      return "utf-8-bom";
    }
    if (input.charCodeAt(0) === 65534) {
      return "utf-16le";
    }
    if (input.charCodeAt(0) === 65279 && input.charCodeAt(1) === 0) {
      return "utf-16be";
    }
    if (/[\u4e00-\u9fa5]/.test(input)) {
      return "utf-8";
    }
    return "ascii";
  }
  Encoding2.detectEncoding = detectEncoding;
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
export {
  Encoding
};
//# sourceMappingURL=encoding.js.map
