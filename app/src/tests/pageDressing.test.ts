import { afterEach, describe, expect, it, vi } from "vitest";
import { dressedLaid } from "../ui/pageDressing";

vi.mock("../inUse", async (original) => ({
  ...(await original<typeof import("../inUse")>()),
  serve: (reference: string) => Promise.resolve(`C:/kept/${reference}`),
}));

vi.mock("@tauri-apps/api/core", async (original) => ({
  ...(await original<typeof import("@tauri-apps/api/core")>()),
  convertFileSrc: (path: string) => `asset://${path}`,
}));

const TABLE =
  '<div class="table" data-at="0-30"><table><thead><tr><th data-type="text">a</th><th data-type="check">b</th></tr></thead><tbody><tr><td>1</td><td></td></tr></tbody></table></div>';

afterEach(() => {
  document.body.innerHTML = "";
});

const sheetWith = (inner: string) => {
  document.body.innerHTML = `<article class="page">${inner}</article>`;
  const sheet = document.querySelector(".page") as Element;
  return { sheet, laid: Array.from(sheet.children) };
};

const settled = () => new Promise((done) => setTimeout(done, 0));

describe("the blocks the core draws again on the page written in place", () => {
  it("dress a table laid at the top of the page, itself the block", () => {
    const { sheet, laid } = sheetWith(TABLE);
    dressedLaid(sheet, laid);
    expect(sheet.querySelector(".table")?.classList.contains("dressed")).toBe(true);
    expect(sheet.querySelector(".table-bar")).not.toBeNull();
  });

  it("bring their pictures as the page does", async () => {
    const { sheet, laid } = sheetWith('<p data-at="0-20"><img src="picture.png" alt=""></p>');
    dressedLaid(sheet, laid);
    await settled();
    expect(sheet.querySelector("img")?.getAttribute("src")).toBe("asset://C:/kept/picture.png");
  });

  it("give a diagram its bar to open it and send it out, once", () => {
    const { sheet, laid } = sheetWith(
      '<figure class="diagram" data-at="0-40"><pre><code>flowchart LR</code></pre></figure>',
    );
    dressedLaid(sheet, laid);
    dressedLaid(sheet, laid);
    const asked = Array.from(sheet.querySelectorAll(".diagram-bar button"), (one) =>
      one.getAttribute("data-asked"),
    );
    expect(asked).toEqual(["diagramExport", "diagram"]);
  });
});
