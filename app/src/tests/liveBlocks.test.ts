import { afterEach, describe, expect, it, vi } from "vitest";
import { blockDrawer } from "../ui/liveBlocks";

const drawn = vi.hoisted(() => ({ html: "", asked: [] as string[] }));

vi.mock("../core", async (original) => ({
  ...(await original<typeof import("../core")>()),
  docDrawn: (body: string) => {
    drawn.asked.push(body);
    return Promise.resolve(drawn.html);
  },
}));

vi.mock("../inUse", async (original) => ({
  ...(await original<typeof import("../inUse")>()),
  serve: (reference: string) => Promise.resolve(`C:/kept/${reference}`),
}));

vi.mock("@tauri-apps/api/core", async (original) => ({
  ...(await original<typeof import("@tauri-apps/api/core")>()),
  convertFileSrc: (path: string) => `asset://${path}`,
}));

const TABLE = "| a | b |\n| --- | --- |\n| 1 | 2 |";
const TABLE_HTML =
  '<table><thead><tr><th data-type="text">a</th><th data-type="check">b</th></tr></thead><tbody><tr><td>1</td><td></td></tr></tbody></table>';

afterEach(() => {
  document.body.innerHTML = "";
  drawn.asked = [];
});

const pageWith = (inner: string) => {
  const page = document.implementation.createHTMLDocument("page");
  page.body.innerHTML = `<article class="page">${inner}</article>`;
  return page;
};

const settled = () => new Promise((done) => setTimeout(done, 0));

describe("a block drawn for the whole-document editor", () => {
  it("is copied from the page already drawn, with no page bytes and its table dressed", () => {
    const page = pageWith(`<div class="table" data-at="0-${TABLE.length}">${TABLE_HTML}</div>`);
    const into = page.createElement("div");
    page.body.append(into);
    blockDrawer(page, `${TABLE}\n`)(TABLE, into, () => {});
    expect(drawn.asked).toEqual([]);
    expect(into.querySelector("[data-at]")).toBeNull();
    expect(into.querySelector('[data-act="column"]')).not.toBeNull();
    expect(into.querySelector('[data-act="rowAdd"]')).not.toBeNull();
    expect(into.querySelector(".col-grip")).not.toBeNull();
    expect(into.querySelector('[data-act="toggle"]')).not.toBeNull();
  });

  it("is drawn anew by the core once its words changed, and measured when it lands", async () => {
    drawn.html = `<div class="front">props</div><div class="table" data-at="0-9">${TABLE_HTML}</div>`;
    const page = pageWith("");
    const into = page.createElement("div");
    page.body.append(into);
    const measured = vi.fn();
    blockDrawer(page, "")("| new |", into, measured);
    expect(into.textContent).toBe("| new |");
    expect(into.classList.contains("cm-rich-waiting")).toBe(true);
    await settled();
    expect(drawn.asked).toEqual(["| new |"]);
    expect(into.querySelector(".front")).toBeNull();
    expect(into.querySelector("[data-at]")).toBeNull();
    expect(into.querySelector('[data-act="column"]')).not.toBeNull();
    expect(into.classList.contains("cm-rich-waiting")).toBe(false);
    expect(measured).toHaveBeenCalled();
  });

  it("brings its pictures as the page does", async () => {
    drawn.html = '<p><img src="rina.png" alt="Rina"></p>';
    const page = pageWith("");
    const into = page.createElement("div");
    page.body.append(into);
    blockDrawer(page, "")("![Rina](rina.png)", into, () => {});
    await settled();
    await settled();
    const img = into.querySelector("img");
    expect(img?.dataset.kept).toBe("rina.png");
    expect(img?.getAttribute("src")).toBe("asset://C:/kept/rina.png");
  });

  it("leaves a block that left the page before its drawing came back", async () => {
    drawn.html = "<p>late</p>";
    const page = pageWith("");
    const into = page.createElement("div");
    blockDrawer(page, "")("late", into, () => {});
    await settled();
    expect(into.textContent).toBe("late");
    expect(into.classList.contains("cm-rich-waiting")).toBe(true);
  });
});
