import type { Field } from "./front";
import type { Head } from "./heads";
import type { Paper } from "./paper";
import { besideAbout } from "./sideAbout";
import { besideMore } from "./sideRows";
import type { Say } from "./words";

// The words a body holds, its code and its marks left out, as the details count them.
export const wordsIn = (body: string): number =>
  body
    .replace(/```[\s\S]*?```/g, " ")
    .replace(/[#>*_`~[\]()!|-]/g, " ")
    .split(/\s+/)
    .filter(Boolean).length;

export type Turn = { readonly on: boolean; readonly onTurn: (on: boolean) => void };

export type Step = { readonly key: string; readonly label: string; readonly onPick: () => void };

export type Leaf = {
  readonly key: string;
  readonly title: string;
  readonly current?: boolean;
  readonly away?: boolean;
  readonly archived?: boolean;
  readonly flagged?: boolean;
  readonly onPick: () => void;
};

export type Book = {
  readonly up?: { readonly title: string; readonly onPick: () => void };
  readonly said: string;
  readonly pages: readonly Leaf[];
};

export type Facts = {
  readonly body: string;
  readonly author?: string;
  readonly born?: string;
  readonly editor?: string;
  /** Already in the reader's words, as the host tells dates. */
  readonly made?: string;
  readonly wrote?: string;
  readonly weighs?: string;
  readonly holds?: string;
  readonly fields?: readonly Field[];
  readonly overgrown?: boolean;
  readonly onFront?: () => void;
  readonly tags?: readonly { readonly name: string; readonly times: number }[];
};

export type Seal = {
  readonly verdict: "holds" | "revoked" | "refused" | "unreadable" | "unsigned";
  readonly who?: string;
  /** The name or alias the writer chose to sign with, which only their key vouches for. */
  readonly name?: string;
  /** When they signed, already said in the reader's language. */
  readonly signedOn?: string;
  readonly fingerprint?: string;
  readonly sender?: string;
};

/** A link to hand out, shown whole so it can be read before it is copied. */
export type Link = {
  readonly key: string;
  readonly label: string;
  readonly url: string;
  readonly note?: string;
};

export type Act = {
  readonly key: string;
  readonly label: string;
  readonly main?: boolean;
  readonly href?: string;
  readonly download?: string;
  readonly onPick?: (at: HTMLElement) => void;
};

// Every panel a sheet can have beside it; each place shows the ones it needs, in its order, and the rest are left out.
export type Panel =
  | {
      readonly kind: "title";
      readonly title: string;
      readonly under?: string;
      readonly trail?: readonly Step[];
      readonly onApart?: () => void;
      readonly onShut?: () => void;
    }
  | {
      readonly kind: "paper";
      readonly leaf: Paper;
      readonly onLeaf: (leaf: Paper) => void;
      readonly sheets?: Turn;
      readonly landscape?: Turn;
      readonly raw?: Turn;
      readonly onExport?: (at: HTMLElement) => void;
      readonly exporting?: boolean;
    }
  | { readonly kind: "index"; readonly heads: readonly Head[]; readonly book?: Book }
  | { readonly kind: "details"; readonly facts: Facts; readonly open?: boolean }
  | { readonly kind: "seal"; readonly seal: Seal }
  | {
      readonly kind: "shared";
      readonly gist: string;
      readonly tone?: "good" | "warn" | "bad";
      readonly said?: string;
      readonly links?: readonly Link[];
      readonly acts?: readonly Act[];
      readonly open?: boolean;
    }
  | { readonly kind: "acts"; readonly acts: readonly Act[] };

export type About = Extract<Panel, { kind: "title" | "paper" | "index" }>;
export type More = Exclude<Panel, About>;

const ABOUT = new Set<Panel["kind"]>(["title", "paper", "index"]);

const isAbout = (panel: Panel): panel is About => ABOUT.has(panel.kind);

// One row open at a time; a row the reader opened or shut stays so while the panels are drawn again.
const folding = (host: HTMLElement) => {
  if (host.dataset.folding) return;
  host.dataset.folding = "on";
  host.addEventListener("click", (event) => {
    const summary = (event.target as Element | null)?.closest?.("summary");
    const row = summary?.parentElement;
    if (row instanceof HTMLDetailsElement) row.dataset.chosen = "";
  });
  host.addEventListener(
    "toggle",
    (event) => {
      const row = event.target as HTMLDetailsElement;
      if (!row.open) return;
      for (const other of host.querySelectorAll<HTMLDetailsElement>("details[data-row][open]")) {
        if (other === row) continue;
        other.open = false;
        other.dataset.chosen = "";
      }
    },
    true,
  );
};

const PRESSABLE = "button, a, summary";

// A control is known again by what it is and says, not by its place, which moves when an outline grows.
const named = (one: Element): string =>
  `${one.tagName}.${one.className}|${one.getAttribute("aria-label") ?? one.textContent}`;

// What the reader was at before the panels are drawn again: where the outline was scrolled and what had the keys.
const heldOn = (host: HTMLElement) => {
  const active = host.ownerDocument.activeElement;
  return {
    scrolled: host.querySelector(".side-index")?.scrollTop ?? 0,
    focused: active && host.contains(active) ? named(active) : null,
  };
};

const heldAgain = (host: HTMLElement, { scrolled, focused }: ReturnType<typeof heldOn>) => {
  const index = host.querySelector(".side-index");
  if (index) index.scrollTop = scrolled;
  if (focused === null) return;
  const again = Array.from(host.querySelectorAll<HTMLElement>(PRESSABLE));
  again.find((one) => named(one) === focused)?.focus();
};

// Drawn again whole on every change: it holds a few dozen nodes, and what the reader chose stays chosen.
export const sided = (host: HTMLElement, say: Say, panels: readonly Panel[]) => {
  const page = host.ownerDocument;
  const held = heldOn(host);
  const chosen = new Map(
    Array.from(host.querySelectorAll<HTMLDetailsElement>("details[data-chosen]"), (one) => [
      one.dataset.row,
      one.open,
    ]),
  );
  const top = page.createElement("section");
  top.className = "side-card side-top";
  const more = page.createElement("section");
  more.className = "side-card side-more";
  for (const panel of panels) {
    if (isAbout(panel)) top.append(besideAbout(page, say, panel));
    else more.append(besideMore(page, say, panel));
  }
  host.replaceChildren(...[top, more].filter((card) => card.childElementCount));
  folding(host);
  for (const one of host.querySelectorAll<HTMLDetailsElement>("details[data-row]")) {
    const was = chosen.get(one.dataset.row);
    if (was === undefined) continue;
    one.open = was;
    one.dataset.chosen = "";
  }
  heldAgain(host, held);
};
