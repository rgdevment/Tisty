import { existsSync, readdirSync, readFileSync } from "node:fs";

const read = (path) => JSON.parse(readFileSync(path, "utf8"));
const root = read("package.json");
const app = read("app/package.json");
const declared = { ...app.devDependencies, ...app.dependencies };
const wrong = [];

for (const one of readdirSync("packages")) {
  const at = `packages/${one}/package.json`;
  if (!existsSync(at)) continue;
  const { peerDependencies, devDependencies } = read(at);
  for (const [name, range] of Object.entries({ ...peerDependencies, ...devDependencies })) {
    if (range === "*" && !declared[name]) wrong.push(`${one} takes ${name} from the app, which does not declare it`);
  }
}

// An override cannot point at a workspace's range, and two versions would ship two copies under one stylesheet.
if (root.overrides.mermaid.katex !== app.dependencies.katex) {
  wrong.push(`Mermaid's katex override says ${root.overrides.mermaid.katex}, and the app says ${app.dependencies.katex}`);
}

for (const one of wrong) console.log(one);
process.exit(wrong.length > 0 ? 1 : 0);
