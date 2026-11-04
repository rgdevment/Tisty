import { coloredAll } from "./coloring";
import { drawing } from "./diagrams";
import { flattened } from "./flattening";

/** A document's HTML as it travels, and how many of its diagrams could not be drawn, which the person is told. */
export type Finished = { readonly html: string; readonly notDrawn: number };

type Draw = (source: string, dark: boolean) => Promise<string | null>;

// The set is heavy, so it comes only for a page that names an icon, as it does for the window.
const iconsIn = async (page: Document) => {
  const named = [...page.querySelectorAll<HTMLElement>("span.ico[data-ico]")];
  if (!named.length) return;
  const { markup } = await import("./glyphs");
  for (const one of named) {
    const drawn = markup(one.dataset.ico ?? "");
    if (drawn === null) continue;
    const glyph = page.createElementNS("http://www.w3.org/2000/svg", "svg");
    glyph.setAttribute("viewBox", "0 0 24 24");
    glyph.setAttribute("aria-hidden", "true");
    glyph.setAttribute("fill", "none");
    glyph.setAttribute("stroke", "currentColor");
    glyph.setAttribute("stroke-width", "2");
    glyph.setAttribute("stroke-linecap", "round");
    glyph.setAttribute("stroke-linejoin", "round");
    glyph.innerHTML = drawn;
    for (const solid of glyph.querySelectorAll(".solid")) {
      solid.removeAttribute("class");
      solid.setAttribute("fill", "currentColor");
      solid.setAttribute("stroke", "none");
    }
    one.replaceChildren(glyph);
  }
};

const flat = async (
  source: string,
  dark: boolean,
  prefix: string,
  draw: Draw,
  room: HTMLElement,
) => {
  const drawn = await draw(source, dark);
  if (!drawn) return null;
  room.innerHTML = drawn;
  const live = room.querySelector("svg");
  return live ? flattened(live, prefix) : null;
};

const asCode = (page: Document, source: string) => {
  const figure = page.createElement("figure");
  figure.className = "code";
  const pre = page.createElement("pre");
  const code = page.createElement("code");
  code.textContent = source;
  pre.append(code);
  figure.append(pre);
  return figure;
};

// Named after what it draws: with no frame around them, drawings from two documents share a page, and a line reaches the first marker of its name.
const nameOf = (source: string) => {
  let hash = 0x811c9dc5;
  for (const one of source) hash = Math.imul(hash ^ (one.codePointAt(0) ?? 0), 0x01000193);
  return (hash >>> 0).toString(36);
};

const diagramsIn = async (page: Document, draw: Draw) => {
  const figures = [...page.querySelectorAll("figure.diagram")];
  if (!figures.length) return 0;
  // Only in this window's page does a drawing have a style to read, so each is laid out of sight for a moment.
  const room = document.createElement("div");
  room.setAttribute("aria-hidden", "true");
  room.style.cssText = "position:absolute;left:-99999px;top:0;width:1600px;visibility:hidden";
  document.body.append(room);
  let notDrawn = 0;
  try {
    for (const [at, figure] of figures.entries()) {
      const source = figure.querySelector("pre > code")?.textContent ?? "";
      const named = `d${nameOf(source)}-${at}`;
      const light = await flat(source, false, `${named}-l`, draw, room);
      const dark = light && (await flat(source, true, `${named}-d`, draw, room));
      if (!light || !dark) {
        notDrawn += 1;
        figure.replaceWith(asCode(page, source));
        continue;
      }
      const held = page.createElement("template");
      held.innerHTML = light + dark;
      // A drawing can hold others, as icons, so only the two at the top are the two looks.
      const [one, other] = held.content.children;
      one.setAttribute("data-theme", "light");
      other.setAttribute("data-theme", "dark");
      figure.replaceChildren(one, other);
    }
  } finally {
    room.remove();
  }
  return notDrawn;
};

/** What only a window can do to a document before it travels: colours its code, draws its icons and its diagrams, and leaves nothing to work out. Rust gives it the references in place and looks at the result again before sealing. */
export const finished = async (html: string, draw: Draw = drawing): Promise<Finished> => {
  const page = new DOMParser().parseFromString(`<!doctype html><body>${html}</body>`, "text/html");
  await coloredAll(page);
  await iconsIn(page);
  const notDrawn = await diagramsIn(page, draw);
  return { html: page.body.innerHTML, notDrawn };
};
