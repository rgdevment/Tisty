import { beforeEach, describe, expect, it, vi } from "vitest";

const ipc = vi.hoisted(() => ({ reply: null as unknown }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () => Promise.resolve(ipc.reply),
}));

import { docDrawn } from "../core";
import { diaryLaid } from "../diary";
import { docPlaced } from "../placing";

const HTML = '<h1 data-at="0-9">Año ñu</h1><p data-at="10-14">→ 🌱</p>';
const BYTES = new TextEncoder().encode(HTML);

const SHAPES: [string, () => unknown][] = [
  ["the custom protocol's buffer", () => BYTES.slice().buffer],
  ["a typed array", () => BYTES.slice()],
  ["the postMessage fallback's plain numbers", () => Array.from(BYTES)],
];

const DRAWERS: [string, () => Promise<string>][] = [
  ["a document", () => docDrawn("# Año ñu")],
  ["a page being written", () => docPlaced("# Año ñu")],
  ["the diary", () => diaryLaid("diario", "# Año ñu")],
];

beforeEach(() => {
  ipc.reply = null;
});

describe("a page drawn by the core reaches the window whole", () => {
  for (const [drawer, draw] of DRAWERS)
    it.each(SHAPES)(`${drawer} reads the same words from %s`, async (_, shape) => {
      ipc.reply = shape();
      expect(await draw()).toBe(HTML);
    });
});
