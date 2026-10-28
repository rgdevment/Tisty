import { fill, t } from "../locales";
import { asking, tablesIn } from "./tabled";

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

/** A day as the reader's language says it, and how far it is from today. */
export const dayOf = (
  iso: string,
  today: string,
  locale: string,
): { day: string; when: string } => {
  const at = new Date(civil(iso));
  const day = new Intl.DateTimeFormat(locale, {
    day: "numeric",
    month: "short",
    year: "numeric",
    timeZone: "UTC",
  }).format(at);
  const days = Math.round((civil(iso) - civil(today)) / DAY);
  const when = new Intl.RelativeTimeFormat(locale, { numeric: "auto" }).format(days, "day");
  return { day, when };
};

const chip = (page: Document, said: string, hues: readonly string[]): HTMLElement => {
  const one = page.createElement("span");
  one.className = "table-option";
  one.style.setProperty("--hue", `var(--tisty-hue-${hueOf(said, hues)})`);
  one.textContent = said;
  return one;
};

type Seeing = { readonly today: string; readonly locale: string; readonly hues: readonly string[] };

// Each typed value is drawn as it is used: a date in the reader's words, stars and a switch to press, options in colour.
export const celled = (page: Document, seeing: Seeing) => {
  for (const box of tablesIn(page)) {
    if (!box.classList.contains("dressed") || box.classList.contains("celled")) continue;
    box.classList.add("celled");
    const types = Array.from(box.querySelectorAll<HTMLElement>(":scope > table > thead th")).map(
      (head) => head.dataset.type ?? "",
    );
    for (const row of box.querySelectorAll<HTMLTableRowElement>(":scope > table > tbody > tr")) {
      const at = Number(row.dataset.row);
      Array.from(row.cells).forEach((cell, column) => {
        drawn(page, cell, types[column], { row: at, column }, seeing);
      });
    }
  }
};

const drawn = (
  page: Document,
  cell: HTMLTableCellElement,
  type: string,
  place: { row: number; column: number },
  { today, locale, hues }: Seeing,
) => {
  const key = cell.dataset.key;
  const grip = cell.querySelector(":scope > .row-grip");
  const keep = (...made: Node[]) => cell.replaceChildren(...(grip ? [grip] : []), ...made);
  if (type === "date" && key) {
    const { day, when } = dayOf(key, today, locale);
    const told = page.createElement("small");
    told.className = "table-when";
    told.textContent = when;
    keep(page.createTextNode(day), told);
  }
  if (type === "rating" && key !== undefined) {
    const lit = Number(key);
    keep(
      ...[1, 2, 3, 4, 5].map((n) => {
        // A second press on the star already lit takes the rating down to nothing.
        const star = asking(page, "star", fill("tableStars", String(n)), "★", {
          ...place,
          to: n === lit ? 0 : n,
        });
        star.classList.add("table-star");
        star.setAttribute("aria-pressed", String(n <= lit));
        return star;
      }),
    );
  }
  if (type === "check" && key !== undefined) {
    const on = key === "1";
    const toggle = asking(page, "toggle", t("tableTypeCheck"), "", { ...place, to: on ? 0 : 1 });
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
