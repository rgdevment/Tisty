import type { Asking } from "tisty-editor/edit";
import { describe, expect, it, vi } from "vitest";
import { editorAsking } from "../ui/sheetAsking";

describe("what the editor is asked with", () => {
  it("carries every field the window gives, the links among them", () => {
    const window: Asking = {
      kinds: ["doc"],
      tags: ["casa"],
      linkable: () => [],
      headsOf: () => [],
      pasted: () => Promise.resolve(""),
      lost: () => {},
      ask: vi.fn(),
    };

    const said = editorAsking(
      window,
      () => {},
      () => "",
    );

    for (const key of Object.keys(window) as (keyof Asking)[])
      if (key !== "ask") expect(said[key], key).toBe(window[key]);
    said.ask("doc", { x: 1, y: 2 }, () => true, false);
    expect(window.ask).toHaveBeenCalledWith("doc", { x: 1, y: 2 }, expect.any(Function), false);
  });

  it("reads the steps from the words as they are now, and says a failure to the sheet", () => {
    let words = "- [ ] uno";
    const failed = vi.fn();
    const said = editorAsking(undefined, failed, () => words);

    words = "- [ ] uno\n- [ ] dos";
    said.failed?.("x");

    expect(said.kinds).toEqual([]);
    expect(said.steps?.()).toEqual(["uno", "dos"]);
    expect(failed).toHaveBeenCalledWith("x");
  });
});
