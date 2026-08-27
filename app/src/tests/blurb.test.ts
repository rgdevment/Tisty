import { describe, expect, it } from "vitest";
import { glimpsed } from "../ui/Editor";

describe("the blurb under a document's title", () => {
  it("reads an icon by the name it carries, not by the tag it travels in", () => {
    const said = glimpsed(
      '# Plan\nLanzamiento <span data-ico="rocket-thing" data-hue="blue">:rocket-thing:</span> el jueves',
    );

    expect(said).toBe("Lanzamiento :rocket-thing: el jueves");
  });

  it("leaves an ordinary line alone", () => {
    expect(glimpsed("# Plan\n- uno\n- dos")).toBe("uno · dos");
  });
});
