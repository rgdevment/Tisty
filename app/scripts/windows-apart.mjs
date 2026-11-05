import { readdirSync, readFileSync } from "node:fs";

const NEVER_PRELOADS = {
  Quick: ["App", "Page", "Sheet"],
  Moving: ["App", "Viewing"],
  Viewing: ["App", "Sheet"],
};

const dist = new URL("../dist/", import.meta.url);

const refuse = (why) => {
  console.error(`x  ${why}`);
  process.exit(1);
};

const isChunkOf = (name) => (file) => file.split("/").pop().startsWith(`${name}-`);

const entryChunk = () => {
  const page = readFileSync(new URL("index.html", dist), "utf8");
  const src = page.match(/<script type="module"[^>]*\ssrc="\/(assets\/index-[^"]+\.js)"/)?.[1];
  if (!src) refuse("dist/index.html names no assets/index-*.js");
  return readFileSync(new URL(src, dist), "utf8");
};

const preloadTable = (entry) => {
  const table = entry.match(/__vite__mapDeps=.*?m\.f=(\[.*?\])/s)?.[1];
  if (!table) refuse("the entry chunk has no __vite__mapDeps table");
  return JSON.parse(table);
};

const preloadedWith = (name, entry, table) => {
  const at = entry.search(new RegExp(`import\\(.\\./${name}-`));
  if (at < 0) refuse(`the entry chunk does not import ${name}`);
  const indexes = entry.slice(at).match(/__vite__mapDeps\(\[([\d,]*)\]\)/)?.[1];
  if (!indexes) refuse(`the import of ${name} has no preload list`);
  return indexes.split(",").map((index) => table[Number(index)]);
};

const built = readdirSync(new URL("assets/", dist));
for (const name of new Set(Object.entries(NEVER_PRELOADS).flat(2))) {
  if (!built.some(isChunkOf(name))) refuse(`no chunk is called ${name}, so this check would look at nothing`);
}

const entry = entryChunk();
const table = preloadTable(entry);
const wrong = Object.entries(NEVER_PRELOADS).flatMap(([name, others]) => {
  const list = preloadedWith(name, entry, table);
  const crossed = others.filter((other) => list.some(isChunkOf(other))).map((other) => `${name} preloads ${other}`);
  return list.some(isChunkOf(name)) ? crossed : [...crossed, `${name} does not preload itself`];
});

if (wrong.length > 0) refuse(`the windows are not apart: ${wrong.join(", ")}`);
console.log("ok each window preloads only its own code");
