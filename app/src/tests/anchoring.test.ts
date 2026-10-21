import { describe, expect, it } from "vitest";
import { changedIn, reanchored, spanOf, touches } from "../ui/anchoring";

describe("the block being written when words arrive from outside", () => {
  const was = "# Acta\n\nuno\n\nlo mio\n\ntres\n";
  const from = was.indexOf("lo mio");
  const end = from + "lo mio".length;

  it("stays where it was when the change lands after it", () => {
    expect(reanchored(was, `${was}\ncuatro\n`, from, end)).toEqual([from, end]);
  });

  it("moves along when the change lands before it", () => {
    const now = was.replace("uno", "uno y medio");
    const at = reanchored(was, now, from, end);
    expect(at).toEqual([from + 8, end + 8]);
    expect(at && now.slice(...at)).toBe("lo mio");
  });

  it("follows its words when changes land on both sides of it", () => {
    const now = "# Acta de hoy\n\nuno\n\nlo mio\n\ntres\n\ncuatro\n";
    const at = reanchored(was, now, from, end);
    expect(at && now.slice(...at)).toBe("lo mio");
  });

  it("ends when the change lands inside it", () => {
    expect(reanchored(was, was.replace("lo mio", "lo suyo"), from, end)).toBeNull();
  });
});

describe("what words from outside changed", () => {
  it("names the bytes of the new text that differ, counting accents as the page does", () => {
    const was = "# Acción\n\nuno\n";
    const now = "# Acción\n\nuno y dos\n";
    expect(changedIn(was, now)).toEqual([14, 20]);
  });

  it("names nothing when nothing changed", () => {
    expect(changedIn("# Igual\n", "# Igual\n")).toBeNull();
  });
});

describe("the blocks a change touches", () => {
  it("counts the blocks either side of a passage taken out", () => {
    expect(touches([10, 20], [20, 20])).toBe(true);
    expect(touches([20, 30], [20, 20])).toBe(true);
    expect(touches([30, 40], [20, 20])).toBe(false);
  });

  it("counts only the blocks a change runs into", () => {
    expect(touches([10, 20], [20, 25])).toBe(false);
    expect(touches([20, 30], [20, 25])).toBe(true);
  });
});

describe("the one edit an open editor takes from outside", () => {
  it("replaces only the words that differ", () => {
    expect(spanOf("uno dos tres", "uno DOS tres")).toEqual({ from: 4, to: 7, insert: "DOS" });
  });

  it("inserts where the new words begin and removes nothing", () => {
    expect(spanOf("uno tres", "uno dos tres")).toEqual({ from: 4, to: 4, insert: "dos " });
  });

  it("removes what was taken out and inserts nothing", () => {
    expect(spanOf("uno dos tres", "uno tres")).toEqual({ from: 4, to: 8, insert: "" });
  });

  it("adds at the end what was appended", () => {
    expect(spanOf("uno", "uno\n\ndos")).toEqual({ from: 3, to: 3, insert: "\n\ndos" });
  });

  it("asks for nothing when nothing changed", () => {
    expect(spanOf("igual", "igual")).toEqual({ from: 5, to: 5, insert: "" });
  });
});
