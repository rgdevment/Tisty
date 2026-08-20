import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const store = vi.hoisted(() => ({ toggles: 0 }));

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    label: "main",
    minimize: () => Promise.resolve(),
    close: () => Promise.resolve(),
    toggleMaximize: () => {
      store.toggles += 1;
      return Promise.resolve();
    },
    onFocusChanged: () => Promise.resolve(() => {}),
  }),
}));

const MAC = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)";
const WINDOWS = "Mozilla/5.0 (Windows NT 10.0; Win64; x64)";

const chrome = async (agent: string) => {
  vi.resetModules();
  vi.spyOn(navigator, "userAgent", "get").mockReturnValue(agent);
  const { default: WindowChrome } = await import("../ui/WindowChrome");
  render(
    <div>
      <WindowChrome />
      <div data-tauri-drag-region data-testid="bar" />
    </div>,
  );
  return screen.getByTestId("bar");
};

const at = (x: number, y: number, detail = 2) => ({ button: 0, detail, clientX: x, clientY: y });

describe("double-clicking the top bar", () => {
  beforeEach(() => {
    store.toggles = 0;
  });

  it("grows the window on macOS, where Tauri only knows how to shrink it", async () => {
    const bar = await chrome(MAC);

    fireEvent.mouseDown(bar, at(200, 12));
    fireEvent.mouseUp(bar, at(200, 12));

    expect(store.toggles).toBe(1);
  });

  it("lets the gesture go when the pointer moved in between", async () => {
    const bar = await chrome(MAC);

    fireEvent.mouseDown(bar, at(200, 12));
    fireEvent.mouseUp(bar, at(260, 12));

    expect(store.toggles).toBe(0);
  });

  it("stays out of the way of a single click", async () => {
    const bar = await chrome(MAC);

    fireEvent.mouseDown(bar, at(200, 12, 1));
    fireEvent.mouseUp(bar, at(200, 12, 1));

    expect(store.toggles).toBe(0);
  });

  it("does nothing where the bar is not a drag region", async () => {
    await chrome(MAC);
    const plain = document.createElement("div");
    document.body.appendChild(plain);

    fireEvent.mouseDown(plain, at(200, 12));
    fireEvent.mouseUp(plain, at(200, 12));
    plain.remove();

    expect(store.toggles).toBe(0);
  });

  it("silences Tauri's own handler, so the two do not undo each other", async () => {
    const bar = await chrome(MAC);
    const theirs = vi.fn();
    document.addEventListener("mouseup", theirs);

    fireEvent.mouseDown(bar, at(200, 12));
    fireEvent.mouseUp(bar, at(200, 12));
    document.removeEventListener("mouseup", theirs);

    expect(store.toggles).toBe(1);
    expect(theirs).not.toHaveBeenCalled();
  });

  it("keeps Tauri's handler on Windows, where it is the one that works", async () => {
    const bar = await chrome(WINDOWS);
    const theirs = vi.fn();
    document.addEventListener("mouseup", theirs);

    fireEvent.mouseDown(bar, at(200, 12));
    fireEvent.mouseUp(bar, at(200, 12));
    document.removeEventListener("mouseup", theirs);

    expect(theirs).toHaveBeenCalled();
  });

  it("leaves Windows to Tauri, which grows the window there already", async () => {
    const bar = await chrome(WINDOWS);

    fireEvent.mouseDown(bar, at(200, 12));
    fireEvent.mouseUp(bar, at(200, 12));

    expect(store.toggles).toBe(0);
  });
});
