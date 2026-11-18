import { describe, expect, it, vi } from "vitest";

const loaded = vi.hoisted(() => [] as string[]);

vi.mock("tisty-sheet/callouts.css", () => {
  loaded.push("callouts");
  return {};
});
vi.mock("tisty-sheet/code.css", () => {
  loaded.push("code");
  return {};
});
vi.mock("tisty-sheet/diagrams.css", () => {
  loaded.push("diagrams");
  return {};
});
vi.mock("tisty-sheet/tables.css", () => {
  throw new Error("this sheet could not be read");
});

import { styledFor } from "../src/styles";

describe("the looks a page's blocks bring", () => {
  it("loads nothing for a page of words alone", async () => {
    await styledFor(["<h1>Riego</h1>", "<p>Los tomates, cada mañana.</p>"]);
    expect(loaded).toEqual([]);
  });

  it("loads only the sheet of each block the page holds, and shows the words when one fails", async () => {
    await expect(
      styledFor([
        '<blockquote class="callout" data-callout="tip"><p>Ojo</p></blockquote>',
        '<div class="table"><table></table></div>',
      ]),
    ).resolves.toBeDefined();
    expect(loaded).toEqual(["callouts"]);

    await styledFor([
      '<figure class="code"><pre><code>x</code></pre></figure>',
      '<figure class="diagram"></figure>',
    ]);
    expect([...loaded].sort((one, other) => one.localeCompare(other))).toEqual([
      "callouts",
      "code",
      "diagrams",
    ]);
  });
});
