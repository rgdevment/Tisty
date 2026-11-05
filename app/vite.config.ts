import { dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

const repository = dirname(dirname(fileURLToPath(import.meta.url)));

export default defineConfig(async () => ({
  plugins: [react(), tailwindcss()],

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
      exclude: ["src/tests/**", "src/main.tsx", "src/vite-env.d.ts"],
    },
  },

  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    fs: { allow: [repository] },
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
