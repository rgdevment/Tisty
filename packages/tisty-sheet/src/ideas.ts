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

const STAMP = /^(\d{4})-(\d{2})-(\d{2})(?: (\d{2}):(\d{2}))?$/;

export const ideaWhen = (heading: string, locale: string): string => {
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
};

const shade = (mix: string) => `stop-color: color-mix(in srgb, var(--pin), ${mix})`;

export const pinMarkup = (id: string): string =>
  `<svg aria-hidden="true" viewBox="0 0 24 44" class="sticky-pin"><defs><radialGradient id="${id}-cap" cx="0.35" cy="0.3" r="0.8"><stop offset="0" style="${shade("white 45%")}"/><stop offset="0.45" style="stop-color: var(--pin)"/><stop offset="1" style="${shade("black 45%")}"/></radialGradient><linearGradient id="${id}-neck" x1="0" x2="1"><stop offset="0" style="${shade("black 35%")}"/><stop offset="0.35" style="${shade("white 25%")}"/><stop offset="1" style="${shade("black 45%")}"/></linearGradient></defs><line x1="12" y1="30" x2="14.5" y2="43" style="stroke: var(--tisty-idea-needle)" stroke-width="1.6" stroke-linecap="round"/><ellipse cx="12" cy="28.5" rx="9" ry="3.6" fill="url(#${id}-cap)"/><path d="M8 9 L16 9 L14.2 27 L9.8 27 Z" fill="url(#${id}-neck)"/><ellipse cx="12" cy="8" rx="8" ry="6.5" fill="url(#${id}-cap)"/><ellipse cx="9.5" cy="5.5" rx="2.6" ry="1.6" fill="white" opacity="0.6"/></svg>`;

// A note in a document leans a quarter of what one on the board does, so its words stay easy to read.
export const pinned = (page: Document, locale: string) => {
  const notes = page.querySelectorAll<HTMLElement>('blockquote[data-callout="idea"]');
  notes.forEach((note, nth) => {
    const shape = shapeOf(note.dataset.when ?? note.textContent ?? "");
    note.dataset.hold = shape.hold;
    note.dataset.pin = String(shape.pin);
    note.style.setProperty("--tilt", `${shape.tilt / 4}deg`);
    note.style.setProperty("--angle", `${shape.angle}deg`);
    note.style.setProperty("--along", `${shape.along}%`);
    if (note.dataset.when) note.dataset.said = ideaWhen(note.dataset.when, locale);
    note.querySelector(":scope > .sticky-pin")?.remove();
    if (shape.hold === "pin") note.insertAdjacentHTML("afterbegin", pinMarkup(`idea-pin-${nth}`));
  });
};

const two = (n: number) => String(n).padStart(2, "0");

export const ideaStamp = (at: Date): string =>
  `${at.getFullYear()}-${two(at.getMonth() + 1)}-${two(at.getDate())} ${two(at.getHours())}:${two(at.getMinutes())}`;
