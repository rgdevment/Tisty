import { attached, docDrawn, type Paper } from "../core";
import { t } from "../locales";
import { named, pictured } from "../previews";
import { iconed } from "./dressing";
import { coloredAll, escaped, saidKinds, shell } from "./paging";
import { PAGE } from "./sheets";

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
    said += String.fromCharCode(...bytes.slice(at, at + 0x8000));
  return btoa(said);
};

const worded = (page: Document, said: string): HTMLSpanElement => {
  const one = page.createElement("span");
  one.textContent = said;
  return one;
};

// A page that travels alone carries its pictures inside it, since the files they name stay here.
const carried = async (page: Document) => {
  for (const img of page.querySelectorAll<HTMLImageElement>(".page img[src]")) {
    const at = img.getAttribute("src") ?? "";
    if (!at || /^(https?|data):/i.test(at)) continue;
    const words = img.getAttribute("alt") || named(at);
    if (!pictured(at)) {
      img.replaceWith(worded(page, words));
      continue;
    }
    try {
      img.setAttribute("src", `data:${kindOf(at)};base64,${coded(await attached(at))}`);
    } catch {
      img.replaceWith(worded(page, words));
    }
  }
};

const unlinked = (page: Document) => {
  for (const card of page.querySelectorAll<HTMLElement>("a.card[data-doc]"))
    card.replaceWith(worded(page, card.textContent ?? ""));
  for (const link of page.querySelectorAll<HTMLAnchorElement>('a[href^="tisty:"]'))
    link.replaceWith(...link.childNodes);
  for (const one of page.querySelectorAll("[data-at]")) one.removeAttribute("data-at");
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

type Sheet = { readonly leaf: Paper; readonly signed?: string };

const papered = (page: Document, { leaf, signed }: Sheet) => {
  page.head.insertAdjacentHTML(
    "beforeend",
    `<style>@page{size:${PAGE[leaf]};margin:22mm 20mm}html:not(.tall),html:not(.tall) body{background:#fff;padding:0}html:not(.tall) .page{max-width:none;margin:0;padding:0;border-radius:0;box-shadow:none;background:none}.page>*{content-visibility:visible}.page>hr{break-after:page;height:0;margin:0;background:none}.page>hr::before,.page>hr::after{display:none}.signed{margin-top:2em;font-size:.8em;color:var(--tisty-faint)}</style>`,
  );
  for (const one of page.querySelectorAll("details")) one.setAttribute("open", "");
  if (!signed) return;
  const said = page.createElement("p");
  said.className = "signed";
  said.textContent = signed;
  page.querySelector("article.page")?.append(said);
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
