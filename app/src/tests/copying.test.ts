import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { beforeEach, describe, expect, it, vi } from "vitest";

const disk = vi.hoisted(() => ({ body: "lo viejo", pending: "" }));
const board = vi.hoisted(() => ({ text: null as string | null, refuses: false }));

vi.mock("../core", () => ({
  docRead: () => Promise.resolve(disk.body),
  docPlain: (body: string) => Promise.resolve(`plain: ${body}`),
  copied: (text: string) => {
    if (board.refuses) return Promise.reject(new Error("no clipboard here"));
    board.text = text;
    return Promise.resolve();
  },
  noteTrouble: () => Promise.resolve(),
}));

vi.mock("../saving", () => ({
  settled: async () => {
    await Promise.resolve();
    if (disk.pending) disk.body = disk.pending;
  },
}));

import { asPlain } from "../copying";
import { saidPlainly } from "../refusal";

beforeEach(() => {
  disk.body = "lo viejo";
  disk.pending = "";
  board.text = null;
  board.refuses = false;
});

describe("copying a document as markdown", () => {
  it("waits for what is still being written before it reads the disk", async () => {
    disk.pending = "lo recién escrito";

    await asPlain("mac0-0001");

    expect(board.text).toBe("plain: lo recién escrito");
  });

  it("hands over the document as the core says it to a reader outside Tisty", async () => {
    disk.body = "algo ++subrayado++ aquí";

    await asPlain("mac0-0001");

    expect(board.text).toBe("plain: algo ++subrayado++ aquí");
  });

  it("says the clipboard failed, rather than blaming something internal", async () => {
    board.refuses = true;

    await expect(asPlain("mac0-0001")).rejects.toMatchObject({ code: "noClipboard" });
    expect(saidPlainly(await asPlain("mac0-0001").catch((e) => e))).toBe(
      "Tisty could not reach the clipboard",
    );
  });

  it("never reaches for the browser clipboard, which the packaged app does not have", () => {
    const walk = (at: string): string[] =>
      readdirSync(at, { withFileTypes: true }).flatMap((one) => {
        const here = join(at, one.name);
        if (one.isDirectory()) return one.name === "tests" ? [] : walk(here);
        return /\.tsx?$/.test(one.name) ? [here] : [];
      });

    const guilty = walk("src").filter((one) =>
      /navigator\s*\.\s*clipboard/.test(readFileSync(one, "utf8")),
    );

    expect(guilty).toEqual([]);
  });
});
