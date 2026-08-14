import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { CATCHES, handTo, takesFiles, whenFilesLand } from "../dropped";

type Payload =
  | { type: "enter"; paths: string[]; position: { x: number; y: number } }
  | { type: "over"; position: { x: number; y: number } }
  | { type: "drop"; paths: string[]; position: { x: number; y: number } }
  | { type: "leave" };

const webview = vi.hoisted(() => ({
  fire: (_p: Payload) => {},
}));

vi.mock("@tauri-apps/api/webview", () => ({
  getCurrentWebview: () => ({
    onDragDropEvent: (handler: (e: { payload: Payload }) => void) => {
      webview.fire = (payload) => handler({ payload });
      return Promise.resolve(() => {});
    },
  }),
}));

function field() {
  const box = document.createElement("div");
  box.setAttribute(CATCHES, "");
  document.body.append(box);
  return box;
}

let looked: (x: number, y: number) => Element | null = () => null;
beforeEach(() => {
  document.elementFromPoint = ((x: number, y: number) => looked(x, y)) as typeof document.elementFromPoint;
});

afterEach(() => {
  document.body.innerHTML = "";
  window.devicePixelRatio = 1;
  looked = () => null;
});

const settled = () => new Promise((done) => setTimeout(done, 0));

describe("a file drag from the system", () => {
  it("does not attach on the way in, only on the drop", async () => {
    const box = field();
    looked = () => box;
    const caught = vi.fn();
    whenFilesLand(caught);
    await settled();

    webview.fire({ type: "enter", paths: ["/a.png"], position: { x: 10, y: 10 } });
    expect(caught).not.toHaveBeenCalled();
    expect(box.classList.contains("catching")).toBe(true);

    webview.fire({ type: "drop", paths: ["/a.png"], position: { x: 10, y: 10 } });
    expect(caught).toHaveBeenCalledWith(box, ["/a.png"], expect.anything());
    expect(box.classList.contains("catching")).toBe(false);
  });

  it("takes macOS at its word, where the position is already in css pixels", async () => {
    const { asCss } = await import("../dropped");

    expect(asCss(200, 100, true, 2)).toEqual([200, 100]);
  });

  it("brings the rest down to css pixels, where the position is in real ones", async () => {
    const { asCss } = await import("../dropped");

    expect(asCss(200, 100, false, 2)).toEqual([100, 50]);
    expect(asCss(200, 100, false, 1)).toEqual([200, 100]);
    expect(asCss(200, 100, false, 0)).toEqual([200, 100]);
  });

  it("hands over where it landed, not only what landed", async () => {
    const box = field();
    looked = () => box;
    const caught = vi.fn();
    whenFilesLand(caught);
    await settled();

    webview.fire({ type: "drop", paths: ["/a.png"], position: { x: 200, y: 100 } });

    expect(caught).toHaveBeenCalledWith(box, ["/a.png"], { left: 200, top: 100 });
  });

  it("stops highlighting when the drag leaves", async () => {
    const box = field();
    looked = () => box;
    whenFilesLand(vi.fn());
    await settled();

    webview.fire({ type: "over", position: { x: 1, y: 1 } });
    expect(box.classList.contains("catching")).toBe(true);
    webview.fire({ type: "leave" });
    expect(box.classList.contains("catching")).toBe(false);
  });

  it("keeps nothing over an element that takes no files", async () => {
    const plain = document.createElement("div");
    document.body.append(plain);
    looked = () => plain;
    const caught = vi.fn();
    whenFilesLand(caught);
    await settled();

    webview.fire({ type: "drop", paths: ["/a.png"], position: { x: 1, y: 1 } });
    expect(caught).not.toHaveBeenCalled();
  });
});

describe("handing the file to the field", () => {
  it("goes to the element that registered, not to a name", () => {
    const one = field();
    const other = field();
    const took = vi.fn();
    takesFiles(one, took);

    expect(handTo(other, "![a](<b>)")).toBe(false);
    expect(took).not.toHaveBeenCalled();

    expect(handTo(one, "![a](<b>)")).toBe(true);
    expect(took).toHaveBeenCalledWith("![a](<b>)", undefined);
  });

  it("says no once the field is gone, instead of losing the file", () => {
    const box = field();
    const off = takesFiles(box, vi.fn());
    off();

    expect(handTo(box, "![a](<b>)")).toBe(false);
  });
});
