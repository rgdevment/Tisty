import { act, renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { useWholeKeys } from "../ui/wholeKeys";

const press = (key: string, init: KeyboardEventInit = {}, on: EventTarget = document.body) => {
  const event = new KeyboardEvent("keydown", {
    key,
    ctrlKey: true,
    bubbles: true,
    cancelable: true,
    ...init,
  });
  act(() => {
    on.dispatchEvent(event);
  });
  return event;
};

describe("the keys a sheet answers to from the window", () => {
  it("opens the whole of a document with Ctrl+E, and takes the key from the page", () => {
    const keys = renderHook(() => useWholeKeys(true, true));
    const pressed = press("e");
    expect(keys.result.current.wholeAsked).toBe(1);
    expect(pressed.defaultPrevented).toBe(true);
    press("E", { metaKey: true, ctrlKey: false });
    expect(keys.result.current.wholeAsked).toBe(2);
  });

  it("leaves Ctrl+E alone on a task's page, which is written by block", () => {
    const keys = renderHook(() => useWholeKeys(true, false));
    const pressed = press("e");
    expect(keys.result.current.wholeAsked).toBe(0);
    expect(pressed.defaultPrevented).toBe(false);
  });

  it("chooses every word of a page just opened, which has not the focus yet", () => {
    const keys = renderHook(() => useWholeKeys(true, false));
    press("a");
    expect(keys.result.current.allAsked).toBe(1);
  });

  it("lets the page choose its own words when Ctrl+A is pressed inside it", () => {
    const keys = renderHook(() => useWholeKeys(true, true));
    press("a", {}, window);
    expect(keys.result.current.allAsked).toBe(0);
  });

  it("keeps a field's keys and any other chord its own", () => {
    const keys = renderHook(() => useWholeKeys(true, true));
    const field = document.createElement("input");
    document.body.append(field);
    press("e", {}, field);
    press("a", {}, field);
    press("e", { shiftKey: true });
    press("e", { altKey: true });
    press("e", { repeat: true });
    press("e", { ctrlKey: false });
    expect(keys.result.current).toEqual({ wholeAsked: 0, allAsked: 0 });
    field.remove();
  });

  it("hears nothing while no page is shown", () => {
    const keys = renderHook(() => useWholeKeys(false, true));
    press("e");
    press("a");
    expect(keys.result.current).toEqual({ wholeAsked: 0, allAsked: 0 });
  });
});
