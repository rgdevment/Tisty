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
  { code: "en", file: "en.md", start: "Start" },
  { code: "es", file: "es.md", start: "Inicio" },
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

const dressed = (html) => {
  const page = new JSDOM(`<article class="page">${html}</article>`).window.document;
  saidKinds(page, (kind) => t(`said${kind}`));
  iconed(page);
  coloredAll(page);
  for (const one of page.querySelectorAll("[data-at]")) one.removeAttribute("data-at");
  return page;
};

const build = ({ code, file, start }) => {
  adopt(code);
  const text = readFileSync(join(root, "crates", "tisty-markdown", "reference", file), "utf8");
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
    const slug = part.at === 0 ? "index" : plain(part.heading) || `topic-${part.at}`;
    part.slug = slug;
    for (const head of page.querySelectorAll("[id]")) slugs.set(head.id, slug);
    const top = page.querySelector("h2[id]");
    if (top) tops.add(top.id);
  }

  const into = join(out, code);
  mkdirSync(into, { recursive: true });
  const listed = [];
  for (const { part, page } of pages) {
    for (const link of page.querySelectorAll('a[href^="#"]')) {
      const id = decodeURIComponent(link.getAttribute("href").slice(1));
      const slug = slugs.get(id);
      if (!slug || slug === part.slug) continue;
      link.setAttribute("href", tops.has(id) ? `${slug}.html` : `${slug}.html#${id}`);
    }
    const title = part.heading ?? page.querySelector("h1")?.textContent ?? "Tisty Markdown";
    writeFileSync(join(into, `${part.slug}.html`), page.querySelector("article").outerHTML);
    listed.push({ slug: part.slug, title, label: part.at === 0 ? start : title });
  }
  writeFileSync(join(into, "topics.json"), `${JSON.stringify(listed, null, 2)}\n`);
};

rmSync(out, { recursive: true, force: true });
mkdirSync(out, { recursive: true });
for (const language of LANGUAGES) build(language);
writeFileSync(join(out, "page.css"), sheet);
writeFileSync(join(out, "guide.css"), readFileSync(join(app, "scripts", "guide.css"), "utf8"));
console.log(`the guide is in ${out}`);
