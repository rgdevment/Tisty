import { readdirSync, readFileSync } from "node:fs";

const HELD_ONLY_BY = {
  Page: ["App", "Viewing"],
  Sheet: ["App"],
};

const app = new URL("../", import.meta.url);
const dist = new URL("dist/", app);

const refuse = (why) => {
  console.error(`x  ${why}`);
  process.exit(1);
};

const isChunkOf = (name) => (file) => file.split("/").pop().startsWith(`${name}-`);

const builtFile = (file) => readFileSync(new URL(file, dist), "utf8");

const entryFile = () => {
  const src = builtFile("index.html").match(/<script type="module"[^>]*\ssrc="\/(assets\/index-[^"]+\.js)"/)?.[1];
  if (!src) refuse("dist/index.html names no assets/index-*.js");
  return src;
};

const windowsOfMain = () => {
  const main = readFileSync(new URL("src/main.tsx", app), "utf8");
  const names = [...main.matchAll(/import\("\.\/(\w+)"\)/g)].map(([, name]) => name);
  if (names.length === 0) refuse("src/main.tsx imports no window");
  return names;
};

const preloadTable = (code) => {
  const table = code.match(/__vite__mapDeps=.*?m\.f=(\[.*?\])/s)?.[1];
  return table ? JSON.parse(table) : [];
};

const preloadListsOf = (name, entry, table) => {
  const sites = [...entry.matchAll(new RegExp(`import\\(.\\./${name}-`, "g"))];
  if (sites.length === 0) refuse(`the entry chunk does not import ${name}`);
  return sites.map(({ index }) => {
    const indexes = entry.slice(index).match(/__vite__mapDeps\(\[([\d,]*)\]\)/)?.[1];
    if (!indexes) refuse(`an import of ${name} has no preload list`);
    return indexes.split(",").map((one) => table[Number(one)]);
  });
};

const reachedFrom = (list, entry) => {
  const reached = new Set(list);
  for (const file of reached) {
    if (!file.endsWith(".js") || file === entry) continue;
    for (const more of preloadTable(builtFile(file))) reached.add(more);
  }
  return [...reached];
};

const windows = windowsOfMain();
const forbiddenTo = (name) => [
  ...windows.filter((other) => other !== name),
  ...Object.keys(HELD_ONLY_BY).filter((chunk) => !HELD_ONLY_BY[chunk].includes(name)),
];

const built = readdirSync(new URL("assets/", dist));
for (const name of [...windows, ...Object.keys(HELD_ONLY_BY)]) {
  if (!built.some(isChunkOf(name))) refuse(`no chunk is called ${name}, so this check would look at nothing`);
}

const entry = entryFile();
const entryCode = builtFile(entry);
const table = preloadTable(entryCode);
if (table.length === 0) refuse("the entry chunk has no __vite__mapDeps table");

const wrong = windows.flatMap((name) =>
  preloadListsOf(name, entryCode, table).flatMap((list) => {
    const reached = reachedFrom(list, entry);
    const crossed = forbiddenTo(name)
      .filter((other) => reached.some(isChunkOf(other)))
      .map((other) => `${name} reaches ${other}`);
    return list.some(isChunkOf(name)) ? crossed : [...crossed, `${name} does not preload itself`];
  }),
);

if (wrong.length > 0) refuse(`the windows are not apart: ${[...new Set(wrong)].join(", ")}`);
console.log(`ok each of ${windows.join(", ")} reaches only its own code`);
