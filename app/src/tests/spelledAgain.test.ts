import { afterEach, describe, expect, it, vi } from "vitest";
import { spelledAgain } from "../ui/spelledAgain";

const sheetWithFrames = () => {
  const sheet = document.createElement("div");
  sheet.spellcheck = true;
  document.body.append(sheet);
  const frames: FrameRequestCallback[] = [];
  vi.spyOn(window, "requestAnimationFrame").mockImplementation((frame) => frames.push(frame));
  return { sheet, frames };
};

afterEach(() => vi.restoreAllMocks());

describe("a sheet spelled again", () => {
  it("turns its spellcheck off now and back on at the next frame", () => {
    const { sheet, frames } = sheetWithFrames();

    spelledAgain(sheet);

    expect(sheet.spellcheck).toBe(false);
    expect(frames).toHaveLength(1);
    frames[0](0);
    expect(sheet.spellcheck).toBe(true);
  });

  it("does nothing when there is no sheet", () => {
    const { frames } = sheetWithFrames();

    expect(() => spelledAgain(null)).not.toThrow();
    expect(() => spelledAgain(undefined)).not.toThrow();
    expect(frames).toHaveLength(0);
  });
});
