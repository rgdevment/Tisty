export const VERSION = WHOLE.version;
export const CONTRACTS = WHOLE.contracts;
export const ALLOWED = WHOLE.allowed;

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

const base64urlOf = (said, length) => said.length === length && /^[A-Za-z0-9_-]+$/.test(said);

const lengthOf = (said) => [...said].length;

const isLanguage = (said) => /^[a-z]{2,3}(-[A-Z][a-z]{3})?(-([A-Z]{2}|[0-9]{3}))?$/.test(said);

const isTagWord = (said) =>
  said === said.toLowerCase() &&
  said === said.normalize("NFD").replace(/\p{M}/gu, "") &&
  /^[\p{Alphabetic}\p{N}]+(-[\p{Alphabetic}\p{N}]+)*$/u.test(said);

const checkType = (type, value, at, out) => {
  if (type === "text") {
    if (typeof value !== "string") out.push(at + " is not text");
  } else if (type === "bool") {
    if (typeof value !== "boolean") out.push(at + " is not true or false");
  } else if (type === "whole") {
    if (!Number.isSafeInteger(value) || value < 0) out.push(at + " is not a whole number");
  } else if (type === "number") {
    if (typeof value !== "number") out.push(at + " is not a number");
  } else if (type === "time") {
    if (typeof value !== "string" || !isTime(value)) out.push(at + " is not a date in whole seconds");
  } else if (type === "language") {
    if (typeof value !== "string" || !isLanguage(value)) out.push(at + " is not a language tag");
  } else if (type === "tag_word") {
    if (typeof value !== "string" || !isTagWord(value)) out.push(at + " is not a tag");
  } else if (type === "key") {
    if (typeof value !== "string" || !base64urlOf(value, 43)) out.push(at + " is not a key");
  } else if (type === "hash") {
    if (typeof value !== "string" || !base64urlOf(value, 43)) out.push(at + " is not a hash");
  } else if (type === "signed") {
    if (typeof value !== "string" || !base64urlOf(value, 86)) out.push(at + " is not a signature");
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
      if (said.required || value[said.required_when] === true) out.push(here + " is missing");
      continue;
    }
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
    if (typeof size === "number" && (typeof given === "string" || said.type === "whole" || said.type === "number")) {
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
  (rule.starts_with !== undefined && !value.startsWith(rule.starts_with));

// Browsers drop spaces and control characters and decode entities, so a scheme hidden behind either counts as wrong.
const wrongUrl = (value) => {
  const bare = [...value].filter((one) => one > " ").join("");
  const head = bare.slice(0, until(bare, /[/?#]/));
  if (head.includes("&")) return true;
  const colon = head.indexOf(":");
  return colon >= 0 && !WHOLE.urls.schemes.includes(lowered(head.slice(0, colon)));
};

const isSizeIn = (said, unit) => said.endsWith(unit) && /^\d+(\.\d+)?$/.test(said.slice(0, said.length - unit.length));

const wrongStyles = (tag, value) =>
  value
    .split(";")
    .map((one) => one.trim())
    .filter(Boolean)
    .filter((one) => {
      const colon = one.indexOf(":");
      const property = lowered(one.slice(0, Math.max(colon, 0))).trim();
      const said = lowered(one.slice(colon + 1)).trim();
      return !WHOLE.styles.some(
        (rule) => rule.on === tag && rule.property === property && (rule.values ? rule.values.includes(said) : isSizeIn(said, rule.unit)),
      );
    });

export function checkHtml(html) {
  const out = [];
  const through = WHOLE.passthrough.attributes;
  for (const { tag, attributes, unclosed } of opened(html)) {
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
          const known = drawn?.classes.includes(one) || drawn?.class_starts.some((start) => one.startsWith(start));
          if (!known) out.push("the class " + one + " on <" + tag + "> is in no contract");
        }
      }
      if (WHOLE.urls.attributes.includes(name) && wrongUrl(value)) out.push(name + " on <" + tag + "> goes where no contract lets it");
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
