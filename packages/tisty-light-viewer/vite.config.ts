import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import solid from "vite-plugin-solid";
import { defineConfig } from "vitest/config";

const here = dirname(fileURLToPath(import.meta.url));

export default defineConfig(({ command }) => ({
  root: here,
  base: command === "serve" ? "/viewer/" : "./",
  plugins: [solid()],
  build: {
    outDir: join(here, "../../app/dist/viewer"),
    emptyOutDir: true,
    manifest: true,
    target: "es2022",
  },
  test: {
    environment: "jsdom",
    css: { include: [/tisty-sheet[\\/]src[\\/]page\.css/] },
    include: ["tests/**/*.test.ts?(x)"],
    env: { TZ: "UTC" },
    server: { deps: { inline: [/solid-js/, /@solidjs/] } },
    coverage: {
      provider: "v8" as const,
      reporter: ["text-summary", "lcov"],
      reportsDirectory: "coverage",
      include: ["src/**/*.{ts,tsx}"],
      exclude: ["src/main.tsx", "src/host.ts"],
    },
  },
}));
