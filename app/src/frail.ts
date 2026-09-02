const KEPT = /^\/?(u|mark(\s+data-pen="[a-z]+")?)$/i;
const TITLED = /\btitle="(?:[^"\\]|\\.)*"/;
const NAMED = /^[a-z0-9#]+$/i;
const WHY = [
  "frailFront",
  "frailHtml",
  "frailComments",
  "frailEntities",
  "frailNotes",
  "frailRefs",
  "frailFence",
];

const spacing = (said: string): number => {
  let wide = 0;
  for (const one of said) {
    if (one === " ") wide += 1;
    else if (one === "\t") wide += 4;
    else break;
  }
  return wide;
};

const flat = (said: string): string => said.replace(/^[ \t\r]+|[ \t\r]+$/g, "");

const quoted = (line: string): [number, number, string] => {
  let said = line;
  let deep = 0;
  let wide = spacing(said);

  while (wide < 4) {
    const rest = flat(said);
    if (!rest.startsWith(">")) break;
    said = rest.slice(1);
    if (said.startsWith(" ")) said = said.slice(1);
    deep += 1;
    wide = spacing(said);
  }
  return [deep, wide, flat(said)];
};

const bullet = (said: string): number | null => {
  const found = /^([-*+]|\d{1,9}[.)])( +)/.exec(said);
  return found ? found[1].length + found[2].length : null;
};

const listed = (base: number, wide: number, said: string): number => {
  if (!said) return base;
  const after = bullet(said);
  if (after !== null && wide <= base) return wide + after;
  return wide < base ? 0 : base;
};

const fenceless = (text: string): { bare: string; told: boolean } => {
  const out: string[] = [];
  let open: { mark: string; many: number; held: number; room: number } | null = null;
  let base = 0;
  let told = false;

  for (const line of text.split("\n")) {
    const [deep, held, held2] = quoted(line);
    if (!open) base = listed(base, held, held2);
    const after = bullet(held2);
    const wide = after === null ? held : held + after;
    const said = after === null ? held2 : held2.slice(after);
    const found = wide < base + 4 ? /^(`{3,}|~{3,})(.*)$/.exec(said) : null;
    const marker = found && (found[1][0] === "~" || !found[2].includes("`")) ? found : null;
    if (open) {
      if (!said) continue;
      if (deep >= open.held && wide >= open.room) {
        const shuts = marker?.[1][0] === open.mark && marker[1].length >= open.many;
        if (shuts && deep === open.held) open = null;
        continue;
      }
      open = null;
    }
    if (marker) {
      told = told || marker[2].replace(TITLED, "").trim().split(/\s+/).filter(Boolean).length > 1;
      open = { mark: marker[1][0], many: marker[1].length, held: deep, room: wide };
      continue;
    }
    out.push(line);
  }
  return { bare: out.join("\n"), told };
};

const fronted = (text: string): boolean => {
  const lines = text.replace(/^﻿/, "").split("\n");
  if (lines[0]?.trim() !== "---") return false;
  return lines.slice(1).some((line) => line.trim() === "---");
};

const spanless = (line: string): string => {
  const ticks = (from: number) => {
    let many = 0;
    while (line[from + many] === "`") many += 1;
    return many;
  };
  let out = "";
  let at = 0;

  while (at < line.length) {
    if (line[at] !== "`") {
      out += line[at];
      at += 1;
      continue;
    }
    const open = ticks(at);
    let scan = at + open;
    let shut = -1;
    while (scan < line.length) {
      if (line[scan] !== "`") {
        scan += 1;
        continue;
      }
      const many = ticks(scan);
      if (many === open) {
        shut = scan + many;
        break;
      }
      scan += many;
    }
    if (shut < 0) {
      out += line.slice(at, at + open);
      at += open;
    } else {
      at = shut;
    }
  }
  return out;
};

const entity = (from: string): boolean => {
  const shut = from.indexOf(";");
  if (shut < 0) return false;
  const name = from.slice(1, shut);
  return name.length > 1 && name.length < 12 && NAMED.test(name);
};

const anchored = (said: string): boolean => {
  const open = said.lastIndexOf("[");
  return open >= 0 && !said.slice(open + 1).includes("]");
};

const markup = (line: string): string | null => {
  for (let at = 0; at < line.length; at += 1) {
    if (line[at] === "&" && entity(line.slice(at))) return "frailEntities";
    if (line[at] !== "<") continue;
    const rest = line.slice(at + 1);
    if (rest.startsWith("!--")) return "frailComments";
    const end = rest.indexOf(">");
    const inner = end < 0 ? rest : rest.slice(0, end);
    if (inner.includes("://") || (inner.includes("@") && !inner.includes(" "))) continue;
    if (line.slice(0, at).endsWith("](") && !inner.includes("<") && anchored(line.slice(0, at - 2)))
      continue;
    if (KEPT.test(inner)) continue;
    if (/^[a-z/!?]/i.test(rest)) return "frailHtml";
  }
  return null;
};

const noted = (line: string): boolean => {
  let at = 0;
  for (;;) {
    const start = line.indexOf("[^", at);
    if (start < 0) return false;
    at = start + 2;
    if (start > 0 && line[start - 1] === "\\") continue;
    const end = line.indexOf("]", at);
    if (end < 0) return false;
    if (end > at && line[end + 1] !== "(" && line[end + 1] !== "[") return true;
    at = end + 1;
  }
};

const labelled = (rest: string): [string, string] | null => {
  let at = 0;
  while (at < rest.length) {
    if (rest[at] === "\\") at += 2;
    else if (rest[at] !== "]") at += 1;
    else if (rest[at + 1] === ":") return [rest.slice(0, at), rest.slice(at + 2)];
    else return null;
  }
  return null;
};

const linked = (line: string, next: string): boolean => {
  if (!line.startsWith("[")) return false;
  const found = labelled(line.slice(1));
  if (!found) return false;
  const [label, told] = found;
  if (!label || label.startsWith("^")) return false;
  return told.trim() !== "" || next.trim() !== "";
};

export const frail = (text: string): string[] => {
  const said = fenceless(text);
  const bare = said.bare.replace(/^ {4}.*$/gm, "").replace(/^\t.*$/gm, "");
  const seen = new Set<string>();

  if (fronted(text)) seen.add("frailFront");
  if (said.told) seen.add("frailFence");

  const lines = bare.split("\n");
  for (const [at, line] of lines.entries()) {
    const plain = spanless(line);
    const why = markup(plain);
    if (why) seen.add(why);
    const one = plain.trim();
    if (noted(one)) seen.add("frailNotes");
    if (linked(one, lines[at + 1] ?? "")) seen.add("frailRefs");
  }

  return WHY.filter((one) => seen.has(one));
};
