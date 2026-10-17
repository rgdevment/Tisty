import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { JSDOM } from "jsdom";
import { createServer } from "vite";
import { createHash } from "node:crypto";
import {
  escaped,
  headers,
  href,
  LANGUAGES,
  mapped,
  missing,
  robots,
  sheetOf,
} from "./guide-site.mjs";

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
const { iconed } = await vite.ssrLoadModule("/src/ui/dressing.ts");
const { coloredAll, saidKinds } = await vite.ssrLoadModule("/src/ui/paging.ts");
const { adopt, t } = await vite.ssrLoadModule("/src/locales.ts");
const { named, pictured } = await vite.ssrLoadModule("/src/previews.ts");
const sheet = (await vite.ssrLoadModule("/src/ui/page.css?raw")).default;
await vite.close();

const drawn = (texts) =>
  JSON.parse(
    execFileSync("cargo", ["run", "-q", "-p", "tisty-markdown", "--example", "drawn"], {
      cwd: root,
      input: JSON.stringify(texts),
      maxBuffer: 256 * 1024 * 1024,
    }).toString(),
  );

const opening = (line) => /^(`{3,}|~{3,})/.exec(line)?.[1];

// Each `##` heading starts a topic; the start page is what comes before them and the first one, the glance.
const topics = (text) => {
  const found = [{ heading: null, lines: [] }];
  let fence = null;
  let glanced = false;
  for (const line of text.replace(/\r\n/g, "\n").split("\n")) {
    if (fence) {
      if (line.trim() === fence) fence = null;
    } else if (opening(line)) {
      fence = opening(line);
    } else if (line.startsWith("## ") && glanced) {
      found.push({ heading: line.slice(3).trim(), lines: [] });
    } else if (line.startsWith("## ")) {
      glanced = true;
    }
    found[found.length - 1].lines.push(line);
  }
  return found;
};

const plain = (said) =>
  said
    .normalize("NFD")
    .replace(/\p{M}/gu, "")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");

const kit = readFileSync(join(app, "src-tauri", "src", "widget.css"), "utf8");

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

const dressed = (page) => {
  saidKinds(page, (kind) => t(`said${kind}`));
  iconed(page);
  coloredAll(page);
  kept(page);
  lent(page);
  for (const one of page.querySelectorAll("[data-at]")) one.removeAttribute("data-at");
  return page;
};

// What a reader sees of a page, without its heading or the examples as written.
const shown = (page) => {
  const copy = page.querySelector("article").cloneNode(true);
  for (const one of copy.querySelectorAll("h2, .written, figcaption")) one.remove();
  const blocks = Array.from(copy.children).flatMap((one) =>
    one.matches("figure.example") ? Array.from(one.querySelector(".drawn").children) : [one],
  );
  return blocks.map((one) => one.textContent).join(" ");
};

const build = (language) => {
  adopt(language.code);
  const text = readFileSync(
    join(root, "crates", "tisty-markdown", "reference", language.file),
    "utf8",
  );
  const parts = topics(text).map((topic, at) => ({ ...topic, at }));
  const sheets = drawn(parts.map((part) => part.lines.join("\n"))).map(
    (html) => new JSDOM(`<article class="page">${html}</article>`).window.document,
  );
  // An example is any `markdown` fence, wherever it sits, shown as written and as Tisty draws it.
  const examples = sheets.flatMap((page) =>
    Array.from(page.querySelectorAll("figure.code > pre > code.language-markdown")),
  );
  const looks = drawn(examples.map((code) => code.textContent));
  examples.forEach((code, at) => {
    const page = code.ownerDocument;
    const figure = page.createElement("figure");
    figure.className = "example";
    figure.innerHTML = `<pre class="written" data-said="${language.written}"><code>${escaped(code.textContent.replace(/\n$/, ""))}</code></pre><div class="drawn" data-said="${language.drawn}">${looks[at]}</div>`;
    code.closest("figure").replaceWith(figure);
  });
  const pages = parts.map((part, at) => ({ part, page: dressed(sheets[at]) }));

  const slugs = new Map();
  const tops = new Set();
  for (const { part, page } of pages) {
    part.slug = part.at === 0 ? "index" : plain(part.heading) || `topic-${part.at}`;
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
    const first = (
      page.querySelector("article > p")?.textContent ||
      shown(page) ||
      ""
    )
      .replace(/\s+/g, " ")
      .trim();
    return {
      at: part.at,
      slug: part.slug,
      title,
      label: part.at === 0 ? language.start : title,
      subs: Array.from(page.querySelectorAll("article > h3[id]")).map((head) => ({
        id: head.id,
        title: head.textContent,
      })),
      said: first.length > 160 ? `${first.slice(0, 157).trimEnd()}…` : first,
      html: page.querySelector("article").outerHTML,
    };
  });
};

const reference = (language) =>
  readFileSync(join(root, "crates", "tisty-markdown", "reference", language.file), "utf8");

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

const index = readFileSync(join(app, "src", "index.css"), "utf8");
const light = /:root\s*\{[^}]*\}/.exec(index)[0];
const dark = /\[data-theme="dark"\]\s*\{([^}]*)\}/.exec(index)[1];

rmSync(out, { recursive: true, force: true });
const built = new Map(LANGUAGES.map((language) => [language.code, build(language)]));
const logo = readFileSync(join(app, "src", "assets", "tisty.svg"));
const appLight = light.replace(/^:roots*{|}$/g, "");
const css = `${light}
html.dark{${dark}}
${sheet}
${readFileSync(join(app, "scripts", "guide.css"), "utf8")}
figure.example,figure.widget{${appLight}}
html.dark figure.example,html.dark figure.widget{${dark}}
`;
const style = `guide.${createHash("sha256").update(css).digest("hex").slice(0, 10)}.css`;
for (const language of LANGUAGES) {
  const into = join(out, language.dir);
  mkdirSync(into, { recursive: true });
  for (const page of built.get(language.code))
    writeFileSync(join(into, `${page.slug}.html`), sheetOf(built, language, page, style));
  writeFileSync(join(into, "rina.jpg"), readFileSync(join(app, "src-tauri", "resources", "guide", language.code, "rina.jpg")));
  writeFileSync(join(into, "tisty.svg"), logo);
  writeFileSync(join(into, "llms-full.txt"), reference(language));
}
writeFileSync(join(out, "404.html"), missing(style));
writeFileSync(join(out, "_headers"), headers(style));
writeFileSync(join(out, "sitemap.xml"), mapped(built));
writeFileSync(join(out, "robots.txt"), robots);
writeFileSync(join(out, "llms.txt"), llms(built));
writeFileSync(join(out, style), css);
console.log(`the guide is in ${out}`);
