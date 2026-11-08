export const VERSION = WHOLE.version;
export const CONTRACTS = WHOLE.contracts;
export const ALLOWED = WHOLE.allowed;
export const PASSTHROUGH = WHOLE.passthrough;
export const STYLES = WHOLE.styles;
export const URLS = WHOLE.urls;
export const WIDGET = WHOLE.widget;

// A tag called constructor must not find what every object inherits.
const own = (held, name) => (held && Object.hasOwn(held, name) ? held[name] : undefined);

const daysIn = (year, month) => {
  const leap = year % 4 === 0 && (year % 100 !== 0 || year % 400 === 0);
  return [31, leap ? 29 : 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][month - 1];
};

const isTime = (said) => {
  const found = /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})Z$/.exec(said);
  if (!found) return false;
  const [year, month, day, hour, minute, second] = found.slice(1).map(Number);
  return month >= 1 && month <= 12 && day >= 1 && day <= daysIn(year, month) && hour <= 23 && minute <= 59 && second <= 59;
};

// A moment nothing signs may hold a fraction of a second, as the log keeps them.
const isMoment = (said) => isTime(said.replace(/\.\d{1,9}Z$/, "Z"));

const UNSEEN = /[\u00AD\u034F\u061C\u115F\u1160\u17B4\u17B5\u180B-\u180F\u200B\u200E\u200F\u2028-\u202E\u2060-\u206F\u2800\u3164\uFEFF\uFFA0\uFFF9-\uFFFB\u{1D173}-\u{1D17A}\u{E0000}-\u{E007F}\p{Cc}]/u;

// Many scripts write with a joiner, and it only hides a letter where no script needs one: between Latin, Greek or Cyrillic.
const needsNoJoiner = (one) => one === undefined || one < 0x0590 || (one >= 0x1e00 && one <= 0x1fff);

const isLine = (said) => {
  const points = [...said].map((one) => one.codePointAt(0));
  const joined = (one, at) => (one !== 0x200c && one !== 0x200d) || (!needsNoJoiner(points[at - 1]) && !needsNoJoiner(points[at + 1]));
  return !UNSEEN.test(said) && points.every(joined);
};

const isFileName = (said) => isLine(said) && !/[/\\]/.test(said);

const isName = (said) => {
  const letters = said.match(/\p{Alphabetic}/gu)?.length ?? 0;
  const digits = said.match(/[0-9]/g)?.length ?? 0;
  return said.trim() === said && [...said].length <= 80 && letters > 0 && digits <= letters && isLine(said);
};

const BASE64URL = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

// The last character holds spare bits, and they are zero: one key has one spelling, so comparing two is comparing them.
const base64urlOf = (said, length, spare) =>
  said.length === length && /^[A-Za-z0-9_-]+$/.test(said) && BASE64URL.indexOf(said[length - 1]) % spare === 0;

const SMALL_ORDER = new Set([
  "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
  "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIA",
  "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
  "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIA",
  "JuiVj8KyJ7BFw_SJ8u-Y8NXfrAXTxjM5sTgCiG1T_AU",
  "JuiVj8KyJ7BFw_SJ8u-Y8NXfrAXTxjM5sTgCiG1T_IU",
  "xxdqcD1N2E-6PAt2DRBnDyogU_osOczGTsf9d5KsA3o",
  "xxdqcD1N2E-6PAt2DRBnDyogU_osOczGTsf9d5KsA_o",
  "7P_______________________________________38",
  "7P________________________________________8",
  "7f_______________________________________38",
  "7f________________________________________8",
  "7v_______________________________________38",
  "7v________________________________________8",
]);

const isId = (said) => /^[A-Za-z0-9_-]+$/.test(said);

const spacesOff = (said) => said.replace(/^[\t\n\f\r ]+|[\t\n\f\r ]+$/g, "");

const lengthOf = (said) => [...said].length;

const isLanguage = (said) => /^[a-z]{2,3}(-[A-Z][a-z]{3})?(-([A-Z]{2}|[0-9]{3}))?$/.test(said);

// Accents come off the alphabets that have them, and elsewhere a mark is how the word is written.
const isTagWord = (said) =>
  said === said.toLowerCase() &&
  !/[\u0000-\u052F\u1E00-\u1FFF]\p{M}/u.test(said.normalize("NFD")) &&
  /^[\p{Alphabetic}\p{M}\p{N}]+(-[\p{Alphabetic}\p{M}\p{N}]+)*$/u.test(said);

const checkType = (type, value, at, out) => {
  if (type === "text") {
    if (typeof value !== "string") out.push(at + " is not text");
  } else if (type === "widget_body") {
    if (typeof value !== "string") out.push(at + " is not text");
    else for (const one of checkWidget(value)) out.push(at + " holds " + one);
  } else if (type === "bool") {
    if (typeof value !== "boolean") out.push(at + " is not true or false");
  } else if (type === "whole") {
    if (!Number.isSafeInteger(value) || value < 0) out.push(at + " is not a whole number");
  } else if (type === "id") {
    if (typeof value !== "string" || !isId(value)) out.push(at + " is not an id");
  } else if (type === "time") {
    if (typeof value !== "string" || !isTime(value)) out.push(at + " is not a date in whole seconds");
  } else if (type === "moment") {
    if (typeof value !== "string" || !isMoment(value)) out.push(at + " is not a date");
  } else if (type === "name") {
    if (typeof value !== "string" || !isName(value)) out.push(at + " is not a name a person can read as written");
  } else if (type === "line") {
    if (typeof value !== "string" || !isLine(value)) out.push(at + " is not one line a person can read as written");
  } else if (type === "file_name") {
    if (typeof value !== "string" || !isFileName(value)) out.push(at + " is not the name of a file");
  } else if (type === "digits") {
    if (typeof value !== "string" || !/^(0|[1-9][0-9]*)$/.test(value)) out.push(at + " is not a whole number written as text");
  } else if (type === "language") {
    if (typeof value !== "string" || !isLanguage(value)) out.push(at + " is not a language tag");
  } else if (type === "tag_word") {
    if (typeof value !== "string" || !isTagWord(value)) out.push(at + " is not a tag");
  } else if (type === "key") {
    if (typeof value !== "string" || !base64urlOf(value, 43, 4) || SMALL_ORDER.has(value)) out.push(at + " is not a key");
  } else if (type === "hash") {
    if (typeof value !== "string" || !base64urlOf(value, 43, 4)) out.push(at + " is not a hash");
  } else if (type === "signed") {
    if (typeof value !== "string" || !base64urlOf(value, 86, 16)) out.push(at + " is not a signature");
  } else if (WHOLE.enums[type]) {
    if (!WHOLE.enums[type].includes(value)) out.push(at + " is none of " + WHOLE.enums[type].join(", "));
  } else {
    checkFields(type, value, at, out);
  }
};

function checkFields(name, value, at, out) {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    out.push(at + " is not a " + name);
    return;
  }
  for (const field of WHOLE.contracts[name].except ?? []) {
    if (value[field] !== undefined) out.push((at === "" ? field : at + "." + field) + " is not something a " + name + " has");
  }
  for (const [field, said] of Object.entries(WHOLE.contracts[name].fields ?? {})) {
    const here = at === "" ? field : at + "." + field;
    const given = value[field];
    if (given === undefined || given === null) {
      if (said.required || (said.required_when && value[said.required_when] === true)) out.push(here + " is missing");
      continue;
    }
    if (said.only_when && value[said.only_when] !== true) out.push(here + " is only for what is " + said.only_when);
    // A word this version does not list wears its default, so a later one can add it.
    if (said.otherwise !== undefined && typeof given === "string") continue;
    if (said.type === "list") {
      if (!Array.isArray(given)) {
        out.push(here + " is not a list");
        continue;
      }
      if (said.at_least !== undefined && given.length < said.at_least) out.push(here + " has too few");
      if (said.at_most !== undefined && given.length > said.at_most) out.push(here + " has too many");
      given.forEach((one, n) => checkType(said.of, one, here + "[" + n + "]", out));
      continue;
    }
    checkType(said.type, given, here, out);
    const size = typeof given === "string" ? lengthOf(given) : given;
    if (typeof given === "string" && said.bytes_at_most !== undefined && new TextEncoder().encode(given).length > said.bytes_at_most) {
      out.push(here + " is too long");
    }
    if (typeof size === "number" && (typeof given === "string" || said.type === "whole")) {
      if (said.at_least !== undefined && size < said.at_least) out.push(here + " is too short or too low");
      if (said.at_most !== undefined && size > said.at_most) out.push(here + " is too long or too high");
    }
  }
}

export function check(contract, value) {
  if (!own(WHOLE.contracts, contract)) return ["there is no contract called " + contract];
  const out = [];
  checkFields(contract, value, "", out);
  return out;
}

const lowered = (said) => said.replace(/[A-Z]/g, (letter) => letter.toLowerCase());

const until = (said, stops) => {
  const at = said.search(stops);
  return at < 0 ? said.length : at;
};

const words = (said) => said.split(/[\t\n\f\r ]+/).filter(Boolean);

// Each tag the HTML opens, with its attributes. It walks as check_html does in Rust, so one page gets one verdict.
const opened = (html) => {
  const out = [];
  let rest = html;
  for (let start = rest.indexOf("<"); start >= 0; start = rest.indexOf("<")) {
    rest = rest.slice(start + 1);
    if (rest.startsWith("!") || rest.startsWith("?")) out.push({ tag: "!", attributes: [], unclosed: false });
    // A closing tag says nothing but its name: a quote in one would hide from here what a browser goes on to run.
    if (rest.startsWith("/")) {
      const closes = /^\/([a-zA-Z][^\t\n\f\r />]*)[\t\n\f\r ]*>/.exec(rest);
      out.push({ tag: closes ? "</" + lowered(closes[1]) : "/", attributes: [], unclosed: false });
    }
    if (!/^[a-zA-Z]/.test(rest)) continue;
    let end = until(rest, /[\t\n\f\r />]/);
    const one = { tag: lowered(rest.slice(0, end)), attributes: [], unclosed: false };
    rest = rest.slice(end);
    for (;;) {
      rest = rest.replace(/^[\t\n\f\r /]+/, "");
      if (rest === "") break;
      if (rest[0] === ">") {
        rest = rest.slice(1);
        break;
      }
      end = until(rest, /[\t\n\f\r =/>]/);
      const name = lowered(rest.slice(0, end));
      rest = rest.slice(end).replace(/^[\t\n\f\r ]+/, "");
      if (rest[0] !== "=") {
        one.attributes.push([name, ""]);
        continue;
      }
      rest = rest.slice(1).replace(/^[\t\n\f\r ]+/, "");
      if (rest[0] === '"' || rest[0] === "'") {
        const close = rest.indexOf(rest[0], 1);
        if (close < 0) {
          one.unclosed = true;
          rest = "";
          break;
        }
        one.attributes.push([name, rest.slice(1, close)]);
        rest = rest.slice(close + 1);
      } else {
        end = until(rest, /[\t\n\f\r >]/);
        one.attributes.push([name, rest.slice(0, end)]);
        rest = rest.slice(end);
      }
    }
    out.push(one);
  }
  return out;
};

const brokenBy = (rule, value) =>
  (rule.never ?? []).some((one) => words(value).includes(one)) ||
  (rule.starts_with !== undefined && !value.startsWith(rule.starts_with)) ||
  (rule.only !== undefined && !rule.only.includes(value));

// One spelling for a place too, and it fits a whole number in every language.
const referenceIn = (bare) => {
  const place = bare.slice(bare.indexOf(":") + 1);
  const slash = place.indexOf("/");
  const kind = place.slice(0, Math.max(slash, 0));
  const at = place.slice(slash + 1);
  return slash >= 0 && WHOLE.urls.references.includes(kind) && /^(0|[1-9][0-9]{0,14})$/.test(at) ? { kind, at: Number(at) } : null;
};

// Browsers drop spaces and control characters and decode entities, so a scheme hidden behind either counts as wrong.
// A link says where it goes: nothing without a scheme means anything once it travels, except a heading of its own page.
const wrongUrl = (value) => {
  const bare = [...value].filter((one) => one > " ").join("");
  if (bare.startsWith("#")) return false;
  const colon = bare.indexOf(":");
  if (colon < 0) return true;
  const scheme = lowered(bare.slice(0, colon));
  if (!WHOLE.urls.schemes.includes(scheme)) return true;
  return scheme === "tisty" && referenceIn(bare) === null;
};

// A paint or a marker names something in its own drawing, and nothing else.
const wrongPaint = (value) => {
  const said = spacesOff(lowered(value));
  if (said.includes("&") || said.includes("\\")) return true;
  if (!said.includes("url(")) return false;
  return !/^url\(#[^\t\n\f\r ()]+\)$/.test(said);
};

const isSizeIn = (said, unit) => said.endsWith(unit) && /^\d+(\.\d+)?$/.test(said.slice(0, said.length - unit.length));

const isSized = (said, units) => {
  if (/^-?0*\.?0+$/.test(said)) return true;
  return units.some((unit) => said.endsWith(unit) && /^-?\d*\.?\d+$/.test(said.slice(0, said.length - unit.length)));
};

const isColor = (said) => /^#([0-9a-f]{3}|[0-9a-f]{6}|[0-9a-f]{8})$/.test(said);

const holdsStyle = (rule, said) => {
  if (rule.unit) return isSizeIn(said, rule.unit);
  if (rule.sizes) return isSized(said, rule.sizes);
  if (rule.color) return isColor(said);
  return rule.values.includes(said);
};

const wrongStyles = (tag, value) =>
  value
    .split(";")
    .map(spacesOff)
    .filter(Boolean)
    .filter((one) => {
      const colon = one.indexOf(":");
      const property = spacesOff(lowered(one.slice(0, Math.max(colon, 0))));
      const said = spacesOff(lowered(one.slice(colon + 1)));
      return !WHOLE.styles.some((rule) => rule.on === tag && rule.property === property && holdsStyle(rule, said));
    });

// Each place inside what travels that the HTML leads to, read as a browser reads its attributes.
export function referencesIn(html) {
  const out = [];
  for (const { attributes } of opened(html)) {
    for (const [name, value] of attributes) {
      const bare = [...value].filter((one) => one > " ").join("");
      const one = WHOLE.urls.attributes.includes(name) && lowered(bare).startsWith("tisty:") ? referenceIn(bare) : null;
      if (one) out.push(one);
    }
  }
  return out;
}

// Each picture the HTML draws, with the size it holds for it.
export function picturesIn(html) {
  const out = [];
  for (const { tag, attributes } of opened(html)) {
    if (tag !== "img") continue;
    const said = (name) => attributes.find(([one]) => one === name)?.[1];
    const one = said("src")?.startsWith("tisty:picture/") ? referenceIn(said("src")) : null;
    if (one) out.push({ at: one.at, width: said("width"), height: said("height") });
  }
  return out;
}

export function checkHtml(html) {
  const out = [];
  const through = WHOLE.passthrough.attributes;
  for (const { tag, attributes, unclosed } of opened(html)) {
    if (tag === "!") {
      out.push("a comment is in no contract");
      continue;
    }
    if (tag === "/") {
      out.push("a closing tag that says more than its name is in no contract");
      continue;
    }
    if (tag.startsWith("</")) {
      if (!own(WHOLE.allowed, tag.slice(2)) && !WHOLE.passthrough.tags.includes(tag.slice(2))) out.push(tag + "> is in no contract");
      continue;
    }
    for (const rule of WHOLE.html_rules) {
      if (rule.tag === tag && rule.requires?.some((name) => !attributes.some(([said]) => said === name))) out.push(rule.says);
    }
    const drawn = own(WHOLE.allowed, tag);
    if (!drawn && !WHOLE.passthrough.tags.includes(tag)) {
      out.push("<" + tag + "> is in no contract");
      continue;
    }
    if (unclosed) out.push("<" + tag + "> leaves a quote open");
    for (const [name, value] of attributes) {
      const listed = drawn?.attributes.includes(name) || through["*"].includes(name) || own(through, tag)?.includes(name);
      if (!listed) out.push(name + " on <" + tag + "> is in no contract");
      if (name === "class") {
        for (const one of words(value)) {
          const known =
            !one.includes("&") &&
            (drawn?.classes.includes(one) ||
            drawn?.class_starts.some((start) => one.startsWith(start)) ||
              drawn?.class_ends.some((end) => one.endsWith(end)));
          if (!known) out.push("the class " + one + " on <" + tag + "> is in no contract");
        }
      }
      if (WHOLE.urls.attributes.includes(name) && wrongUrl(value)) out.push(name + " on <" + tag + "> goes where no contract lets it");
      if (WHOLE.urls.paints.includes(name) && wrongPaint(value)) out.push(name + " on <" + tag + "> asks for what is not in its drawing");
      if (name === "style") {
        for (const one of wrongStyles(tag, value)) out.push("the style " + one + " on <" + tag + "> is in no contract");
      }
      for (const rule of WHOLE.html_rules) {
        if (rule.tag === tag && rule.attribute === name && brokenBy(rule, value)) out.push(rule.says);
      }
    }
  }
  return [...new Set(out)];
}

const bareOf = (said) => [...said].filter((one) => one > " ").join("");

// What a style shows between quotes runs nothing, and spaces and comments hide nothing; only an escape is kept from the quotes.
const flatCss = (said) => {
  let out = ";";
  let at = 0;
  while (at < said.length) {
    const one = said[at];
    if (one === '"' || one === "'") {
      const end = said.indexOf(one, at + 1);
      const shown = said.slice(at + 1, end < 0 ? said.length : end);
      out += one + shown.replace(/[^\\]/g, "") + one;
      at = end < 0 ? said.length : end + 1;
    } else if (said.startsWith("/*", at)) {
      const end = said.indexOf("*/", at + 2);
      at = end < 0 ? said.length : end + 2;
    } else {
      if (one > " ") out += one;
      at += 1;
    }
  }
  return out;
};

// A word that ends in a colon is a property, so only where a property starts.
const saysNever = (flat, one) =>
  one.endsWith(":") ? flat.includes(";" + one) || flat.includes("{" + one) : flat.includes(one);

// Each thing a stylesheet says that would reach outside, or that hides a word so nothing can tell.
const wrongInCss = (css) => {
  const said = lowered(css);
  const flat = flatCss(said);
  const out = WHOLE.widget.css_never.filter((one) => saysNever(flat, one));
  for (let at = said.indexOf("url("); at >= 0; at = said.indexOf("url(", at + 4)) {
    const close = said.indexOf(")", at);
    const inside = bareOf(said.slice(at + 4, close < 0 ? said.length : close)).replace(/^["']/, "");
    if (!inside.startsWith("#") && !inside.startsWith("data:image/")) out.push("url(" + inside.slice(0, 40) + ")");
  }
  return out;
};

const sheetsIn = (html) => {
  const low = lowered(html);
  const out = [];
  for (let at = low.indexOf("<style"); at >= 0; at = low.indexOf("<style", at + 1)) {
    const opens = low.indexOf(">", at);
    if (opens < 0) break;
    const close = low.indexOf("</style", opens);
    out.push(html.slice(opens + 1, close < 0 ? html.length : close));
  }
  return out;
};

const holdsPicture = (value) => !value.includes("&") && lowered(bareOf(value)).startsWith("data:image/");

const wrongInTag = (tag, attributes) => {
  const { tags, anywhere, starts } = WHOLE.widget;
  const out = [];
  for (const [name, value] of attributes) {
    const listed = anywhere.includes(name) || own(tags, tag).includes(name) || starts.some((one) => name.startsWith(one));
    if (!listed) out.push("<" + tag + " " + name + ">");
    else if (name === "src" && !holdsPicture(value)) out.push("<" + tag + " src>");
    else if (name === "style") {
      if (value.includes("&")) out.push("<" + tag + " style>");
      out.push(...wrongInCss(value));
    } else if (WHOLE.urls.paints.includes(name) && wrongPaint(value)) out.push("<" + tag + " " + name + ">");
  }
  return out;
};

export function checkWidget(body) {
  const out = [];
  for (const { tag, attributes, unclosed } of opened(body)) {
    if (tag === "!") out.push("<!");
    else if (tag === "/") out.push("</");
    else if (tag.startsWith("</")) {
      if (!own(WHOLE.widget.tags, tag.slice(2))) out.push(tag + ">");
    } else if (!own(WHOLE.widget.tags, tag)) out.push("<" + tag + ">");
    else {
      if (unclosed) out.push("<" + tag + ' "');
      out.push(...wrongInTag(tag, attributes));
    }
  }
  for (const sheet of sheetsIn(body)) out.push(...wrongInCss(sheet));
  return [...new Set(out)];
}

export const widgetPage = (body, dark) =>
  "<!doctype html><html" +
  (dark ? ' class="dark"' : "") +
  '><head><meta charset="utf-8"><meta http-equiv="Content-Security-Policy" content="' +
  WHOLE.widget.policy +
  '"><meta name="color-scheme" content="light dark"><style>' +
  WHOLE.widget.kit +
  '</style></head><body><main class="w">' +
  body +
  "</main></body></html>";

// What a browser makes of the words has the last say, for a reader that has one.
export function strayInWidget(body) {
  const { tags, anywhere, starts } = WHOLE.widget;
  const out = [];
  for (const one of new DOMParser().parseFromString(widgetPage(body, false), "text/html").body.querySelectorAll("*")) {
    const tag = lowered(one.localName);
    if (!own(tags, tag)) {
      out.push("<" + tag + ">");
      continue;
    }
    for (const said of one.getAttributeNames()) {
      const name = lowered(said);
      const listed = anywhere.includes(name) || own(tags, tag).includes(name) || starts.some((start) => name.startsWith(start));
      if (!listed) out.push("<" + tag + " " + name + ">");
    }
  }
  return [...new Set(out)];
}

const says = (bytes, at, said) => {
  for (let one = 0; one < said.length; one += 1) if (bytes[at + one] !== said.charCodeAt(one)) return false;
  return true;
};

const OGG_SOUNDS = ["OpusHead", "\x01vorbis", "\x7fFLAC"];
const EBML_HEADER = 64;

const isOggSound = (bytes) => OGG_SOUNDS.some((said) => says(bytes, 27 + (bytes[26] ?? 0), said));

const isWebm = (bytes) => {
  for (let at = 4; at < Math.min(bytes.length, EBML_HEADER); at += 1) {
    if (says(bytes, at, "\x42\x82\x84webm")) return true;
  }
  return false;
};

const brandedFormat = (bytes) => {
  if (bytes[0] * 0x1000000 + (bytes[1] << 16) + (bytes[2] << 8) + bytes[3] < 16) return null;
  const brand = String.fromCharCode(...bytes.subarray(8, 12));
  return Object.keys(WHOLE.plays).find((format) => WHOLE.plays[format].brands?.includes(brand)) ?? null;
};

// Two bytes of sync say little on their own, so the values no frame can hold rule it out.
const frameFormat = (bytes) => {
  if (bytes.length < 4 || bytes[0] !== 0xff || (bytes[1] & 0xe0) !== 0xe0) return null;
  const layer = bytes[1] & 0x06;
  if (layer === 0) return (bytes[1] & 0xf0) === 0xf0 && ((bytes[2] >> 2) & 0x0f) <= 12 ? "aac" : null;
  const fair = layer === 0x02 && ((bytes[1] >> 3) & 0x03) !== 1 && bytes[2] >> 4 !== 0x0f && ((bytes[2] >> 2) & 0x03) !== 3;
  return fair ? "mp3" : null;
};

const formatOf = (bytes) => {
  if (says(bytes, 0, "RIFF") && says(bytes, 8, "WAVE")) return "wav";
  if (says(bytes, 0, "fLaC")) return "flac";
  if (says(bytes, 0, "OggS")) return isOggSound(bytes) ? "ogg" : null;
  if (says(bytes, 0, "\x1a\x45\xdf\xa3")) return isWebm(bytes) ? "webm" : null;
  if (says(bytes, 4, "ftyp")) return brandedFormat(bytes);
  if (says(bytes, 0, "ID3") && bytes[3] < 0xff && bytes[4] < 0xff) return "mp3";
  return frameFormat(bytes);
};

export function playableAs(bytes) {
  const format = formatOf(bytes);
  if (!format) return null;
  const { player, type } = WHOLE.plays[format];
  return { format, player, type };
}
