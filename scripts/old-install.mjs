import { existsSync, lstatSync, readdirSync, rmSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const packages = join(root, "packages");
const appModules = join(root, "app", "node_modules");

for (const one of readdirSync(packages)) {
  const linkToTheApp = join(packages, one, "node_modules");
  if (lstatSync(linkToTheApp, { throwIfNoEntry: false })?.isSymbolicLink()) rmSync(linkToTheApp, { force: true });
}
// A workspace keeps this file at the root, so here it can only mean an install from before the workspace.
if (existsSync(join(appModules, ".package-lock.json"))) rmSync(appModules, { recursive: true, force: true });
