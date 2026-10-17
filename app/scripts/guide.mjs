import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { JSDOM } from "jsdom";
import { createServer } from "vite";

const app = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const root = resolve(app, "..");
const out = resolve(process.argv[2] ?? join(root, "target", "guide"));
const LANGUAGES = [
  {
    code: "en",
    file: "en.md",
    dir: "",
    start: "Start",
    topics: "Guide",
    near: "Next and previous",
    home: "https://www.tisty.app/",
  },
  {
    code: "es",
    file: "es.md",
    dir: "es/",
    start: "Inicio",
    topics: "Guía",
    near: "Anterior y siguiente",
    home: "https://www.tisty.app/es/",
  },
];

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

const escaped = (said) => said.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

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

// A fence that says `markdown` is an example: shown as written and as Tisty draws it.
const pieces = (lines) => {
  const found = [];
  let prose = [];
  let example = null;
  for (const line of lines) {
    if (example) {
      if (line.trim() === example.fence) {
        found.push({ source: example.lines.join("\n") });
        example = null;
      } else example.lines.push(line);
      continue;
    }
    const fence = /^(`{3,})markdown\s*$/.exec(line)?.[1];
    if (fence) {
      found.push({ prose: prose.join("\n") });
      prose = [];
      example = { fence, lines: [] };
    } else prose.push(line);
  }
  found.push({ prose: prose.join("\n") });
  return found.filter((one) => one.source !== undefined || one.prose.trim());
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
      img.setAttribute("src", "photo.svg");
      continue;
    }
    const card = page.createElement("a");
    card.className = "card";
    card.textContent = img.getAttribute("alt") || named(at);
    img.replaceWith(card);
  }
};

// A widget runs in a frame of its own, dressed by the same kit the window gives it.
const lent = (page) => {
  for (const code of page.querySelectorAll("figure.code > pre > code.language-widget")) {
    const figure = code.closest("figure");
    const frame = page.createElement("iframe");
    frame.className = "widget";
    frame.setAttribute("sandbox", "allow-scripts");
    frame.setAttribute("title", figure.querySelector("figcaption")?.textContent ?? "Widget");
    frame.setAttribute(
      "srcdoc",
      `<!doctype html><html><head><meta charset="utf-8"><style>${kit}</style></head><body><main class="w">${code.textContent}</main></body></html>`,
    );
    figure.replaceWith(frame);
  }
};

const dressed = (html) => {
  const page = new JSDOM(`<article class="page">${html}</article>`).window.document;
  saidKinds(page, (kind) => t(`said${kind}`));
  iconed(page);
  coloredAll(page);
  kept(page);
  lent(page);
  for (const one of page.querySelectorAll("[data-at]")) one.removeAttribute("data-at");
  return page;
};

const build = (language) => {
  adopt(language.code);
  const text = readFileSync(
    join(root, "crates", "tisty-markdown", "reference", language.file),
    "utf8",
  );
  const parts = topics(text).map((topic, at) => ({ ...topic, at, pieces: pieces(topic.lines) }));
  const texts = parts.flatMap((part) => part.pieces.map((one) => one.source ?? one.prose));
  const html = drawn(texts);
  let next = 0;
  const pages = parts.map((part) => {
    const body = part.pieces
      .map((one) => {
        const said = html[next++];
        if (one.source === undefined) return said;
        return `<figure class="example"><pre class="written"><code>${escaped(one.source)}</code></pre><div class="drawn">${said}</div></figure>`;
      })
      .join("");
    return { part, page: dressed(body) };
  });

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
      link.setAttribute("href", tops.has(id) ? `${slug}.html` : `${slug}.html#${id}`);
    }
    const title = part.heading ?? page.querySelector("h1")?.textContent ?? "Tisty Markdown";
    const first = page.querySelector("article > p")?.textContent ?? "";
    return {
      at: part.at,
      slug: part.slug,
      title,
      label: part.at === 0 ? language.start : title,
      said: first.length > 160 ? `${first.slice(0, 157).trimEnd()}…` : first,
      html: page.querySelector("article").outerHTML,
    };
  });
};

const SITE = "https://markdown.tisty.app/";

const address = (language, page) =>
  `${SITE}${language.dir}${page.slug === "index" ? "" : page.slug}`;

// Plain HTML with one local stylesheet and relative links, so the same folder serves the web and the app.
const sheetOf = (language, page, built) => {
  const other = LANGUAGES.find((one) => one !== language);
  const pages = built.get(language.code);
  const twin = built.get(other.code)[page.at] ?? built.get(other.code)[0];
  const up = language.dir ? "../" : "";
  const listed = pages
    .map(
      (one) =>
        `<li><a href="${one.slug}.html"${one.slug === page.slug ? ' aria-current="page"' : ""}>${escaped(one.label)}</a></li>`,
    )
    .join("");
  const before = pages[page.at - 1];
  const after = pages[page.at + 1];
  const near = [
    before && `<a rel="prev" href="${before.slug}.html">← ${escaped(before.label)}</a>`,
    after && `<a rel="next" href="${after.slug}.html">${escaped(after.label)} →</a>`,
  ]
    .filter(Boolean)
    .join("");
  const title = page.slug === "index" ? "Tisty Markdown" : `${page.title} · Tisty Markdown`;
  return `<!doctype html>
<html lang="${language.code}">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>${escaped(title)}</title>
<meta name="description" content="${escaped(page.said).replace(/"/g, "&quot;")}">
<link rel="canonical" href="${address(language, page)}">
<link rel="alternate" hreflang="${language.code}" href="${address(language, page)}">
<link rel="alternate" hreflang="${other.code}" href="${address(other, twin)}">
<link rel="stylesheet" href="${up}guide.css">
</head>
<body>
<header class="top"><a class="brand" href="index.html">Tisty Markdown</a><a class="out" href="${language.home}">www.tisty.app ↗</a><a class="lang" lang="${other.code}" hreflang="${other.code}" href="${language.dir ? "../" : other.dir}${twin.slug}.html">${other.code.toUpperCase()}</a></header>
<div class="wrap">
<nav class="topics" aria-label="${language.topics}"><ol>${listed}</ol></nav>
<main>${page.html}<nav class="near" aria-label="${language.near}">${near}</nav></main>
</div>
</body>
</html>
`;
};

const tokens = /:root\s*\{[^}]*\}/.exec(readFileSync(join(app, "src", "index.css"), "utf8"))[0];

rmSync(out, { recursive: true, force: true });
const built = new Map(LANGUAGES.map((language) => [language.code, build(language)]));
for (const language of LANGUAGES) {
  const into = join(out, language.dir);
  mkdirSync(into, { recursive: true });
  for (const page of built.get(language.code))
    writeFileSync(join(into, `${page.slug}.html`), sheetOf(language, page, built));
  writeFileSync(join(into, "photo.svg"), readFileSync(join(app, "scripts", "guide-photo.svg")));
}
writeFileSync(
  join(out, "guide.css"),
  `${tokens}\n${sheet}\n${readFileSync(join(app, "scripts", "guide.css"), "utf8")}`,
);
console.log(`the guide is in ${out}`);
