import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vitest/config";

const packages = dirname(dirname(fileURLToPath(import.meta.url)));
const reader = join(packages, "tisty-reader");
const studio = join(packages, "tisty-diagrams");

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
