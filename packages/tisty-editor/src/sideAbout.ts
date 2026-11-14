import { glyph, type ReadWord, type Say } from "tisty-sheet";
import { offered, type Paper } from "./paper";
import type { About, Book, Tongue, Turn } from "./side";
import { made, pressed, toggle, tooled } from "./sideParts";

type Of<K extends About["kind"]> = Extract<About, { kind: K }>;

const LEAF: Record<Paper, ReadWord> = {
  a4: "leafA4",
  letter: "leafLetter",
  folio: "leafFolio",
  tabloid: "leafTabloid",
};

const IN_ONE_ROW = 3;
const FIND_FROM = 9;

const trailed = (page: Document, panel: Of<"title">) => {
  const line = made(page, "p", "side-under");
  if (panel.under) line.append(panel.under);
  for (const [at, step] of (panel.trail ?? []).entries()) {
    if (at > 0 || panel.under) line.append(made(page, "span", "side-slash", " / "));
    line.append(pressed(page, "side-step", step.label, () => step.onPick()));
  }
  return line;
};

const plain = (said: string) => said.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase();

const segmented = (page: Document, say: Say, tongues: readonly Tongue[]) => {
  const row = made(page, "div", "side-leaves side-tongues");
  row.setAttribute("role", "group");
  row.setAttribute("aria-label", say("tonguesOf"));
  for (const one of tongues) {
    const button = pressed(page, "side-leaf", one.label, () => !one.on && one.onPick());
    button.setAttribute("lang", one.lang);
    button.setAttribute("aria-pressed", String(one.on));
    button.title = one.label;
    button.disabled = one.away === true;
    row.append(button);
  }
  return row;
};

const chosen = (page: Document, one: Tongue, shut: () => void) => {
  const item = pressed(page, one.on ? "side-tongue-item on" : "side-tongue-item", "", () => {
    shut();
    if (!one.on) one.onPick();
  });
  item.setAttribute("role", "option");
  item.setAttribute("aria-selected", String(one.on));
  item.setAttribute("lang", one.lang);
  item.dataset.find = plain(`${one.label} ${one.lang}`);
  item.append(
    made(page, "span", "side-tongue-code", one.lang.toUpperCase()),
    made(page, "span", "side-tongue-label", one.label),
  );
  if (one.on) item.append(made(page, "span", "side-tongue-mark", "✓"));
  return item;
};

const finder = (page: Document, say: Say, items: HTMLElement[]) => {
  const find = made(page, "input", "side-tongue-find");
  find.type = "search";
  find.placeholder = say("tonguesFind");
  find.setAttribute("aria-label", say("tonguesFind"));
  find.addEventListener("input", () => {
    const asked = plain(find.value.trim());
    for (const item of items) item.hidden = !item.dataset.find?.includes(asked);
  });
  return find;
};

const stepped = (live: HTMLElement[], from: EventTarget | null, by: number) => {
  const at = live.indexOf(from as HTMLElement);
  if (at < 0) return live[by > 0 ? 0 : live.length - 1];
  return live[(at + by + live.length) % live.length];
};

const picker = (page: Document, say: Say, tongues: readonly Tongue[]) => {
  const now = tongues.find((one) => one.on) ?? tongues[0];
  const live = tongues.filter((one) => !one.away);
  const box = made(page, "div", "side-tongue-pick");
  const list = made(page, "div", "side-tongue-list");
  list.hidden = true;
  list.setAttribute("role", "listbox");
  list.setAttribute("aria-label", say("tonguesOf"));
  const opener = made(page, "button", "side-tongue-now");
  opener.type = "button";
  const away = (event: Event) => {
    if (!box.contains(event.target as Node)) shut();
  };
  const moved = (event: Event) => {
    if (!list.contains(event.target as Node)) shut();
  };
  const shut = () => {
    list.hidden = true;
    opener.setAttribute("aria-expanded", "false");
    page.removeEventListener("mousedown", away, true);
    page.removeEventListener("scroll", moved, true);
    page.defaultView?.removeEventListener("resize", shut);
  };
  const items = live.map((one) => chosen(page, one, shut));
  const find = live.length >= FIND_FROM ? finder(page, say, items) : null;
  opener.addEventListener("click", () => {
    if (!list.hidden) return shut();
    const at = opener.getBoundingClientRect();
    list.style.top = `${at.bottom + 4}px`;
    list.style.left = `${at.left}px`;
    list.style.width = `${at.width}px`;
    list.hidden = false;
    opener.setAttribute("aria-expanded", "true");
    page.addEventListener("mousedown", away, true);
    page.addEventListener("scroll", moved, true);
    page.defaultView?.addEventListener("resize", shut);
    (find ?? items.find((one) => one.classList.contains("on")) ?? items[0])?.focus();
  });
  opener.setAttribute("aria-haspopup", "listbox");
  opener.setAttribute("aria-expanded", "false");
  opener.setAttribute("aria-label", `${say("tonguesOf")} · ${now.label} · ${live.length}`);
  opener.append(
    glyph(page, "language"),
    made(page, "span", "side-tongue-name", now.label),
    made(page, "span", "side-tongue-many", `${live.length}`),
    glyph(page, "chevron"),
  );
  list.addEventListener("keydown", (event) => {
    const live = items.filter((one) => !one.hidden);
    if (event.key === "Escape") {
      event.stopPropagation();
      shut();
      opener.focus();
    } else if (event.key === "Enter" && event.target === find) {
      event.preventDefault();
      live[0]?.click();
    } else if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      stepped(live, event.target, event.key === "ArrowDown" ? 1 : -1)?.focus();
    }
  });
  if (find) list.append(find);
  list.append(...items);
  box.append(opener, list);
  return box;
};

const tongued = (page: Document, say: Say, tongues: readonly Tongue[]) =>
  tongues.filter((one) => !one.away).length > IN_ONE_ROW
    ? picker(page, say, tongues)
    : segmented(page, say, tongues);

// The page as drawn or as written, chosen side by side as the languages are.
const viewed = (page: Document, say: Say, view: Turn) => {
  const row = made(page, "div", "side-leaves side-tongues");
  row.setAttribute("role", "group");
  for (const [markdown, word] of [
    [false, "viewPreview"],
    [true, "viewMarkdown"],
  ] as const) {
    const button = pressed(page, "side-leaf", say(word), () => {
      if (view.on !== markdown) view.onTurn(markdown);
    });
    button.setAttribute("aria-pressed", String(view.on === markdown));
    row.append(button);
  }
  return row;
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
  if (panel.tongues?.length) head.append(tongued(page, say, panel.tongues));
  if (panel.view) head.append(viewed(page, say, panel.view));
  return head;
};

const papered = (page: Document, say: Say, panel: Of<"paper">) => {
  const row = made(page, "div", "side-paper");
  const leaves = made(page, "div", "side-leaves");
  leaves.setAttribute("role", "group");
  leaves.setAttribute("aria-label", say("leafIs"));
  // Only where sheets can be turned on: elsewhere the paper would have no way to change at all.
  const waiting = panel.sheets !== undefined && !panel.sheets.on;
  if (waiting) leaves.title = say("leafNeedsSheets");
  for (const one of offered(panel.leaf)) {
    const leaf = pressed(page, "side-leaf", say(LEAF[one]), () => panel.onLeaf(one));
    leaf.setAttribute("aria-pressed", String(one === panel.leaf));
    leaf.disabled = waiting;
    leaves.append(leaf);
  }
  if (panel.sheets) row.append(toggle(page, panel.sheets, say("sheetsShown"), "⇕"));
  row.append(leaves);
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
