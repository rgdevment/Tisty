import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// Stryker runs the tests from a copy of the app, so the packages are looked for up the folders.
const packageAt = (name: string, from: string): string => {
  const here = join(from, "packages", name);
  if (existsSync(here)) return here;
  const up = dirname(from);
  if (up === from) throw new Error(`packages/${name} is not above the app`);
  return packageAt(name, up);
};
const appAt = dirname(fileURLToPath(import.meta.url));
const studio = packageAt("tisty-diagrams", appAt);
const reader = packageAt("tisty-editor", appAt);
const web = packageAt("tisty-share-web", appAt);

export default defineConfig(async () => ({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: { "tisty-diagrams": join(studio, "src"), "tisty-editor": join(reader, "src") },
  },

  test: {
    environment: "jsdom",
    setupFiles: "./src/tests/setup.ts",
    include: ["src/**/*.test.ts?(x)", "../packages/*/tests/**/*.test.ts?(x)"],
    env: { TZ: "UTC" },
    testTimeout: 30000,
    coverage: {
      provider: "v8" as const,
      reporter: ["text-summary", "lcov"],
      reportsDirectory: "coverage",
      // The packages live beside the app, outside its root, and are measured all the same.
      allowExternal: true,
      include: ["src/**/*.{ts,tsx}", "../packages/*/src/**/*.{ts,tsx}"],
      exclude: [
        "src/tests/**",
        "src/main.tsx",
        "src/vite-env.d.ts",
        "../packages/tisty-share-web/src/main.ts",
        "../packages/tisty-share-web/src/vite-env.d.ts",
      ],
    },
  },

  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    fs: { allow: [".", studio, reader, web] },
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
