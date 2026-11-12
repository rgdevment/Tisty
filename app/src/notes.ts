export interface Note {
  heading: string;
  text: string;
}

export interface Board {
  head: string;
  notes: Note[];
}

const SECTION = /^## (.*)$/;
const FENCE = /^(```|~~~)/;
const STAMP = /^(\d{4})-(\d{2})-(\d{2}) (\d{2}):(\d{2})(?: · [a-z]+)?$/;
const PAINTED = / · ([a-z]+)$/;

export const COLOURS = [
  "pink",
  "coral",
  "orange",
  "yellow",
  "lime",
  "green",
  "teal",
  "cyan",
  "blue",
  "lavender",
  "violet",
  "magenta",
];

export const colourOf = (heading: string): number => {
  const said = PAINTED.exec(heading);
  return said ? COLOURS.indexOf(said[1]) : -1;
};

export function boardOf(body: string): Board {
  const lines = body.replace(/\r\n/g, "\n").split("\n");
  const head: string[] = [];
  const notes: { heading: string; lines: string[] }[] = [];
  let fenced = false;
  for (const line of lines) {
    const section = fenced ? null : SECTION.exec(line);
    if (FENCE.test(line)) fenced = !fenced;
    if (section) {
      notes.push({ heading: section[1].trim(), lines: [] });
    } else if (notes.length) {
      notes[notes.length - 1].lines.push(line);
    } else {
      head.push(line);
    }
  }
  return {
    head: head.join("\n").trimEnd(),
    notes: notes.map((one) => ({ heading: one.heading, text: one.lines.join("\n").trim() })),
  };
}

export function bodyOf(board: Board): string {
  const sections = board.notes.map((one) =>
    one.text ? `## ${one.heading}\n\n${one.text}` : `## ${one.heading}`,
  );
  return `${[board.head, ...sections].filter((one) => one !== "").join("\n\n")}\n`;
}

export const rewritten = (body: string, at: number, text: string): string => {
  const board = boardOf(body);
  board.notes = board.notes.map((one, index) => (index === at ? { ...one, text } : one));
  return bodyOf(board);
};

export const dropped = (body: string, at: number): string => {
  const board = boardOf(body);
  board.notes = board.notes.filter((_, index) => index !== at);
  return bodyOf(board);
};

export const ordered = (body: string, order: number[]): string => {
  const board = boardOf(body);
  board.notes = order.map((at) => board.notes[at]).filter((one) => one !== undefined);
  return bodyOf(board);
};

export function shifted(order: number[], from: number, to: number): number[] {
  const next = [...order];
  const [moving] = next.splice(from, 1);
  next.splice(to, 0, moving);
  return next;
}

// Chosen once, when the idea is born, and never the colour of the idea just before it.
export function bornHeading(body: string, at: Date, roll = Math.random()): string {
  const { notes } = boardOf(body);
  const last = notes.length ? colourOf(notes[notes.length - 1].heading) : -1;
  const many = COLOURS.length;
  let picked = Math.floor(roll * many) % many;
  if (picked === last) picked = (picked + 1 + (Math.floor(roll * 1000) % (many - 1))) % many;
  return `${stampOf(at)} · ${COLOURS[picked]}`;
}

export const added = (body: string, heading: string): string => {
  const board = boardOf(body);
  board.notes.push({ heading, text: "" });
  return bodyOf(board);
};

const two = (n: number) => String(n).padStart(2, "0");

export const stampOf = (at: Date): string =>
  `${at.getFullYear()}-${two(at.getMonth() + 1)}-${two(at.getDate())} ${two(at.getHours())}:${two(at.getMinutes())}`;

export function saidWhen(heading: string, locale: string): string {
  const stamp = STAMP.exec(heading);
  if (!stamp) return heading;
  const [, year, month, day, hour, minute] = stamp.map(Number);
  const at = new Date(year, month - 1, day, hour, minute);
  const date = new Intl.DateTimeFormat(locale, { day: "numeric", month: "short" }).format(at);
  return `${date} · ${two(hour)}:${two(minute)}`;
}

export interface Shape {
  tilt: number;
  hue: number;
  wide: number;
  drop: number;
  nudge: number;
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
    wide: pick(3, 4),
    drop: pick(4, 6),
    nudge: pick(5, 7) - 3,
    hold: HOLDS[pick(6, HOLDS.length)],
    angle: pick(7, 31) - 15,
    along: pick(8, 61) + 20,
    pin: pick(9, 4),
  };
}
