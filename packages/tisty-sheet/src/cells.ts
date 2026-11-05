import { asking, tablesIn } from "./tables";
import type { Say } from "./words";

// The same option always takes the same hue, on every machine, without a word of it in the file.
export const hueOf = (said: string, hues: readonly string[]): string => {
  let sum = 0;
  for (const one of said.toLowerCase()) sum = (sum * 31 + (one.codePointAt(0) ?? 0)) >>> 0;
  return hues[sum % hues.length] ?? "gray";
};

const DAY = 86_400_000;

/** Today on this machine's calendar, as a file writes a day. */
export const today = (): string => {
  const now = new Date();
  const two = (one: number) => String(one).padStart(2, "0");
  return `${now.getFullYear()}-${two(now.getMonth() + 1)}-${two(now.getDate())}`;
};

const civil = (iso: string): number => {
  const [year, month, day] = iso.split("-").map(Number);
  return Date.UTC(year, month - 1, day);
};

type Saying = (iso: string, today: string) => { day: string; when: string };

/** How the reader's language says a day and how far it is from today; its formatters are made once. */
export const sayingDays = (locale: string): Saying => {
  const days = new Intl.DateTimeFormat(locale, {
    day: "numeric",
    month: "short",
    year: "numeric",
    timeZone: "UTC",
  });
  const far = new Intl.RelativeTimeFormat(locale, { numeric: "auto" });
  return (iso, today) => ({
    day: days.format(new Date(civil(iso))),
    when: far.format(Math.round((civil(iso) - civil(today)) / DAY), "day"),
  });
};

const chip = (page: Document, said: string, hues: readonly string[]): HTMLElement => {
  const one = page.createElement("span");
  one.className = "table-option";
  one.style.setProperty("--hue", `var(--tisty-hue-${hueOf(said, hues)})`);
  one.textContent = said;
  return one;
};

type Seeing = {
  readonly today: string;
  readonly locale: string;
  readonly hues: readonly string[];
  /** Whether the table may be written, and so has stars and a switch to press. */
  readonly live: boolean;
  readonly say: Say;
};

type Drawing = Omit<Seeing, "locale"> & { readonly said: Saying };

// Each typed value is drawn as it is used: a date in the reader's words, options in colour, and, where it is written, stars and a switch to press.
export const celled = (page: Document, seeing: Seeing) => {
  const drawing = { ...seeing, said: sayingDays(seeing.locale) };
  for (const box of tablesIn(page)) {
    if (box.classList.contains("celled")) continue;
    box.classList.add("celled");
    const types = Array.from(box.querySelectorAll<HTMLElement>(":scope > table > thead th")).map(
      (head) => head.dataset.type ?? "",
    );
    for (const row of box.querySelectorAll<HTMLTableRowElement>(":scope > table > tbody > tr")) {
      const at = Number(row.dataset.row);
      Array.from(row.cells).forEach((cell, column) => {
        drawn(page, cell, types[column], { row: at, column }, drawing);
      });
    }
  }
};

const drawn = (
  page: Document,
  cell: HTMLTableCellElement,
  type: string,
  place: { row: number; column: number },
  { today, hues, live, said, say }: Drawing,
) => {
  const key = cell.dataset.key;
  const grip = cell.querySelector(":scope > .row-grip");
  const keep = (...made: Node[]) => cell.replaceChildren(...(grip ? [grip] : []), ...made);
  // An empty cell is a rating of none and a «no», so a new row can be set with one press too.
  const empty = key === undefined && cell.dataset.amiss === undefined && !cell.textContent?.trim();
  if (type === "date" && key) {
    const { day, when } = said(key, today);
    const told = page.createElement("small");
    told.className = "table-when";
    told.textContent = when;
    keep(page.createTextNode(day), told);
  }
  if (live && type === "rating" && (key !== undefined || empty)) {
    const lit = Number(key ?? 0);
    keep(
      ...[1, 2, 3, 4, 5].map((n) => {
        // A second press on the star already lit empties the cell: no rating, which no total counts.
        const star = asking(page, "star", say("tableStars", String(n)), "★", {
          ...place,
          to: n === lit ? 0 : n,
        });
        star.classList.add("table-star");
        star.setAttribute("aria-pressed", String(n <= lit));
        return star;
      }),
    );
  }
  if (live && type === "check" && (key !== undefined || empty)) {
    const on = key === "1";
    const toggle = asking(page, "toggle", say("tableTypeCheck"), "", { ...place, to: on ? 0 : 1 });
    toggle.classList.add("table-switch");
    toggle.setAttribute("role", "switch");
    toggle.setAttribute("aria-checked", String(on));
    keep(toggle);
  }
  if ((type === "select" || type === "tags") && key) {
    const said = cell.textContent?.trim() ?? "";
    const parts = type === "tags" ? said.split(",") : [said];
    keep(
      ...parts
        .map((one) => one.trim())
        .filter(Boolean)
        .map((one) => chip(page, one, hues)),
    );
  }
  if (type === "percent" && key) {
    cell.classList.add("table-percent");
    cell.style.setProperty("--fill", `${Math.max(0, Math.min(100, Number(key)))}%`);
  }
};
