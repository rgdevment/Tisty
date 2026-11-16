import { defineConfig, mergeConfig } from "vitest/config";
import app from "./vite.config";

// Each mutant runs every test that reaches it, and the random walks reach the whole page writer for half a minute.
export default defineConfig(async (env) =>
  mergeConfig(await app(env), { test: { exclude: ["src/tests/pageWritingFuzz.test.ts"] } }),
);
