import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { createServer, type Plugin } from "vite";
import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

const host = process.env.TAURI_DEV_HOST;

const app = dirname(fileURLToPath(import.meta.url));
const repository = dirname(app);

// The viewer builds apart so no React meets its Solid; in development its own server answers under the same address.
const lightViewer: Plugin = {
  name: "tisty-light-viewer",
  apply: "serve",
  async configureServer(server) {
    if (process.env.VITEST || !server.httpServer) return;
    const own = await createServer({
      configFile: join(repository, "packages/tisty-light-viewer/vite.config.ts"),
      server: { middlewareMode: true, hmr: false },
      appType: "mpa",
    });
    server.middlewares.use((request, response, next) =>
      request.url?.startsWith("/viewer/") ? own.middlewares(request, response, next) : next(),
    );
    server.httpServer.once("close", () => void own.close());
  },
};

export default defineConfig(async () => ({
  plugins: [react(), tailwindcss(), lightViewer],

  test: {
    environment: "jsdom",
    setupFiles: "./src/tests/setup.ts",
    include: ["src/**/*.test.ts?(x)", "../packages/*/tests/**/*.test.ts?(x)"],
    exclude: ["../packages/tisty-light-viewer/**", "**/node_modules/**"],
    env: { TZ: "UTC" },
    testTimeout: 30000,
    coverage: {
      provider: "v8" as const,
      reporter: ["text-summary", "lcov"],
      reportsDirectory: "coverage",
      // The packages live beside the app, outside its root, and are measured all the same.
      allowExternal: true,
      include: ["src/**/*.{ts,tsx}", "../packages/*/src/**/*.{ts,tsx}"],
      exclude: ["src/tests/**", "src/main.tsx", "src/vite-env.d.ts", "../packages/tisty-light-viewer/**"],
    },
  },

  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    fs: {
      allow: [app, ...["packages", "node_modules", "contracts/dist/typescript"].map((one) => join(repository, one))],
    },
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
