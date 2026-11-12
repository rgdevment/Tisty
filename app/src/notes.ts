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
const STAMP = /^(\d{4})-(\d{2})-(\d{2}) (\d{2}):(\d{2})$/;

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

const mixed = (seed: number, salt: number): number => {
  let h = Math.imul(seed ^ Math.imul(salt + 1, 0x9e3779b9), 0x85ebca6b);
  h = Math.imul(h ^ (h >>> 13), 0xc2b2ae35);
  return (h ^ (h >>> 16)) >>> 0;
};

// Drawn from when it came, not from its words, so editing a note never repaints it.
export function shapeOf(heading: string, nth = 0): Shape {
  let seed = 0x811c9dc5;
  for (const letter of `${heading}#${nth}`) {
    seed = Math.imul(seed ^ (letter.codePointAt(0) ?? 0), 0x01000193);
  }
  const pick = (salt: number, range: number) => mixed(seed, salt) % range;
  return {
    tilt: pick(1, 15) - 7,
    hue: pick(2, 12),
    wide: pick(3, 4),
    drop: pick(4, 6),
    nudge: pick(5, 7) - 3,
    hold: HOLDS[pick(6, HOLDS.length)],
    angle: pick(7, 31) - 15,
    along: pick(8, 61) + 20,
    pin: pick(9, 4),
  };
}
