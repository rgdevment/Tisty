import { renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { usePreviewing } from "../ui/previewing";

describe("a PDF preview filling the window", () => {
  it("hides the window's own controls while it is open, and gives them back when it closes", () => {
    const root = document.documentElement;
    const { rerender, unmount } = renderHook(({ on }) => usePreviewing(on), {
      initialProps: { on: false },
    });
    expect(root.dataset.previewing).toBeUndefined();

    rerender({ on: true });
    expect(root.dataset.previewing).toBe("");

    rerender({ on: false });
    expect(root.dataset.previewing).toBeUndefined();

    rerender({ on: true });
    unmount();
    expect(root.dataset.previewing).toBeUndefined();
  });
});
