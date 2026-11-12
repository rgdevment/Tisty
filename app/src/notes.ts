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

// Drawn from the words, so a note keeps its tilt and colour every time it is shown.
export function shapeOf(note: Note): {
  tilt: number;
  hue: number;
  wide: number;
  drop: number;
  pinned: boolean;
} {
  let seed = 0;
  for (const letter of `${note.heading}\u0000${note.text}`) {
    seed = (seed * 31 + (letter.codePointAt(0) ?? 0)) % 1_000_003;
  }
  const pick = (salt: number, range: number) =>
    ((seed * (salt * 2 + 1) + salt * 7919) >>> 0) % range;
  return {
    tilt: pick(1, 9) - 4,
    hue: pick(2, 5),
    wide: pick(3, 4),
    drop: pick(4, 4),
    pinned: pick(5, 3) === 0,
  };
}
