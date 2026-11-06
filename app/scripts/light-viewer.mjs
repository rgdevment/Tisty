import { readdirSync, readFileSync, statSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const JS_AT_MOST = 80 * 1024;
const CSS_AT_MOST = 50 * 1024;
const REACT = "react.transitional.element";
const SOLID = "_$SOLID_EVENT_OWNER";

const dist = join(dirname(fileURLToPath(import.meta.url)), "..", "dist");
const viewer = join(dist, "viewer");
const manifest = JSON.parse(readFileSync(join(viewer, ".vite", "manifest.json"), "utf8"));
const wrong = [];

const size = (file) => statSync(join(viewer, file)).size;
const entry = Object.values(manifest).find((one) => one.isEntry);
const reached = new Set();
const walk = (one) => {
  if (reached.has(one.file)) return;
  reached.add(one.file);
  for (const key of one.imports ?? []) walk(manifest[key]);
};
walk(entry);

const js = [...reached].reduce((sum, file) => sum + size(file), 0);
const css = (entry.css ?? []).reduce((sum, file) => sum + size(file), 0);
if (js > JS_AT_MOST) wrong.push(`${js} bytes of JS before the first words; ${JS_AT_MOST} is the budget`);
if (css > CSS_AT_MOST) wrong.push(`${css} bytes of CSS before the first words; ${CSS_AT_MOST} is the budget`);

const scripts = (folder) =>
  readdirSync(join(folder, "assets"))
    .filter((file) => file.endsWith(".js"))
    .map((file) => join(folder, "assets", file));

const carrying = (folder, mark) =>
  scripts(folder).filter((file) => readFileSync(file, "utf8").includes(mark));

for (const file of carrying(viewer, REACT)) wrong.push(`${file} carries React`);
for (const file of carrying(dist, SOLID)) wrong.push(`${file} carries Solid`);
if (carrying(viewer, SOLID).length === 0) wrong.push("no script of the viewer carries Solid: this check no longer knows what to look for");
if (carrying(dist, REACT).length === 0) wrong.push("no script of the app carries React: this check no longer knows what to look for");

for (const one of wrong) console.log(one);
if (wrong.length === 0) console.log(`${js} bytes of JS and ${css} of CSS before the first words, each framework in its own build`);
process.exit(wrong.length > 0 ? 1 : 0);
