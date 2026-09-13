// @ts-nocheck
import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

//  process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

const sandboxed = import.meta.dirname.includes(".stryker-tmp");

const readTheCrates = [
  "src/tests/catalogue.test.tsx",
  "src/tests/depth.test.ts",
  "src/tests/refusalcodes.test.ts",
  "src/tests/scrolling.test.tsx",
  "src/tests/synonyms.test.ts",
  "src/tests/windowing.test.tsx",
];

export default defineConfig(async () => ({
  plugins: [react(), tailwindcss()],

  test: {
    environment: "jsdom",
    setupFiles: "./src/tests/setup.ts",
    include: ["src/**/*.test.ts?(x)"],
    exclude: ["**/node_modules/**", "**/dist/**", ...(sandboxed ? readTheCrates : [])],
    env: { TZ: "UTC" },
    testTimeout: 30000,
    coverage: {
      provider: "v8" as const,
      reporter: ["text-summary", "lcov"],
      reportsDirectory: "coverage",
      include: ["src/**/*.{ts,tsx}"],
      exclude: ["src/tests/**", "src/main.tsx", "src/vite-env.d.ts"],
    },
  },

  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
