import { beforeEach, describe, expect, it, vi } from "vitest";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  replies: [] as unknown[],
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args });
    if (cmd === "doc_placed") return Promise.resolve(new TextEncoder().encode("<p>Uno</p>").buffer);
    return Promise.resolve(ipc.replies.shift());
  },
}));

import { checkOf, docPlaced, docPlacedNear } from "../placing";

const BYTES = new TextEncoder();
const NEAR = { count: 1, sum: [1, 2], from: 0, to: 1, html: "<p>Unos</p>", heads: null };
const SENT = {
  body: "Unos",
  was: checkOf("Uno"),
  now: checkOf("Unos"),
  edit: { from: 3, to: 3, insert: "s" },
};
const KEY = { edit: { ...SENT.edit, was: SENT.was } };

beforeEach(() => {
  ipc.calls.length = 0;
  ipc.replies.length = 0;
});

describe("a key told to the core", () => {
  it("sends only the key, with the check of the text it was typed on and of the one it made", async () => {
    ipc.replies.push(NEAR);
    expect(await docPlacedNear(SENT, 3, 4)).toEqual(NEAR);
    expect(ipc.calls).toEqual([
      { cmd: "doc_placed_near", args: { sent: KEY, now: SENT.now, from: 3, to: 4 } },
    ]);
  });

  it("sends the whole body in the same step when the core holds another text", async () => {
    ipc.replies.push(null, NEAR);
    expect(await docPlacedNear(SENT, 3, 4)).toEqual(NEAR);
    expect(ipc.calls.map((one) => one.args)).toEqual([
      { sent: KEY, now: SENT.now, from: 3, to: 4 },
      { sent: { body: "Unos" }, now: SENT.now, from: 3, to: 4 },
    ]);
  });

  it("seeds the core with the check of the whole page it draws", async () => {
    expect(await docPlaced("Uno")).toBe("<p>Uno</p>");
    expect(ipc.calls).toEqual([
      { cmd: "doc_placed", args: { body: "Uno", check: checkOf("Uno") } },
    ]);
  });
});

describe("the reckoning of a text the core is told", () => {
  it("counts its bytes as the core does and tells apart texts one letter apart", () => {
    for (const text of [
      "",
      "a",
      "ñandú",
      "😀 y 😁",
      "uno\r\ndos",
      "# Hola\n\nUno 😀 dos ñandú\r\n",
    ]) {
      expect(checkOf(text).bytes).toBe(BYTES.encode(text).length);
    }
    expect(checkOf("Uno dos")).toEqual(checkOf("Uno dos"));
    const [fnv, mixed] = checkOf("Uno dos").hash;
    const [otherFnv, otherMixed] = checkOf("Uno des").hash;
    expect(fnv).not.toBe(otherFnv);
    expect(mixed).not.toBe(otherMixed);
  });
});
