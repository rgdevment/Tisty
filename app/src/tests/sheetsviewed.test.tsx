import { act, renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { useSheets } from "../ui/viewer/useSheets";

const ACROSS = "---\nsheets: on\norientation: landscape\n---\n# Plan\n";

describe("sheets in the viewer", () => {
  it("start as the file asks", () => {
    expect(renderHook(() => useSheets("a", "# Plan\n")).result.current).toMatchObject({
      sheets: false,
      landscape: false,
    });
    expect(renderHook(() => useSheets("a", ACROSS)).result.current).toMatchObject({
      sheets: true,
      landscape: true,
    });
  });

  it("come back in the file's orientation when they are turned off and on again", () => {
    const shown = renderHook(() => useSheets("a", ACROSS));
    act(() => shown.result.current.onSheets(false));
    expect(shown.result.current).toMatchObject({ sheets: false, landscape: false });
    act(() => shown.result.current.onSheets(true));
    expect(shown.result.current).toMatchObject({ sheets: true, landscape: true });
  });

  it("keep a chosen orientation, and turn on when one is chosen", () => {
    const shown = renderHook(() => useSheets("a", "# Plan\n"));
    act(() => shown.result.current.onLandscape(true));
    expect(shown.result.current).toMatchObject({ sheets: true, landscape: true });
    act(() => shown.result.current.onSheets(false));
    act(() => shown.result.current.onSheets(true));
    expect(shown.result.current.landscape).toBe(true);
  });

  it("belong to their file, so the next one opens as it asks from its first frame", () => {
    const seen: boolean[] = [];
    const shown = renderHook(
      ({ which, body }) => {
        const shape = useSheets(which, body);
        seen.push(shape.sheets);
        return shape;
      },
      { initialProps: { which: "a", body: "# A\n" } },
    );
    act(() => shown.result.current.onSheets(true));
    seen.length = 0;
    shown.rerender({ which: "b", body: "# B\n" });
    expect(seen.length).toBeGreaterThan(0);
    expect(seen.every((one) => !one)).toBe(true);
    shown.rerender({ which: "a", body: "# A\n" });
    expect(shown.result.current.sheets).toBe(false);
  });
});
