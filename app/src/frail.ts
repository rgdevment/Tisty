import { known } from "./glyphs";

const HUES = "red|orange|amber|green|teal|blue|indigo|purple|pink|brown|gray";
const KEPT = new RegExp(
  String.raw`^(\/(u|mark|span)|u|mark(\s+data-pen="(?:yellow|${HUES})")?|span\s+data-ico="([a-z0-9-]+|\p{Extended_Pictographic}[^"]*)"(\s+data-hue="(?:${HUES})")?)$`,
  "iu",
);

// A tag of an older Tisty is kept only while it names a colour of the palette and an icon there is.
const kept = (inner: string): boolean => {
  const found = KEPT.exec(inner);
  if (!found) return false;
  const icon = found[4];
  return !icon || known(icon) || /\p{Extended_Pictographic}/u.test(icon);
};
const WHY = ["frailHtml"];

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
  const found = /^([-*+]|\d{1,9}[.)])([ \t]+)/.exec(said);
  return found ? found[1].length + found[2].length : null;
};

const listed = (base: number, wide: number, said: string): number => {
  if (!said) return base;
  const after = bullet(said) ?? (/^([-*+]|\d{1,9}[.)])$/.test(said) ? said.length + 1 : null);
  if (after !== null && wide <= base) return wide + after;
  return wide < base ? 0 : base;
};

const fenceless = (text: string): string => {
  const out: string[] = [];
  let open: { mark: string; many: number; held: number; room: number } | null = null;
  let base = 0;

  for (const line of text.split("\n")) {
    const [deep, held, held2] = quoted(line);
    if (!open) base = listed(base, held, held2);
    const after = bullet(held2);
    const wide = after === null ? held : held + after;
    const said = after === null ? held2 : held2.slice(after);
    const found = wide < base + 4 ? /^(`{3,}|~{3,})(.*)$/.exec(said) : null;
    const wrong = found?.[1][0] === "`" && found[2].includes("`");
    const marker = found && !wrong ? found : null;
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
      open = { mark: marker[1][0], many: marker[1].length, held: deep, room: Math.min(base, wide) };
      continue;
    }
    out.push(line);
  }
  return out.join("\n");
};

// What the front matter takes at the top, the way the core's `front::span` finds it.
const unfronted = (text: string): string => {
  const lines = text.split("\n");
  if (lines[0]?.trimEnd() !== "---" || !lines[1]?.trim()) return text;
  const shut = lines.findIndex((line, at) => at > 1 && /^(---|\.\.\.)$/.test(line.trimEnd()));
  return shut < 0 ? text : lines.slice(shut + 1).join("\n");
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

const anchored = (said: string): boolean => {
  const open = said.lastIndexOf("[");
  return open >= 0 && !said.slice(open + 1).includes("]");
};

const markup = (line: string): string | null => {
  for (let at = 0; at < line.length; at += 1) {
    if (line[at] !== "<") continue;
    const rest = line.slice(at + 1);
    if (rest.startsWith("!--")) continue;
    const end = rest.indexOf(">");
    const inner = end < 0 ? rest : rest.slice(0, end);
    if (inner.includes("://") || (inner.includes("@") && !inner.includes(" "))) continue;
    if (line.slice(0, at).endsWith("](") && !inner.includes("<") && anchored(line.slice(0, at - 2)))
      continue;
    if (kept(inner)) continue;
    if (/^[a-z/!?]/i.test(rest)) return "frailHtml";
  }
  return null;
};

export const frail = (whole: string): string[] => {
  const text = whole.replace(/^\ufeff+/, "");
  const seen = new Set<string>();
  let aside = false;

  for (const line of fenceless(unfronted(text)).split("\n")) {
    if (!aside && (line.startsWith("    ") || line.startsWith("\t"))) continue;
    let plain = "";
    let rest = spanless(line);
    while (rest) {
      const wall = rest.indexOf(aside ? "-->" : "<!--");
      if (wall < 0) {
        if (!aside) plain += rest;
        break;
      }
      const cut = aside ? wall + 3 : wall;
      if (!aside) plain += rest.slice(0, cut);
      rest = rest.slice(cut);
      aside = !aside;
    }
    const why = markup(plain);
    if (why) seen.add(why);
  }

  return WHY.filter((one) => seen.has(one));
};
