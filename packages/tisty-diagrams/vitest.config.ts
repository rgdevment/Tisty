import { defineConfig } from "vitest/config";

// The studio's own run, for Stryker: its logic is tested here; the app runs the same tests with its own.
export default defineConfig({
  test: {
    environment: "jsdom",
    include: ["tests/**/*.test.ts?(x)"],
    env: { TZ: "UTC" },
    testTimeout: 30000,
  },
});
