import { attached, docDrawn, type Paper } from "../core";
import { t } from "../locales";
import { named, pictured } from "../previews";
import { iconed } from "./dressing";
import { coloredAll, escaped, saidKinds, shell } from "./paging";
import type { Stamp } from "./printing";
import { PRINTED, sizeOf } from "./sheets";

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

type Sheet = { readonly leaf: Paper; readonly landscape?: boolean; readonly stamp?: Stamp };

const STAMPED =
  ".stamp{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:.85em 2em;margin:0 0 2em;padding:1em 1.25em;border:1px solid var(--tisty-line);border-radius:10px;background:var(--tisty-rail);break-inside:avoid}.stamp>div{min-width:0}.stamp dt{margin:0 0 .2em;font-size:.68em;font-weight:600;letter-spacing:.08em;text-transform:uppercase;color:var(--tisty-faint)}.stamp dd{margin:0;font-size:.9em;color:var(--tisty-ink);overflow-wrap:anywhere}.credit{margin-top:3em;text-align:center;font-size:.7em;color:var(--tisty-faint);opacity:.75}.credit a{color:inherit;text-decoration:none}";

const cardOn = (page: Document, card: Stamp["card"]) => {
  const sheet = page.querySelector("article.page");
  if (!sheet || !card?.length) return;
  const list = page.createElement("dl");
  list.className = "stamp";
  for (const { label, said } of card) {
    const pair = page.createElement("div");
    const named = page.createElement("dt");
    named.textContent = label;
    const told = page.createElement("dd");
    told.textContent = said;
    pair.append(named, told);
    list.append(pair);
  }
  const head = Array.from(sheet.children).find((one) => !one.matches(".front, .fronted"));
  if (head && /^H[1-3]$/.test(head.tagName)) head.after(list);
  else if (head) head.before(list);
  else sheet.append(list);
};

const creditOn = (page: Document) => {
  const said = page.createElement("p");
  said.className = "credit";
  const home = page.createElement("a");
  home.href = "https://www.tisty.app";
  home.textContent = t("pdfCredit");
  said.append(home);
  page.querySelector("article.page")?.append(said);
};

const papered = (page: Document, { leaf, landscape, stamp }: Sheet) => {
  page.head.insertAdjacentHTML(
    "beforeend",
    `<style>@page{size:${sizeOf(leaf, landscape)};${PRINTED}}html:not(.tall),html:not(.tall) body{background:#fff;padding:0}html:not(.tall) .page{max-width:none;margin:0;padding:0;border-radius:0;box-shadow:none;background:none}.page>*{content-visibility:visible}.page>hr{break-after:page;height:0;margin:0;background:none}.page>hr::before,.page>hr::after{display:none}.page>:is(h1,h2,h3,h4,h5,h6){break-after:avoid;page-break-after:avoid}${STAMPED}</style>`,
  );
  for (const one of page.querySelectorAll("details")) one.setAttribute("open", "");
  cardOn(page, stamp?.card);
  if (stamp?.credited) creditOn(page);
};

export const webPage = async (bodies: string[], title: string, sheet?: Sheet): Promise<string> => {
  const drawn = await Promise.all(bodies.map(docDrawn));
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
  unlinked(page);
  if (sheet) papered(page, sheet);
  await carried(page);
  return `<!doctype html>${page.documentElement.outerHTML}`;
};
