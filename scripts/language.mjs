import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const WIKI = "https://raw.githubusercontent.com/wiki/rgdevment/Tisty";
const PAGES = [
  ["Tisty-Markdown.md", "en.md"],
  ["Lenguaje-Tisty.md", "es.md"],
];

// Inside the app there is no other wiki page to go to, so a link to one keeps only its words.
export const unlinked = (text) =>
  text.replace(/\[([^\]]+)\]\(([^)\s#:]+)\)/g, (whole, words, to) =>
    /\.[a-z0-9]{2,5}$/i.test(to) ? whole : words,
  );

const checking = process.argv.includes("--check");
let stale = 0;

for (const [page, kept] of PAGES) {
  const answer = await fetch(`${WIKI}/${encodeURIComponent(page)}`);
  if (!answer.ok) throw new Error(`${page}: the wiki answered ${answer.status}`);
  const text = unlinked((await answer.text()).replace(/\r\n/g, "\n"));
  const at = join(root, "crates", "tisty-markdown", "reference", kept);
  if (readFileSync(at, "utf8") === text) continue;
  stale += 1;
  if (checking) console.error(`${kept} differs from the wiki's ${page}`);
  else writeFileSync(at, text);
}

if (checking && stale) process.exit(1);
console.log(checking ? "the reference matches the wiki" : `${stale} page(s) brought from the wiki`);
