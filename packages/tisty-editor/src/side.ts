import type { Field } from "./front";
import type { Head } from "./heads";
import { offered, type Paper } from "./paper";
import type { ReadWord, Say } from "./words";

// The words a body holds, its code and its marks left out, as the details count them.
export const wordsIn = (body: string): number =>
  body
    .replace(/```[\s\S]*?```/g, " ")
    .replace(/[#>*_`~[\]()!|-]/g, " ")
    .split(/\s+/)
    .filter(Boolean).length;

type Turn = { readonly on: boolean; readonly onTurn: (on: boolean) => void };

export type Facts = {
  readonly body: string;
  readonly author?: string;
  readonly born?: string;
  readonly editor?: string;
  /** Already in the reader's words, as the host tells dates. */
  readonly made?: string;
  readonly wrote?: string;
  readonly weighs?: string;
  /** What it holds besides its words: pages, documents, files. */
  readonly holds?: string;
  readonly fields?: readonly Field[];
};

export type Seal = {
  readonly verdict: "holds" | "revoked" | "refused" | "unreadable" | "unsigned";
  readonly who?: string;
  readonly fingerprint?: string;
  readonly sender?: string;
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
      readonly onShut?: () => void;
    }
  | {
      readonly kind: "paper";
      readonly leaf: Paper;
      readonly onLeaf: (leaf: Paper) => void;
      readonly sheets: Turn;
      /** Only while the sheets are laid. */
      readonly landscape?: Turn;
      readonly raw?: Turn;
    }
  | { readonly kind: "index"; readonly heads: readonly Head[] }
  | { readonly kind: "details"; readonly facts: Facts }
  | { readonly kind: "seal"; readonly seal: Seal }
  | { readonly kind: "shared"; readonly gist: string; readonly said?: string }
  | { readonly kind: "acts"; readonly acts: readonly Act[] };

const LEAF: Record<Paper, ReadWord> = {
  a4: "leafA4",
  letter: "leafLetter",
  folio: "leafFolio",
  tabloid: "leafTabloid",
};

const made = <K extends keyof HTMLElementTagNameMap>(
  page: Document,
  tag: K,
  className: string,
  said = "",
): HTMLElementTagNameMap[K] => {
  const one = page.createElement(tag);
  one.className = className;
  one.textContent = said;
  return one;
};

const pressed = (
  page: Document,
  className: string,
  said: string,
  run: (at: HTMLElement) => void,
) => {
  const one = made(page, "button", className, said);
  one.type = "button";
  one.addEventListener("click", () => run(one));
  return one;
};

const toggle = (
  page: Document,
  turn: Turn,
  said: string,
  glyph: string,
  className = "side-tool",
) => {
  const one = pressed(page, className, glyph, () => turn.onTurn(!turn.on));
  one.setAttribute("aria-label", said);
  one.setAttribute("aria-pressed", String(turn.on));
  one.title = said;
  return one;
};

const titled = (page: Document, say: Say, panel: Extract<Panel, { kind: "title" }>) => {
  const head = made(page, "header", "side-title-row");
  const line = made(page, "div", "side-title-line");
  line.append(made(page, "h1", "side-title", panel.title));
  if (panel.onShut) {
    const shut = pressed(page, "side-shut", "✕", () => panel.onShut?.());
    shut.setAttribute("aria-label", say("besideShut"));
    shut.title = say("besideShut");
    line.append(shut);
  }
  head.append(line);
  if (panel.under) head.append(made(page, "p", "side-under", panel.under));
  return head;
};

const papered = (page: Document, say: Say, panel: Extract<Panel, { kind: "paper" }>) => {
  const row = made(page, "div", "side-paper");
  const leaves = made(page, "div", "side-leaves");
  leaves.setAttribute("role", "group");
  leaves.setAttribute("aria-label", say("leafIs"));
  for (const one of offered(panel.leaf)) {
    const leaf = pressed(page, "side-leaf", say(LEAF[one]), () => panel.onLeaf(one));
    leaf.setAttribute("aria-pressed", String(one === panel.leaf));
    leaves.append(leaf);
  }
  row.append(leaves, toggle(page, panel.sheets, say("sheetsShown"), "⇕"));
  if (panel.landscape) row.append(toggle(page, panel.landscape, say("sheetsAcross"), "⇔"));
  if (panel.raw) row.append(toggle(page, panel.raw, say("viewSource"), "</>", "side-tool is-code"));
  return row;
};

const indexed = (page: Document, say: Say, panel: Extract<Panel, { kind: "index" }>) => {
  const nav = made(page, "nav", "side-index");
  nav.setAttribute("aria-label", say("outline"));
  nav.append(made(page, "h2", "side-label", say("outline")));
  const heads = panel.heads.filter((head) => head.level > 0);
  if (!heads.length) nav.append(made(page, "p", "side-none", say("outlineNone")));
  for (const one of heads) {
    const go = pressed(page, "side-go", one.text, () => one.go());
    go.style.setProperty("--depth", String(Math.max(0, one.level - 1)));
    nav.append(go);
  }
  return nav;
};

type Line = {
  readonly key: string;
  readonly label: string;
  readonly gist: string;
  readonly tone?: string;
};

const rowed = (page: Document, line: Line, fill?: (into: HTMLElement) => void) => {
  const label = made(page, "strong", "side-row-label", line.label);
  const gist = made(page, "span", `side-gist${line.tone ? ` is-${line.tone}` : ""}`, line.gist);
  if (!fill) {
    const still = made(page, "div", "side-row");
    still.append(label, gist);
    return still;
  }
  const open = made(page, "details", "side-row");
  open.dataset.row = line.key;
  const summary = made(page, "summary", "side-row-head");
  summary.append(label, gist);
  const body = made(page, "div", "side-row-body");
  fill(body);
  open.append(summary, body);
  return open;
};

const listed = (page: Document, pairs: readonly (readonly [string, string | undefined])[]) => {
  const list = made(page, "dl", "side-facts");
  for (const [term, said] of pairs) {
    if (!said) continue;
    const pair = made(page, "div", "side-fact");
    pair.append(made(page, "dt", "", term), made(page, "dd", "", said));
    list.append(pair);
  }
  return list;
};

const detailed = (page: Document, say: Say, { facts }: Extract<Panel, { kind: "details" }>) => {
  const words = wordsIn(facts.body);
  const long = words ? say("paperWords", `${words}`) : say("paperEmpty");
  const gist = [long, facts.weighs].filter(Boolean).join(" · ");
  return rowed(page, { key: "details", label: say("besideDetails"), gist }, (into) => {
    into.append(
      listed(page, [
        [say("paperAuthor"), facts.author],
        [say("paperBorn"), facts.born],
        [say("paperEditor"), facts.editor],
        [say("paperMade"), facts.made],
        [say("paperWrote"), facts.wrote],
        [say("paperLong"), long],
        [say("paperWeighs"), facts.weighs],
        [say("paperHolds"), facts.holds],
        ...(facts.fields ?? []).map((one) => [one.key, one.values.join(", ")] as const),
      ]),
    );
  });
};

const VERDICT = {
  holds: ["signedHolds", "signedHoldsSaid", "good"],
  revoked: ["signedRevoked", "signedRevokedSaid", "warn"],
  refused: ["signedRefused", "signedRefusedSaid", "bad"],
  unreadable: ["signedUnreadable", "signedUnreadableSaid", "bad"],
} as const;

// Folded until asked: whose key, its print and what it says; a body nobody signed has nothing more to show.
const sealed = (page: Document, say: Say, { seal }: Extract<Panel, { kind: "seal" }>) => {
  if (seal.verdict === "unsigned")
    return rowed(page, { key: "seal", label: say("signedTitle"), gist: say("signedNone") });
  const [short, long, tone] = VERDICT[seal.verdict];
  const gist = [seal.who, say(short)].filter(Boolean).join(" · ");
  return rowed(page, { key: "seal", label: say("signedTitle"), gist, tone }, (into) => {
    into.append(made(page, "p", "side-said", say(long)));
    if (seal.fingerprint) into.append(made(page, "code", "side-print", seal.fingerprint));
    if (seal.sender)
      into.append(made(page, "p", "side-said", `${say("signedSentBy")} ${seal.sender}`));
  });
};

const shared = (page: Document, say: Say, panel: Extract<Panel, { kind: "shared" }>) => {
  const said = panel.said;
  const fill = said
    ? (into: HTMLElement) => into.append(made(page, "p", "side-said", said))
    : undefined;
  return rowed(page, { key: "shared", label: say("besideShared"), gist: panel.gist }, fill);
};

const acted = (page: Document, act: Act) => {
  const className = `side-act${act.main ? " is-main" : ""}`;
  if (!act.href) return pressed(page, className, act.label, (at) => act.onPick?.(at));
  const link = made(page, "a", className, act.label);
  link.href = act.href;
  if (act.download) link.download = act.download;
  if (act.onPick) link.addEventListener("click", () => act.onPick?.(link));
  return link;
};

const drawn = (page: Document, say: Say, panel: Panel): HTMLElement => {
  switch (panel.kind) {
    case "title":
      return titled(page, say, panel);
    case "paper":
      return papered(page, say, panel);
    case "index":
      return indexed(page, say, panel);
    case "details":
      return detailed(page, say, panel);
    case "seal":
      return sealed(page, say, panel);
    case "shared":
      return shared(page, say, panel);
    case "acts": {
      const acts = made(page, "div", "side-acts");
      for (const act of panel.acts) acts.append(acted(page, act));
      return acts;
    }
  }
};

// What names the sheet sits in the first card, with its index; what is said about it, and what can be done, in the second.
const ABOUT = new Set<Panel["kind"]>(["title", "paper", "index"]);

// Drawn again whole on every change: it holds a few dozen nodes, and a row left open stays open.
export const sided = (host: HTMLElement, say: Say, panels: readonly Panel[]) => {
  const page = host.ownerDocument;
  const opened = new Set(
    Array.from(
      host.querySelectorAll<HTMLDetailsElement>("details[open]"),
      (one) => one.dataset.row,
    ),
  );
  const top = made(page, "section", "side-card side-top");
  const more = made(page, "section", "side-card side-more");
  for (const panel of panels) (ABOUT.has(panel.kind) ? top : more).append(drawn(page, say, panel));
  host.replaceChildren(...[top, more].filter((card) => card.childElementCount));
  for (const one of host.querySelectorAll<HTMLDetailsElement>("details[data-row]"))
    one.open = opened.has(one.dataset.row);
};
