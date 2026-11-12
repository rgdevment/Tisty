import { CONTRACTS } from "@tisty/contracts";

type Style = { values: { callouts: { idea: { hues: string[] } } } };

export const IDEA_HUES: readonly string[] = (CONTRACTS.style as Style).values.callouts.idea.hues;

const STAMPED = /^\d{4}-\d{2}-\d{2}(?: \d{2}:\d{2})?$/;

export interface IdeaHead {
  when?: string;
  hue?: string;
  side?: "left" | "right";
}

export const ideaParts = (words: string): IdeaHead => {
  const head: IdeaHead = {};
  for (const part of words.split(" · ").map((one) => one.trim())) {
    const lower = part.toLowerCase();
    if (IDEA_HUES.includes(lower)) head.hue = lower;
    else if (lower === "left" || lower === "right") head.side = lower;
    else if (!head.when && STAMPED.test(part)) head.when = part;
  }
  return head;
};

export const saysAny = (head: IdeaHead): boolean => Boolean(head.when || head.hue || head.side);

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

export const shapeOf = (heading: string, nth = 0): Shape => {
  const pick = (salt: number, range: number) => hashed(`${salt}|${heading}#${nth}`) % range;
  return {
    tilt: pick(1, 15) - 7,
    hue: pick(2, IDEA_HUES.length),
    wide: pick(3, 7),
    drop: pick(4, 6),
    nudge: pick(5, 7) - 3,
    space: pick(10, 6),
    hold: HOLDS[pick(6, HOLDS.length)],
    angle: pick(7, 31) - 15,
    along: pick(8, 61) + 20,
    pin: pick(9, 4),
  };
};

// A note in a document leans a quarter of what one on the board does, so its words stay easy to read.
export const pinned = (page: Document) => {
  for (const note of page.querySelectorAll<HTMLElement>('blockquote[data-callout="idea"]')) {
    const shape = shapeOf(note.dataset.when ?? note.textContent ?? "");
    note.dataset.hold = shape.hold;
    note.dataset.pin = String(shape.pin);
    note.style.setProperty("--tilt", `${shape.tilt / 4}deg`);
    note.style.setProperty("--angle", `${shape.angle}deg`);
    note.style.setProperty("--along", `${shape.along}%`);
  }
};

const two = (n: number) => String(n).padStart(2, "0");

export const ideaStamp = (at: Date): string =>
  `${at.getFullYear()}-${two(at.getMonth() + 1)}-${two(at.getDate())} ${two(at.getHours())}:${two(at.getMinutes())}`;
