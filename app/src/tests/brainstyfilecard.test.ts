import { describe, expect, it } from "vitest";
import type { Neuron } from "../brainsty";
import { usersOf } from "../ui/brainsty/cardFacts";
import { fileGlyphOf, fileKindOf } from "../ui/brainsty/FileCard";

const nodes: Neuron[] = [
  { id: "zeta.md", kind: "doc", title: "Zeta" },
  { id: "01A", kind: "task", title: "alpha" },
  { id: "attachments/map-1a2b3c4d.png", kind: "file", title: "map-1a2b3c4d.png" },
  { id: "hidden.md", kind: "doc", title: "Hidden by a filter" },
  { id: "#home", kind: "tag", title: "home" },
];
const edges: [number, number, number][] = [
  [0, 2, 4],
  [1, 2, 4],
  [3, 2, 4],
  [0, 4, 3],
  [0, 1, 0],
];

describe("a file's card", () => {
  it("names who uses the file, by title, leaving out what the filters hide", () => {
    const shown = [false, false, false, true, false];

    expect(usersOf({ nodes, edges }, 2, shown).map((one) => one.title)).toEqual(["alpha", "Zeta"]);
    expect(usersOf({ nodes, edges }, 4, shown)).toEqual([]);
  });

  it("draws the glyph of the kind of file it is, and a clip for one it does not know", () => {
    expect(
      ["a.png", "b.mp4", "c.mp3", "d.zip", "e.xlsx", "f.json", "g.pptx", "h.pdf", "i.xyz", "j"].map(
        (one) => fileGlyphOf(`attachments/${one}`),
      ),
    ).toEqual([
      "file-image",
      "file-video",
      "file-audio",
      "file-archive",
      "sheet",
      "table",
      "book",
      "page",
      "clip",
      "clip",
    ]);
  });

  it("says the kind in words and by its ending, or just that it is a file", () => {
    expect(fileKindOf("attachments/report-1a2b3c4d.pdf")).toBe("PDF");
    expect(fileKindOf("attachments/hill-0714d2ee.jpg")).toBe("Image · JPG");
    expect(fileKindOf("attachments/thing.xyz")).toBe("File · XYZ");
    expect(fileKindOf("attachments/bare")).toBe("File");
  });
});
