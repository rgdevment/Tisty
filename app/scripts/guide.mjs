import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { JSDOM } from "jsdom";
import { build as bundled, createServer } from "vite";
import { createHash } from "node:crypto";
import {
  changedOn,
  commitsIn,
  describedAs,
  escaped,
  headers,
  href,
  LANGUAGES,
  mapped,
  missing,
  muted,
  raised,
  robots,
  sheetOf,
} from "./guide-site.mjs";
import { drawnInChrome } from "./chrome.mjs";

const app = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const root = resolve(app, "..");
const out = resolve(process.argv[2] ?? join(root, "target", "guide"));

// The window's own code dresses the page, so it is loaded the way the window loads it.
const window = new JSDOM("").window;
globalThis.window = window;
globalThis.document = window.document;
const vite = await createServer({
  root: app,
  appType: "custom",
  logLevel: "error",
  server: { middlewareMode: true, hmr: false },
});
const { carded, coloredAll, hosting, HUES, iconed, PAGE_CSS: sheet, piece, saidKinds } =
  await vite.ssrLoadModule("tisty-editor/read");
const { adopt, t } = await vite.ssrLoadModule("/src/locales.ts");
const { named, pictured } = await vite.ssrLoadModule("/src/previews.ts");
const { SAY } = await vite.ssrLoadModule("/src/ui/saying.ts");
const { examplesIn, shownBeside, topicsOf } = await vite.ssrLoadModule("/src/guide/reference.ts");

const drawn = (texts) =>
  JSON.parse(
    execFileSync("cargo", ["run", "-q", "-p", "tisty-markdown", "--example", "drawn"], {
      cwd: root,
      input: JSON.stringify(texts),
      maxBuffer: 256 * 1024 * 1024,
    }).toString(),
  );

const kit = readFileSync(join(app, "..", "contracts", "documents", "contains", "widget.css"), "utf8");

// The examples name files only a store holds, so a picture shows the guide's own and a file its card.
const kept = (page) => {
  for (const img of page.querySelectorAll("img[src]")) {
    const at = img.getAttribute("src");
    if (/^(https?|data):/i.test(at)) continue;
    if (pictured(at)) {
      img.setAttribute("src", "rina.jpg");
      continue;
    }
    const card = page.createElement("a");
    card.className = "card";
    card.textContent = img.getAttribute("alt") || named(at);
    img.replaceWith(card);
  }
};

// A widget runs in a frame of its own, dressed by the same kit the window gives it, and says how tall it is.
const TELLS = `<script>(()=>{const say=()=>parent.postMessage({tisty:"height",h:document.documentElement.scrollHeight},"*");new ResizeObserver(say).observe(document.documentElement);addEventListener("load",say);addEventListener("message",(e)=>{if(e.data&&e.data.tisty==="theme")document.documentElement.classList.toggle("dark",e.data.dark)})})()</script>`;

const lent = (page) => {
  for (const code of page.querySelectorAll("figure.code > pre > code.language-widget")) {
    const figure = code.closest("figure");
    const name = figure.querySelector("figcaption")?.textContent ?? "";
    const holder = page.createElement("figure");
    holder.className = "widget";
    const bar = page.createElement("div");
    bar.className = "widget-bar";
    const said = page.createElement("span");
    said.className = "widget-name";
    said.textContent = name;
    bar.append(said);
    const frame = page.createElement("iframe");
    frame.className = "widget";
    frame.setAttribute("sandbox", "allow-scripts");
    frame.setAttribute("title", name || "Widget");
    frame.setAttribute(
      "srcdoc",
      `<!doctype html><html><head><meta charset="utf-8"><style>${kit}</style></head><body><main class="w">${code.textContent}</main>${TELLS}</body></html>`,
    );
    holder.append(bar, frame);
    figure.replaceWith(holder);
  }
};

// Cards are drawn by the window's own code where it can be borrowed; a link card shows its site's letter, as one the app has not looked up yet.
const cards = (page) => {
  for (const drawn of page.querySelectorAll("figure.example > .drawn")) {
    drawn.classList.add("page");
    const titles = new Map(
      Array.from(drawn.querySelectorAll("a.card[data-doc]")).map((one) => [one.dataset.doc, one.textContent]),
    );
    carded(page, { card: (file) => ({ kind: "doc", title: titles.get(file) ?? "" }), say: SAY }, () => true);
    for (const link of drawn.querySelectorAll(":scope > p > a[href]")) {
      const held = link.parentElement;
      const at = link.getAttribute("href");
      const host = hosting(at);
      if (!host || held.children.length !== 1 || held.textContent.trim() !== link.textContent.trim()) continue;
      const box = page.createElement("a");
      box.className = "doc-card is-web is-bare";
      box.href = at;
      const tile = piece(page, "card-tile is-letter", host.charAt(0).toUpperCase());
      let sum = 0;
      for (const one of host) sum += one.charCodeAt(0);
      tile.style.setProperty("--hue", `var(--tisty-hue-${HUES[sum % HUES.length]})`);
      const meta = piece(page, "card-meta");
      meta.append(piece(page, "kind", host), piece(page, "card-go", "›"));
      const name = link.textContent.replace(/^https?:\/\//, "");
      box.append(tile, piece(page, "card-name", name), piece(page, "card-blurb"), meta);
      link.replaceWith(box);
    }
    drawn.classList.remove("page");
  }
};

const dressed = async (page) => {
  saidKinds(page, (kind) => t(`said${kind}`));
  await iconed(page);
  await coloredAll(page);
  kept(page);
  lent(page);
  cards(page);
  muted(page);
  for (const one of page.querySelectorAll("[data-at]")) one.removeAttribute("data-at");
  return page;
};

const REFERENCE = "crates/tisty-markdown/reference";

const git = (...args) => execFileSync("git", args, { cwd: root, maxBuffer: 16 * 1024 * 1024, stdio: "pipe" }).toString();

// A shallow clone or an archive cannot say when a topic changed, and a wrong date is worse than none.
const wholeHistory = () => {
  try {
    return git("rev-parse", "--is-shallow-repository").trim() === "false";
  } catch {
    return false;
  }
};
const dated = wholeHistory();
if (!dated) {
  // What goes live must not lose its dates to a checkout nobody meant to be shallow.
  if (process.env.GUIDE_DATES === "required") throw new Error("this checkout has no whole git history, and this guide must go out with its dates");
  console.warn("no whole git history here, so the guide's pages go out without dates");
}

const reference = (language) => readFileSync(join(root, REFERENCE, language.file), "utf8");

// Newest first, starting from what is on disk, which may say more than the last commit.
const revisionsOf = (file, text) => {
  const path = `${REFERENCE}/${file}`;
  return [
    { date: new Date().toISOString().slice(0, 10), topics: topicsOf(text) },
    ...commitsIn(git("log", "--first-parent", "--format=%H %cs", "--", path)).map(({ hash, date }) => ({
      date,
      topics: topicsOf(git("show", `${hash}:${path}`)),
    })),
  ];
};

const build = async (language) => {
  adopt(language.code);
  const text = reference(language);
  const parts = topicsOf(text);
  for (const part of parts) describedAs(language, part.slug);
  const changed = dated ? changedOn(revisionsOf(language.file, text)) : new Map();
  const sheets = drawn(parts.map((part) => part.body)).map(
    (html) => new JSDOM(`<article class="page">${html}</article>`).window.document,
  );
  const examples = sheets.flatMap((page) => examplesIn(page));
  const looks = drawn(examples.map((code) => code.textContent));
  examples.forEach((code, at) => shownBeside(code, looks[at], language));
  const pages = await Promise.all(
    parts.map(async (part, at) => ({ part, page: await dressed(sheets[at]) })),
  );

  const slugs = new Map();
  const tops = new Set();
  for (const { part, page } of pages) {
    for (const head of page.querySelectorAll("[id]")) slugs.set(head.id, part.slug);
    const top = page.querySelector("h2[id]");
    if (top) tops.add(top.id);
  }

  return pages.map(({ part, page }) => {
    for (const link of page.querySelectorAll('a[href^="#"]')) {
      const id = decodeURIComponent(link.getAttribute("href").slice(1));
      const slug = slugs.get(id);
      if (!slug || slug === part.slug) continue;
      link.setAttribute("href", tops.has(id) ? href(slug) : `${href(slug)}#${id}`);
    }
    const title = part.heading ?? page.querySelector("h1")?.textContent ?? "Tisty Markdown";
    const subs = Array.from(page.querySelectorAll("article > h3[id]")).map((head) => ({
      id: head.id,
      title: head.textContent,
    }));
    if (part.at > 0) raised(page);
    return {
      at: part.at,
      slug: part.slug,
      title,
      label: part.at === 0 ? language.start : title,
      subs,
      said: language.said[part.slug],
      changed: changed.get(part.slug),
      html: page.querySelector("article").outerHTML,
    };
  });
};

// What an assistant reads best is the language as written, so it gets the reference itself.
const llms = (built) => {
  const lines = [
    "# Tisty Markdown",
    "",
    "> The language of Tisty documents: CommonMark and GitHub's additions, plus a few words of Tisty's own for what Markdown cannot say.",
    "",
  ];
  for (const language of LANGUAGES) {
    lines.push(`## ${language.name}`, "");
    for (const page of built.get(language.code)) {
      const at = `https://markdown.tisty.app/${language.dir}${page.slug === "index" ? "" : page.slug}`;
      lines.push(`- [${page.title}](${at}): ${page.said}`);
    }
    lines.push(
      "",
      `- [The whole language in one file](https://markdown.tisty.app/${language.dir}llms-full.txt)`,
      "",
    );
  }
  return lines.join("\n");
};

const DRAWER = ".drawer";

const waitingIn = (document) => [...document.querySelectorAll("figure.diagram > pre > code")];

const sheetFor = (document, look, svg) => {
  const sheet = document.createElement("div");
  sheet.className = "diagram-sheet";
  sheet.dataset.look = look;
  // The example's source, written beside it, is what a search should find.
  sheet.setAttribute("data-pagefind-ignore", "");
  sheet.innerHTML = svg;
  // A Gantt chart marks the day it was drawn, which on a page kept for months would pass for today.
  for (const today of sheet.querySelectorAll("g.today")) today.remove();
  return sheet;
};

// Each diagram goes into its page finished, in each look, so a reader downloads no drawing code.
const drawDiagrams = async (pages, files, style) => {
  const held = pages
    .map((one) => ({ one, document: new JSDOM(one.html).window.document }))
    .filter(({ document }) => document.querySelector("figure.diagram"));
  if (!held.length) return;
  await bundled({
    root: app,
    configFile: false,
    logLevel: "error",
    build: {
      outDir: join(files, DRAWER),
      emptyOutDir: true,
      lib: { entry: join(app, "src", "guide", "diagramming.ts"), formats: ["es"], fileName: () => "drawer.js" },
    },
  });
  const sources = [...new Set(held.flatMap(({ document }) => waitingIn(document).map((code) => code.textContent)))];
  const answer = await drawnInChrome(
    sources,
    files,
    `<!doctype html><html><head><meta charset="utf-8"><style>${style}</style></head><body><script type="module">import("/${DRAWER}/drawer.js").then((one)=>one.drawAll()).catch((error)=>fetch("/failed",{method:"POST",body:String(error?.stack??error)}))</script></body></html>`,
    "the guide's diagrams",
  );
  const drawings = new Map(answer.map((one) => [one.source, one]));
  rmSync(join(files, DRAWER), { recursive: true });
  const unreadable = sources.filter((one) => !drawings.get(one)?.light || !drawings.get(one)?.dark);
  if (unreadable.length) throw new Error(`diagrams that draw nothing:\n${unreadable.join("\n---\n")}`);
  for (const { one, document } of held) {
    for (const code of waitingIn(document)) {
      const figure = code.closest("figure");
      const { light, dark } = drawings.get(code.textContent);
      figure.replaceChildren(sheetFor(document, "light", light), sheetFor(document, "dark", dark));
      figure.classList.add("drawn");
    }
    // Nothing draws them later, so a figure this missed would reach the reader as its source.
    if (document.querySelector("figure.diagram:not(.drawn)")) throw new Error(`a diagram on ${one.slug} was not drawn`);
    one.html = document.querySelector("article").outerHTML;
  }
};

const index = readFileSync(join(app, "src", "index.css"), "utf8");
const light = /:root\s*\{[^}]*\}/.exec(index)[0];
const dark = /\[data-theme="dark"\]\s*\{([^}]*)\}/.exec(index)[1];

rmSync(out, { recursive: true, force: true });
const printOf = (bytes) => createHash("sha256").update(bytes).digest("hex").slice(0, 10);
const fromApp = createRequire(join(app, "package.json"));
const faces = ["inter", "space-grotesk", "jetbrains-mono"].flatMap((name) => {
  const from = dirname(fromApp.resolve(`@fontsource-variable/${name}/wght.css`));
  const sheet = readFileSync(join(from, "wght.css"), "utf8");
  return [...sheet.matchAll(/\/\* [\w-]+-latin(?:-ext)?-wght-normal \*\/\s*(@font-face\s*\{[^}]*\})/g)].map(([, face]) => {
    const file = /url\(\.\/files\/([^)]+)\)/.exec(face)[1];
    const bytes = readFileSync(join(from, "files", file));
    const named = file.replace(".woff2", `.${printOf(bytes)}.woff2`);
    mkdirSync(join(out, "fonts"), { recursive: true });
    writeFileSync(join(out, "fonts", named), bytes);
    return face.replace(`./files/${file}`, `/fonts/${named}`);
  });
});
// One language at a time, since the window's words follow the language adopted last.
const built = new Map();
for (const language of LANGUAGES) built.set(language.code, await build(language));
// The sheet brings its icons and code colours on demand, so the server stays up until the pages are drawn.
await vite.close();
const logo = readFileSync(join(app, "src", "assets", "tisty.svg"));
const appLight = /\{([^}]*)\}/.exec(light)[1];
const css = `${faces.join("\n")}
${light}
html.dark{${dark}}
${sheet}
${readFileSync(join(app, "scripts", "guide.css"), "utf8")}
figure.example,figure.widget{${appLight}}
html.dark figure.example,html.dark figure.widget{${dark}}
`;
const mark = readFileSync(join(app, "..", "assets", "icons", "tisty-light.svg"));
const logoName = `tisty-light.${printOf(mark)}.svg`;
const font = faces.join("\n").match(/\/fonts\/inter-latin-wght-normal\.[^)]+/)[0];
const look = { style: css, logo: logoName, font };
await drawDiagrams([...built.values()].flat(), out, css);
for (const language of LANGUAGES) {
  const into = join(out, language.dir);
  mkdirSync(into, { recursive: true });
  for (const page of built.get(language.code))
    writeFileSync(join(into, `${page.slug}.html`), sheetOf(built, language, page, look));
  writeFileSync(join(into, "rina.jpg"), readFileSync(join(app, "src-tauri", "resources", "guide", language.code, "rina.jpg")));
  writeFileSync(join(into, "tisty.svg"), logo);
  writeFileSync(join(into, "llms-full.txt"), reference(language));
}
writeFileSync(join(out, "404.html"), missing(look));
writeFileSync(join(out, "_headers"), headers(logoName, built));
writeFileSync(join(out, "sitemap.xml"), mapped(built));
writeFileSync(join(out, "robots.txt"), robots);
writeFileSync(join(out, "llms.txt"), llms(built));
writeFileSync(join(out, logoName), mark);
console.log(`the guide is in ${out}`);
