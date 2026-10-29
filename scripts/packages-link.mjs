// The packages have no install of their own: each finds the app's dependencies through a link
// to app/node_modules, made after every `npm install` or `npm ci` in app/.
import { existsSync, lstatSync, readdirSync, rmSync, symlinkSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const modules = join(root, "app", "node_modules");
const packages = join(root, "packages");

if (!existsSync(modules)) {
  console.error("app/node_modules is not there yet, so the packages have nothing to link to");
  process.exit(1);
}
for (const one of readdirSync(packages, { withFileTypes: true })) {
  if (!one.isDirectory()) continue;
  const link = join(packages, one.name, "node_modules");
  if (existsSync(link) || lstatSync(link, { throwIfNoEntry: false })) rmSync(link, { recursive: false, force: true });
  // A junction needs no rights on Windows; elsewhere a relative link survives moving the checkout.
  if (process.platform === "win32") symlinkSync(modules, link, "junction");
  else symlinkSync(relative(dirname(link), modules), link, "dir");
}
