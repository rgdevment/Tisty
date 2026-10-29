import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vitest/config";

// Stryker runs the tests from a copy of the reader, so the packages are looked for up the folders.
const packageAt = (name: string, from: string): string => {
  const here = join(from, "packages", name);
  if (existsSync(here)) return here;
  const up = dirname(from);
  if (up === from) throw new Error(`packages/${name} is not above the reader`);
  return packageAt(name, up);
};
const studio = packageAt("tisty-diagrams", dirname(fileURLToPath(import.meta.url)));

// The reader's own run, for Stryker: its logic is tested here; the app runs the same tests with its own.
export default defineConfig({
  resolve: { alias: { "tisty-diagrams": join(studio, "src") } },
  test: {
    environment: "jsdom",
    include: ["tests/**/*.test.ts?(x)"],
    env: { TZ: "UTC" },
    testTimeout: 30000,
  },
});
