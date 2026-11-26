import { fireEvent, render } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import NewDocKey from "../ui/NewDocKey";

describe("the shortcut that makes a document", () => {
  it("makes one on Ctrl+N from anywhere in the window", () => {
    const onNew = vi.fn();
    render(<NewDocKey onNew={onNew} />);

    const pressed = fireEvent.keyDown(document.body, { key: "n", code: "KeyN", ctrlKey: true });

    expect(onNew).toHaveBeenCalledTimes(1);
    expect(pressed).toBe(false);
  });

  it("makes one on Cmd+N too", () => {
    const onNew = vi.fn();
    render(<NewDocKey onNew={onNew} />);

    fireEvent.keyDown(document.body, { key: "n", code: "KeyN", metaKey: true });

    expect(onNew).toHaveBeenCalledTimes(1);
  });

  it("leaves a plain N, and N with Shift or Alt, to what is being typed", () => {
    const onNew = vi.fn();
    render(<NewDocKey onNew={onNew} />);

    fireEvent.keyDown(document.body, { key: "n", code: "KeyN" });
    fireEvent.keyDown(document.body, { key: "N", code: "KeyN", ctrlKey: true, shiftKey: true });
    fireEvent.keyDown(document.body, { key: "n", code: "KeyN", ctrlKey: true, altKey: true });
    fireEvent.keyDown(document.body, { key: "m", code: "KeyM", ctrlKey: true });

    expect(onNew).not.toHaveBeenCalled();
  });

  it("calls the latest function without listening again", () => {
    const first = vi.fn();
    const second = vi.fn();
    const { rerender } = render(<NewDocKey onNew={first} />);
    rerender(<NewDocKey onNew={second} />);

    fireEvent.keyDown(document.body, { key: "n", code: "KeyN", ctrlKey: true });

    expect(first).not.toHaveBeenCalled();
    expect(second).toHaveBeenCalledTimes(1);
  });

  it("stops listening once it is gone", () => {
    const onNew = vi.fn();
    const { unmount } = render(<NewDocKey onNew={onNew} />);
    unmount();

    fireEvent.keyDown(document.body, { key: "n", code: "KeyN", ctrlKey: true });

    expect(onNew).not.toHaveBeenCalled();
  });

  it("is ignored while a modal dialog is open", () => {
    const onNew = vi.fn();
    render(<NewDocKey onNew={onNew} />);
    const dialog = document.createElement("div");
    dialog.setAttribute("aria-modal", "true");
    document.body.appendChild(dialog);

    fireEvent.keyDown(document.body, { key: "n", code: "KeyN", ctrlKey: true });
    dialog.remove();

    expect(onNew).not.toHaveBeenCalled();
  });
});
