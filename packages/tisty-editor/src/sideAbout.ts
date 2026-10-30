import { glyph } from "./cards";
import { offered, type Paper } from "./paper";
import type { About, Book } from "./side";
import { made, pressed, toggle, tooled } from "./sideParts";
import type { ReadWord, Say } from "./words";

type Of<K extends About["kind"]> = Extract<About, { kind: K }>;

const LEAF: Record<Paper, ReadWord> = {
  a4: "leafA4",
  letter: "leafLetter",
  folio: "leafFolio",
  tabloid: "leafTabloid",
};

const trailed = (page: Document, panel: Of<"title">) => {
  const line = made(page, "p", "side-under");
  if (panel.under) line.append(panel.under);
  for (const [at, step] of (panel.trail ?? []).entries()) {
    if (at > 0 || panel.under) line.append(made(page, "span", "side-slash", " / "));
    line.append(pressed(page, "side-step", step.label, () => step.onPick()));
  }
  return line;
};

const titled = (page: Document, say: Say, panel: Of<"title">) => {
  const head = made(page, "header", "side-title-row");
  const line = made(page, "div", "side-title-line");
  line.append(made(page, "h2", "side-title", panel.title));
  const { onApart, onShut } = panel;
  if (onApart)
    line.append(tooled(page, "side-apart", say("viewDocApart"), onApart, glyph(page, "window")));
  if (onShut) line.append(tooled(page, "side-shut", say("besideShut"), onShut, "✕"));
  head.append(line);
  if (panel.under || panel.trail?.length) head.append(trailed(page, panel));
  return head;
};

const papered = (page: Document, say: Say, panel: Of<"paper">) => {
  const row = made(page, "div", "side-paper");
  const leaves = made(page, "div", "side-leaves");
  leaves.setAttribute("role", "group");
  leaves.setAttribute("aria-label", say("leafIs"));
  for (const one of offered(panel.leaf)) {
    const leaf = pressed(page, "side-leaf", say(LEAF[one]), () => panel.onLeaf(one));
    leaf.setAttribute("aria-pressed", String(one === panel.leaf));
    leaves.append(leaf);
  }
  row.append(leaves);
  if (panel.sheets) row.append(toggle(page, panel.sheets, say("sheetsShown"), "⇕"));
  if (panel.landscape) row.append(toggle(page, panel.landscape, say("sheetsAcross"), "⇔"));
  if (panel.raw) row.append(toggle(page, panel.raw, say("viewSource"), "</>", "side-tool is-code"));
  const { onExport } = panel;
  if (onExport) {
    const said = say(panel.exporting ? "makingPdf" : "exportIt");
    const opens = glyph(page, "chevron");
    opens.classList.add("is-small");
    const out = tooled(page, "side-tool", said, onExport, glyph(page, "download"), opens);
    out.setAttribute("aria-haspopup", "menu");
    out.setAttribute("aria-busy", String(Boolean(panel.exporting)));
    row.append(out);
  }
  return row;
};

const leafLabel = (say: Say, one: Book["pages"][number]): string | undefined => {
  const named = one.title || say("untitledDoc");
  if (one.away) return `${named} — ${say("isArchived")}`;
  if (one.flagged) return `${named} — ${say("docFlagged")}`;
  return undefined;
};

const booked = (page: Document, say: Say, book: Book, nav: HTMLElement) => {
  const { up } = book;
  if (up) {
    nav.append(made(page, "h4", "side-label is-sub", say("besidePageOf")));
    const back = pressed(page, "side-go is-page is-up", "", () => up.onPick());
    back.append(glyph(page, "chevron"), made(page, "span", "", up.title || say("untitledDoc")));
    nav.append(back);
  }
  if (book.pages.length) nav.append(made(page, "h4", "side-label is-sub", book.said));
  for (const one of book.pages) {
    const go = pressed(page, `side-go is-page${one.away ? " is-away" : ""}`, "", () =>
      one.onPick(),
    );
    go.append(
      glyph(page, one.archived ? "archive" : "alignleft"),
      made(page, "span", "", one.title || say("untitledDoc")),
    );
    if (one.flagged && !one.away) go.append(made(page, "span", "side-flag", "◆"));
    const label = leafLabel(say, one);
    if (label) go.setAttribute("aria-label", label);
    if (one.current) go.setAttribute("aria-current", "true");
    nav.append(go);
  }
};

const indexed = (page: Document, say: Say, panel: Of<"index">) => {
  const nav = made(page, "nav", "side-index");
  nav.setAttribute("aria-label", say("outline"));
  nav.append(made(page, "h3", "side-label", say("outline")));
  const heads = panel.heads.filter((head) => head.level > 0);
  const { book } = panel;
  if (!heads.length && !book?.pages.length && !book?.up)
    nav.append(made(page, "p", "side-none", say("outlineNone")));
  for (const one of heads) {
    const go = pressed(page, "side-go", one.text, () => one.go());
    go.style.setProperty("--depth", String(Math.max(0, one.level - 1)));
    nav.append(go);
  }
  if (book) booked(page, say, book, nav);
  return nav;
};

export const besideAbout = (page: Document, say: Say, panel: About): HTMLElement => {
  if (panel.kind === "title") return titled(page, say, panel);
  if (panel.kind === "paper") return papered(page, say, panel);
  return indexed(page, say, panel);
};
