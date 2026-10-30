import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vitest/config";

// Stryker runs the tests from a copy of the page, so the packages are looked for up the folders.
const packageAt = (name: string, from: string): string => {
  const here = join(from, "packages", name);
  if (existsSync(here)) return here;
  const up = dirname(from);
  if (up === from) throw new Error(`packages/${name} is not above the page`);
  return packageAt(name, up);
};
const pageAt = dirname(fileURLToPath(import.meta.url));
const reader = packageAt("tisty-reader", pageAt);
const studio = packageAt("tisty-diagrams", pageAt);

export default defineConfig({
  resolve: {
    alias: { "tisty-diagrams": join(studio, "src"), "tisty-reader": join(reader, "src") },
  },
  build: {
    // The wasm loads as a file of its own, so the page's CSP never has to let a script be built from a string.
    assetsInlineLimit: 0,
    target: "es2022",
  },
  server: { fs: { allow: [".", reader, studio] } },
  test: {
    environment: "jsdom",
    include: ["tests/**/*.test.ts"],
    env: { TZ: "UTC" },
  },
});
