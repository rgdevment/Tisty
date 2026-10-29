import { fileURLToPath } from "node:url";
import { defineConfig } from "vitest/config";

// The reader's own run, for Stryker: its logic is tested here; the app runs the same tests with its own.
export default defineConfig({
  resolve: {
    alias: { "tisty-diagrams": fileURLToPath(new URL("../tisty-diagrams/src", import.meta.url)) },
  },
  test: {
    environment: "jsdom",
    include: ["tests/**/*.test.ts?(x)"],
    env: { TZ: "UTC" },
    testTimeout: 30000,
  },
});
