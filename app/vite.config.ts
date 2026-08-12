import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [react(), tailwindcss()],

  test: {
    environment: "jsdom",
    setupFiles: "./src/tests/setup.ts",
    include: ["src/**/*.test.ts?(x)"],
    // The archive groups by the reader's own calendar, on purpose: a task
    // finished at 00:30Z was finished in August for anyone west of UTC. Fixing
    // the zone here is what makes the tests say the same thing everywhere —
    // the setup file already does it for the locale.
    env: { TZ: "UTC" },
  },

  // Or Vite's own output buries the rust errors underneath it.
  clearScreen: false,
  server: {
    // Tauri looks for this exact port and cannot discover another.
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
