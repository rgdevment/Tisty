import { afterEach, describe, expect, it, vi } from "vitest";
import type { Menued } from "../ui/quickly";

vi.mock("../glyphs", () => ({ markup: () => '<path d="M1 1"/>' }));

const MAC = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)";
const WINDOWS = "Mozilla/5.0 (Windows NT 10.0; Win64; x64)";

afterEach(() => {
  vi.restoreAllMocks();
  document.body.innerHTML = "";
});

const menuOn = async (agent: string) => {
  vi.resetModules();
  vi.spyOn(navigator, "userAgent", "get").mockReturnValue(agent);
  const { quickly } = await import("../ui/quickly");
  const frame = document.createElement("iframe");
  document.body.append(frame);
  const page = frame.contentDocument as Document;
  const view = frame.contentWindow as Window & typeof globalThis;
  page.body.innerHTML =
    '<article class="page"><p data-at="0-20">Rina likes her walks</p></article>';
  const asked: Menued[] = [];
  quickly(page, () => ({ onMenu: (one) => asked.push(one) }));
  const block = page.querySelector("p") as HTMLElement;
  const words = block.firstChild as Text;
  const selection = page.getSelection() as Selection;
  const chosen = (from: number, to: number) => {
    const range = page.createRange();
    range.setStart(words, from);
    range.setEnd(words, to);
    selection.removeAllRanges();
    selection.addRange(range);
  };
  const pressed = (type: string, init: MouseEventInit) =>
    block.dispatchEvent(new view.MouseEvent(type, { bubbles: true, cancelable: true, ...init }));
  const rightClicked = (init: MouseEventInit = { button: 2 }) => {
    pressed("mousedown", init);
    chosen(11, 14);
    pressed("contextmenu", init);
  };
  const caret = () => {
    const range = selection.getRangeAt(0);
    return { collapsed: range.collapsed, at: range.startOffset };
  };
  return { asked, words, selection, chosen, pressed, rightClicked, caret };
};

describe("a right-click on the page on a Mac", () => {
  it("puts the caret back where it stood, so the menu is the block's and no word is replaced", async () => {
    const { asked, words, selection, rightClicked, caret } = await menuOn(MAC);
    selection.collapse(words, 4);
    rightClicked();
    expect(caret()).toEqual({ collapsed: true, at: 4 });
    expect(asked[0]?.picked).toBeNull();
    expect(asked[0]?.block?.at).toEqual([0, 20]);
  });

  it("does the same for a click with Control held, and for a page with no caret at all", async () => {
    const { asked, words, selection, rightClicked, caret } = await menuOn(MAC);
    selection.collapse(words, 2);
    rightClicked({ button: 0, ctrlKey: true });
    expect(caret()).toEqual({ collapsed: true, at: 2 });
    selection.removeAllRanges();
    rightClicked();
    expect(selection.rangeCount).toBe(0);
    expect(asked.map((one) => one.picked)).toEqual([null, null]);
  });

  it("keeps words chosen before the right-click, and offers them", async () => {
    const { asked, chosen, rightClicked, caret } = await menuOn(MAC);
    chosen(11, 14);
    rightClicked();
    expect(caret()).toEqual({ collapsed: false, at: 11 });
    expect(asked[0]?.picked?.said).toBe("her");
  });

  it("leaves a menu asked with no press before it alone", async () => {
    const { asked, words, selection, chosen, pressed } = await menuOn(MAC);
    selection.collapse(words, 4);
    pressed("mousedown", { button: 0 });
    chosen(11, 14);
    pressed("contextmenu", {});
    expect(asked[0]?.picked?.said).toBe("her");
  });
});

describe("a right-click on the page elsewhere", () => {
  it("leaves the selection as the engine left it, as before", async () => {
    const { asked, words, selection, rightClicked, caret } = await menuOn(WINDOWS);
    selection.collapse(words, 4);
    rightClicked();
    expect(caret()).toEqual({ collapsed: false, at: 11 });
    expect(asked[0]?.picked?.said).toBe("her");
  });
});
