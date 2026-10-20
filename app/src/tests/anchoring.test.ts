import { describe, expect, it } from "vitest";
import { changedIn, reanchored } from "../ui/anchoring";

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
    expect(at).toEqual([from + 7, end + 7]);
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
