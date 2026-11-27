import { act, fireEvent, render, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { dropLater, fireAll } from "../dropLater";
import DropUndo from "../ui/DropUndo";

const waiting = () => {
  const run = vi.fn(() => Promise.resolve());
  act(() => {
    dropLater({ hides: ["d1"], said: "gone", run });
  });
  return run;
};

const undoKey = (on: Window | Element = window, more: KeyboardEventInit = {}) =>
  fireEvent.keyDown(on, { key: "z", code: "KeyZ", ctrlKey: true, ...more });

afterEach(async () => {
  fireAll();
  await act(() => Promise.resolve());
});

describe("the undo key after a delete", () => {
  it("brings back the last delete that is waiting, and it never runs", () => {
    const { container } = render(<DropUndo />);
    const run = waiting();
    expect(container.querySelector("output")).not.toBeNull();

    undoKey();
    fireAll();

    expect(container.querySelector("output")).toBeNull();
    expect(run).not.toHaveBeenCalled();
  });

  it("does nothing to a text field", async () => {
    render(
      <>
        <DropUndo />
        <input aria-label="campo" />
        <textarea aria-label="area" />
      </>,
    );
    const run = waiting();

    for (const label of ["campo", "area"]) {
      const field = document.querySelector<HTMLElement>(`[aria-label="${label}"]`) as HTMLElement;
      expect(undoKey(field)).toBe(true);
    }
    fireAll();

    await waitFor(() => expect(run).toHaveBeenCalledTimes(1));
  });

  it("does nothing to an editable area either", async () => {
    render(<DropUndo />);
    const area = document.createElement("div");
    area.contentEditable = "true";
    Object.defineProperty(area, "isContentEditable", { value: true });
    document.body.append(area);
    const run = waiting();

    undoKey(area);
    fireAll();
    area.remove();

    await waitFor(() => expect(run).toHaveBeenCalledTimes(1));
  });

  it("does nothing when a page already took the key for its own undo", async () => {
    render(<DropUndo />);
    const run = waiting();
    const own = (e: Event) => e.preventDefault();
    document.body.addEventListener("keydown", own);

    undoKey(document.body);
    document.body.removeEventListener("keydown", own);
    fireAll();

    await waitFor(() => expect(run).toHaveBeenCalledTimes(1));
  });

  it("does nothing with nothing waiting", () => {
    render(<DropUndo />);

    expect(undoKey()).toBe(true);
  });
});
