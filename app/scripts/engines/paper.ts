import "./page.ts";
import "tisty-editor/tokens.css";
import { LIVELY_CSS as lively } from "tisty-sheet/lively";

const { framed, dressedAsWindow } = await import("/src/ui/framed.ts");
const { fitted, worn } = await import("/src/ui/dressing.ts");
const { docPlaced } = await import("/src/placing.ts");
const { webPage } = await import("/src/ui/webPage.ts");
const { seenIn } = await import("/src/ui/printing.ts");
const { diagrams, sheeting, sheetFor } = await import("tisty-editor/read");

type Mode = "lively" | "sheeted" | "tall" | "untitled" | "plain" | "pdf";

type Asked = {
  readonly mode: Mode;
  readonly width: number;
  readonly height?: number;
  readonly dark?: boolean;
  readonly wide?: number;
  readonly bare?: boolean;
  readonly row?: boolean;
  readonly measured?: boolean;
};

const SEEN = [
  "display",
  "font-size",
  "line-height",
  "font-weight",
  "letter-spacing",
  "color",
  "background-color",
  "border-radius",
  "box-shadow",
  "margin-top",
  "margin-right",
  "margin-bottom",
  "margin-left",
  "padding-top",
  "padding-right",
  "padding-bottom",
  "padding-left",
  "border-bottom-width",
  "border-bottom-color",
  "max-width",
];

const KINDS = [
  "h2",
  "h3",
  "p",
  "ul",
  "ol",
  "blockquote",
  "blockquote.callout",
  ".table",
  "figure.code",
  "figure.diagram",
  "details",
  "hr",
  ".sheet-turn",
];

const rounded = (n: number) => Math.round(n * 100) / 100;

const boxOf = (one: Element) => {
  const box = one.getBoundingClientRect();
  return [box.x, box.y, box.width, box.height].map(rounded);
};

const styled = (one: Element) => {
  const style = one.ownerDocument.defaultView?.getComputedStyle(one);
  return Object.fromEntries(SEEN.map((name) => [name, style?.getPropertyValue(name) ?? ""]));
};

const described = (one: Element | null) =>
  one && { tag: one.tagName, box: boxOf(one), ...styled(one) };

const kept = (one: Element | null, ...at: number[]) => {
  const seen = described(one);
  return seen && { ...seen, box: at.map((index) => seen.box[index]) };
};

const chOf = (sheet: Element) => {
  const probe = sheet.ownerDocument.createElement("span");
  probe.style.cssText = "position:absolute;width:1ch;visibility:hidden";
  sheet.append(probe);
  const wide = probe.getBoundingClientRect().width;
  probe.remove();
  return wide;
};

const onTop = (page: Document, one: Element | null) => {
  const box = one?.getBoundingClientRect();
  return box ? page.elementFromPoint(box.x + box.width / 2, box.y + box.height / 2) === one : null;
};

const laid = (page: Document) => {
  const sheet = page.querySelector(".page");
  if (!sheet) throw new Error("the frame drew no page");
  const blocks = Array.from(sheet.children);
  return {
    ch: chOf(sheet),
    printed: page.defaultView?.matchMedia("print").matches ?? false,
    classes: page.documentElement.className,
    root: styled(page.documentElement),
    row: described(page.querySelector(".page-more-row > button")),
    rowOnTop: onTop(page, page.querySelector(".page-more-row > button")),
    body: styled(page.body),
    page: kept(sheet, 0, 2),
    title: described(sheet.querySelector(":scope > .title")),
    first: blocks.slice(0, 12).map(described),
    kinds: Object.fromEntries(
      KINDS.map((kind) => [kind, kept(sheet.querySelector(`:scope > ${kind}`), 0, 2, 3)]),
    ),
  };
};

const settled = async (page: Document) => {
  await page.fonts?.ready;
  let last = "";
  let same = 0;
  for (let tries = 0; tries < 80 && same < 4; tries += 1) {
    await new Promise((done) => setTimeout(done, 150));
    const now = `${page.documentElement.scrollHeight}:${page.querySelectorAll("svg").length}`;
    same = now === last ? same + 1 : 0;
    last = now;
  }
};

const loaded = (held: HTMLIFrameElement, html: string) =>
  new Promise<Document>((ready, failed) => {
    held.addEventListener("load", () => {
      const page = held.contentDocument;
      if (page) ready(page);
      else failed(new Error("the frame never loaded"));
    });
    held.srcdoc = html;
  });

const STILL = "<style>*,*::before,*::after{transition:none!important;animation:none!important}</style>";

const without = (html: string) => html.replace(lively, "");

const PROBES = Array.from({ length: 81 }, (_, at) => 200 + at * 10);

const needs = (page: Document) => PROBES.map((across) => seenIn(page, across).wide.length);

const dressedAs = async (page: Document, asked: Asked, body: string) => {
  const root = page.documentElement;
  dressedAsWindow(page);
  root.classList.toggle("lively", asked.mode === "lively" || asked.mode === "sheeted");
  root.classList.toggle("tall", asked.mode === "tall" || asked.mode === "untitled");
  root.classList.toggle("untitled", asked.mode === "untitled");
  fitted(page, asked.wide);
  worn(page, () => true, { body, onDoc: () => {} });
  sheeting(page, () => (asked.mode === "sheeted" ? sheetFor("a4", false, "—") : undefined));
  await diagrams(page, Boolean(asked.dark), () => true);
};

const rowed = (page: Document) => {
  const row = page.createElement("div");
  row.className = "page-more-row";
  row.append(page.createElement("button"));
  page.body.prepend(row);
};

let shown: HTMLIFrameElement | null = null;

const lay = async (body: string, asked: Asked) => {
  shown?.remove();
  if (asked.dark) document.documentElement.setAttribute("data-theme", "dark");
  else document.documentElement.removeAttribute("data-theme");
  const held = document.createElement("iframe");
  held.style.cssText = `display:block;width:${asked.width}px;height:${asked.height ?? 900}px;border:0`;
  document.getElementById("root")?.append(held);
  shown = held;
  const drawn =
    asked.mode === "pdf"
      ? await webPage([body], "", { leaf: "a4" })
      : framed(await docPlaced(body));
  if (asked.mode === "pdf" && drawn.includes(lively)) throw new Error("the PDF's page carries lively.css");
  if (asked.mode !== "pdf" && !drawn.includes(lively)) throw new Error("the frame carries no lively.css");
  const page = await loaded(held, (asked.bare ? without(drawn) : drawn).replace("</head>", `${STILL}</head>`));
  page.querySelector(".page")?.removeAttribute("inert");
  if (asked.mode !== "pdf") await dressedAs(page, asked, body);
  if (asked.row) rowed(page);
  await settled(page);
  return asked.measured ? { needs: needs(page) } : laid(page);
};

Object.assign(window, { paper: { lay }, paperReady: true });
