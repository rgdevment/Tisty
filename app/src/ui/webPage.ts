import { attached, docDrawn, type Paper } from "../core";
import { t } from "../locales";
import { named, pictured } from "../previews";
import { diagrams } from "./diagrams";
import { iconed } from "./dressing";
import { coloredAll, escaped, saidKinds, shell } from "./paging";
import type { Stamp } from "./printing";
import { PRINTED, sizeOf } from "./sheets";
import { coverOn, creditOn } from "./stamped";

const KINDS: Record<string, string> = {
  png: "image/png",
  jpg: "image/jpeg",
  jpeg: "image/jpeg",
  gif: "image/gif",
  webp: "image/webp",
  avif: "image/avif",
  bmp: "image/bmp",
  svg: "image/svg+xml",
};

const kindOf = (at: string): string =>
  KINDS[at.split(/[?#]/)[0].split(".").pop()?.toLowerCase() ?? ""] ?? "application/octet-stream";

const coded = (bytes: number[]): string => {
  let said = "";
  for (let at = 0; at < bytes.length; at += 0x8000)
    said += String.fromCodePoint(...bytes.slice(at, at + 0x8000));
  return btoa(said);
};

const worded = (page: Document, said: string): HTMLSpanElement => {
  const one = page.createElement("span");
  one.textContent = said;
  return one;
};

// A page that travels alone carries its pictures inside it, since the files they name stay here.
const carried = (page: Document): Promise<unknown> =>
  Promise.all(
    Array.from(page.querySelectorAll<HTMLImageElement>(".page img[src]")).map(async (img) => {
      // A picture waiting to be scrolled to would never be, in a window nobody sees.
      img.removeAttribute("loading");
      const at = img.getAttribute("src") ?? "";
      if (!at || /^(https?|data):/i.test(at)) return;
      const words = img.getAttribute("alt") || named(at);
      if (!pictured(at)) {
        img.replaceWith(worded(page, words));
        return;
      }
      try {
        img.setAttribute("src", `data:${kindOf(at)};base64,${coded(await attached(at))}`);
      } catch {
        img.replaceWith(worded(page, words));
      }
    }),
  );

const unlinked = (page: Document) => {
  for (const card of page.querySelectorAll<HTMLElement>("a.card[data-doc]"))
    card.replaceWith(worded(page, card.textContent ?? ""));
  for (const link of page.querySelectorAll<HTMLAnchorElement>('a[href^="tisty:"]'))
    link.replaceWith(...link.childNodes);
  for (const one of page.querySelectorAll<HTMLElement>("[data-at]")) delete one.dataset.at;
  page.querySelector("article.page")?.removeAttribute("inert");
};

// A page read elsewhere is read on white, whichever theme the window wears here.
const lit = (): string => {
  const found: string[] = [];
  for (const sheet of Array.from(document.styleSheets)) {
    let rules: CSSRuleList;
    try {
      rules = sheet.cssRules;
    } catch {
      continue;
    }
    for (const rule of Array.from(rules)) {
      if (!(rule instanceof CSSStyleRule) || rule.selectorText !== ":root") continue;
      for (const name of Array.from(rule.style))
        if (name.startsWith("--tisty-"))
          found.push(`${name}:${rule.style.getPropertyValue(name).trim()}`);
    }
  }
  return found.filter((one) => !/[<>{}]/.test(one)).join(";");
};

export type PaperSheet = {
  readonly leaf: Paper;
  readonly landscape?: boolean;
  readonly stamp?: Stamp;
  /** How the first body is seen on screen: where its sheets turn, which folds are open. */
  readonly turns?: readonly string[];
  readonly opened?: readonly string[];
};

// Paper turns where the screen's sheets turn and folds what the screen folds, so what is read there is what is printed.
const asSeen = (drawn: string, { turns = [], opened = [] }: PaperSheet): string => {
  if (!turns.length && !opened.length) return drawn;
  const held = new DOMParser().parseFromString(`<body>${drawn}</body>`, "text/html").body;
  for (const one of held.children) {
    const at = (one as HTMLElement).dataset.at ?? "";
    if (turns.includes(at)) one.classList.add("turned");
    if (one.tagName === "DETAILS" && opened.includes(at)) one.setAttribute("open", "");
  }
  return held.innerHTML;
};

const papered = (page: Document, { leaf, landscape, stamp }: PaperSheet) => {
  page.head.insertAdjacentHTML(
    "beforeend",
    `<style>@page{size:${sizeOf(leaf, landscape)};${PRINTED}}html:not(.tall),html:not(.tall) body{background:#fff;padding:0}html:not(.tall) .page{max-width:none;margin:0;padding:0;border-radius:0;box-shadow:none;background:none}.page>*{content-visibility:visible}.page>hr{break-after:page;height:0;margin:0;background:none}.page>hr::before,.page>hr::after{display:none}.page>:is(h1,h2,h3,h4,h5,h6){break-after:avoid;page-break-after:avoid}.page>:not(hr){break-inside:avoid;page-break-inside:avoid}.page>.turned{break-before:page;page-break-before:always}.page>hr+*,.page>.turned{margin-top:0}</style>`,
  );
  coverOn(page, stamp?.card);
  if (stamp?.credited) creditOn(page);
};

export const webPage = async (
  bodies: string[],
  title: string,
  sheet?: PaperSheet,
): Promise<string> => {
  const drawn = await Promise.all(bodies.map(docDrawn));
  if (drawn.length && sheet) drawn[0] = asSeen(drawn[0], sheet);
  const page = new DOMParser().parseFromString(shell(drawn.join("<hr>")), "text/html");
  page.documentElement.classList.remove("dark");
  const light = lit();
  const tokens = page.head.querySelector("style");
  if (light && tokens) tokens.textContent = `:root{${light}}`;
  page.head.insertAdjacentHTML(
    "afterbegin",
    `<meta name="viewport" content="width=device-width, initial-scale=1"><title>${escaped(title)}</title>`,
  );
  saidKinds(page, (kind) => t(`said${kind}` as Parameters<typeof t>[0]));
  iconed(page);
  coloredAll(page);
  await diagrams(page, false, () => true);
  unlinked(page);
  if (sheet) papered(page, sheet);
  await carried(page);
  return `<!doctype html>${page.documentElement.outerHTML}`;
};
