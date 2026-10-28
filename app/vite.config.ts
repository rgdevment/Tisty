import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// Stryker runs the tests from a copy of the app, so the studio is looked for up the folders.
const studioAt = (from: string): string => {
  const here = join(from, "packages", "tisty-diagrams");
  if (existsSync(here)) return here;
  const up = dirname(from);
  if (up === from) throw new Error("packages/tisty-diagrams is not above the app");
  return studioAt(up);
};
const studio = studioAt(dirname(fileURLToPath(import.meta.url)));

export default defineConfig(async () => ({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: { "tisty-diagrams": join(studio, "src") },
    // The studio has no node_modules of its own: what it imports comes from the app's.
    dedupe: ["react", "react-dom", "@testing-library/react", "@testing-library/user-event", "mermaid", "vitest"],
  },

  test: {
    environment: "jsdom",
    setupFiles: "./src/tests/setup.ts",
    include: ["src/**/*.test.ts?(x)", "../packages/tisty-diagrams/tests/**/*.test.ts?(x)"],
    env: { TZ: "UTC" },
    testTimeout: 30000,
    coverage: {
      provider: "v8" as const,
      reporter: ["text-summary", "lcov"],
      reportsDirectory: "coverage",
      // The studio lives beside the app, outside its root, and is measured all the same.
      allowExternal: true,
      include: ["src/**/*.{ts,tsx}", "../packages/tisty-diagrams/src/**/*.{ts,tsx}"],
      exclude: ["src/tests/**", "src/main.tsx", "src/vite-env.d.ts"],
    },
  },

  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    fs: { allow: [".", studio] },
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
