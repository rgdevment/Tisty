import { renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { Seat } from "../ui/pageWriter";
import { useSeated } from "../ui/seated";

const landing = (said: string[]) => (next: (now: string) => string | null) => {
  const now = next("words");
  if (now !== null) said.push(now);
  return now !== null;
};

describe("the seat a sheet holds for the document it shows", () => {
  it("is held while the sheet is there, and lays late words through whoever shows it", () => {
    const said: string[] = [];
    const view = renderHook(() => useSeated("a.md", undefined, landing(said)));
    const seat = view.result.current.current as Seat;

    expect(seat.current?.((now) => `${now}!`)).toBe(true);
    expect(said).toEqual(["words!"]);

    view.unmount();
    expect(view.result.current.current).toBeNull();
    expect(seat.current).toBeNull();
  });

  it("is a new one when the sheet moves to another file, and the old one lays nothing", () => {
    const view = renderHook(({ file }) => useSeated(file, undefined, landing([])), {
      initialProps: { file: "a.md" },
    });
    const first = view.result.current.current as Seat;

    view.rerender({ file: "b.md" });

    expect(view.result.current.current).not.toBe(first);
    expect(first.current).toBeNull();
  });

  it("stays the one its holder gives across a new file name and a new sheet", () => {
    const given: Seat = { current: null };
    const said: string[] = [];
    const first = renderHook(({ file }) => useSeated(file, given, landing([])), {
      initialProps: { file: "unwritten:one" },
    });
    first.rerender({ file: "t_one-0001" });
    expect(first.result.current.current).toBe(given);

    first.unmount();
    const second = renderHook(() => useSeated("t_one-0001", given, landing(said)));

    expect(second.result.current.current).toBe(given);
    expect(given.current?.((now) => now)).toBe(true);
    expect(said).toEqual(["words"]);
  });
});
