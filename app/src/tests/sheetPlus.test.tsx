import { render, screen, waitFor } from "@testing-library/react";
import type { Asking } from "tisty-editor/edit";
import { afterEach, describe, expect, it, vi } from "vitest";
import { t } from "../locales";
import Sheet from "../ui/Sheet";

const DRAWN: Record<string, string> = {
  "- \n": '<ul data-at="11-13"><li data-at="11-13"></li></ul>',
  "## \n": '<h2 data-at="11-14"></h2>',
  "leche\n": '<p data-at="11-17"><span data-src="11-16">leche</span></p>',
  "- uno\n- dos\n- tres\n":
    '<ul data-at="11-31">' +
    '<li data-at="11-17"><span data-src="13-16">uno</span></li>' +
    '<li data-at="17-23"><span data-src="19-22">dos</span></li>' +
    '<li data-at="23-31"><span data-src="25-29">tres</span></li></ul>',
};
const FOLLOWED = "- \n\n## \n\nOtro parrafo\n";
DRAWN[FOLLOWED] =
  '<ul data-at="11-13"><li data-at="11-13"></li></ul><h2 data-at="14-17"></h2>' +
  '<p data-at="18-31"><span data-src="18-30">Otro parrafo</span></p>';
DRAWN["- \n\nOtro parrafo\n"] =
  '<ul data-at="11-13"><li data-at="11-13"></li></ul>' +
  '<p data-at="14-27"><span data-src="14-26">Otro parrafo</span></p>';
const HEAD = '<h1 data-at="0-10"><span data-src="2-9">Compras</span></h1>';

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args: { body?: string }) => {
    if (cmd === "doc_drawn" || cmd === "doc_placed") {
      const body = (args?.body ?? "").replace("# Compras\n\n", "");
      const drawn = `${HEAD}${DRAWN[body] ?? ""}`;
      return Promise.resolve(new TextEncoder().encode(drawn).buffer);
    }
    return Promise.resolve(null);
  },
  convertFileSrc: (path: string) => path,
}));

const asking: Asking = { kinds: [], ask: () => {}, lost: () => {} };
const settled = () => new Promise((done) => setTimeout(done, 50));

type Box = { top: number; bottom: number; left?: number; right?: number };

// The frame's own window and document, with a layout laid by hand since jsdom lays nothing out.
const laid = async (container: HTMLElement, boxes: (page: Document) => Map<Element, Box>) => {
  const frame = await waitFor(() => {
    const found = container.querySelector("iframe");
    expect(found).toBeTruthy();
    return found as HTMLIFrameElement;
  });
  await waitFor(() => expect(frame.srcdoc).toContain("<article"));
  const page = frame.contentDocument as Document;
  page.open();
  page.write(frame.srcdoc);
  page.close();
  frame.dispatchEvent(new Event("load"));
  await waitFor(() =>
    expect(page.querySelector(".page")?.getAttribute("contenteditable")).toBe("true"),
  );
  const view = page.defaultView as Window & typeof globalThis;
  const placed = boxes(page);
  view.Element.prototype.getBoundingClientRect = function (this: Element) {
    const box = placed.get(this) ?? { top: 0, bottom: 0 };
    const [top, bottom] = [box.top, box.bottom];
    return {
      top,
      bottom,
      height: bottom - top,
      left: box.left ?? 0,
      right: box.right ?? 0,
    } as DOMRect;
  };
  view.getComputedStyle = (() => ({ paddingLeft: "120px", lineHeight: "20px" })) as never;
  Object.defineProperty(view, "innerHeight", { configurable: true, value: 600 });
  const measured = view.Range.prototype as unknown as { getBoundingClientRect?: () => unknown };
  measured.getBoundingClientRect ??= () => new DOMRect();
  return page;
};

const sheetBox = { top: 0, bottom: 600, left: 100, right: 700 };

const hover = (page: Document, clientY: number) =>
  page.dispatchEvent(new MouseEvent("mousemove", { clientX: 300, clientY, bubbles: true }));

const plusOf = async (page: Document) => {
  const plus = page.querySelector<HTMLElement>(".page-plus");
  await waitFor(() => expect(plus?.hidden).toBe(false));
  return plus as HTMLElement;
};

const mount = (value: string, writes: string[] = []) =>
  render(
    <Sheet
      file="compras.md"
      value={`# Compras\n\n${value}`}
      label="Compras"
      asking={asking}
      onWrite={(text) => writes.push(text)}
    />,
  );

afterEach(() => vi.restoreAllMocks());

const chosen = async (value: string, tops: Map<string, Box>, y: number) => {
  const writes: string[] = [];
  const view = mount(value, writes);
  const page = await laid(view.container, (doc) => {
    const boxes = new Map<Element, Box>([[doc.querySelector(".page") as Element, sheetBox]]);
    for (const [selector, box] of tops) boxes.set(doc.querySelector(selector) as Element, box);
    return boxes;
  });
  hover(page, y);
  const plus = await plusOf(page);
  plus.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  const heading = await screen.findByRole("menuitem", { name: new RegExp(t("bigTitle")) });
  heading.click();
  await settled();
  return writes;
};

const alone = (selector: string) => new Map([[selector, { top: 100, bottom: 120 }]]);

describe("the + beside a bare marker", () => {
  it("turns an empty list item into the block picked, not a line begun under it", async () => {
    const writes = await chosen("- \n", alone(".page > ul"), 110);

    expect(writes[writes.length - 1]).toBe("# Compras\n\n# \n");
  });

  it("turns an empty heading into the block picked", async () => {
    const writes = await chosen("## \n", alone(".page > h2"), 110);

    expect(writes[writes.length - 1]).toBe("# Compras\n\n# \n");
  });

  it("turns that very item when an empty heading and a paragraph follow it", async () => {
    const writes = await chosen(FOLLOWED, alone(".page > ul"), 110);

    expect(writes[writes.length - 1]).toBe("# Compras\n\n# \n\n## \n\nOtro parrafo\n");
  });

  it("turns that very item when only a paragraph follows it", async () => {
    const writes = await chosen("- \n\nOtro parrafo\n", alone(".page > ul"), 110);

    expect(writes[writes.length - 1]).toBe("# Compras\n\n# \n\nOtro parrafo\n");
  });

  it("still begins a line under a block with words", async () => {
    const writes = await chosen("leche\n", alone(".page > p"), 110);

    expect(writes[writes.length - 1]).toMatch(/^# Compras\n\nleche\n\n# ?\n?$/);
  });
});

describe("where the + stands", () => {
  const LIST = "- uno\n- dos\n- tres\n";
  const items = new Map<string, Box>([
    [".page > ul", { top: 100, bottom: 200 }],
    ["li:nth-child(1)", { top: 100, bottom: 120 }],
    ["li:nth-child(2)", { top: 140, bottom: 160 }],
    ["li:nth-child(3)", { top: 180, bottom: 200 }],
  ]);
  const listed = async () => {
    const view = mount(LIST);
    const page = await laid(view.container, (doc) => {
      const boxes = new Map<Element, Box>([[doc.querySelector(".page") as Element, sheetBox]]);
      for (const [selector, box] of items) boxes.set(doc.querySelector(selector) as Element, box);
      return boxes;
    });
    return page;
  };
  const topAfter = async (page: Document, y: number, was: string) => {
    hover(page, y);
    const plus = await plusOf(page);
    await waitFor(() => expect(plus.style.top).not.toBe(was));
    return plus.style.top;
  };

  it("is beside the list item under the pointer", async () => {
    const page = await listed();
    hover(page, 150);
    const plus = await plusOf(page);

    expect(plus.style.top).toBe("150px");
  });

  it("follows the pointer from one item to the next within the list", async () => {
    const page = await listed();
    hover(page, 110);
    const plus = await plusOf(page);
    expect(plus.style.top).toBe("110px");

    expect(await topAfter(page, 190, plus.style.top)).toBe("190px");
  });

  it("stays by the item above while the pointer is in the gap between two", async () => {
    const page = await listed();
    hover(page, 190);
    const plus = await plusOf(page);
    expect(plus.style.top).toBe("190px");

    expect(await topAfter(page, 170, plus.style.top)).toBe("150px");
  });

  it("is clamped into view for a block whose top is above the window", async () => {
    const view = mount("leche\n");
    const page = await laid(
      view.container,
      (doc) =>
        new Map<Element, Box>([
          [doc.querySelector(".page") as Element, sheetBox],
          [doc.querySelector(".page > p") as Element, { top: -50, bottom: 300 }],
        ]),
    );
    hover(page, 100);
    const plus = await plusOf(page);

    expect(plus.style.top).toBe("10px");
  });
});

describe("the + on a key typed", () => {
  const typing = (page: Document, times: number) => {
    for (let at = 0; at < times; at++)
      page.dispatchEvent(new KeyboardEvent("keydown", { key: "a", bubbles: true }));
  };
  const watched = async () => {
    const view = mount("leche\n");
    const page = await laid(
      view.container,
      (doc) =>
        new Map<Element, Box>([
          [doc.querySelector(".page") as Element, sheetBox],
          [doc.querySelector(".page > p") as Element, { top: 100, bottom: 120 }],
        ]),
    );
    const plus = page.querySelector(".page-plus") as HTMLElement;
    const observer = new (page.defaultView as Window & typeof globalThis).MutationObserver(
      () => {},
    );
    observer.observe(plus, { attributes: true });
    return { page, plus, observer };
  };

  it("is not written to at all while it is already hidden", async () => {
    const { page, plus, observer } = await watched();
    expect(plus.hidden).toBe(true);
    observer.takeRecords();

    typing(page, 5);

    expect(observer.takeRecords()).toHaveLength(0);
  });

  it("is hidden by one key, with a single write", async () => {
    const { page, plus, observer } = await watched();
    hover(page, 110);
    await plusOf(page);
    observer.takeRecords();

    typing(page, 5);

    expect(plus.hidden).toBe(true);
    expect(observer.takeRecords()).toHaveLength(1);
  });
});
