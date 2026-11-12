import { IDEA_HUES, ideaParts, saysAny } from "tisty-editor/read";

export interface Note {
  heading: string;
  text: string;
}

export interface Board {
  head: string;
  notes: Note[];
}

const OPENER = /^\s*>\s*\[!idea\](.*)$/i;
const FENCE = /^[>\s]*(```|~~~)/;
const QUOTED = /^\s*>/;
const STAMP = /^(\d{4})-(\d{2})-(\d{2})(?: (\d{2}):(\d{2}))?$/;

export const IDEA_AT_MOST = 140;

export const COLOURS = IDEA_HUES;

export const colourOf = (heading: string): number => COLOURS.indexOf(ideaParts(heading).hue ?? "");

interface Cut {
  ending: string;
  head: string[];
  blocks: string[][];
}

function cut(body: string): Cut {
  const ending = body.includes("\r\n") ? "\r\n" : "\n";
  const head: string[] = [];
  const blocks: string[][] = [];
  let fenced = false;
  let fencedInQuote = false;
  let quoting = false;
  for (const line of body.split(/\r?\n/)) {
    const quoted = QUOTED.test(line);
    // Markdown ends a fence opened inside a quote where the quote ends, closed or not.
    if (fenced && fencedInQuote && !quoted) fenced = false;
    const opens = !fenced && !quoting && OPENER.test(line);
    if (FENCE.test(line)) {
      fencedInQuote = !fenced && quoted;
      fenced = !fenced;
    }
    quoting = quoted;
    if (opens) blocks.push([line]);
    else if (blocks.length) blocks[blocks.length - 1].push(line);
    else head.push(line);
  }
  return { ending, head, blocks };
}

const unquoted = (line: string): string => line.replace(/^\s*> ?/, "");

const noteOf = (block: string[]): Note => ({
  heading: (OPENER.exec(block[0])?.[1] ?? "").trim(),
  text: block.slice(1).map(unquoted).join("\n").trim(),
});

const blockOf = (heading: string, text: string): string[] => [
  `> [!IDEA] ${heading}`,
  ...(text ? text.split("\n").map((line) => (line ? `> ${line}` : ">")) : []),
  "",
];

// Untouched notes go back byte for byte, so one edit never rewrites the whole file in sync.
function sewn({ ending, head, blocks }: Cut): string {
  const spaced = blocks.map((block, at) =>
    at < blocks.length - 1 && block[block.length - 1] !== "" ? [...block, ""] : block,
  );
  const lines = [...head, ...spaced.flat()];
  while (lines.length && lines[lines.length - 1] === "") lines.pop();
  return `${lines.join(ending)}${ending}`;
}

export function boardOf(body: string): Board {
  const { head, blocks } = cut(body);
  return { head: head.join("\n").trimEnd(), notes: blocks.map(noteOf) };
}

export function introOf(body: string): string {
  const { head } = cut(body);
  let at = 0;
  if (head[0]?.trim() === "---") {
    const closes = head.findIndex(
      (line, i) => i > 0 && (line.trim() === "---" || line.trim() === "..."),
    );
    at = closes < 0 ? 0 : closes + 1;
  }
  const rest = head.slice(at);
  const title = rest.findIndex((line) => line.startsWith("# "));
  return (title < 0 ? rest : rest.slice(title + 1)).join("\n").trim();
}

export const rewritten = (body: string, at: number, text: string): string => {
  const sheet = cut(body);
  sheet.blocks = sheet.blocks.map((block, index) =>
    index === at ? blockOf(noteOf(block).heading, text) : block,
  );
  return sewn(sheet);
};

export const dropped = (body: string, at: number): string => {
  const sheet = cut(body);
  sheet.blocks = sheet.blocks.filter((_, index) => index !== at);
  return sewn(sheet);
};

// By what it says, not where it sits: the sheet may have moved while the idea was away.
export const without = (body: string, idea: Note): string => {
  const at = cut(body).blocks.findIndex((block) => {
    const one = noteOf(block);
    return one.heading === idea.heading && one.text === idea.text;
  });
  return at < 0 ? body : dropped(body, at);
};

export const ordered = (body: string, order: number[]): string => {
  const sheet = cut(body);
  sheet.blocks = order.map((at) => sheet.blocks[at]).filter((one) => one !== undefined);
  return sewn(sheet);
};

export function shifted(order: number[], from: number, to: number): number[] {
  const next = [...order];
  const [moving] = next.splice(from, 1);
  next.splice(to, 0, moving);
  return next;
}

export function bornHeading(body: string, at: Date, roll = Math.random()): string {
  const { notes } = boardOf(body);
  const last = notes.length ? colourOf(notes[notes.length - 1].heading) : -1;
  const many = COLOURS.length;
  let picked = Math.floor(roll * many) % many;
  if (picked === last) picked = (picked + 1 + (Math.floor(roll * 1000) % (many - 1))) % many;
  return `${stampOf(at)} · ${COLOURS[picked]}`;
}

export const added = (body: string, heading: string): string => {
  const sheet = cut(body);
  sheet.blocks.push(blockOf(heading, ""));
  return sewn(sheet);
};

const two = (n: number) => String(n).padStart(2, "0");

export const stampOf = (at: Date): string =>
  `${at.getFullYear()}-${two(at.getMonth() + 1)}-${two(at.getDate())} ${two(at.getHours())}:${two(at.getMinutes())}`;

export function saidWhen(heading: string, locale: string): string {
  const stamp = STAMP.exec(ideaParts(heading).when ?? "");
  if (!stamp) {
    return heading
      .split(" · ")
      .map((part) => part.trim())
      .filter((part) => part && !saysAny(ideaParts(part)))
      .join(" · ");
  }
  const [year, month, day] = stamp.slice(1, 4).map(Number);
  const date = new Intl.DateTimeFormat(locale, { day: "numeric", month: "short" }).format(
    new Date(year, month - 1, day),
  );
  return stamp[4] ? `${date} · ${stamp[4]}:${stamp[5]}` : date;
}

export interface Shape {
  tilt: number;
  hue: number;
  wide: number;
  drop: number;
  nudge: number;
  space: number;
  hold: "pin" | "tape" | "corner-left" | "corner-right";
  angle: number;
  along: number;
  pin: number;
}

const HOLDS: Shape["hold"][] = ["pin", "tape", "corner-left", "corner-right"];

const hashed = (text: string): number => {
  let h = 0x811c9dc5;
  for (const letter of text) h = Math.imul(h ^ (letter.codePointAt(0) ?? 0), 0x01000193);
  h ^= h >>> 16;
  h = Math.imul(h, 0x85ebca6b);
  h ^= h >>> 13;
  h = Math.imul(h, 0xc2b2ae35);
  return (h ^ (h >>> 16)) >>> 0;
};

export const HUES = 12;

export function shapeOf(heading: string, nth = 0): Shape {
  const pick = (salt: number, range: number) => hashed(`${salt}|${heading}#${nth}`) % range;
  return {
    tilt: pick(1, 15) - 7,
    hue: pick(2, HUES),
    wide: pick(3, 7),
    drop: pick(4, 6),
    nudge: pick(5, 7) - 3,
    space: pick(10, 6),
    hold: HOLDS[pick(6, HOLDS.length)],
    angle: pick(7, 31) - 15,
    along: pick(8, 61) + 20,
    pin: pick(9, 4),
  };
}
